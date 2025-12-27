use crate::algorithm::log_set;
use crate::algorithm::reachability::ReachabilityAlgorithm;
use crate::algorithm::scc::SccConfig;
use crate::algorithm::scc::scc_config::TrimSetting;
use crate::algorithm::scc::scc_config::TrimSetting::Both;
use crate::algorithm::trimming::{TrimSinks, TrimSources};
use crate::algorithm_trait::Incomplete::Working;
use crate::algorithm_trait::{Completable, DynComputable, GeneratorStep};
use TrimSetting::{Sinks, Sources};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use log::{debug, info};
use num_bigint::BigUint;
use std::marker::PhantomData;

pub struct FwdBwdState {
    computing: Option<IterationState>,
    to_process: Vec<GraphColoredVertices>,
}

enum IterationState {
    TrimSources(TrimSources),
    TrimSinks(TrimSinks),
    FwdBwd {
        universe: GraphColoredVertices,
        forward: DynComputable<GraphColoredVertices>,
        backward: DynComputable<GraphColoredVertices>,
    },
}

pub struct FwdBwdStep<FWD: ReachabilityAlgorithm, BWD: ReachabilityAlgorithm> {
    _phantom: PhantomData<(FWD, BWD)>,
}

impl IterationState {
    fn new<FWD: ReachabilityAlgorithm, BWD: ReachabilityAlgorithm>(
        context: &SccConfig,
        value: GraphColoredVertices,
    ) -> Self {
        if context.should_trim == Both || context.should_trim == Sources {
            Self::TrimSources(TrimSources::configure(&context.graph, value))
        } else {
            Self::new_trimmed_sources::<FWD, BWD>(context, value)
        }
    }

    fn new_trimmed_sources<FWD: ReachabilityAlgorithm, BWD: ReachabilityAlgorithm>(
        context: &SccConfig,
        value: GraphColoredVertices,
    ) -> Self {
        if context.should_trim == Both || context.should_trim == Sinks {
            Self::TrimSinks(TrimSinks::configure(&context.graph, value))
        } else {
            Self::new_trimmed::<FWD, BWD>(context, value)
        }
    }

    fn new_trimmed<FWD: ReachabilityAlgorithm, BWD: ReachabilityAlgorithm>(
        context: &SccConfig,
        value: GraphColoredVertices,
    ) -> Self {
        let graph = context.graph.restrict(&value);
        let pivot = value.pick_vertex();
        Self::FwdBwd {
            forward: FWD::configure_dyn(graph.clone(), pivot.clone()),
            backward: BWD::configure_dyn(graph.clone(), pivot.clone()),
            universe: value,
        }
    }
}

impl From<&SymbolicAsyncGraph> for FwdBwdState {
    fn from(value: &SymbolicAsyncGraph) -> Self {
        FwdBwdState {
            computing: None,
            to_process: vec![value.mk_unit_colored_vertices()],
        }
    }
}

impl<FWD: ReachabilityAlgorithm, BWD: ReachabilityAlgorithm>
    GeneratorStep<SccConfig, FwdBwdState, GraphColoredVertices> for FwdBwdStep<FWD, BWD>
{
    fn step(
        context: &SccConfig,
        state: &mut FwdBwdState,
    ) -> Completable<Option<GraphColoredVertices>> {
        if let Some(iteration) = state.computing.as_mut() {
            match iteration {
                IterationState::TrimSources(trim) => {
                    let trimmed = trim.try_compute()?;
                    state.computing = Some(IterationState::new_trimmed_sources::<FWD, BWD>(
                        context,
                        trimmed.clone(),
                    ));
                    Err(Working)
                }
                IterationState::TrimSinks(trim) => {
                    let trimmed = trim.try_compute()?;
                    state.computing = Some(IterationState::new_trimmed::<FWD, BWD>(
                        context,
                        trimmed.clone(),
                    ));
                    Err(Working)
                }
                IterationState::FwdBwd {
                    forward,
                    backward,
                    universe,
                } => {
                    // If we are processing a specific component right now, continue the iteration.
                    let backward = backward.try_compute()?;
                    let forward = forward.try_compute()?;
                    let scc = backward.intersect(forward);
                    debug!("Extracted raw SCC ({})", log_set(&scc));

                    // Enqueue the remaining states for further processing.
                    let remaining_backward = backward.minus(forward);
                    let remaining_forward = forward.minus(backward);
                    let remaining_rest = universe.minus(backward).minus(forward);

                    debug!(
                        "Pushed remaining FWD ({}), BWD ({}), and REST ({}) sets.",
                        log_set(&remaining_forward),
                        log_set(&remaining_backward),
                        log_set(&remaining_rest),
                    );

                    if !remaining_backward.is_empty() {
                        state.to_process.push(remaining_backward);
                    }
                    if !remaining_forward.is_empty() {
                        state.to_process.push(remaining_forward);
                    }
                    if !remaining_rest.is_empty() {
                        state.to_process.push(remaining_rest);
                    }

                    // Remove colors where the SCC is a singleton state:
                    let valid_colors = scc.minus(&scc.pick_vertex()).colors();
                    let scc = scc.intersect_colors(&valid_colors);
                    state.computing = None;
                    if scc.is_empty() {
                        // Iteration is done, but we have not found a new non-trivial SCC.
                        debug!("The SCC is trivial.");
                        Err(Working)
                    } else {
                        // Iteration is done, and we have a new non-trivial SCC.
                        info!("Returning non-trivial SCC ({}).", log_set(&scc));
                        Ok(Some(scc))
                    }
                }
            }
        } else {
            // We are in-between iterations. We need to pick a new set for processing.
            // In theory, this choice does not matter because all sets need to be processed
            // eventually. However, since the algorithm can be also terminated early, it
            // is preferred to return as many SCCs as quickly as possible. As such, we always
            // pick the smallest set (in terms of BDD size).

            if state.to_process.is_empty() {
                // If there is nothing to process, we are done.
                return Ok(None);
            }

            let total_used = state
                .to_process
                .iter()
                .map(|it| it.symbolic_size())
                .sum::<usize>();
            let total_elements = state
                .to_process
                .iter()
                .map(|it| it.exact_cardinality())
                .sum::<BigUint>();
            state.to_process.sort_by_cached_key(|it| it.symbolic_size());
            state.to_process.reverse();

            info!(
                "{} sets remaining (elements={}; BDD nodes={})",
                state.to_process.len(),
                total_elements,
                total_used
            );

            let todo = state
                .to_process
                .pop()
                .expect("Correctness violation: Nothing to process");

            assert!(state.computing.is_none());
            state.computing = Some(IterationState::new::<FWD, BWD>(context, todo));
            Err(Working)
        }
    }
}
