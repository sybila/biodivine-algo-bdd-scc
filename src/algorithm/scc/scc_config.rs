use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;

/// A configuration object for various reachability problems.
pub struct SccConfig {
    /// The graph used for SCC computation.
    pub graph: SymbolicAsyncGraph,
    /// Indicate that the algorithm should try to trim trivial components (default: true).
    pub should_trim: bool,
}

impl From<SymbolicAsyncGraph> for SccConfig {
    fn from(value: SymbolicAsyncGraph) -> Self {
        SccConfig::new(value)
    }
}

impl From<&SymbolicAsyncGraph> for SccConfig {
    fn from(value: &SymbolicAsyncGraph) -> Self {
        SccConfig::new(value.clone())
    }
}

impl SccConfig {
    /// Create a new instance of [`SccConfig`] from a [`SymbolicAsyncGraph`]
    /// with trimming enabled.
    pub fn new(graph: SymbolicAsyncGraph) -> SccConfig {
        SccConfig {
            graph,
            should_trim: true,
        }
    }

    /// Enabled/disable trimming of trivial components (default: enabled).
    pub fn should_trim(mut self, should_trim: bool) -> Self {
        self.should_trim = should_trim;
        self
    }
}
