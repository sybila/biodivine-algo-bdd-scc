mod iterative_subtraction;
mod step_operators;

#[cfg(test)]
mod tests;

use crate::algorithm::reachability::ReachabilityComputation;
use crate::algorithm::trimming::step_operators::RelativeSinksAndSources;
use crate::algorithm_trait::{DynComputable, IdentityComputation};
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
pub use iterative_subtraction::IterativeSubtraction;
pub use step_operators::{RelativeSinks, RelativeSources};

pub type TrimSinks = ReachabilityComputation<IterativeSubtraction<RelativeSinks>>;
pub type TrimSources = ReachabilityComputation<IterativeSubtraction<RelativeSources>>;
pub type TrimSinksAndSources =
    ReachabilityComputation<IterativeSubtraction<RelativeSinksAndSources>>;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
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
            TrimSetting::Both => Box::new(TrimSinksAndSources::configure(graph, set)),
            TrimSetting::Sources => Box::new(TrimSources::configure(graph, set)),
            TrimSetting::Sinks => Box::new(TrimSinks::configure(graph, set)),
            TrimSetting::None => Box::new(IdentityComputation::<GraphColoredVertices>::configure(
                (),
                set,
            )),
        }
    }
}
