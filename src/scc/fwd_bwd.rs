use crate::log_set;
use crate::reachability::ReachabilityAlgorithm;
use crate::scc::{SccConfig, filter_scc};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use computation_process::Incomplete::Suspended;
use computation_process::{Completable, DynComputable, GeneratorStep};
use log::{debug, info};
use std::marker::PhantomData;

/// Internal state for the forward-backward SCC algorithm.
///
/// This struct tracks the current computation phase and pending work items.
pub struct FwdBwdState {
    computing: Step,
    to_process: Vec<GraphColoredVertices>,
}

/// Step implementation for the forward-backward SCC algorithm.
///
/// This type is parameterized by forward and backward reachability algorithms
/// and implements the [`GeneratorStep`] trait for SCC enumeration.
pub struct FwdBwdStep<FWD: ReachabilityAlgorithm, BWD: ReachabilityAlgorithm> {
    _phantom: PhantomData<(FWD, BWD)>,
}

impl From<&SymbolicAsyncGraph> for FwdBwdState {
    fn from(value: &SymbolicAsyncGraph) -> Self {
        FwdBwdState::from(value.mk_unit_colored_vertices())
    }
}

impl From<GraphColoredVertices> for FwdBwdState {
    fn from(value: GraphColoredVertices) -> Self {
        FwdBwdState {
            computing: Step::Idle,
            to_process: vec![value],
        }
    }
}

impl From<&GraphColoredVertices> for FwdBwdState {
    fn from(value: &GraphColoredVertices) -> Self {
        FwdBwdState::from(value.clone())
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
            Step::Idle => {
                // Pick a new state for processing.

                let Some(todo) = state.to_process.pop() else {
                    // If there is nothing to process, we are done.
                    return Ok(None);
                };

                let Some(todo) = context.apply_long_lived_filter(&todo) else {
                    // The set is not long-lived, we can ignore it.
                    debug!("Candidate set empty after long-lived filtering.");
                    return Err(Suspended);
                };

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

                state.computing = Step::Trimming(Step1::new(context, todo));
                Err(Suspended)
            }
            Step::Trimming(step) => {
                let Some(trimmed) = step.try_advance::<BWD>(context)? else {
                    // If the set is empty after trimming/filtering, reset the state and stop.
                    state.computing = Step::Idle;
                    return Err(Suspended);
                };

                state.computing = Step::Backward(trimmed);
                Err(Suspended)
            }
            Step::Backward(step) => {
                state.computing = Step::Forward(step.try_advance::<FWD>(context)?);
                Err(Suspended)
            }
            Step::Forward(step) => {
                let result = step.try_advance(context)?;
                let scc = result.scc;
                let forward = result.forward;
                let backward = result.backward;
                let universe = result.universe;

                // Enqueue the remaining states for further processing.
                let remaining_backward = backward.minus(&forward);
                let remaining_forward = forward.minus(&backward);
                let remaining_rest = universe.minus(&backward).minus(&forward);

                debug!(
                    "Adding remaining FWD ({}), BWD ({}), and REST ({}) sets.",
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

                state.computing = Step::Idle;
                if let Some(scc) = scc {
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
    Backward(Step2),
    Forward(Step3),
}

struct Step1 {
    universe: DynComputable<GraphColoredVertices>,
}

struct Step2 {
    pivot: GraphColoredVertices,
    universe: GraphColoredVertices,
    backward: DynComputable<GraphColoredVertices>,
}

struct Step3 {
    universe: GraphColoredVertices,
    backward: GraphColoredVertices,
    forward: DynComputable<GraphColoredVertices>,
}

struct IterationResult {
    universe: GraphColoredVertices,
    forward: GraphColoredVertices,
    backward: GraphColoredVertices,
    scc: Option<GraphColoredVertices>,
}

impl Step1 {
    pub fn new(context: &SccConfig, set: GraphColoredVertices) -> Step1 {
        Step1 {
            universe: context.should_trim.build_computation(&context.graph, set),
        }
    }

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

        let graph = context.graph.restrict(&universe);
        let pivot = universe.pick_vertex();
        Ok(Some(Step2 {
            backward: BWD::configure(&graph, pivot.clone()).dyn_computable(),
            universe,
            pivot,
        }))
    }
}

impl Step2 {
    pub fn try_advance<FWD: ReachabilityAlgorithm>(
        &mut self,
        context: &SccConfig,
    ) -> Completable<Step3> {
        let backward = self.backward.try_compute()?;
        let graph = context.graph.restrict(&self.universe);

        let mut result = Step3 {
            forward: FWD::configure(&graph, self.pivot.clone()).dyn_computable(),
            universe: context.graph.mk_empty_colored_vertices(),
            backward,
        };

        std::mem::swap(&mut result.universe, &mut self.universe);

        Ok(result)
    }
}

impl Step3 {
    pub fn try_advance(&mut self, context: &SccConfig) -> Completable<IterationResult> {
        let forward = self.forward.try_compute()?;
        let scc = forward.intersect(&self.backward);
        debug!("Extracted raw SCC ({})", log_set(&scc));

        let mut result = IterationResult {
            universe: context.graph.mk_empty_colored_vertices(),
            backward: context.graph.mk_empty_colored_vertices(),
            scc: filter_scc(context, scc),
            forward,
        };

        std::mem::swap(&mut result.universe, &mut self.universe);
        std::mem::swap(&mut result.backward, &mut self.backward);

        Ok(result)
    }
}
