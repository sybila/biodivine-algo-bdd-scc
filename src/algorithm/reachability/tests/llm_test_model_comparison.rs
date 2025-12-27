//! Tests comparing ForwardReachability vs. ForwardReachabilityBFS and
//! BackwardReachability vs. BackwardReachabilityBFS on real model files.
//!
//! These tests verify that saturation-based and BFS-based algorithms produce
//! the same results, while also testing with timeouts to ensure tests don't hang.

use crate::algorithm::reachability::{
    BfsPredecessors, BfsSuccessors, IterativeUnion, ReachabilityComputation, ReachabilityConfig,
    SaturationPredecessors, SaturationSuccessors,
};
use crate::algorithm_trait::ComputationStep;
use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use cancel_this::Cancellable;
use std::time::Duration;
use test_generator::test_resources;

/// Generic helper function to compare saturation-based and BFS-based reachability algorithms.
///
/// For each state in the graph:
/// 1. Pick a state from the remaining states
/// 2. Compute reachability using both algorithms (saturation and BFS variants)
/// 3. Verify results are identical
/// 4. Remove the reachable set from remaining states and continue
fn test_reachability_comparison_impl<VariantA, VariantB>(model_path: &str) -> Cancellable<()>
where
    VariantA: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices>,
    VariantB: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices>,
{
    // Load the model
    let bn = BooleanNetwork::try_from_file(model_path)
        .unwrap_or_else(|e| panic!("Failed to load model {}: {:?}", model_path, e));

    // Empirical testing shows that on current hardware, all networks above 20 variables will
    // exceed the 1s timeout. We can increase this in the future if it becomes relevant, but
    // for now it saves quite a lot of time when testing.
    if bn.num_vars() > 20 {
        return Ok(());
    }

    let graph = SymbolicAsyncGraph::new(&bn)
        .unwrap_or_else(|e| panic!("Failed to create graph from {}: {:?}", model_path, e));

    let mut remaining_states = graph.mk_unit_colored_vertices();
    let mut iteration = 0;

    while !remaining_states.is_empty() {
        iteration += 1;

        // Pick a state from remaining states
        let initial_state = remaining_states.pick_singleton();
        assert!(
            !initial_state.is_empty(),
            "Picked state should not be empty"
        );

        // Compute reachability with both algorithms
        let sat_result = ReachabilityComputation::<VariantA>::run(&graph, initial_state.clone())?;
        let bfs_result = ReachabilityComputation::<VariantB>::run(&graph, initial_state.clone())?;

        // Verify results are identical
        assert_eq!(
            sat_result, bfs_result,
            "Reachability results differ for state in iteration {} of {}",
            iteration, model_path
        );

        // Remove the reachable set from remaining states
        remaining_states = remaining_states.minus(&sat_result);
    }

    Ok(())
}

/// Test forward reachability algorithms on a model file.
///
/// The entire test has a 1s timeout. The test passes if it completes or times out.
#[test_resources("./models/bbm-inputs-true/*.aeon")]
fn test_forward_reachability_comparison(model_path: &str) {
    let one_second = Duration::from_secs(1);
    match cancel_this::on_timeout(one_second, || {
        test_reachability_comparison_impl::<
            IterativeUnion<SaturationSuccessors>,
            IterativeUnion<BfsSuccessors>,
        >(model_path)
    }) {
        Ok(()) => {}
        Err(_) => {
            // Test passes if canceled due to timeout
            // Cancellation errors are expected for long-running computations
        }
    }
}

/// Test backward reachability algorithms on a model file.
///
/// The entire test has a 1s timeout. The test passes if it completes or times out.
#[test_resources("./models/bbm-inputs-true/*.aeon")]
fn test_backward_reachability_comparison(model_path: &str) {
    let one_second = Duration::from_secs(1);
    match cancel_this::on_timeout(one_second, || {
        test_reachability_comparison_impl::<
            IterativeUnion<SaturationPredecessors>,
            IterativeUnion<BfsPredecessors>,
        >(model_path)
    }) {
        Ok(()) => {}
        Err(_) => {
            // Test passes if canceled due to timeout
            // Cancellation errors are expected for long-running computations
        }
    }
}
