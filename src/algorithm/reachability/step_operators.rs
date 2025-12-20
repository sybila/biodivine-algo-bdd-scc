use crate::algorithm::reachability::{ReachabilityState, StepOperator};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use cancel_this::{Cancellable, is_cancelled};
use log::trace;

/// Computes the direct successors of the current reachable set, excluding values that are already
/// in the reachable set.
pub struct AllSuccessors;

/// Computes the direct predecessors of the current reachable set, excluding values that are
/// already in the reachable set.
pub struct AllPredecessors;

/// Find the greatest variable which admits successors (excluding current reachable values) in
/// the current reachable set and return those successors (or empty set otherwise).
pub struct SaturationSuccessors;

/// Find the greatest variable which admits predecessors (excluding current reachable values) in
/// the current reachable set and return those predecessors (or empty set otherwise).
pub struct SaturationPredecessors;

impl StepOperator for AllSuccessors {
    fn step(state: &ReachabilityState) -> Cancellable<GraphColoredVertices> {
        let mut successors = state.graph.mk_empty_colored_vertices();
        for var in state.variables.iter().rev() {
            is_cancelled!()?;
            let var_successors = state.graph.var_post_out(*var, &state.set);
            if !var_successors.is_empty() {
                successors = successors.union(&var_successors);
                trace!(
                    "Successors updated using `{}` to `{}` elements (`{}` BDD nodes).",
                    var,
                    successors.exact_cardinality(),
                    successors.symbolic_size()
                );
            }
        }
        Ok(successors)
    }
}

impl StepOperator for AllPredecessors {
    fn step(state: &ReachabilityState) -> Cancellable<GraphColoredVertices> {
        let mut predecessors = state.graph.mk_empty_colored_vertices();
        for var in state.variables.iter().rev() {
            is_cancelled!()?;
            let var_predecessors = state.graph.var_pre_out(*var, &state.set);
            if !var_predecessors.is_empty() {
                predecessors = predecessors.union(&var_predecessors);
                trace!(
                    "Predecessors updated using `{}` to `{}` elements (`{}` BDD nodes).",
                    var,
                    predecessors.exact_cardinality(),
                    predecessors.symbolic_size()
                );
            }
        }
        Ok(predecessors)
    }
}

impl StepOperator for SaturationSuccessors {
    fn step(state: &ReachabilityState) -> Cancellable<GraphColoredVertices> {
        for var in state.variables.iter().rev() {
            is_cancelled!()?;
            let step = state.graph.var_post_out(*var, &state.set);
            if !step.is_empty() {
                trace!(
                    "Found successors using `{}` with `{}` elements (`{}` BDD nodes).",
                    var,
                    step.exact_cardinality(),
                    step.symbolic_size()
                );
                return Ok(step);
            }
        }

        Ok(state.graph.mk_empty_colored_vertices())
    }
}

impl StepOperator for SaturationPredecessors {
    fn step(state: &ReachabilityState) -> Cancellable<GraphColoredVertices> {
        for var in state.variables.iter().rev() {
            is_cancelled!()?;
            let step = state.graph.var_pre_out(*var, &state.set);
            if !step.is_empty() {
                trace!(
                    "Found predecessors using `{}` with `{}` elements (`{}` BDD nodes).",
                    var,
                    step.exact_cardinality(),
                    step.symbolic_size()
                );
                return Ok(step);
            }
        }

        Ok(state.graph.mk_empty_colored_vertices())
    }
}
