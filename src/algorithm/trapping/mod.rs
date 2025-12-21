use crate::Algorithm;
use crate::algorithm::reachability::{
    Reachability, ReachabilityState, SaturationPredecessors, SaturationSuccessors, StepOperator,
};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use cancel_this::Cancellable;
use log::info;
use std::any::type_name;

#[cfg(test)]
mod tests;

/// Computes the greatest forward trapped subset of the initial set. That is, a subset of states
/// that cannot reach anything outside the original state.
///
/// Internally, the algorithm translates the problem into
/// [`crate::algorithm::reachability::BackwardReachability`]. This is only a type alias,
/// meaning you can use any other backward reachability procedure instead
/// (e.g. [`crate::algorithm::reachability::BackwardReachabilityBFS`]).
pub type TrappingForward = Trapping<SaturationPredecessors>;

/// Computes the greatest backward trapped subset of the initial set. That is, a subset of states
/// that cannot be reached by anything outside the original state.
///
/// Internally, the algorithm translates the problem into
/// [`crate::algorithm::reachability::ForwardReachability`]. This is only a type alias,
/// meaning you can use any other backward reachability procedure instead
/// (e.g. [`crate::algorithm::reachability::BackwardReachabilityBFS`]).
pub type TrappingBackward = Trapping<SaturationSuccessors>;

/// Trapping is a helper algorithm that computes a forward/backward trap set by translating
/// the problem into reachability.
///
/// Here, the relationship is as follows:
///
///  - Assume we want to find a forward trapped subset of a given set `X`.
///  - Recall that a set is forward trapped if none of its states has an outgoing transition
///    leading out of the set.
///  - We compute the complement of `X`, and we use it as initial set for backward reachability.
///  - Once backwards reachability terminates, we again compute the complement of the final set.
///  - This means we are left with a subset of states that cannot reach outside the original
///    set, i.e. a trap set.
///
/// For backward trapped set, the relationship is symmetrical.
pub struct Trapping<S: StepOperator>(Reachability<S>);

impl<S: StepOperator> Algorithm for Trapping<S> {
    type State = ReachabilityState;
    type Output = GraphColoredVertices;

    fn create<T: Into<Self::State>>(initial_state: T) -> Self
    where
        Self: Sized,
    {
        let mut state: ReachabilityState = initial_state.into();

        info!(
            "Initializing trap set computation (elements=`{}`; BDD nodes=`{}`; defers to `{}` reachability).",
            state.set.exact_cardinality(),
            state.set.symbolic_size(),
            type_name::<S>().split("::").last().unwrap_or("?"),
        );

        // Invert the initial set
        state.set = state.graph.mk_unit_colored_vertices().minus(&state.set);
        Trapping(Reachability::create(state))
    }

    fn advance(&mut self) -> Cancellable<Option<Self::Output>> {
        self.0.advance().map(|result| {
            result.map(|output| {
                // Invert the output set
                let output = self
                    .0
                    .state()
                    .graph
                    .mk_unit_colored_vertices()
                    .minus(&output);

                info!(
                    "Trap set computed with {} elements (BDD nodes=`{}`).",
                    output.exact_cardinality(),
                    output.symbolic_size()
                );

                output
            })
        })
    }
}
