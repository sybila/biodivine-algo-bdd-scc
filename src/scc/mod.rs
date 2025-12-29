//! Symbolic SCC detection algorithms for Boolean networks.
//!
//! This module provides algorithms for detecting strongly connected components (SCCs)
//! in the asynchronous state transition graph of a Boolean network.
//!
//! # Algorithms
//!
//! - [`FwdBwdScc`]: Classic forward-backward algorithm. Picks a pivot, computes forward
//!   and backward reachable sets, and extracts their intersection as an SCC.
//! - [`ChainScc`]: Chain-based algorithm that uses backward reachability to find basins
//!   and then forward reachability within each basin to find SCCs. Can sometimes handle
//!   larger networks.
//!
//! Both algorithms only report **non-trivial SCCs** (containing more than one state).
//!
//! # Configuration
//!
//! Use [`SccConfig`] to customize algorithm behavior:
//!
//! - **Trimming**: Remove trivial sink/source states before SCC computation
//! - **Long-lived filtering**: Only report SCCs that cannot be escaped by updating
//!   a single variable
//!
//! # Example
//!
//! ```no_run
//! use biodivine_algo_bdd_scc::scc::{FwdBwdScc, SccConfig};
//! use biodivine_lib_param_bn::BooleanNetwork;
//! use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
//! use computation_process::Stateful;
//!
//! let bn = BooleanNetwork::try_from_file("model.aeon").unwrap();
//! let graph = SymbolicAsyncGraph::new(&bn).unwrap();
//!
//! // Configure with trimming and long-lived filtering
//! let config = SccConfig::new(graph.clone())
//!     .filter_long_lived(true);
//!
//! for scc in FwdBwdScc::configure(config, &graph) {
//!     let scc = scc.unwrap();
//!     println!("Found SCC with {} states", scc.exact_cardinality());
//! }
//! ```

mod chain;
mod fwd_bwd;
mod scc_config;

#[cfg(test)]
mod tests;

use crate::reachability::{
    BackwardReachability, BackwardReachabilityBfs, ForwardReachability, ForwardReachabilityBfs,
};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
pub use chain::{ChainState, ChainStep};
use computation_process::{GenAlgorithm, Generator};
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

/// A very basic forward-backward algorithm for finding strongly connected components.
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

/// Variant of [`FwdBwdScc`] that uses BFS reachability. This is not very practical (the rigid
/// BFS order is not required for `fwd-bwd` to work) and is mostly just intended for benchmarking.
pub type FwdBwdSccBfs = Generator<
    SccConfig,
    FwdBwdState,
    GraphColoredVertices,
    FwdBwdStep<ForwardReachabilityBfs, BackwardReachabilityBfs>,
>;

/// An SCC detection algorithm that uses "chain-like" exploration. It is generally faster
/// than the `fwd-bwd` algorithm, but not exclusively so. Generally, we recommend
/// `chain` as the default SCC detection algorithm, but for hard instances it may be useful
/// to test both approaches.
///
/// > Note that this algorithm is slightly different from the one presented in
/// > [A Truly Symbolic Linear-Time Algorithm for SCC Decomposition](https://link.springer.com/chapter/10.1007/978-3-031-30820-8_22).
/// > Mainly, it does not select the pivot vertex from the "last level" of the reachability
/// > procedure. The main reason is that this requires BFS reachability, which is in practice
/// > much slower than saturation.
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

/// Remove colors that correspond to trivial and short-lived SCCs (if configured to do so).
fn filter_scc(context: &SccConfig, scc: GraphColoredVertices) -> Option<GraphColoredVertices> {
    // First, remove all colors in which the SCC is trivial.
    let valid_colors = scc.minus(&scc.pick_vertex()).colors();
    let non_trivial_scc = scc.intersect_colors(&valid_colors);

    if non_trivial_scc.is_empty() {
        info!("The SCC is trivial.");
        return None;
    }

    let long_lived_scc = if context.filter_long_lived {
        retain_long_lived(&context.graph, &non_trivial_scc)
    } else {
        non_trivial_scc.clone()
    };

    if long_lived_scc.is_empty() {
        info!("The SCC is short-lived.");
        return None;
    }

    Some(long_lived_scc)
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
