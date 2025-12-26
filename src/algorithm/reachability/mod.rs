use crate::algorithm_trait::{Algorithm, Computation};
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use cancel_this::Cancellable;

#[cfg(test)]
mod tests;

mod iterative_union;
mod reachability_config;
mod step_operators;

pub use iterative_union::IterativeUnion;
pub use reachability_config::ReachabilityConfig;
pub use step_operators::{
    BfsPredecessors, BfsSuccessors, SaturationPredecessors, SaturationSuccessors,
};

/// A helper alias which allows us to use [`ReachabilityComputation`] as shorthand for
/// `Computation<Context = ReachabilityConfig, State = GraphColoredVertices>`.
pub type ReachabilityComputation<STEP> =
    Computation<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices, STEP>;

/// A helper trait which allows us to use [`ReachabilityAlgorithm`] as shorthand for
/// `Algorithm<Context = ReachabilityConfig, State = GraphColoredVertices>`.
pub trait ReachabilityAlgorithm:
    Algorithm<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static
{
}
impl<T: Algorithm<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static>
    ReachabilityAlgorithm for T
{
}

/// A type alias for the recommended forward reachability configuration using saturation.
pub type ForwardReachability = ReachabilityComputation<IterativeUnion<SaturationSuccessors>>;

/// A type alias for a forward reachability procedure that always explores
/// the graph in the BFS order.
///
/// This means each step computes exactly one additional layer of vertices further
/// from the initial set. Note that this is generally slower than [`crate::algorithm::reachability::ForwardReachability`]
/// because it tends to produce larger BDDs, but it is sometimes required exactly due to this
/// specific order of computation.
pub type ForwardReachabilityBfs = ReachabilityComputation<IterativeUnion<BfsSuccessors>>;

/// A type alias for the recommended backward reachability configuration using saturation.
pub type BackwardReachability = ReachabilityComputation<IterativeUnion<SaturationPredecessors>>;

/// A type alias for a backward reachability procedure that always explores
/// the graph in the BFS order.
///
/// This means each step computes exactly one additional layer of vertices further
/// from the initial set. Note that this is generally slower than [`crate::algorithm::reachability::BackwardReachability`]
/// because it tends to produce larger BDDs, but it is sometimes required exactly due to this
/// specific order of computation.
pub type BackwardReachabilityBfs = ReachabilityComputation<IterativeUnion<BfsPredecessors>>;

/// Used to reduce code repetition in various reachability-like algorithms.
pub trait ReachabilityStep {
    fn step(
        context: &ReachabilityConfig,
        state: &GraphColoredVertices,
    ) -> Cancellable<GraphColoredVertices>;
}
