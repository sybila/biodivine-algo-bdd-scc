use crate::algorithm::scc::retain_long_lived;
use crate::algorithm::trimming::TrimSetting;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

/// A configuration object for various reachability problems.
#[derive(Clone)]
pub struct SccConfig {
    /// The graph used for SCC computation.
    pub graph: SymbolicAsyncGraph,
    /// Indicate that the algorithm should try to trim trivial components (default: both).
    pub should_trim: TrimSetting,
    /// Indicate that only long-lived components should be reported.
    ///
    /// A component is long-lived if it cannot be escaped by updating a single variable.
    pub filter_long_lived: bool,
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
            filter_long_lived: false,
        }
    }

    /// Set trimming of trivial components (none/sinks/sources/both; default: both).
    pub fn should_trim(mut self, should_trim: TrimSetting) -> Self {
        self.should_trim = should_trim;
        self
    }

    /// Enable/disable long lived filtering (default: false).
    pub fn filter_long_lived(mut self, filter: bool) -> Self {
        self.filter_long_lived = filter;
        self
    }

    /// If long-lived filtering is enabled, apply it. Otherwise, return the same set.
    pub fn apply_long_lived_filter(
        &self,
        set: &GraphColoredVertices,
    ) -> Option<GraphColoredVertices> {
        let filtered = if self.filter_long_lived {
            retain_long_lived(&self.graph, set)
        } else {
            set.clone()
        };

        if filtered.is_empty() {
            None
        } else {
            Some(filtered)
        }
    }
}
