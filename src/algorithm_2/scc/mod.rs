mod fwd_bwd;

#[cfg(test)]
mod llm_tests;

use crate::algorithm_trait_2::Generator;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
pub use fwd_bwd::{FwdBwdIterationState, FwdBwdState, FwdBwdStep};

/// A very basic algorithm for finding strongly connected components.
///
/// Basic algorithm idea:
///  - Pick a pivot vertex.
///  - Compute all forward and backward reachable vertices from pivot.
///  - SCC is the intersection of these two sets.
///  - Recursively continue in `FWD \ SCC`, `BWD \ SCC` and `ALL \ FWD \ BWD`.
///  - As with all other SCC algorithms here, only non-trivial SCCs are returned.
pub type FwdBwdScc = Generator<SymbolicAsyncGraph, FwdBwdState, GraphColoredVertices, FwdBwdStep>;
