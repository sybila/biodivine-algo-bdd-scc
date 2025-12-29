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

pub type TrimSinks = ReachabilityComputation<IterativeSubtraction<RelativeSinks>>;
pub type TrimSources = ReachabilityComputation<IterativeSubtraction<RelativeSources>>;
pub type TrimSinksAndSources =
    ReachabilityComputation<IterativeSubtraction<RelativeSinksAndSources>>;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum TrimSetting {
    Both,
    Sources,
    Sinks,
    None,
}

impl TrimSetting {
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
