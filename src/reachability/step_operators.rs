use crate::log_set;
use crate::reachability::{ReachabilityConfig, ReachabilityStep};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use cancel_this::{Cancellable, is_cancelled};
use log::trace;

/// Computes the direct successors of the current reachable set, excluding values that are
/// already in the reachable set.
pub struct BfsSuccessors;

/// Computes the direct predecessors of the current reachable set, excluding values that are
/// already in the reachable set.
pub struct BfsPredecessors;

/// Find the greatest variable which produces successors (excluding current reachable values) in
/// the current reachable set and return those successors (or empty set otherwise).
pub struct SaturationSuccessors;

/// Find the greatest variable which produces predecessors (excluding current reachable values) in
/// the current reachable set and return those predecessors (or empty set otherwise).
pub struct SaturationPredecessors;

impl ReachabilityStep for BfsSuccessors {
    fn step(
        context: &ReachabilityConfig,
        state: &GraphColoredVertices,
    ) -> Cancellable<GraphColoredVertices> {
        let mut post = context.graph.mk_empty_colored_vertices();
        for var in context.variables.iter().rev() {
            is_cancelled!()?;
            let var_successors = context.graph.var_post_out(*var, state);
            if !var_successors.is_empty() {
                is_cancelled!()?;
                post = post.union(&var_successors);

                trace!("Successors updated using `{var}` ({}).", log_set(&post));
            }
        }
        Ok(post)
    }
}

impl ReachabilityStep for BfsPredecessors {
    fn step(
        context: &ReachabilityConfig,
        state: &GraphColoredVertices,
    ) -> Cancellable<GraphColoredVertices> {
        let mut pre = context.graph.mk_empty_colored_vertices();
        for var in context.variables.iter().rev() {
            is_cancelled!()?;
            let var_predecessors = context.graph.var_pre_out(*var, state);
            if !var_predecessors.is_empty() {
                pre = pre.union(&var_predecessors);

                trace!("Predecessors updated using `{var}` ({}).", log_set(&pre));
            }
        }
        Ok(pre)
    }
}

impl ReachabilityStep for SaturationSuccessors {
    fn step(
        context: &ReachabilityConfig,
        state: &GraphColoredVertices,
    ) -> Cancellable<GraphColoredVertices> {
        for var in context.variables.iter().rev() {
            is_cancelled!()?;
            let step = context.graph.var_post_out(*var, state);
            if !step.is_empty() {
                trace!("Found successors using `{var}` ({}).", log_set(&step));
                return Ok(step);
            }
        }

        Ok(context.graph.mk_empty_colored_vertices())
    }
}

impl ReachabilityStep for SaturationPredecessors {
    fn step(
        context: &ReachabilityConfig,
        state: &GraphColoredVertices,
    ) -> Cancellable<GraphColoredVertices> {
        for var in context.variables.iter().rev() {
            is_cancelled!()?;
            let step = context.graph.var_pre_out(*var, state);
            if !step.is_empty() {
                trace!("Found predecessors using `{var}` ({}).", log_set(&step));
                return Ok(step);
            }
        }

        Ok(context.graph.mk_empty_colored_vertices())
    }
}
