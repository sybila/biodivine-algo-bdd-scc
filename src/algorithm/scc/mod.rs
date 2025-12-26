mod fwd_bwd;
mod scc_config;

#[cfg(test)]
mod llm_tests;

use crate::algorithm::reachability::{
    BackwardReachability, BackwardReachabilityBfs, ForwardReachability, ForwardReachabilityBfs,
};
use crate::algorithm_trait::{GenAlgorithm, Generator};
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;

pub use fwd_bwd::{FwdBwdState, FwdBwdStep};
pub use scc_config::SccConfig;

/// A helper trait which allows us to use [`SccAlgorithm`] as shorthand for
/// `GenAlgorithm<Context = SymbolicAsyncGraph, Output = GraphColoredVertices>`.
pub trait SccAlgorithm<STATE>:
    GenAlgorithm<SccConfig, STATE, GraphColoredVertices> + 'static
{
}
impl<STATE, T: GenAlgorithm<SccConfig, STATE, GraphColoredVertices> + 'static> SccAlgorithm<STATE>
    for T
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
    SccConfig,
    FwdBwdState,
    GraphColoredVertices,
    FwdBwdStep<ForwardReachability, BackwardReachability>,
>;

/// Variant of [`FwdBwdScc`] that uses BFS reachability. This is not really necessary and is
/// mostly just for benchmark comparisons.
pub type FwdBwdSccBfs = Generator<
    SccConfig,
    FwdBwdState,
    GraphColoredVertices,
    FwdBwdStep<ForwardReachabilityBfs, BackwardReachabilityBfs>,
>;
