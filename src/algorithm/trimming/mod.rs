use crate::Algorithm;
use crate::algorithm::reachability::{Reachability, ReachabilityState, StepOperator};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use cancel_this::{Cancellable, is_cancelled};
use log::{info, trace};
use std::any::type_name;

#[cfg(test)]
mod tests;

/// A variant of [`Trimming`] which eliminates all "local source states" from the initial set.
pub type TrimSources = Trimming<RelativeSources>;

/// A variant of [`Trimming`] which eliminates all "local sink states" from the initial set.
pub type TrimSinks = Trimming<RelativeSinks>;

/// Trimming is a helper algorithm that removes all sink/source states from a set by translating
/// the problem into reachability.
///
/// A common use of this procedure is to remove trivial source/sink SCCs before searching for
/// complex cycles.
pub struct Trimming<S: StepOperator>(Reachability<S>);

impl<S: StepOperator> Algorithm for Trimming<S> {
    type State = ReachabilityState;
    type Output = GraphColoredVertices;

    fn create<T: Into<Self::State>>(initial_state: T) -> Self
    where
        Self: Sized,
    {
        let mut state: ReachabilityState = initial_state.into();

        info!(
            "Initializing trimming computation (elements=`{}`; BDD nodes=`{}`; defers to `{}` reachability).",
            state.set.exact_cardinality(),
            state.set.symbolic_size(),
            type_name::<S>().split("::").last().unwrap_or("?"),
        );

        // Invert the initial set
        state.set = state.graph.mk_unit_colored_vertices().minus(&state.set);
        Trimming(Reachability::create(state))
    }

    fn advance(&mut self) -> Cancellable<Option<Self::Output>> {
        self.0.advance().map(|result| {
            result.map(|output| {
                // Invert the output set
                let output = self
                    .0
                    .state()
                    .graph
                    .mk_unit_colored_vertices()
                    .minus(&output);

                info!(
                    "Trimmed set computed with {} elements (BDD nodes=`{}`).",
                    output.exact_cardinality(),
                    output.symbolic_size()
                );

                output
            })
        })
    }
}

/// [`RelativeSources`] is a special [`StepOperator`] intended for extending the initial set with
/// trivial "connecting components" from the remaining set of states.
///
/// It returns the set of states that are not in the current set `X`, but are "sources" within
/// the remaining states `Y` (complement of `X`). This means they do not have an incoming
/// regulation from within `Y` itself (they can still have an incoming regulation from within
/// `X`, but this is not required).
///
/// In particular, such "source states" cannot cross SCC boundaries, i.e. adding them to an
/// initial set can never add a new complex SCC to the set. Furthermore, note that these
/// source states do not need to be connected to the initial set by transitions. In particular,
/// any "global" source state outside of the initial set is always added in the first iteration.
pub struct RelativeSources;

/// Same as [`RelativeSources`], but produces sinks, i.e. states with no outgoing transition
/// within the candidate set.
pub struct RelativeSinks;

impl StepOperator for RelativeSources {
    fn step(state: &ReachabilityState) -> Cancellable<GraphColoredVertices> {
        let candidates = state.graph.mk_unit_colored_vertices().minus(&state.set);
        // A "source" is a state that has no predecessors within the candidate set.
        let mut sources = candidates.clone();
        for var in state.variables.iter().rev() {
            is_cancelled!()?;
            let has_predecessor = state.graph.var_can_pre_within(*var, &candidates);
            let new_sources = sources.minus(&has_predecessor);
            if new_sources != sources {
                sources = new_sources;
                trace!(
                    "Source states updated using `{}` to `{}` elements (`{}` BDD nodes).",
                    var,
                    sources.exact_cardinality(),
                    sources.symbolic_size()
                );
            }
        }

        Ok(sources)
    }
}

impl StepOperator for RelativeSinks {
    fn step(state: &ReachabilityState) -> Cancellable<GraphColoredVertices> {
        let candidates = state.graph.mk_unit_colored_vertices().minus(&state.set);
        // A "sink" is a state that has no successor within the candidate set.
        let mut sinks = candidates.clone();
        for var in state.variables.iter().rev() {
            is_cancelled!()?;
            let has_successor = state.graph.var_can_post_within(*var, &candidates);
            let new_sinks = sinks.minus(&has_successor);
            if new_sinks != sinks {
                sinks = new_sinks;
                trace!(
                    "Sink states updated using `{}` to `{}` elements (`{}` BDD nodes).",
                    var,
                    sinks.exact_cardinality(),
                    sinks.symbolic_size()
                );
            }
        }

        Ok(sinks)
    }
}
