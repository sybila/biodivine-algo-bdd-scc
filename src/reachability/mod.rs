//! Symbolic reachability algorithms for Boolean networks.
//!
//! This module provides algorithms for computing forward and backward reachable sets
//! in the asynchronous state transition graph of a Boolean network.
//!
//! # Algorithm Variants
//!
//! Two strategies are available for reachability computation:
//!
//! - **Saturation** (default): Processes one variable at a time, finding new reachable
//!   states before moving to the next. Generally produces smaller intermediate BDDs.
//! - **BFS**: Computes all successors/predecessors at each step, exploring the graph
//!   layer by layer. Useful when exploration order matters.
//!
//! # Type Aliases
//!
//! For convenience, the module exports type aliases for common configurations:
//!
//! - [`ForwardReachability`]: Forward reachability using saturation
//! - [`BackwardReachability`]: Backward reachability using saturation
//! - [`ForwardReachabilityBfs`]: Forward reachability using BFS
//! - [`BackwardReachabilityBfs`]: Backward reachability using BFS
//!
//! # Example
//!
//! ```no_run
//! use biodivine_algo_bdd_scc::reachability::ForwardReachability;
//! use biodivine_lib_param_bn::BooleanNetwork;
//! use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
//! use computation_process::Algorithm;
//!
//! let bn = BooleanNetwork::try_from_file("model.aeon").unwrap();
//! let graph = SymbolicAsyncGraph::new(&bn).unwrap();
//!
//! let initial = graph.mk_unit_colored_vertices().pick_vertex();
//! let reachable = ForwardReachability::run(&graph, initial).unwrap();
//! ```

use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use cancel_this::Cancellable;
use computation_process::{Algorithm, Computation};

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
/// from the initial set. Note that this is generally slower than [`ForwardReachability`]
/// because it tends to produce larger BDDs, but it is sometimes required exactly due to this
/// specific order of computation.
pub type ForwardReachabilityBfs = ReachabilityComputation<IterativeUnion<BfsSuccessors>>;

/// A type alias for the recommended backward reachability configuration using saturation.
pub type BackwardReachability = ReachabilityComputation<IterativeUnion<SaturationPredecessors>>;

/// A type alias for a backward reachability procedure that always explores
/// the graph in the BFS order.
///
/// This means each step computes exactly one additional layer of vertices further
/// from the initial set. Note that this is generally slower than [`BackwardReachability`]
/// because it tends to produce larger BDDs, but it is sometimes required exactly due to this
/// specific order of computation.
pub type BackwardReachabilityBfs = ReachabilityComputation<IterativeUnion<BfsPredecessors>>;

/// Used to reduce code repetition in various reachability-like algorithms.
///
/// Implementors define a single step of a reachability procedure, which is then
/// iterated by higher-level algorithms like [`IterativeUnion`].
pub trait ReachabilityStep {
    /// Perform a single step of reachability computation.
    ///
    /// Returns the set of newly discovered states (not already in `state`),
    /// or an empty set if no more states can be reached.
    fn step(
        context: &ReachabilityConfig,
        state: &GraphColoredVertices,
    ) -> Cancellable<GraphColoredVertices>;
}
