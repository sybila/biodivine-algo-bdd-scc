use crate::reachability::ReachabilityConfig;
use biodivine_lib_param_bn::VariableId;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use std::collections::BTreeSet;

/// A configuration object for various attractor detection algorithms.
#[derive(Clone)]
pub struct AttractorConfig {
    /// The graph used for SCC computation.
    pub graph: SymbolicAsyncGraph,
    /// If it is known that only a subset of variables can update (e.g., when exploring
    /// a trap space), this can be indicated by restricting the considered variables.
    pub active_variables: BTreeSet<VariableId>,
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
            variables: value.active_variables.clone(),
        }
    }
}

impl AttractorConfig {
    /// Create a new instance of [`AttractorConfig`] from a [`SymbolicAsyncGraph`].
    pub fn new(graph: SymbolicAsyncGraph) -> AttractorConfig {
        AttractorConfig {
            active_variables: graph.variables().collect(),
            graph,
        }
    }

    /// Configure the overall state space that this attractor detection is allowed to explore.
    ///
    /// States that are not in this state space are not fully removed, but their transitions
    /// are ignored, meaning they do not contribute towards reachability.
    pub fn restrict_state_space(mut self, state_space: &GraphColoredVertices) -> Self {
        self.graph = self.graph.restrict(state_space);
        self
    }

    /// Set the variables that can update inside an attractor. All other
    /// variables will be ignored.
    ///
    /// # Panics
    ///
    /// The method will panic if you submit a variable ID that is not valid in the current graph.
    pub fn restrict_variables(mut self, variables: &[VariableId]) -> Self {
        let variables = BTreeSet::from_iter(variables.iter().copied());

        // Check that all variables are valid in this graph. Using the maximal variable
        // is enough because valid variables are always a continuous range.
        if let Some(last) = variables.last() {
            assert!(last.to_index() < self.graph.num_vars());
        }

        self.active_variables = variables;
        self
    }
}
