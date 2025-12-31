use crate::log_set;
use crate::reachability::{ReachabilityConfig, ReachabilityStep};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use cancel_this::{Cancellable, is_cancelled};
use log::trace;

/// Identifies states that have a successor *outside* of the current reachable set using
/// saturation (one variable at a time).
pub struct HasSuccessorSaturation;

/// Identifies states that have a predecessor *outside* of the current reachable set using
/// saturation (one variable at a time).
pub struct HasPredecessorSaturation;

impl ReachabilityStep for HasSuccessorSaturation {
    fn step(
        context: &ReachabilityConfig,
        state: &GraphColoredVertices,
    ) -> Cancellable<GraphColoredVertices> {
        for var in context.active_variables.iter().rev() {
            is_cancelled!()?;
            let step = context.graph.var_can_post_out(*var, state);
            if !step.is_empty() {
                trace!("[{var}] States with successors found ({}).", log_set(&step));
                return Ok(step);
            }
        }

        Ok(context.graph.mk_empty_colored_vertices())
    }
}

impl ReachabilityStep for HasPredecessorSaturation {
    fn step(
        context: &ReachabilityConfig,
        state: &GraphColoredVertices,
    ) -> Cancellable<GraphColoredVertices> {
        for var in context.active_variables.iter().rev() {
            is_cancelled!()?;
            let step = context.graph.var_can_pre_out(*var, state);
            if !step.is_empty() {
                trace!(
                    "[{var}] States with predecessors found ({}).",
                    log_set(&step)
                );
                return Ok(step);
            }
        }

        Ok(context.graph.mk_empty_colored_vertices())
    }
}
