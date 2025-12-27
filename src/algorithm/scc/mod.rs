mod chain;
mod fwd_bwd;
mod scc_config;

#[cfg(test)]
mod tests;

use crate::algorithm::log_set;
use crate::algorithm::reachability::{
    BackwardReachability, BackwardReachabilityBfs, ForwardReachability, ForwardReachabilityBfs,
};
use crate::algorithm_trait::Incomplete::Working;
use crate::algorithm_trait::{Completable, GenAlgorithm, Generator};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
pub use chain::{ChainState, ChainStep};
pub use fwd_bwd::{FwdBwdState, FwdBwdStep};
use log::{debug, info};
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

pub type ChainScc = Generator<
    SccConfig,
    ChainState,
    GraphColoredVertices,
    ChainStep<ForwardReachability, BackwardReachability>,
>;

fn try_report_scc(scc: GraphColoredVertices) -> Completable<Option<GraphColoredVertices>> {
    if scc.is_empty() {
        // Iteration is done, but we have not found a new non-trivial SCC.
        debug!("The SCC is trivial.");
        Err(Working)
    } else {
        // Iteration is done, and we have a new non-trivial SCC.
        info!("Returning non-trivial SCC ({}).", log_set(&scc));
        Ok(Some(scc))
    }
}
