use crate::attractor::AttractorConfig;
use crate::log_set;
use crate::reachability::{
    ReachabilityConfig, ReachabilityStep, SaturationPredecessors, SaturationSuccessors,
};
use biodivine_lib_param_bn::VariableId;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use computation_process::Incomplete::Suspended;
use computation_process::{Completable, ComputationStep};
use log::{debug, info};
use std::cmp::Reverse;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ItgrState {
    /// Current set of remaining states (the result of the computation).
    remaining_set: GraphColoredVertices,
    /// Reachability config reflecting the current `remaining_set`. Is used to ensure we
    /// never leave the remaining set when computing reachability.
    remaining_reachability: ReachabilityConfig,
    /// In the next iteration, the given set of states should be removed from `remaining_set`.
    to_discard: Option<GraphColoredVertices>,
    /// Remaining transition-guided reductions.
    reductions: Vec<(VariableId, Step)>,
}

pub struct ItgrStep;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct StepForward {
    forward: GraphColoredVertices,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct StepExtendedComponent {
    forward: GraphColoredVertices,
    extended_component: GraphColoredVertices,
    universe_forward: ReachabilityConfig,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct StepForwardBasin {
    forward: GraphColoredVertices,
    basin: GraphColoredVertices,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct StepBottomBasin {
    bottom: GraphColoredVertices,
    basin: GraphColoredVertices,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum Step {
    Forward(StepForward),
    Extended(Box<StepExtendedComponent>),
    ForwardBasin(StepForwardBasin),
    BottomBasin(StepBottomBasin),
}

impl Step {
    pub fn weight(&self) -> usize {
        match self {
            Step::Forward(x) => x.forward.symbolic_size(),
            Step::Extended(x) => x.extended_component.symbolic_size(),
            Step::ForwardBasin(x) => x.basin.symbolic_size(),
            Step::BottomBasin(x) => x.basin.symbolic_size(),
        }
    }

    pub fn restrict_to(&mut self, restrict_to: &GraphColoredVertices) {
        match self {
            Step::Forward(x) => {
                x.forward = x.forward.intersect(restrict_to);
            }
            Step::Extended(x) => {
                x.forward = x.forward.intersect(restrict_to);
                x.extended_component = x.extended_component.intersect(restrict_to);
                x.universe_forward.graph = x.universe_forward.graph.restrict(restrict_to);
            }
            Step::ForwardBasin(x) => {
                x.forward = x.forward.intersect(restrict_to);
                x.basin = x.basin.intersect(restrict_to);
            }
            Step::BottomBasin(x) => {
                x.bottom = x.bottom.intersect(restrict_to);
                x.basin = x.basin.intersect(restrict_to);
            }
        }
    }
}

impl ItgrState {
    pub fn new(graph: &SymbolicAsyncGraph, universe: &GraphColoredVertices) -> Self {
        ItgrState {
            remaining_set: universe.clone(),
            remaining_reachability: graph.restrict(universe).into(),
            to_discard: None,
            reductions: graph
                .variables()
                .map(|it| {
                    let step = Step::Forward(StepForward {
                        forward: graph.var_can_post(it, universe),
                    });
                    (it, step)
                })
                .collect(),
        }
    }

    /// Returns an iterator over variables that are still considered "active" (i.e., not eliminated)
    /// in the current state of reduction.
    ///
    /// The result of this method is typically used to restrict the variable set for subsequent
    /// algorithms (like [`XieBeerelAttractors`](crate::attractor::XieBeerelAttractors)).
    pub fn active_variables(&self) -> impl Iterator<Item = VariableId> {
        self.remaining_reachability.variables.iter().copied()
    }
}

/// A helper function used in debug asserts to verify algorithm invariants.
fn is_forward_closed(graph: &SymbolicAsyncGraph, set: &GraphColoredVertices) -> bool {
    for var in graph.variables() {
        let post = graph.var_can_post_out(var, set);
        if !post.is_empty() {
            return false;
        }
    }
    true
}

impl ComputationStep<AttractorConfig, ItgrState, GraphColoredVertices> for ItgrStep {
    fn step(context: &AttractorConfig, state: &mut ItgrState) -> Completable<GraphColoredVertices> {
        // First, if we have some states to remove, remove them from all remaining reductions:
        if let Some(to_discard) = state.to_discard.take() {
            state.remaining_set = state.remaining_set.minus(&to_discard);
            state.remaining_reachability.graph = state
                .remaining_reachability
                .graph
                .restrict(&state.remaining_set);

            // Note: This is not super impactful, but can save some overhead
            // for very large network.
            for var in state.remaining_reachability.variables.clone() {
                let can_post = state
                    .remaining_reachability
                    .graph
                    .var_can_post_within(var, &state.remaining_set);
                if can_post.is_empty() {
                    debug!("Variable {} eliminated.", var);
                    state.remaining_reachability.variables.remove(&var);
                }
            }

            debug_assert!(is_forward_closed(&context.graph, &state.remaining_set));

            info!(
                "Remaining set reduced ({}). Active tasks: {}",
                log_set(&state.remaining_set),
                state.reductions.len()
            );
            for (_var, x) in state.reductions.iter_mut() {
                x.restrict_to(&state.remaining_set);
            }
        }

        // Basin computation takes priority against everything else.
        let last_is_bottom_basin =
            matches!(state.reductions.last(), Some(&(_, Step::BottomBasin(_))));
        let last_is_forward_basin =
            matches!(state.reductions.last(), Some(&(_, Step::ForwardBasin(_))));
        if !last_is_bottom_basin && !last_is_forward_basin {
            // Sort reductions by symbolic size (should be a fairly inexpensive operation).
            state
                .reductions
                .sort_by_cached_key(|(_, it)| Reverse(it.weight()));
        }

        // Second, try to advance the last reduction:
        if let Some((var, reduction)) = state.reductions.last_mut() {
            let var = *var; // local copy to make the borrow checker happy
            match reduction {
                Step::Forward(x) => {
                    // Forward set just needs to be fully computed while staying confined
                    // to the remaining states.
                    let post =
                        SaturationSuccessors::step(&state.remaining_reachability, &x.forward)?;

                    if post.is_empty() {
                        // Forward reachability done.
                        let forward = x.forward.clone();
                        debug_assert!(is_forward_closed(&context.graph, &forward));
                        debug_assert!(forward.is_subset(&state.remaining_set));

                        state.reductions.pop();
                        info!(
                            "[{}] Forward set done. Spawning extended component computation ({}).",
                            var,
                            log_set(&forward)
                        );

                        let mut forward_config = state.remaining_reachability.clone();
                        forward_config.graph = forward_config.graph.restrict(&forward);
                        state.reductions.push((
                            var,
                            Step::Extended(Box::new(StepExtendedComponent {
                                universe_forward: forward_config,
                                extended_component: context
                                    .graph
                                    .var_can_post(var, &state.remaining_set),
                                forward: forward.clone(),
                            })),
                        ));

                        // If forward != remaining, there should also be a basin that we can remove.
                        let basin_candidates = state.remaining_set.minus(&forward);
                        if !basin_candidates.is_empty() {
                            info!(
                                "[{}] Spawning forward-basin reduction ({})",
                                var,
                                log_set(&basin_candidates)
                            );
                            state.reductions.push((
                                var,
                                Step::ForwardBasin(StepForwardBasin {
                                    forward: forward.clone(),
                                    basin: forward,
                                }),
                            ));
                        }

                        Err(Suspended)
                    } else {
                        x.forward = x.forward.union(&post);
                        debug!("[{}] Forward increased ({}).", var, log_set(&x.forward));
                        Err(Suspended)
                    }
                }
                Step::Extended(x) => {
                    // An extended component needs to compute the backward reachability.
                    let pre =
                        SaturationPredecessors::step(&x.universe_forward, &x.extended_component)?;
                    if pre.is_empty() {
                        // Backward reachability is done. If the bottom set is not empty,
                        // we can try to remove its basin.
                        info!(
                            "[{}] Extended component done ({})",
                            var,
                            log_set(&x.extended_component)
                        );
                        let bottom = x.forward.minus(&x.extended_component);
                        debug_assert!(x.extended_component.is_subset(&state.remaining_set));
                        debug_assert!(x.extended_component.is_subset(&x.forward));
                        debug_assert!(is_forward_closed(&context.graph, &x.forward));

                        state.reductions.pop();
                        if !bottom.is_empty() {
                            info!(
                                "[{}] Spawning bottom-basin reduction ({})",
                                var,
                                log_set(&bottom)
                            );
                            state.reductions.push((
                                var,
                                Step::BottomBasin(StepBottomBasin {
                                    bottom: bottom.clone(),
                                    basin: bottom,
                                }),
                            ));
                        }
                        Err(Suspended)
                    } else {
                        x.extended_component = x.extended_component.union(&pre);
                        debug!(
                            "[{}] Extended component increased ({}).",
                            var,
                            log_set(&x.extended_component)
                        );
                        Err(Suspended)
                    }
                }
                Step::ForwardBasin(x) => {
                    let pre =
                        SaturationPredecessors::step(&state.remaining_reachability, &x.basin)?;
                    if pre.is_empty() {
                        info!("[{}] Forward basin done ({})", var, log_set(&x.basin));
                        let to_discard = x.basin.minus(&x.forward);
                        if !to_discard.is_empty() {
                            info!(
                                "[{}] Discarding with forward basin ({})",
                                var,
                                log_set(&to_discard)
                            );
                            state.to_discard = Some(to_discard);
                        } else {
                            info!("[{}] Cannot discard anything using forward basin", var);
                        }
                        state.reductions.pop();
                        Err(Suspended)
                    } else {
                        x.basin = x.basin.union(&pre);
                        debug!("[{}] Forward basin increased ({}).", var, log_set(&x.basin));
                        Err(Suspended)
                    }
                }
                Step::BottomBasin(x) => {
                    let pre =
                        SaturationPredecessors::step(&state.remaining_reachability, &x.basin)?;
                    if pre.is_empty() {
                        info!(
                            "[{}] Bottom basin done ({}) with universe ({})",
                            var,
                            log_set(&x.basin),
                            log_set(&state.remaining_set)
                        );
                        let to_discard = x.basin.minus(&x.bottom);
                        if !to_discard.is_empty() {
                            info!(
                                "[{}] Discarding with bottom basin ({})",
                                var,
                                log_set(&to_discard)
                            );
                            state.to_discard = Some(to_discard);
                        } else {
                            info!("[{}] Cannot discard anything using bottom basin", var);
                        }
                        state.reductions.pop();
                        Err(Suspended)
                    } else {
                        x.basin = x.basin.union(&pre);
                        debug!("[{}] Bottom basin increased ({}).", var, log_set(&x.basin));
                        Err(Suspended)
                    }
                }
            }
        } else {
            // All reductions are done. We can return remaining states.
            Ok(state.remaining_set.clone())
        }
    }
}
