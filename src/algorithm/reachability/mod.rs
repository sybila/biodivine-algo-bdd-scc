use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use cancel_this::Cancellable;

mod reachability_algorithm;
mod reachability_state;
mod step_operators;

#[cfg(test)]
mod tests;

use crate::Algorithm;

pub use reachability_algorithm::Reachability;
pub use reachability_state::ReachabilityState;
pub use step_operators::BfsPredecessors;
pub use step_operators::BfsSuccessors;
pub use step_operators::SaturationPredecessors;
pub use step_operators::SaturationSuccessors;

/// A helper trait which allows us to use [`ReachabilityAlgorithm`] as a shorthand for
/// `Algorithm<State = ReachabilityState, Output = GraphColoredVertices>`.
pub trait ReachabilityAlgorithm:
    Algorithm<State = ReachabilityState, Output = GraphColoredVertices>
{
}
impl<A: Algorithm<State = ReachabilityState, Output = GraphColoredVertices>> ReachabilityAlgorithm
    for A
{
}

/// A type alias for the recommended forward reachability configuration using saturation.
pub type ForwardReachability = Reachability<SaturationSuccessors>;

/// A type alias for forward reachability procedure that always explores
/// the graph in the BFS order.
///
/// This means each step computes exactly one additional layer of vertices further
/// from the initial set. Note that this is generally slower than [`ForwardReachability`]
/// because it tends to produce larger BDDs, but it is sometimes required exactly due to this
/// specific order of computation.
pub type ForwardReachabilityBfs = Reachability<BfsSuccessors>;

/// A type alias for the recommended backward reachability configuration using saturation.
pub type BackwardReachability = Reachability<SaturationPredecessors>;

/// A type alias for backward reachability procedure that always explores
/// the graph in the BFS order.
///
/// This means each step computes exactly one additional layer of vertices further
/// from the initial set. Note that this is generally slower than [`BackwardReachability`]
/// because it tends to produce larger BDDs, but it is sometimes required exactly due to this
/// specific order of computation.
pub type BackwardReachabilityBfs = Reachability<BfsPredecessors>;

/// An internal trait implemented by various algorithmic reachability structs that compute
/// the successors or predecessors of a particular reachable set.
pub trait StepOperator {
    fn step(state: &ReachabilityState) -> Cancellable<GraphColoredVertices>;
}
