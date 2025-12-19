use crate::Reachability;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use cancel_this::{Cancellable, is_cancelled};
use log::{debug, info};

/// Naive reachability methods that always follow the breadth-first exploration order.
impl Reachability {
    /// Compute the colored set of elements that can be reached within `graph` from the given
    /// set of `initial` elements (i.e. there is a path from an initial vertex to the returned
    /// vertex consistently using a single color).
    pub fn reach_forward_naive(
        graph: &SymbolicAsyncGraph,
        initial: GraphColoredVertices,
    ) -> Cancellable<GraphColoredVertices> {
        info!(
            "Computing naive forward reachability with {} elements (using {} BDD nodes).",
            initial.exact_cardinality(),
            initial.symbolic_size()
        );
        let mut result = initial;
        loop {
            let step = graph.post(&result);
            is_cancelled!()?;

            if step.is_subset(&result) {
                break;
            }

            result = result.union(&step);
            is_cancelled!()?;

            debug!(
                "Reachable set extended to {} elements (using {} BDD nodes).",
                result.exact_cardinality(),
                result.symbolic_size()
            );
        }
        info!(
            "Naive forward reachability terminated with {} elements (using {} BDD nodes).",
            result.exact_cardinality(),
            result.symbolic_size()
        );
        Ok(result)
    }

    /// Compute the colored set of elements that can within `graph` reach the given
    /// set of `initial` elements (i.e. there is a path into an initial vertex from the returned
    /// vertex consistently using a single color).
    pub fn reach_backward_naive(
        graph: &SymbolicAsyncGraph,
        initial: GraphColoredVertices,
    ) -> Cancellable<GraphColoredVertices> {
        info!(
            "Computing naive backward reachability with {} elements (using {} BDD nodes).",
            initial.exact_cardinality(),
            initial.symbolic_size()
        );
        let mut result = initial;
        loop {
            let step = graph.pre(&result);
            is_cancelled!()?;

            if step.is_subset(&result) {
                break;
            }

            result = result.union(&step);
            is_cancelled!()?;

            debug!(
                "Reachable set extended to {} elements (using {} BDD nodes).",
                result.exact_cardinality(),
                result.symbolic_size()
            );
        }
        info!(
            "Naive forward reachability terminated with {} elements (using {} BDD nodes).",
            result.exact_cardinality(),
            result.symbolic_size()
        );
        Ok(result)
    }

    /// Compute the smallest subset of `initial` that is forward-trapped, i.e. it has no outgoing
    /// transitions leading out of the set. In particular, running forward reachability on
    /// a forward trap will always produce the same set.
    pub fn trap_forward_naive(
        graph: &SymbolicAsyncGraph,
        initial: GraphColoredVertices,
    ) -> Cancellable<GraphColoredVertices> {
        info!(
            "Computing naive forward trap with {} elements (using {} BDD nodes).",
            initial.exact_cardinality(),
            initial.symbolic_size()
        );
        let mut result = initial;
        loop {
            let step = graph.can_post_out(&result);
            is_cancelled!()?;

            if step.is_empty() {
                break;
            }

            result = result.minus(&step);
            is_cancelled!()?;

            debug!(
                "Trap set reduced to {} elements (using {} BDD nodes).",
                result.exact_cardinality(),
                result.symbolic_size()
            );
        }
        info!(
            "Naive forward trap terminated with {} elements (using {} BDD nodes).",
            result.exact_cardinality(),
            result.symbolic_size()
        );
        Ok(result)
    }

    /// Compute the smallest subset of `initial` that is backward-trapped, i.e. it has no incoming
    /// transitions leading into the set. In particular, running backward reachability on
    /// a backward trap will always produce the same set.
    pub fn trap_backward_naive(
        graph: &SymbolicAsyncGraph,
        initial: GraphColoredVertices,
    ) -> Cancellable<GraphColoredVertices> {
        info!(
            "Computing naive backward trap with {} elements (using {} BDD nodes).",
            initial.exact_cardinality(),
            initial.symbolic_size()
        );
        let mut result = initial;
        loop {
            let step = graph.can_pre_out(&result);
            is_cancelled!()?;

            if step.is_empty() {
                break;
            }

            result = result.minus(&step);
            is_cancelled!()?;

            debug!(
                "Trap set reduced to {} elements (using {} BDD nodes).",
                result.exact_cardinality(),
                result.symbolic_size()
            );
        }
        info!(
            "Naive backward trap terminated with {} elements (using {} BDD nodes).",
            result.exact_cardinality(),
            result.symbolic_size()
        );
        Ok(result)
    }
}
