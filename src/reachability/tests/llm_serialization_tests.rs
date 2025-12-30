//! Tests for serialization of reachability algorithms.
//!
//! These tests verify that reachability `Computation` objects can be serialized
//! and deserialized mid-execution, and that deserialized computations can be
//! resumed to produce the same results as uninterrupted computations.

use crate::reachability::{
    BfsPredecessors, BfsSuccessors, IterativeUnion, ReachabilityComputation, ReachabilityConfig,
    ReachabilityState, SaturationPredecessors, SaturationSuccessors,
};
use crate::test_utils::llm_example_network::create_test_network;
use crate::test_utils::llm_example_network::states::*;
use crate::test_utils::{init_logger, mk_state};
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use cancel_this::Cancellable;
use computation_process::{Algorithm, Computable, ComputationStep, Stateful};
use serde_json;

// ========== Helper functions ==========

/// Test that a reachability computation can be serialized and deserialized mid-execution,
/// and that the deserialized computation produces the same result when resumed.
fn test_computation_serialization_roundtrip_impl<STEP>() -> Cancellable<()>
where
    STEP: ComputationStep<ReachabilityConfig, ReachabilityState, GraphColoredVertices> + 'static,
    ReachabilityComputation<STEP>:
        Algorithm<ReachabilityConfig, ReachabilityState, GraphColoredVertices>,
{
    init_logger();
    let graph = create_test_network();
    let initial = mk_state(&graph, S010);

    // Create a computation
    let config = ReachabilityConfig::new(graph.clone());
    let mut computation = <ReachabilityComputation<STEP> as Stateful<
        ReachabilityConfig,
        ReachabilityState,
    >>::configure(config.clone(), initial.clone());

    // Run for a few steps using try_compute
    // This will partially execute the computation
    for _ in 0..3 {
        match computation.try_compute() {
            Ok(_) => {
                // Computation completed - this is fine, we can still test serialization
            }
            Err(_) => {
                // Computation suspended - continue
            }
        }
    }

    // Serialize the computation
    let json = serde_json::to_string(&computation).expect("Failed to serialize Computation");

    // Deserialize the computation
    let mut deserialized_computation: ReachabilityComputation<STEP> =
        serde_json::from_str(&json).expect("Failed to deserialize Computation");

    // Resume the deserialized computation to completion
    let deserialized_result = loop {
        match deserialized_computation.try_compute() {
            Ok(result) => break result,
            Err(_) => {
                // Continue until completion
            }
        }
    };

    // Run a fresh computation to completion for comparison
    let mut fresh_computation = <ReachabilityComputation<STEP> as Stateful<
        ReachabilityConfig,
        ReachabilityState,
    >>::configure(config, initial);
    let fresh_result = loop {
        match fresh_computation.try_compute() {
            Ok(result) => break result,
            Err(_) => {
                // Continue until completion
            }
        }
    };

    // Results should be identical
    assert_eq!(
        deserialized_result, fresh_result,
        "Deserialized computation should produce the same result as a fresh computation"
    );

    Ok(())
}

/// Test serialization/deserialization with a computation that requires multiple iterations.
fn test_computation_serialization_multiple_steps_impl<STEP>() -> Cancellable<()>
where
    STEP: ComputationStep<ReachabilityConfig, ReachabilityState, GraphColoredVertices> + 'static,
    ReachabilityComputation<STEP>: Algorithm<ReachabilityConfig, ReachabilityState, GraphColoredVertices>
        + Stateful<ReachabilityConfig, ReachabilityState>,
{
    init_logger();
    let graph = create_test_network();
    let initial = mk_state(&graph, S010);

    // Create a computation
    let config = ReachabilityConfig::new(graph.clone());
    let mut computation = <ReachabilityComputation<STEP> as Stateful<
        ReachabilityConfig,
        ReachabilityState,
    >>::configure(config.clone(), initial.clone());

    // Execute one step
    let step1_result = computation.try_compute();
    let was_suspended_after_step1 = step1_result.is_err();

    // Serialize after the first step
    let json_step1 =
        serde_json::to_string(&computation).expect("Failed to serialize Computation after step 1");

    // Deserialize
    let mut deserialized_step1: ReachabilityComputation<STEP> =
        serde_json::from_str(&json_step1).expect("Failed to deserialize Computation after step 1");

    // Continue execution
    if was_suspended_after_step1 {
        // Continue from where we left off
        let _ = deserialized_step1.try_compute();
    }

    // Execute a few more steps
    for _ in 0..5 {
        match deserialized_step1.try_compute() {
            Ok(_) => break,
            Err(_) => continue,
        }
    }

    // Serialize again after more steps
    let json_step2 = serde_json::to_string(&deserialized_step1)
        .expect("Failed to serialize Computation after step 2");

    // Deserialize again
    let mut deserialized_step2: ReachabilityComputation<STEP> =
        serde_json::from_str(&json_step2).expect("Failed to deserialize Computation after step 2");

    // Complete the computation
    let final_result = loop {
        match deserialized_step2.try_compute() {
            Ok(result) => break result,
            Err(_) => {
                // Continue until completion
            }
        }
    };

    // Compare with a fresh computation
    let mut fresh_computation = <ReachabilityComputation<STEP> as Stateful<
        ReachabilityConfig,
        ReachabilityState,
    >>::configure(config, initial);
    let fresh_result = loop {
        match fresh_computation.try_compute() {
            Ok(result) => break result,
            Err(_) => {
                // Continue until completion
            }
        }
    };

    assert_eq!(
        final_result, fresh_result,
        "Computation serialized/deserialized multiple times should produce the same result"
    );

    Ok(())
}

// ========== Tests for ForwardReachability (Saturation) ==========

#[test]
fn test_computation_serialization_roundtrip_forward_sat() -> Cancellable<()> {
    test_computation_serialization_roundtrip_impl::<IterativeUnion<SaturationSuccessors>>()
}

#[test]
fn test_computation_serialization_multiple_steps_forward_sat() -> Cancellable<()> {
    test_computation_serialization_multiple_steps_impl::<IterativeUnion<SaturationSuccessors>>()
}

// ========== Tests for ForwardReachabilityBfs ==========

#[test]
fn test_computation_serialization_roundtrip_forward_bfs() -> Cancellable<()> {
    test_computation_serialization_roundtrip_impl::<IterativeUnion<BfsSuccessors>>()
}

#[test]
fn test_computation_serialization_multiple_steps_forward_bfs() -> Cancellable<()> {
    test_computation_serialization_multiple_steps_impl::<IterativeUnion<BfsSuccessors>>()
}

// ========== Tests for BackwardReachability (Saturation) ==========

#[test]
fn test_computation_serialization_roundtrip_backward_sat() -> Cancellable<()> {
    test_computation_serialization_roundtrip_impl::<IterativeUnion<SaturationPredecessors>>()
}

#[test]
fn test_computation_serialization_multiple_steps_backward_sat() -> Cancellable<()> {
    test_computation_serialization_multiple_steps_impl::<IterativeUnion<SaturationPredecessors>>()
}

// ========== Tests for BackwardReachabilityBfs ==========

#[test]
fn test_computation_serialization_roundtrip_backward_bfs() -> Cancellable<()> {
    test_computation_serialization_roundtrip_impl::<IterativeUnion<BfsPredecessors>>()
}

#[test]
fn test_computation_serialization_multiple_steps_backward_bfs() -> Cancellable<()> {
    test_computation_serialization_multiple_steps_impl::<IterativeUnion<BfsPredecessors>>()
}
