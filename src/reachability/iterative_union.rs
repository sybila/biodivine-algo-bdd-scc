use crate::reachability::reachability_state::ReachabilityState;
use crate::reachability::{ReachabilityConfig, ReachabilityStep};
use crate::{log_set, simple_type_name};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use cancel_this::Cancelled;
use computation_process::Incomplete::Suspended;
use computation_process::{Completable, ComputationStep};
use log::debug;
use std::marker::PhantomData;

/// A helper implementation of [`ComputationStep`] that repeatedly calls a [`ReachabilityStep`]
/// function, collecting the results into the current `state`.
pub struct IterativeUnion<S: ReachabilityStep>(PhantomData<S>);

impl<S: ReachabilityStep>
    ComputationStep<ReachabilityConfig, ReachabilityState, GraphColoredVertices>
    for IterativeUnion<S>
{
    fn step(
        context: &ReachabilityConfig,
        state: &mut ReachabilityState,
    ) -> Completable<GraphColoredVertices> {
        if state.iteration >= context.max_iterations {
            debug!(
                "[iteration:{}] Union<{}> canceled (exceeded iteration count).",
                state.iteration,
                simple_type_name::<S>()
            );

            return Err(Cancelled::new("ReachabilityConfig::max_iterations").into());
        } else {
            state.iteration += 1;
        }

        let to_union = S::step(context, &state.set)?;
        if to_union.is_empty() {
            debug!(
                "[iteration:{}] Union<{}> finished with ({}).",
                state.iteration,
                simple_type_name::<S>(),
                log_set(&state.set)
            );

            Ok(state.set.clone())
        } else {
            state.set = state.set.union(&to_union);

            if state.set.symbolic_size() > context.max_symbolic_size {
                debug!(
                    "[iteration:{}] Union<{}> canceled (exceeded symbolic size).",
                    state.iteration,
                    simple_type_name::<S>()
                );

                return Err(Cancelled::new("ReachabilityConfig::max_symbolic_size").into());
            }

            debug!(
                "[iteration:{}] Union<{}> increased to ({}).",
                state.iteration,
                simple_type_name::<S>(),
                log_set(&state.set)
            );

            Err(Suspended)
        }
    }
}
