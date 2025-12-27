use crate::algorithm::log_set;
use crate::algorithm::reachability::ReachabilityAlgorithm;
use crate::algorithm::scc::{SccConfig, slice, try_report_scc};
use crate::algorithm::trimming::TrimSetting;
use crate::algorithm_trait::Incomplete::Working;
use crate::algorithm_trait::{Completable, DynComputable, GeneratorStep};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use log::{debug, info};
use std::marker::PhantomData;

pub struct FwdBwdState {
    computing: IterationState,
    to_process: Vec<GraphColoredVertices>,
}

enum IterationState {
    Idle,
    Trimming(DynComputable<GraphColoredVertices>),
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
    fn new_trim<FWD: ReachabilityAlgorithm, BWD: ReachabilityAlgorithm>(
        context: &SccConfig,
        value: GraphColoredVertices,
    ) -> Self {
        if context.should_trim == TrimSetting::None {
            Self::new_fwd_bwd::<FWD, BWD>(context, value)
        } else {
            Self::Trimming(context.should_trim.build_computation(&context.graph, value))
        }
    }

    fn new_fwd_bwd<FWD: ReachabilityAlgorithm, BWD: ReachabilityAlgorithm>(
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
        let sets = slice(value, value.mk_unit_colored_vertices());
        println!(
            "Total slices: {}; Largest: {}",
            sets.len(),
            sets.iter().map(|it| it.exact_cardinality()).max().unwrap()
        );
        FwdBwdState {
            computing: IterationState::Idle,
            to_process: sets,
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
        match &mut state.computing {
            IterationState::Trimming(trim) => {
                let trimmed = trim.try_compute()?;

                if trimmed.is_empty() {
                    state.computing = IterationState::Idle;
                    debug!("Candidate set trimmed to empty.");
                    return Err(Working);
                }

                debug!("Candidate set trimmed ({}).", log_set(trimmed));

                state.computing = IterationState::new_fwd_bwd::<FWD, BWD>(context, trimmed.clone());
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
                state.computing = IterationState::Idle;
                try_report_scc(scc)
            }
            IterationState::Idle => {
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
                    log_set(&todo),
                    state.to_process.len(),
                    state
                        .to_process
                        .iter()
                        .map(|it| it.symbolic_size())
                        .sum::<usize>()
                );

                state.computing = IterationState::new_trim::<FWD, BWD>(context, todo);
                Err(Working)
            }
        }
    }
}
