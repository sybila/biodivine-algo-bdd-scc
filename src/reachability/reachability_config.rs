use biodivine_lib_param_bn::VariableId;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use std::collections::BTreeSet;

/// A configuration object for various reachability problems.
#[derive(Clone)]
pub struct ReachabilityConfig {
    /// The symbolic graph used for reachability computation.
    pub graph: SymbolicAsyncGraph,
    /// The set of variables that can be updated during reachability.
    pub variables: BTreeSet<VariableId>,
}

impl From<SymbolicAsyncGraph> for ReachabilityConfig {
    fn from(value: SymbolicAsyncGraph) -> Self {
        ReachabilityConfig::new(value)
    }
}

impl From<&SymbolicAsyncGraph> for ReachabilityConfig {
    fn from(value: &SymbolicAsyncGraph) -> Self {
        ReachabilityConfig::new(value.clone())
    }
}

impl ReachabilityConfig {
    /// Create a new instance of [`ReachabilityConfig`] from a [`SymbolicAsyncGraph`] using
    /// all available network variables.
    pub fn new(graph: SymbolicAsyncGraph) -> ReachabilityConfig {
        ReachabilityConfig {
            variables: BTreeSet::from_iter(graph.variables()),
            graph,
        }
    }

    /// Configure the overall state space that this reachability procedure is allowed to explore.
    ///
    /// States that are not in this state space are not fully removed, but their transitions
    /// are ignored, meaning they do not contribute towards reachability. Still, in most cases,
    /// the assumption is that the explored states are a subset of `state_space`.
    pub fn restrict_state_space(mut self, state_space: &GraphColoredVertices) -> Self {
        self.graph = self.graph.restrict(state_space);
        self
    }

    /// Set the variables that can be updated using reachability. All other
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

        self.variables = variables;
        self
    }
}
