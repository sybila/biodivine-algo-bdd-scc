use crate::reachability::{ReachabilityConfig, ReachabilityState, ReachabilityStep};
use crate::{log_set, simple_type_name};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use cancel_this::Cancelled;
use computation_process::Incomplete::Suspended;
use computation_process::{Completable, ComputationStep};
use log::debug;
use std::marker::PhantomData;

/// A helper implementation of [`ComputationStep`] that repeatedly calls a [`ReachabilityStep`]
/// function, *removing* the results from the current `state`.
pub struct IterativeSubtraction<S: ReachabilityStep>(PhantomData<S>);

impl<S: ReachabilityStep>
    ComputationStep<ReachabilityConfig, ReachabilityState, GraphColoredVertices>
    for IterativeSubtraction<S>
{
    fn step(
        context: &ReachabilityConfig,
        state: &mut ReachabilityState,
    ) -> Completable<GraphColoredVertices> {
        if state.iteration >= context.max_iterations {
            debug!(
                "[iteration:{}] Subtraction<{}> canceled (exceeded iteration count).",
                state.iteration,
                simple_type_name::<S>()
            );

            return Err(Cancelled::new("ReachabilityConfig::max_iterations").into());
        } else {
            state.iteration += 1;
        }

        let to_remove = S::step(context, &state.set)?;
        if to_remove.is_empty() {
            debug!(
                "[iteration:{}] Subtraction<{}> finished with ({}).",
                state.iteration,
                simple_type_name::<S>(),
                log_set(&state.set)
            );

            Ok(state.set.clone())
        } else {
            state.set = state.set.minus(&to_remove);

            if state.set.symbolic_size() > context.max_symbolic_size {
                debug!(
                    "[iteration:{}] Subtraction<{}> canceled (exceeded symbolic size).",
                    state.iteration,
                    simple_type_name::<S>()
                );

                return Err(Cancelled::new("ReachabilityConfig::max_symbolic_size").into());
            }

            debug!(
                "[iteration:{}] Subtraction<{}> decreased to ({}).",
                state.iteration,
                simple_type_name::<S>(),
                log_set(&state.set)
            );

            Err(Suspended)
        }
    }
}
