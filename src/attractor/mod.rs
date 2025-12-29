//! Symbolic attractor enumeration algorithms for Boolean networks.
//!
//! This module provides algorithms that enumerate **attractors** (bottom SCCs) of the
//! asynchronous state transition graph of a Boolean network.
//!
//! # Algorithms
//!
//! - [`XieBeerelAttractors`]: A generator based on the Xie–Beerel SCC decomposition scheme,
//!   specialized to enumerate only attractors. This is the main algorithm for exact attractor
//!   enumeration.
//! - [`InterleavedTransitionGuidedReduction`]: A preprocessing reduction (ITGR) that tries to
//!   shrink the explored state space and identify variables that are irrelevant in the remaining
//!   part of the graph.
//!
//! # Typical usage
//!
//! For large models, it is often useful to run ITGR first to reduce the universe, then run
//! Xie–Beerel on the reduced graph. This combined approach is usually significantly faster
//! than running Xie-Beerel directly.
//!
//! ```no_run
//! use biodivine_algo_bdd_scc::attractor::{
//!     AttractorConfig, InterleavedTransitionGuidedReduction, ItgrState, XieBeerelAttractors,
//!     XieBeerelState,
//! };
//! use biodivine_lib_param_bn::BooleanNetwork;
//! use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
//! use computation_process::{Computable, Stateful};
//!
//! let bn = BooleanNetwork::try_from_file("model.aeon").unwrap();
//! let graph = SymbolicAsyncGraph::new(&bn).unwrap();
//!
//! // 1) Reduce the universe using ITGR.
//! let config = AttractorConfig::new(graph.clone());
//! let itgr_state = ItgrState::new(&graph, &graph.mk_unit_colored_vertices());
//! let mut itgr = InterleavedTransitionGuidedReduction::configure(config.clone(), itgr_state);
//! // This step is cancellable and returns the reduced state space (or error).
//! let reduced = itgr.compute().unwrap();
//!
//! // 2) Restrict the attractor search to the reduced state space and variables.
//! //    Note: active_variables() returns an iterator, which `restrict_variables` can consume.
//! let config = config
//!     .restrict_state_space(&reduced)
//!     .restrict_variables(itgr.state().active_variables());
//!
//! // 3) Enumerate attractors.
//! let initial_state = XieBeerelState::from(&reduced);
//! for attractor in XieBeerelAttractors::configure(config, initial_state) {
//!     let attractor = attractor.unwrap();
//!     println!("Attractor has {} states.", attractor.exact_cardinality());
//! }
//! ```

mod attractor_config;
mod itgr;
mod xie_beerel;

#[cfg(test)]
mod tests;

pub use attractor_config::AttractorConfig;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use computation_process::{Computation, Generator};
pub use itgr::{ItgrState, ItgrStep};
pub use xie_beerel::{XieBeerelState, XieBeerelStep};

/// Enumerate attractors using the Xie–Beerel algorithm.
pub type XieBeerelAttractors =
    Generator<AttractorConfig, XieBeerelState, GraphColoredVertices, XieBeerelStep>;

/// Reduce the universe using Interleaved Transition-Guided Reduction (ITGR).
pub type InterleavedTransitionGuidedReduction =
    Computation<AttractorConfig, ItgrState, GraphColoredVertices, ItgrStep>;
