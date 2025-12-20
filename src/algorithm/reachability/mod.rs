use biodivine_lib_param_bn::VariableId;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use cancel_this::Cancellable;

pub mod reachability_algorithm;
mod step_operators;

#[cfg(test)]
mod tests;

pub use reachability_algorithm::Reachability;
pub use step_operators::AllPredecessors;
pub use step_operators::AllSuccessors;
pub use step_operators::SaturationPredecessors;
pub use step_operators::SaturationSuccessors;

/// A type alias for the recommended forward reachability configuration using saturation.
pub type ForwardReachability = Reachability<SaturationSuccessors>;

/// A type alias for forward reachability procedure that always explores
/// the graph in the BFS order.
///
/// This means each step computes exactly one additional layer of vertices further
/// from the initial set. Note that this is generally slower than [`ForwardReachability`]
/// because it tends to produce larger BDDs, but it is sometimes required exactly due to this
/// specific order of computation.
pub type ForwardReachabilityBFS = Reachability<AllSuccessors>;

/// A type alias for the recommended backward reachability configuration using saturation.
pub type BackwardReachability = Reachability<SaturationPredecessors>;

/// A type alias for backward reachability procedure that always explores
/// the graph in the BFS order.
///
/// This means each step computes exactly one additional layer of vertices further
/// from the initial set. Note that this is generally slower than [`ForwardReachability`]
/// because it tends to produce larger BDDs, but it is sometimes required exactly due to this
/// specific order of computation.
pub type BackwardReachabilityBFS = Reachability<AllPredecessors>;

/// An internal trait implemented by various algorithmic reachability structs that compute
/// the successors or predecessors of a particular reachable set.
pub trait StepOperator {
    fn step(state: &ReachabilityState) -> Cancellable<GraphColoredVertices>;
}

/// An internal object used across various reachability algorithms to represent
/// algorithm state and configuration.
///
/// TODO: In the future, this structure must be serializable.
#[derive(Clone)]
pub struct ReachabilityState {
    /// The graph that encodes all system transitions.
    pub graph: SymbolicAsyncGraph,
    /// The *sorted list* of variables that are allowed to be updated.
    pub variables: Vec<VariableId>,
    /// Current set of reachable states.
    pub set: GraphColoredVertices,
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
        let mut variables = variables.to_vec();
        variables.sort();

        // Check that all variables are valid in this graph:
        if let Some(last) = variables.last() {
            assert!(last.to_index() < self.graph.num_vars());
        }

        self.variables = variables;
        self
    }
}
