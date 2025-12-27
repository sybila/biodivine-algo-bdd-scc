use crate::algorithm::trimming::TrimSetting;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;

/// A configuration object for various reachability problems.
#[derive(Clone)]
pub struct SccConfig {
    /// The graph used for SCC computation.
    pub graph: SymbolicAsyncGraph,
    /// Indicate that the algorithm should try to trim trivial components (default: both).
    pub should_trim: TrimSetting,
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
            should_trim: TrimSetting::Both,
        }
    }

    /// Set trimming of trivial components (none/sinks/sources/both; default: both).
    pub fn should_trim(mut self, should_trim: TrimSetting) -> Self {
        self.should_trim = should_trim;
        self
    }
}
