use crate::log_set;
use crate::reachability::ReachabilityAlgorithm;
use crate::scc::{SccConfig, filter_scc};
use crate::trimming::TrimComputation;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use computation_process::Incomplete::Suspended;
use computation_process::{Completable, Computable, GeneratorStep};
use log::{debug, info};
use std::marker::PhantomData;

/// Internal state for the forward-backward SCC algorithm.
///
/// This struct tracks the current computation phase and pending work items.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FwdBwdState<FWD: ReachabilityAlgorithm, BWD: ReachabilityAlgorithm> {
    computing: Step<FWD, BWD>,
    to_process: Vec<GraphColoredVertices>,
}

/// Step implementation for the forward-backward SCC algorithm.
///
/// This type is parameterized by forward and backward reachability algorithms
/// and implements the [`GeneratorStep`] trait for SCC enumeration.
pub struct FwdBwdStep<FWD: ReachabilityAlgorithm, BWD: ReachabilityAlgorithm> {
    _phantom: PhantomData<(FWD, BWD)>,
}

impl<FWD: ReachabilityAlgorithm, BWD: ReachabilityAlgorithm> From<&SymbolicAsyncGraph>
    for FwdBwdState<FWD, BWD>
{
    fn from(value: &SymbolicAsyncGraph) -> Self {
        FwdBwdState::from(value.mk_unit_colored_vertices())
    }
}

impl<FWD: ReachabilityAlgorithm, BWD: ReachabilityAlgorithm> From<GraphColoredVertices>
    for FwdBwdState<FWD, BWD>
{
    fn from(value: GraphColoredVertices) -> Self {
        FwdBwdState {
            computing: Step::Idle,
            to_process: vec![value],
        }
    }
}

impl<FWD: ReachabilityAlgorithm, BWD: ReachabilityAlgorithm> From<&GraphColoredVertices>
    for FwdBwdState<FWD, BWD>
{
    fn from(value: &GraphColoredVertices) -> Self {
        FwdBwdState::from(value.clone())
    }
}

impl<FWD: ReachabilityAlgorithm, BWD: ReachabilityAlgorithm>
    GeneratorStep<SccConfig, FwdBwdState<FWD, BWD>, GraphColoredVertices> for FwdBwdStep<FWD, BWD>
{
    fn step(
        context: &SccConfig,
        state: &mut FwdBwdState<FWD, BWD>,
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

                state.computing = Step::Trimming(Box::new(Step1::new(context, todo)));
                Err(Suspended)
            }
            Step::Trimming(step) => {
                let Some(trimmed) = step.try_advance::<BWD>(context)? else {
                    // If the set is empty after trimming/filtering, reset the state and stop.
                    state.computing = Step::Idle;
                    return Err(Suspended);
                };

                state.computing = Step::Backward(Box::new(trimmed));
                Err(Suspended)
            }
            Step::Backward(step) => {
                state.computing = Step::Forward(Box::new(step.try_advance::<FWD>(context)?));
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

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum Step<FWD: ReachabilityAlgorithm, BWD: ReachabilityAlgorithm> {
    Idle,
    Trimming(Box<Step1>),
    Backward(Box<Step2<BWD>>),
    Forward(Box<Step3<FWD>>),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Step1 {
    universe: TrimComputation,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Step2<BWD: ReachabilityAlgorithm> {
    pivot: GraphColoredVertices,
    universe: GraphColoredVertices,
    backward: BWD,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Step3<FWD> {
    universe: GraphColoredVertices,
    backward: GraphColoredVertices,
    forward: FWD,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    ) -> Completable<Option<Step2<BWD>>> {
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
            backward: BWD::configure(&graph, pivot.clone()),
            universe,
            pivot,
        }))
    }
}

impl<BWD: ReachabilityAlgorithm> Step2<BWD> {
    pub fn try_advance<FWD: ReachabilityAlgorithm>(
        &mut self,
        context: &SccConfig,
    ) -> Completable<Step3<FWD>> {
        let backward = self.backward.try_compute()?;
        let graph = context.graph.restrict(&self.universe);

        let mut result = Step3 {
            forward: FWD::configure(&graph, self.pivot.clone()),
            universe: context.graph.mk_empty_colored_vertices(),
            backward,
        };

        std::mem::swap(&mut result.universe, &mut self.universe);

        Ok(result)
    }
}

impl<FWD: ReachabilityAlgorithm> Step3<FWD> {
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
