//! Trimming algorithms for removing trivial states from Boolean network state spaces.
//!
//! Trimming removes states that cannot be part of non-trivial SCCs:
//!
//! - **Sinks**: States with no successors within the remaining set
//! - **Sources**: States with no predecessors within the remaining set
//!
//! Trimming is applied iteratively until a fixed point is reached, as removing
//! one layer of sinks/sources may expose new ones.
//!
//! # Algorithms
//!
//! - [`TrimSinks`]: Iteratively remove sink states
//! - [`TrimSources`]: Iteratively remove source states
//! - [`TrimSinksAndSources`]: Remove both (more efficient than separate passes)
//!
//! # Configuration
//!
//! Use [`TrimSetting`] to select which trimming strategy to apply:
//!
//! - `TrimSetting::Both` (default): Trim both sinks and sources
//! - `TrimSetting::Sinks`: Only trim sinks
//! - `TrimSetting::Sources`: Only trim sources
//! - `TrimSetting::None`: Skip trimming entirely

mod iterative_subtraction;
mod step_operators;

#[cfg(test)]
mod tests;

use crate::reachability::ReachabilityComputation;
use crate::trimming::step_operators::RelativeSinksAndSources;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use computation_process::{Computable, ComputableIdentity, DynComputable, Stateful};
pub use iterative_subtraction::IterativeSubtraction;
pub use step_operators::{RelativeSinks, RelativeSources};

/// Trimming algorithm that iteratively removes sink states.
pub type TrimSinks = ReachabilityComputation<IterativeSubtraction<RelativeSinks>>;

/// Trimming algorithm that iteratively removes source states.
pub type TrimSources = ReachabilityComputation<IterativeSubtraction<RelativeSources>>;

/// Trimming algorithm that iteratively removes both sink and source states.
pub type TrimSinksAndSources =
    ReachabilityComputation<IterativeSubtraction<RelativeSinksAndSources>>;

/// Configuration for trimming behavior during SCC computation.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum TrimSetting {
    /// Trim both sinks and sources (default).
    Both,
    /// Only trim source states (no predecessors).
    Sources,
    /// Only trim sink states (no successors).
    Sinks,
    /// Skip trimming entirely.
    None,
}

impl TrimSetting {
    /// Build a trimming computation based on the current setting.
    pub fn build_computation(
        &self,
        graph: &SymbolicAsyncGraph,
        set: GraphColoredVertices,
    ) -> DynComputable<GraphColoredVertices> {
        match self {
            TrimSetting::Both => TrimSinksAndSources::configure(graph, set).dyn_computable(),
            TrimSetting::Sources => TrimSources::configure(graph, set).dyn_computable(),
            TrimSetting::Sinks => TrimSinks::configure(graph, set).dyn_computable(),
            TrimSetting::None => ComputableIdentity::from(set).dyn_computable(),
        }
    }
}
