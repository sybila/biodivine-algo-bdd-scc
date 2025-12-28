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
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
pub use chain::{ChainState, ChainStep};
pub use fwd_bwd::{FwdBwdState, FwdBwdStep};
use log::info;
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

/// An SCC detection algorithm that uses "chain-like" exploration. It can sometimes work on
/// larger networks where fwd-bwd fails. But in cases where `fwd-bwd`` works, `fwd-bwd` is
/// often faster because it is quicker to partition the state space into smaller chunks.
///
/// Basic algorithm idea:
///  - Pick a pivot vertex (using a hint set if available).
///  - Compute a backwards reachable set and SCC inside this set (using forward).
///  - Recursively continue in `BWD \ SCC` and `ALL \ BWD`.
///  - The pivot hints are the immediate successors/predecessors of the SCC.
///  - If trimming removes pivot hints, we replace them with immediate predecessors/successors
///    of the trimmed set.
pub type ChainScc = Generator<
    SccConfig,
    ChainState,
    GraphColoredVertices,
    ChainStep<ForwardReachability, BackwardReachability>,
>;

fn try_report_scc(
    context: &SccConfig,
    scc: GraphColoredVertices,
) -> Completable<Option<GraphColoredVertices>> {
    if scc.is_empty() {
        // Iteration is done, but we have not found a new non-trivial SCC.
        info!("The SCC is trivial.");
        Err(Working)
    } else {
        // Iteration is done, and we have a new non-trivial SCC.
        let reduced_scc = if context.filter_long_lived {
            retain_long_lived(&context.graph, &scc)
        } else {
            scc.clone()
        };

        if reduced_scc.is_empty() {
            info!("Skipping short-lived SCC ({}).", log_set(&scc));
            return Err(Working);
        }

        info!("Returning non-trivial SCC ({}).", log_set(&reduced_scc));
        Ok(Some(reduced_scc))
    }
}

/// Return a subset of states that are long-lived, meaning the set cannot be escaped by updating a
/// single variable. This is evaluated per-color, i.e., each color is either fully retained
/// or fully removed.
fn retain_long_lived(
    graph: &SymbolicAsyncGraph,
    set: &GraphColoredVertices,
) -> GraphColoredVertices {
    let colors = set.colors();
    if colors.is_singleton() {
        // For singletons, we can use a simpler algorithm
        for var in graph.variables() {
            let can_post_out = graph.var_can_post_out(var, set);
            if &can_post_out == set {
                return graph.mk_empty_colored_vertices();
            }
        }
        set.clone()
    } else {
        // For colored sets, this is a bit more complicated.
        // A color is safe (long-lived) if for EVERY variable, at least one state stays inside.
        // We start with all colors and intersect with colors that have states staying for each var.
        let mut safe_colors = colors.clone();
        for var in graph.variables() {
            let can_post_out = graph.var_can_post_out(var, set);
            let stays_inside = set.minus(&can_post_out);
            safe_colors = safe_colors.intersect(&stays_inside.colors());
            if safe_colors.is_empty() {
                return graph.mk_empty_colored_vertices();
            }
        }
        set.intersect_colors(&safe_colors)
    }
}
