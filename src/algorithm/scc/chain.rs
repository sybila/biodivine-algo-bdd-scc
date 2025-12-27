use crate::algorithm::log_set;
use crate::algorithm::reachability::ReachabilityAlgorithm;
use crate::algorithm::scc::{SccConfig, try_report_scc};
use crate::algorithm::trimming::TrimSetting;
use crate::algorithm_trait::Incomplete::Working;
use crate::algorithm_trait::{Completable, DynComputable, GeneratorStep};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use log::{debug, info};
use std::marker::PhantomData;

pub struct RemainingSet {
    set: GraphColoredVertices,
    pivot_hint: GraphColoredVertices,
}

pub enum IterationState {
    Trimming {
        universe: GraphColoredVertices,
        pivot_hint: GraphColoredVertices,
        trimmed: DynComputable<GraphColoredVertices>,
    },
    ChainBasin {
        universe: GraphColoredVertices,
        pivot: GraphColoredVertices,
        basin: DynComputable<GraphColoredVertices>,
    },
    ChainScc {
        universe: GraphColoredVertices,
        basin: GraphColoredVertices,
        scc: DynComputable<GraphColoredVertices>,
    },
}

pub struct ChainState {
    computing: Option<IterationState>,
    to_process: Vec<RemainingSet>,
}

impl From<&SymbolicAsyncGraph> for ChainState {
    fn from(value: &SymbolicAsyncGraph) -> Self {
        ChainState {
            computing: None,
            to_process: vec![RemainingSet {
                set: value.mk_unit_colored_vertices(),
                pivot_hint: value.mk_empty_colored_vertices(),
            }],
        }
    }
}

impl IterationState {
    fn new_trim_state<BWD: ReachabilityAlgorithm>(
        context: &SccConfig,
        universe: GraphColoredVertices,
        pivot_hint: GraphColoredVertices,
    ) -> Self {
        if context.should_trim == TrimSetting::None {
            Self::new_basin_state::<BWD>(context, universe, &pivot_hint)
        } else {
            Self::Trimming {
                universe: universe.clone(),
                pivot_hint,
                trimmed: context
                    .should_trim
                    .build_computation(&context.graph, universe),
            }
        }
    }

    fn new_basin_state<BWD: ReachabilityAlgorithm>(
        context: &SccConfig,
        universe: GraphColoredVertices,
        pivot_hint: &GraphColoredVertices,
    ) -> Self {
        let graph = context.graph.restrict(&universe);

        let pivot = if pivot_hint.is_empty() {
            universe.pick_vertex()
        } else {
            pivot_hint.pick_vertex()
        };

        Self::ChainBasin {
            basin: Box::new(BWD::configure(graph, pivot.clone())),
            pivot: pivot.clone(),
            universe,
        }
    }

    fn new_scc<FWD: ReachabilityAlgorithm>(
        context: &SccConfig,
        universe: GraphColoredVertices,
        pivot: GraphColoredVertices,
        basin: GraphColoredVertices,
    ) -> Self {
        // Note that in this case, the reachability is explicitly restricted to the basin,
        // not the universe set.
        let graph = context.graph.restrict(&basin);

        Self::ChainScc {
            scc: Box::new(FWD::configure(graph, pivot)),
            universe,
            basin,
        }
    }
}

pub struct ChainStep<FWD: ReachabilityAlgorithm, BWD: ReachabilityAlgorithm> {
    _phantom: PhantomData<(FWD, BWD)>,
}

impl<FWD: ReachabilityAlgorithm, BWD: ReachabilityAlgorithm>
    GeneratorStep<SccConfig, ChainState, GraphColoredVertices> for ChainStep<FWD, BWD>
{
    fn step(
        context: &SccConfig,
        state: &mut ChainState,
    ) -> Completable<Option<GraphColoredVertices>> {
        if let Some(iteration) = state.computing.as_mut() {
            match iteration {
                IterationState::Trimming {
                    trimmed,
                    pivot_hint,
                    universe,
                } => {
                    // Try to advance the trimming computation. If trimming is done, recompute
                    // the pivot hint (if needed) and move on to basin computation.

                    let trimmed = trimmed.try_compute()?;
                    if trimmed.is_empty() {
                        state.computing = None;
                        debug!("Candidate set trimmed to empty.");
                        return Err(Working);
                    }

                    debug!("Candidate set trimmed ({}).", log_set(trimmed));

                    let mut pivot_hint = trimmed.intersect(pivot_hint);
                    if pivot_hint.is_empty() {
                        // If trimming has removed all hint states, try to find
                        // additional hint states at the border of the trimmed set.

                        let removed = universe.minus(trimmed);
                        for var in context.graph.variables().rev() {
                            let var_post = context.graph.var_post(var, &removed).intersect(trimmed);
                            if !var_post.is_empty() {
                                pivot_hint = var_post;
                                debug!(
                                    "Updated pivot hint after trimming ({}).",
                                    log_set(&pivot_hint)
                                );
                                break;
                            }
                        }
                    }

                    state.computing = Some(IterationState::new_basin_state::<BWD>(
                        context,
                        universe.clone(),
                        &pivot_hint,
                    ));
                    Err(Working)
                }
                IterationState::ChainBasin {
                    universe,
                    pivot,
                    basin,
                } => {
                    // Try to advance the basin computation. If the basin is done, move on to
                    // SCC computation.
                    let basin = basin.try_compute()?;
                    state.computing = Some(IterationState::new_scc::<FWD>(
                        context,
                        universe.clone(),
                        pivot.clone(),
                        basin.clone(),
                    ));
                    Err(Working)
                }
                IterationState::ChainScc {
                    universe,
                    basin,
                    scc,
                } => {
                    // Try to advance the SCC computation. If the SCC is done, we can partition
                    // the remaining states, report the SCC and move on to other iterations.
                    let scc = scc.try_compute()?;
                    debug!("Extracted raw SCC ({})", log_set(scc));

                    // Enqueue the remaining states for further processing.
                    let remaining_basin = basin.minus(scc);
                    let remaining_rest = universe.minus(basin);

                    if !remaining_basin.is_empty() {
                        // Try to find *some* states that are direct predecessors of SCC inside
                        // the remaining basin.
                        let mut hint = context.graph.mk_empty_colored_vertices();
                        for var in context.graph.variables().rev() {
                            let var_pre = context
                                .graph
                                .var_pre_out(var, scc)
                                .intersect(&remaining_basin);
                            if !var_pre.is_empty() {
                                hint = var_pre;
                                break;
                            }
                        }

                        debug!(
                            "Pushed remaining BASIN ({}) with hint ({}).",
                            log_set(&remaining_basin),
                            log_set(&hint),
                        );

                        state.to_process.push(RemainingSet {
                            set: remaining_basin,
                            pivot_hint: hint,
                        });
                    }
                    if !remaining_rest.is_empty() {
                        // Try to find *some* states that are direct successors of SCC inside
                        // the remaining set.
                        let mut hint = context.graph.mk_empty_colored_vertices();
                        for var in context.graph.variables().rev() {
                            let var_pre = context
                                .graph
                                .var_post_out(var, scc)
                                .intersect(&remaining_rest);
                            if !var_pre.is_empty() {
                                hint = var_pre;
                                break;
                            }
                        }

                        debug!(
                            "Pushed remaining REST ({}) with hint ({}).",
                            log_set(&remaining_rest),
                            log_set(&hint),
                        );

                        state.to_process.push(RemainingSet {
                            set: remaining_rest,
                            pivot_hint: hint,
                        });
                    }

                    // Remove colors where the SCC is a singleton state:
                    let valid_colors = scc.minus(&scc.pick_vertex()).colors();
                    let scc = scc.intersect_colors(&valid_colors);
                    state.computing = None;
                    try_report_scc(scc)
                }
            }
        } else {
            // We are in-between iterations. We need to pick a new set for processing.
            // In theory, this choice does not matter because all sets need to be processed
            // eventually.

            if state.to_process.is_empty() {
                // If there is nothing to process, we are done.
                return Ok(None);
            }

            let todo = state
                .to_process
                .pop()
                .expect("Correctness violation: Nothing to process");

            info!(
                "Start processing ({}); {} sets remaining (BDD nodes={})",
                log_set(&todo.set),
                state.to_process.len(),
                state
                    .to_process
                    .iter()
                    .map(|it| it.set.symbolic_size())
                    .sum::<usize>()
            );

            assert!(state.computing.is_none());
            state.computing = Some(IterationState::new_trim_state::<BWD>(
                context,
                todo.set,
                todo.pivot_hint,
            ));
            Err(Working)
        }
    }
}
