use crate::scc::retain_long_lived;
use crate::trimming::TrimSetting;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

/// A configuration object for various SCC detection problems.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SccConfig {
    /// The graph used for SCC computation. You can restrict this graph to a subset
    /// of vertices using [`SymbolicAsyncGraph::restrict`], but keep in mind that this also
    /// eliminates the associated transitions, potentially creating new "fake" SCCs
    /// unless the new restriction is SCC-closed in the original graph.
    ///
    /// If you are only interested in a subset of SCCs, you want to instead limit the
    /// initial set used by the algorithm.
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
            should_trim: TrimSetting::default(),
            filter_long_lived: false,
        }
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
