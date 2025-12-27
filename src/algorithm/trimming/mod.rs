mod iterative_subtraction;
mod step_operators;

#[cfg(test)]
mod tests;

use crate::algorithm::reachability::ReachabilityComputation;
pub use iterative_subtraction::IterativeSubtraction;
pub use step_operators::{RelativeSinks, RelativeSources};

pub type TrimSinks = ReachabilityComputation<IterativeSubtraction<RelativeSinks>>;
pub type TrimSources = ReachabilityComputation<IterativeSubtraction<RelativeSources>>;
