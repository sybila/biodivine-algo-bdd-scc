//! BDD-based algorithms for symbolic SCC detection in Boolean networks.
//!
//! This crate provides efficient symbolic algorithms for computing strongly connected
//! components (SCCs) and reachability in asynchronous Boolean networks, using Binary
//! Decision Diagrams (BDDs) as the underlying representation.
//!
//! # Main Modules
//!
//! - [`reachability`]: Forward and backward reachability algorithms (BFS and saturation)
//! - [`scc`]: SCC detection algorithms (forward-backward and chain-based)
//! - [`trimming`]: Algorithms for removing trivial sink/source states
//!
//! # Quick Start
//!
//! ## SCC Enumeration
//!
//! ```no_run
//! use biodivine_algo_bdd_scc::scc::{FwdBwdScc, SccConfig};
//! use biodivine_lib_param_bn::BooleanNetwork;
//! use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
//! use computation_process::Stateful;
//!
//! // Load a Boolean network
//! let bn = BooleanNetwork::try_from_file("model.aeon").unwrap();
//! let graph = SymbolicAsyncGraph::new(&bn).unwrap();
//!
//! // Enumerate all non-trivial SCCs
//! let config = SccConfig::new(graph.clone());
//! for scc in FwdBwdScc::configure(config, graph.mk_unit_colored_vertices()) {
//!     let scc = scc.unwrap();
//!     println!("Found SCC with {} states", scc.exact_cardinality());
//! }
//! ```
//!
//! ## Reachability Analysis
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
//! // Compute forward reachable set from an initial state
//! let initial = graph.mk_unit_colored_vertices().pick_vertex();
//! let reachable = ForwardReachability::run(&graph, initial).unwrap();
//! ```
//!
//! # Algorithm Variants
//!
//! The crate provides multiple algorithm variants optimized for different scenarios:
//!
//! - **Saturation-based** algorithms (default): Generally faster due to smaller BDD sizes
//! - **BFS-based** algorithms: Useful when a specific exploration order is required
//! - **Chain algorithm**: Can handle some larger networks where forward-backward fails

use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;

#[cfg(test)]
mod test_utils;

pub mod reachability;
pub mod scc;
pub mod trimming;

/// A utility method for printing useful metadata of symbolic sets.
fn log_set(set: &GraphColoredVertices) -> String {
    format!(
        "elements={}; BDD nodes={}",
        set.exact_cardinality(),
        set.symbolic_size()
    )
}

/// Extract the "simple name" of a type argument at compile time.
///
/// In the future, this should be a `const fn`, but `type_name` and `unwrap_or` are not
/// yet stabilized as `const` functions (even thought they probably are).
fn simple_type_name<T>() -> &'static str {
    std::any::type_name::<T>().split("::").last().unwrap_or("?")
}
