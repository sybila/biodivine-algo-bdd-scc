use crate::algorithm::reachability::{ReachabilityConfig, ReachabilityStep};
use crate::algorithm::{log_set, simple_type_name};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use computation_process::Incomplete::Suspended;
use computation_process::{Completable, ComputationStep};
use log::{debug, info};
use std::marker::PhantomData;

/// A helper implementation of [`ComputationStep`] that repeatedly calls a [`ReachabilityStep`]
/// function, collecting the results into the current `state`.
pub struct IterativeUnion<S: ReachabilityStep>(PhantomData<S>);

impl<S: ReachabilityStep>
    ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices>
    for IterativeUnion<S>
{
    fn step(
        context: &ReachabilityConfig,
        state: &mut GraphColoredVertices,
    ) -> Completable<GraphColoredVertices> {
        let to_union = S::step(context, state)?;
        if to_union.is_subset(state) {
            info!(
                "Union[{}] finished ({}).",
                simple_type_name::<S>(),
                log_set(state)
            );
            Ok(state.clone())
        } else {
            *state = state.union(&to_union);
            debug!(
                "Union[{}] increased ({}).",
                simple_type_name::<S>(),
                log_set(state)
            );
            Err(Suspended)
        }
    }
}
