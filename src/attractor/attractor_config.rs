use crate::reachability::ReachabilityConfig;
use biodivine_lib_param_bn::VariableId;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use std::collections::BTreeSet;

/// A configuration object for attractor detection algorithms.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttractorConfig {
    /// The graph used for attractor computation. You can restrict this graph to a subset
    /// of vertices using [`SymbolicAsyncGraph::restrict`], but keep in mind that this also
    /// eliminates the associated transitions, potentially creating new "fake" attractors
    /// unless the new restriction is forward-closed in the original graph.
    ///
    /// If you are only interested in a subset of attractors, you want to instead limit the
    /// initial set used by the algorithm.
    pub graph: SymbolicAsyncGraph,
    /// If it is known that only a subset of variables can update (e.g., when exploring
    /// a trap space), this can be indicated by restricting the considered variables. Note that
    /// (same as with `graph`), if you restrict variables that still can update, you can create
    /// spurious attractors.
    pub active_variables: BTreeSet<VariableId>,
    /// Cancel the procedure if the symbolic representation exceeds the given number of BDD nodes
    /// (default: `usize::MAX`).
    ///
    /// Note: In the future, this could be replaced by a global "symbolic size" cancellation
    /// trigger, but this will likely rely on direct support from the BDD library.
    pub max_symbolic_size: usize,
}

impl From<SymbolicAsyncGraph> for AttractorConfig {
    fn from(value: SymbolicAsyncGraph) -> Self {
        AttractorConfig::new(value)
    }
}

impl From<&SymbolicAsyncGraph> for AttractorConfig {
    fn from(value: &SymbolicAsyncGraph) -> Self {
        AttractorConfig::new(value.clone())
    }
}

impl From<&AttractorConfig> for ReachabilityConfig {
    fn from(value: &AttractorConfig) -> Self {
        ReachabilityConfig {
            graph: value.graph.clone(),
            active_variables: value.active_variables.clone(),
            max_iterations: usize::MAX,
            max_symbolic_size: value.max_symbolic_size,
        }
    }
}

impl AttractorConfig {
    /// Create a new instance of [`AttractorConfig`] from a [`SymbolicAsyncGraph`].
    pub fn new(graph: SymbolicAsyncGraph) -> AttractorConfig {
        AttractorConfig {
            active_variables: graph.variables().collect(),
            max_symbolic_size: usize::MAX,
            graph,
        }
    }
}
