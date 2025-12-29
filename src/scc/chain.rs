use crate::log_set;
use crate::reachability::ReachabilityAlgorithm;
use crate::scc::{SccConfig, filter_scc};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use computation_process::Incomplete::Suspended;
use computation_process::{Completable, DynComputable, GeneratorStep};
use log::{debug, info};
use std::marker::PhantomData;

pub struct ChainState {
    computing: Step,
    to_process: Vec<Step0>,
}

impl From<&SymbolicAsyncGraph> for ChainState {
    fn from(value: &SymbolicAsyncGraph) -> Self {
        ChainState {
            computing: Step::Idle,
            to_process: vec![Step0 {
                full_universe: value.mk_unit_colored_vertices(),
                pivot_hint: value.mk_empty_colored_vertices(),
            }],
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
        match &mut state.computing {
            Step::Idle => {
                // We are in-between iterations. We need to pick a new set for processing.
                // Pick a new state for processing.

                let Some(mut todo) = state.to_process.pop() else {
                    // If there is nothing to process, we are done.
                    return Ok(None);
                };

                let Some(full_universe) = context.apply_long_lived_filter(&todo.full_universe)
                else {
                    // The set is not long-lived, we can ignore it.
                    debug!("Candidate set empty after long-lived filtering.");
                    return Err(Suspended);
                };

                todo.full_universe = full_universe;

                info!(
                    "Start processing ({}); {} sets remaining (BDD nodes={})",
                    log_set(&todo.full_universe),
                    state.to_process.len(),
                    state
                        .to_process
                        .iter()
                        .map(|it| it.full_universe.symbolic_size())
                        .sum::<usize>()
                );

                state.computing = Step::Trimming(todo.advance(context));
                Err(Suspended)
            }
            Step::Trimming(step) => {
                let Some(trimmed) = step.try_advance::<BWD>(context)? else {
                    // If the set is empty after trimming/filtering, reset the state and stop.
                    state.computing = Step::Idle;
                    return Err(Suspended);
                };

                state.computing = Step::Basin(trimmed);
                Err(Suspended)
            }
            Step::Basin(step) => {
                state.computing = Step::Scc(step.try_advance::<FWD>(context)?);
                Err(Suspended)
            }
            Step::Scc(step) => {
                let result = step.try_advance(context)?;
                let raw_scc = result.raw_scc;
                let basin = result.basin;
                let universe = result.universe;

                debug!("Extracted raw SCC ({})", log_set(&raw_scc));

                // Enqueue the remaining states for further processing.
                let remaining_basin = basin.minus(&raw_scc);
                let remaining_rest = universe.minus(&basin);

                if !remaining_basin.is_empty() {
                    // Try to find *some* states that are direct predecessors of SCC inside
                    // the remaining basin.
                    let mut hint = context.graph.mk_empty_colored_vertices();
                    for var in context.graph.variables().rev() {
                        let var_pre = context
                            .graph
                            .var_pre_out(var, &raw_scc)
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

                    state.to_process.push(Step0 {
                        full_universe: remaining_basin,
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
                            .var_post_out(var, &raw_scc)
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

                    state.to_process.push(Step0 {
                        full_universe: remaining_rest,
                        pivot_hint: hint,
                    });
                }

                // Remove colors where the SCC is a singleton state:
                state.computing = Step::Idle;
                if let Some(scc) = filter_scc(context, raw_scc) {
                    Ok(Some(scc))
                } else {
                    Err(Suspended)
                }
            }
        }
    }
}

enum Step {
    Idle,
    Trimming(Step1),
    Basin(Step2),
    Scc(Step3),
}

struct Step0 {
    full_universe: GraphColoredVertices,
    pivot_hint: GraphColoredVertices,
}

struct Step1 {
    full_universe: GraphColoredVertices,
    pivot_hint: GraphColoredVertices,
    universe: DynComputable<GraphColoredVertices>,
}

struct Step2 {
    universe: GraphColoredVertices,
    pivot: GraphColoredVertices,
    basin: DynComputable<GraphColoredVertices>,
}

struct Step3 {
    universe: GraphColoredVertices,
    basin: GraphColoredVertices,
    scc: DynComputable<GraphColoredVertices>,
}

struct IterationResult {
    universe: GraphColoredVertices,
    basin: GraphColoredVertices,
    raw_scc: GraphColoredVertices,
}

impl Step0 {
    pub fn advance(&mut self, context: &SccConfig) -> Step1 {
        let mut result = Step1 {
            universe: context
                .should_trim
                .build_computation(&context.graph, self.full_universe.clone()),
            full_universe: context.graph.mk_empty_colored_vertices(),
            pivot_hint: context.graph.mk_empty_colored_vertices(),
        };

        std::mem::swap(&mut result.full_universe, &mut self.full_universe);
        std::mem::swap(&mut result.pivot_hint, &mut self.pivot_hint);

        result
    }
}

impl Step1 {
    pub fn try_advance<BWD: ReachabilityAlgorithm>(
        &mut self,
        context: &SccConfig,
    ) -> Completable<Option<Step2>> {
        let universe = self.universe.try_compute()?;

        if universe.is_empty() {
            debug!("Candidate set empty after trimming.");
            return Ok(None);
        }

        let Some(universe) = context.apply_long_lived_filter(&universe) else {
            debug!("Candidate set empty after trimming and long-term filtering.");
            return Ok(None);
        };

        debug!("Candidate set trimmed ({}).", log_set(&universe));

        let mut pivot_hint = universe.intersect(&self.pivot_hint);
        if pivot_hint.is_empty() {
            // If trimming has removed all hint states, try to find
            // additional hint states at the border of the trimmed set.

            let removed = self.full_universe.minus(&universe);
            for var in context.graph.variables().rev() {
                let var_post = context.graph.var_post(var, &removed).intersect(&universe);
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

        let pivot = if pivot_hint.is_empty() {
            universe.pick_vertex()
        } else {
            pivot_hint.pick_vertex()
        };

        let graph = context.graph.restrict(&universe);

        let result = Step2 {
            basin: BWD::configure(&graph, pivot.clone()).dyn_computable(),
            universe,
            pivot,
        };

        Ok(Some(result))
    }
}

impl Step2 {
    pub fn try_advance<FWD: ReachabilityAlgorithm>(
        &mut self,
        context: &SccConfig,
    ) -> Completable<Step3> {
        let basin = self.basin.try_compute()?;
        let basin_graph = context.graph.restrict(&basin);
        let mut result = Step3 {
            scc: FWD::configure(&basin_graph, self.pivot.clone()).dyn_computable(),
            universe: basin_graph.mk_empty_colored_vertices(),
            basin,
        };

        std::mem::swap(&mut result.universe, &mut self.universe);

        Ok(result)
    }
}

impl Step3 {
    pub fn try_advance(&mut self, context: &SccConfig) -> Completable<IterationResult> {
        let raw_scc = self.scc.try_compute()?;

        let mut result = IterationResult {
            universe: context.graph.mk_empty_colored_vertices(),
            basin: context.graph.mk_empty_colored_vertices(),
            raw_scc,
        };

        std::mem::swap(&mut result.universe, &mut self.universe);
        std::mem::swap(&mut result.basin, &mut self.basin);

        Ok(result)
    }
}
