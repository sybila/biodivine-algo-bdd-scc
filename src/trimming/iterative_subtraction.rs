use crate::reachability::{ReachabilityConfig, ReachabilityStep};
use crate::{log_set, simple_type_name};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use computation_process::Incomplete::Suspended;
use computation_process::{Completable, ComputationStep};
use log::{debug, info};
use std::marker::PhantomData;

/// A helper implementation of [`ComputationStep`] that repeatedly calls a [`ReachabilityStep`]
/// function, *removing* the results from the current `state`.
pub struct IterativeSubtraction<S: ReachabilityStep>(PhantomData<S>);

impl<S: ReachabilityStep>
    ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices>
    for IterativeSubtraction<S>
{
    fn step(
        context: &ReachabilityConfig,
        state: &mut GraphColoredVertices,
    ) -> Completable<GraphColoredVertices> {
        let to_remove = S::step(context, state)?;
        if to_remove.is_empty() {
            info!(
                "Subtraction[{}] finished ({}).",
                simple_type_name::<S>(),
                log_set(state)
            );
            Ok(state.clone())
        } else {
            *state = state.minus(&to_remove);
            debug!(
                "Subtraction[{}] decreased ({}).",
                simple_type_name::<S>(),
                log_set(state)
            );
            Err(Suspended)
        }
    }
}
