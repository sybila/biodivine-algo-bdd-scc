use biodivine_lib_param_bn::VariableId;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use std::collections::BTreeSet;

/// An internal object used across various reachability algorithms to represent
/// algorithm state and configuration.
///
/// TODO: In the future, this structure must be serializable.
#[derive(Clone)]
pub struct ReachabilityState {
    /// The graph that encodes all system transitions.
    pub graph: SymbolicAsyncGraph,
    /// The *sorted set* of variables that are allowed to be updated.
    pub variables: BTreeSet<VariableId>,
    /// Current set of reachable states.
    pub set: GraphColoredVertices,
}

impl From<(SymbolicAsyncGraph, GraphColoredVertices)> for ReachabilityState {
    fn from(value: (SymbolicAsyncGraph, GraphColoredVertices)) -> Self {
        ReachabilityState::initial(value.0, value.1)
    }
}

impl ReachabilityState {
    /// Create a new [`ReachabilityState`] using a [`SymbolicAsyncGraph`] and initial
    /// [`GraphColoredVertices`] set.
    pub fn initial(graph: SymbolicAsyncGraph, initial: GraphColoredVertices) -> Self {
        ReachabilityState {
            variables: graph.variables().collect(),
            set: initial,
            graph,
        }
    }

    /// Set the variables which reachability is allowed to change. All other
    /// variables will be ignored.
    ///
    /// # Panics
    ///
    /// The method will panic if you submit a variable ID that is not valid in the current graph.
    pub fn restrict_variables(mut self, variables: &[VariableId]) -> Self {
        let variables = BTreeSet::from_iter(variables.iter().copied());

        // Check that all variables are valid in this graph. Using the maximal variable
        // is sufficient because valid variables are always a continuous range.
        if let Some(last) = variables.last() {
            assert!(last.to_index() < self.graph.num_vars());
        }

        self.variables = variables;
        self
    }
}
