mod step_operators;

#[cfg(test)]
mod llm_tests;

use crate::reachability::ReachabilityComputation;
use crate::trimming::IterativeSubtraction;
pub use step_operators::{HasPredecessorSaturation, HasSuccessorSaturation};

/// A type alias for a forward trap set computation (using saturation update).
///
/// Forward trap set is the greatest forward-closed subset of the initial set.
pub type ForwardTrap = ReachabilityComputation<IterativeSubtraction<HasSuccessorSaturation>>;

/// A type alias for a backward trap set computation (using saturation update).
///
/// Backward trap set is the greatest backward-closed subset of the initial set.
pub type BackwardTrap = ReachabilityComputation<IterativeSubtraction<HasPredecessorSaturation>>;
