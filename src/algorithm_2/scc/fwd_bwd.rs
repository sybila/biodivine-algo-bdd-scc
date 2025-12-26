use crate::algorithm_2::log_set;
use crate::algorithm_2::reachability::{BackwardReachability, ForwardReachability};
use crate::algorithm_trait_2::Incomplete::Working;
use crate::algorithm_trait_2::{Completable, GeneratorStep};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use log::{debug, info};
use num_bigint::BigUint;

pub struct FwdBwdState {
    computing: Option<FwdBwdIterationState>,
    to_process: Vec<GraphColoredVertices>,
}

pub struct FwdBwdIterationState {
    universe: GraphColoredVertices,
    forward: ForwardReachability,
    backward: BackwardReachability,
}

pub struct FwdBwdStep;

impl FwdBwdIterationState {
    fn new(graph: &SymbolicAsyncGraph, value: GraphColoredVertices) -> Self {
        let pivot = value.pick_vertex();
        let graph = graph.restrict(&value);
        FwdBwdIterationState {
            forward: ForwardReachability::configure(graph.clone(), pivot.clone()),
            backward: BackwardReachability::configure(graph, pivot),
            universe: value,
        }
    }
}

impl From<&SymbolicAsyncGraph> for FwdBwdState {
    fn from(value: &SymbolicAsyncGraph) -> Self {
        FwdBwdState {
            computing: None,
            to_process: vec![value.mk_unit_colored_vertices()],
        }
    }
}

impl GeneratorStep<SymbolicAsyncGraph, FwdBwdState, GraphColoredVertices> for FwdBwdStep {
    fn step(
        context: &SymbolicAsyncGraph,
        state: &mut FwdBwdState,
    ) -> Completable<Option<GraphColoredVertices>> {
        if let Some(iteration) = state.computing.as_mut() {
            // If we are processing a specific component right now, continue the iteration.
            let backward = iteration.backward.try_compute()?;
            let forward = iteration.forward.try_compute()?;
            let scc = backward.intersect(forward);
            debug!("Extracted raw SCC ({})", log_set(&scc));

            // Enqueue the remaining states for further processing.
            let remaining_backward = backward.minus(forward);
            let remaining_forward = forward.minus(backward);
            let remaining_rest = iteration.universe.minus(backward).minus(forward);

            debug!(
                "Pushed remaining FWD ({}), BWD ({}), and REST ({}) sets.",
                log_set(&remaining_forward),
                log_set(&remaining_backward),
                log_set(&remaining_rest),
            );

            if !remaining_backward.is_empty() {
                state.to_process.push(remaining_backward);
            }
            if !remaining_forward.is_empty() {
                state.to_process.push(remaining_forward);
            }
            if !remaining_rest.is_empty() {
                state.to_process.push(remaining_rest);
            }

            // Remove colors where the SCC is a singleton state:
            let valid_colors = scc.minus(&scc.pick_vertex()).colors();
            let scc = scc.intersect_colors(&valid_colors);
            state.computing = None;
            if scc.is_empty() {
                // Iteration is done, but we have not found a new non-trivial SCC.
                debug!("The SCC is trivial.");
                Err(Working)
            } else {
                // Iteration is done, and we have a new non-trivial SCC.
                info!("Returning non-trivial SCC ({}).", log_set(&scc));
                Ok(Some(scc))
            }
        } else {
            // We are in-between iterations. We need to pick a new set for processing.
            // In theory, this choice does not matter because all sets need to be processed
            // eventually. However, since the algorithm can be also terminated early, it
            // is preferred to return as many SCCs as quickly as possible. As such, we always
            // pick the smallest set (in terms of BDD size).

            if state.to_process.is_empty() {
                // If there is nothing to process, we are done.
                return Ok(None);
            }

            let total_used = state
                .to_process
                .iter()
                .map(|it| it.symbolic_size())
                .sum::<usize>();
            let total_elements = state
                .to_process
                .iter()
                .map(|it| it.exact_cardinality())
                .sum::<BigUint>();
            state.to_process.sort_by_cached_key(|it| it.symbolic_size());
            state.to_process.reverse();

            info!(
                "{} sets remaining (elements={}; BDD nodes={})",
                state.to_process.len(),
                total_elements,
                total_used
            );

            let todo = state
                .to_process
                .pop()
                .expect("Correctness violation: Nothing to process");

            assert!(state.computing.is_none());
            state.computing = Some(FwdBwdIterationState::new(context, todo));
            Err(Working)
        }
    }
}
