use crate::Algorithm;
use crate::algorithm::reachability::{ReachabilityState, StepOperator};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use cancel_this::Cancellable;
use log::{debug, info};
use std::any::type_name;
use std::marker::PhantomData;

/// A generic structure responsible for computing reachability operations by iteratively
/// computing successors/predecessors and adding them to the reachable set until
/// a fixed-point is reached.
pub struct Reachability<S: StepOperator> {
    state: ReachabilityState,
    _phantom: PhantomData<S>,
}

impl<S: StepOperator> Algorithm for Reachability<S> {
    type State = ReachabilityState;
    type Output = GraphColoredVertices;

    fn create<T: Into<Self::State>>(initial_state: T) -> Self
    where
        Self: Sized,
    {
        let initial_state: ReachabilityState = initial_state.into();
        info!(
            "Initializing reachability (step function=`{}`; elements=`{}`; BDD nodes=`{}`).",
            type_name::<S>().split("::").last().unwrap_or("?"),
            initial_state.set.exact_cardinality(),
            initial_state.set.symbolic_size()
        );
        Reachability {
            state: initial_state,
            _phantom: PhantomData,
        }
    }

    fn advance(&mut self) -> Cancellable<Option<Self::Output>> {
        let next_set = S::step(&self.state)?;
        if next_set.is_subset(&self.state.set) {
            info!(
                "Reachability finished with {} elements (BDD nodes=`{}`).",
                self.state.set.exact_cardinality(),
                self.state.set.symbolic_size()
            );
            Ok(Some(self.state.set.clone()))
        } else {
            self.state.set = self.state.set.union(&next_set);
            debug!(
                "Reachability increased to {} elements (BDD nodes=`{}`).",
                self.state.set.exact_cardinality(),
                self.state.set.symbolic_size()
            );
            Ok(None)
        }
    }
}

impl<S: StepOperator> Reachability<S> {
    /// Accessor for the internal reachability state.
    pub fn state(&self) -> &ReachabilityState {
        &self.state
    }

    /// Mutating accessor for the internal reachability state.
    pub fn state_mut(&mut self) -> &mut ReachabilityState {
        &mut self.state
    }
}
