use crate::attractor::AttractorConfig;
use crate::log_set;
use crate::reachability::{
    BackwardReachability, ReachabilityConfig, ReachabilityStep, SaturationSuccessors,
};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use computation_process::Incomplete::Suspended;
use computation_process::{Completable, Computable, GeneratorStep, Stateful};
use log::{debug, info};

/// Internal state of the Xie-Beerel attractor algorithm.
pub struct XieBeerelState {
    computing: Step,
    remaining: GraphColoredVertices,
    //pivot_hint: Option<GraphColoredVertices>,
}

/// Step implementation for the Xie-Beerel attractor algorithm.
///
/// This type is parameterized by forward and backward reachability algorithms
/// and implements the [`GeneratorStep`] trait for SCC enumeration.
pub struct XieBeerelStep;

enum Step {
    Idle,
    Basin(Step1),
    Attractor(Step2),
}

struct Step1 {
    pivot: GraphColoredVertices,
    basin: BackwardReachability,
}

struct Step2 {
    basin: GraphColoredVertices,
    attractor_config: ReachabilityConfig,
    attractor: GraphColoredVertices,
}

impl GeneratorStep<AttractorConfig, XieBeerelState, GraphColoredVertices> for XieBeerelStep {
    fn step(
        context: &AttractorConfig,
        state: &mut XieBeerelState,
    ) -> Completable<Option<GraphColoredVertices>> {
        match &mut state.computing {
            Step::Idle => {
                // Find a new pivot and start basin computation:

                if state.remaining.is_empty() {
                    // If there is nothing to process, we are done.
                    return Ok(None);
                }

                // // Try to use a pivot hint (if any) to select the next pivot:
                // let pivot_hint = if let Some(hint) = state.pivot_hint.take() {
                //     hint.intersect(&state.remaining)
                // } else {
                //     context.graph.mk_empty_colored_vertices()
                // };
                //
                // let pivot = if pivot_hint.is_empty() {
                //     state.remaining.pick_vertex()
                // } else {
                //     pivot_hint.pick_vertex()
                // };

                info!(
                    "Start next iteration. Remaining ({}).",
                    log_set(&state.remaining),
                );

                let pivot = state.remaining.pick_vertex();

                let bwd_config =
                    ReachabilityConfig::from(context).restrict_state_space(&state.remaining);
                state.computing = Step::Basin(Step1 {
                    basin: BackwardReachability::configure(bwd_config, pivot.clone()),
                    pivot,
                });
                Err(Suspended)
            }
            Step::Basin(step) => {
                // Basin is just computed fully without any special treatment:
                let basin = step.basin.try_compute()?;
                state.computing = Step::Attractor(Step2 {
                    basin,
                    attractor: step.pivot.clone(),
                    attractor_config: context.into(),
                });
                Err(Suspended)
            }
            Step::Attractor(step) => {
                let successors =
                    SaturationSuccessors::step(&step.attractor_config, &step.attractor)?;
                if successors.is_subset(&step.attractor) {
                    info!(
                        "Attractor ({}) and basin ({}) iteration done.",
                        log_set(&step.attractor),
                        log_set(&step.basin),
                    );

                    // Attractor computation is done! Remove the basin and report the attractor.
                    let attractor = step.attractor.clone();
                    state.remaining = state.remaining.minus(&step.basin);
                    state.computing = Step::Idle;
                    if attractor.is_empty() {
                        Err(Suspended)
                    } else {
                        Ok(Some(attractor))
                    }
                } else {
                    step.attractor = step.attractor.union(&successors);

                    // Check if some successor escaped the basin. If yes, we want to completely
                    // remove all its colors, because they cannot produce an attractor.
                    let escaped = successors.minus(&step.basin).colors();
                    if !escaped.is_empty() {
                        debug!(
                            "Removing {} colors that escape attractor basin.",
                            escaped.exact_cardinality()
                        );
                        step.attractor = step.attractor.minus_colors(&escaped);
                    }

                    Err(Suspended)
                }
            }
        }
    }
}

impl From<&SymbolicAsyncGraph> for XieBeerelState {
    fn from(value: &SymbolicAsyncGraph) -> Self {
        XieBeerelState::from(value.mk_unit_colored_vertices())
    }
}

impl From<&GraphColoredVertices> for XieBeerelState {
    fn from(value: &GraphColoredVertices) -> Self {
        XieBeerelState::from(value.clone())
    }
}

impl From<GraphColoredVertices> for XieBeerelState {
    fn from(value: GraphColoredVertices) -> Self {
        XieBeerelState {
            computing: Step::Idle,
            remaining: value,
            //pivot_hint: None,
        }
    }
}
