use crate::log_set;
use crate::reachability::{ReachabilityConfig, ReachabilityStep};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use cancel_this::{Cancellable, is_cancelled};
use log::trace;

/// Identifies states that are "sinks" within the given set. These are states that do not
/// have a successor within the given set.
pub struct RelativeSinks;

/// Identifies states that are "sources" within the given set. These are states that do not
/// have a predecessor within the given set.
pub struct RelativeSources;

/// The union of [`RelativeSinks`] and [`RelativeSources`] which allows us to trim a set
/// from "both sides".
pub struct RelativeSinksAndSources;

impl ReachabilityStep for RelativeSinks {
    fn step(
        context: &ReachabilityConfig,
        state: &GraphColoredVertices,
    ) -> Cancellable<GraphColoredVertices> {
        // We compute the result by inversion: union all states that have a successor
        // and then invert the set.
        let mut has_successor = context.graph.mk_empty_colored_vertices();
        for var in context.active_variables.iter().rev() {
            is_cancelled!()?;
            let var_successor = context.graph.var_can_post_within(*var, state);
            if !var_successor.is_subset(&has_successor) {
                has_successor = has_successor.union(&var_successor);
                trace!(
                    "Inverted sinks updated using `{var}` ({}).",
                    log_set(&has_successor)
                );
            }
        }

        Ok(state.minus(&has_successor))
    }
}

impl ReachabilityStep for RelativeSources {
    fn step(
        context: &ReachabilityConfig,
        state: &GraphColoredVertices,
    ) -> Cancellable<GraphColoredVertices> {
        // We compute the result by inversion: union all states that have a predecessor
        // and then invert the set.
        let mut has_predecessor = context.graph.mk_empty_colored_vertices();
        for var in context.active_variables.iter().rev() {
            is_cancelled!()?;
            let var_predecessor = context.graph.var_can_pre_within(*var, state);
            if !var_predecessor.is_subset(&has_predecessor) {
                has_predecessor = has_predecessor.union(&var_predecessor);
                trace!(
                    "Inverted sources updated using `{var}` ({}).",
                    log_set(&has_predecessor)
                );
            }
        }

        Ok(state.minus(&has_predecessor))
    }
}

impl ReachabilityStep for RelativeSinksAndSources {
    fn step(
        context: &ReachabilityConfig,
        state: &GraphColoredVertices,
    ) -> Cancellable<GraphColoredVertices> {
        let sources = RelativeSources::step(context, state)?;
        if !sources.is_empty() {
            Ok(sources)
        } else {
            RelativeSinks::step(context, state)
        }
    }
}
