use biodivine_lib_param_bn::VariableId;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use std::collections::BTreeSet;

/// A "flat" configuration object for various reachability problems.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReachabilityConfig {
    /// The symbolic graph used for reachability computation. Use [`SymbolicAsyncGraph::restrict`]
    /// to restrict the reachability procedure only to a subset of graph vertices.
    ///
    /// # Panics
    ///
    /// The procedure is allowed to panic if initialized with vertices that do not belong
    /// to this graph.
    pub graph: SymbolicAsyncGraph,
    /// The set of variables that can be updated by the reachability procedure (default:
    /// all variables).
    ///
    /// # Panics
    ///
    /// The procedure is allowed to panic if this set contains variables not valid in
    /// the associated [`SymbolicAsyncGraph`].
    pub variables: BTreeSet<VariableId>,
    /// Cancel the procedure if it exceeds the specified number of iterations (default:
    /// `usize::MAX`).
    ///
    /// Note that the definition of "iteration" can depend on the chosen reachability operator.
    pub max_iterations: usize,
    /// Cancel the procedure if the symbolic representation exceeds the given amount of BDD nodes
    /// (default: `usize::MAX`).
    ///
    /// Note: In the future, this could be replaced by a global "symbolic size" cancellation
    /// trigger, but this will likely rely on direct support from the BDD library.
    pub max_symbolic_size: usize,
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
            max_iterations: usize::MAX,
            max_symbolic_size: usize::MAX,
        }
    }
}
