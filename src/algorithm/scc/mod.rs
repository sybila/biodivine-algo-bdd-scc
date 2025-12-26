mod fwd_bwd;

#[cfg(test)]
mod llm_tests;

use crate::algorithm::reachability::{
    BackwardReachability, BackwardReachabilityBfs, ForwardReachability, ForwardReachabilityBfs,
};
use crate::algorithm_trait::{GenAlgorithm, Generator};
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
pub use fwd_bwd::{FwdBwdIterationState, FwdBwdState, FwdBwdStep};

/// A helper trait which allows us to use [`crate::algorithm::reachability::ReachabilityAlgorithm`] as shorthand for
/// `Algorithm<Context = ReachabilityConfig, State = GraphColoredVertices>`.
pub trait SccGenAlgorithm<STATE>:
    GenAlgorithm<SymbolicAsyncGraph, STATE, GraphColoredVertices> + 'static
{
}
impl<STATE, T: GenAlgorithm<SymbolicAsyncGraph, STATE, GraphColoredVertices> + 'static>
    SccGenAlgorithm<STATE> for T
{
}

/// A very basic algorithm for finding strongly connected components.
///
/// Basic algorithm idea:
///  - Pick a pivot vertex.
///  - Compute all forward and backward reachable vertices from pivot.
///  - SCC is the intersection of these two sets.
///  - Recursively continue in `FWD \ SCC`, `BWD \ SCC` and `ALL \ FWD \ BWD`.
///  - As with all other SCC algorithms here, only non-trivial SCCs are returned.
pub type FwdBwdScc = Generator<
    SymbolicAsyncGraph,
    FwdBwdState,
    GraphColoredVertices,
    FwdBwdStep<ForwardReachability, BackwardReachability>,
>;

/// Variant of [`FwdBwdScc`] that uses BFS reachability. This is not really necessary and is
/// mostly just for benchmark comparisons.
pub type FwdBwdSccBfs = Generator<
    SymbolicAsyncGraph,
    FwdBwdState,
    GraphColoredVertices,
    FwdBwdStep<ForwardReachabilityBfs, BackwardReachabilityBfs>,
>;
