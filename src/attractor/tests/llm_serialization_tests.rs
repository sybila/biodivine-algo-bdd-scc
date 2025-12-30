//! Tests for serialization of attractor algorithms.
//!
//! These tests verify that ITGR and XieBeerelAttractors computations can be serialized
//! and deserialized mid-execution, and that deserialized computations can be resumed
//! to produce the same results as uninterrupted computations.

use crate::attractor::{
    AttractorConfig, InterleavedTransitionGuidedReduction, ItgrState, XieBeerelAttractors,
    XieBeerelState,
};
use crate::test_utils::llm_example_network::create_test_network;
use crate::test_utils::llm_example_network::sets::{ATTRACTOR_1, ATTRACTOR_2};
use crate::test_utils::{init_logger, mk_states};
use cancel_this::Cancellable;
use computation_process::{Algorithm, Computable, Stateful};
use serde_json;

// ========== Helper functions ==========

/// Test that an ITGR computation can be serialized and deserialized mid-execution,
/// and that the deserialized computation produces the same result when resumed.
fn test_itgr_serialization_roundtrip_impl() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let universe = graph.mk_unit_colored_vertices();

    // Create an ITGR computation
    let config = AttractorConfig::from(&graph);
    let itgr_state = ItgrState::new(&graph, &universe);
    let mut itgr = InterleavedTransitionGuidedReduction::configure(config.clone(), itgr_state);

    // Run for a few steps using try_compute
    for _ in 0..5 {
        match itgr.try_compute() {
            Ok(_) => {
                // Computation completed
                break;
            }
            Err(_) => {
                // Computation suspended - continue
            }
        }
    }

    // Serialize the computation
    let json = serde_json::to_string(&itgr).expect("Failed to serialize ITGR Computation");

    // Deserialize the computation
    let mut deserialized_itgr: InterleavedTransitionGuidedReduction =
        serde_json::from_str(&json).expect("Failed to deserialize ITGR Computation");

    // Resume the deserialized computation to completion
    let deserialized_result = deserialized_itgr.compute()?;

    // Run a fresh computation to completion for comparison
    let fresh_state = ItgrState::new(&graph, &universe);
    let fresh_result = InterleavedTransitionGuidedReduction::run(config, fresh_state)?;

    // Results should be identical
    assert_eq!(
        deserialized_result, fresh_result,
        "Deserialized ITGR computation should produce the same result as a fresh computation"
    );

    Ok(())
}

/// Test serialization/deserialization with ITGR that requires multiple iterations.
fn test_itgr_serialization_multiple_steps_impl() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let universe = graph.mk_unit_colored_vertices();

    // Create an ITGR computation
    let config = AttractorConfig::new(graph.clone());
    let itgr_state = ItgrState::new(&graph, &universe);
    let mut itgr = InterleavedTransitionGuidedReduction::configure(config.clone(), itgr_state);

    // Execute a few steps
    for _ in 0..3 {
        let _ = itgr.try_compute();
    }

    // Serialize after the first few steps
    let json_step1 =
        serde_json::to_string(&itgr).expect("Failed to serialize ITGR Computation after step 1");

    // Deserialize
    let mut deserialized_step1: InterleavedTransitionGuidedReduction =
        serde_json::from_str(&json_step1)
            .expect("Failed to deserialize ITGR Computation after step 1");

    // Continue execution
    for _ in 0..5 {
        match deserialized_step1.try_compute() {
            Ok(_) => break,
            Err(_) => continue,
        }
    }

    // Serialize again after more steps
    let json_step2 = serde_json::to_string(&deserialized_step1)
        .expect("Failed to serialize ITGR Computation after step 2");

    // Deserialize again
    let mut deserialized_step2: InterleavedTransitionGuidedReduction =
        serde_json::from_str(&json_step2)
            .expect("Failed to deserialize ITGR Computation after step 2");

    // Complete the computation
    let final_result = deserialized_step2.compute()?;

    // Compare with a fresh computation
    let fresh_state = ItgrState::new(&graph, &universe);
    let fresh_result = InterleavedTransitionGuidedReduction::run(config, fresh_state)?;

    assert_eq!(
        final_result, fresh_result,
        "ITGR computation serialized/deserialized multiple times should produce the same result"
    );

    Ok(())
}

/// Test that a XieBeerelAttractors generator can be serialized and deserialized mid-execution,
/// and that the deserialized generator produces the same attractors when resumed.
fn test_xie_beerel_serialization_roundtrip_impl() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();

    // Create a generator
    let config = AttractorConfig::new(graph.clone());
    let initial_state = XieBeerelState::from(&graph);
    let mut generator = XieBeerelAttractors::configure(config.clone(), initial_state);

    // Get the first attractor (if any)
    let mut first_attractor = None;
    for result in &mut generator {
        match result {
            Ok(attr) => {
                first_attractor = Some(attr);
                break;
            }
            Err(_) => {
                // Continue
            }
        }
    }

    // Serialize the generator
    let json = serde_json::to_string(&generator)
        .expect("Failed to serialize XieBeerelAttractors Generator");

    // Deserialize the generator
    let deserialized_generator: XieBeerelAttractors =
        serde_json::from_str(&json).expect("Failed to deserialize XieBeerelAttractors Generator");

    // Collect all remaining attractors from the deserialized generator
    let mut deserialized_attractors = Vec::new();
    if let Some(attr) = first_attractor {
        deserialized_attractors.push(attr);
    }
    for result in deserialized_generator {
        match result {
            Ok(attr) => deserialized_attractors.push(attr),
            Err(_) => {
                // Continue
            }
        }
    }

    // Run a fresh generator to completion for comparison
    let fresh_initial_state = XieBeerelState::from(&graph);
    let fresh_generator = XieBeerelAttractors::configure(config, fresh_initial_state);
    let mut fresh_attractors = Vec::new();
    for result in fresh_generator {
        match result {
            Ok(attr) => fresh_attractors.push(attr),
            Err(_) => {
                // Continue
            }
        }
    }

    // Results should be identical (same attractors, possibly in different order)
    assert_eq!(
        deserialized_attractors.len(),
        fresh_attractors.len(),
        "Deserialized generator should produce the same number of attractors"
    );

    // Verify we get the expected attractors
    let attractor_1 = mk_states(&graph, ATTRACTOR_1);
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);
    assert!(
        deserialized_attractors
            .iter()
            .any(|attr| attr == &attractor_1),
        "Should find attractor 1 (fixed point {{000}})"
    );
    assert!(
        deserialized_attractors
            .iter()
            .any(|attr| attr == &attractor_2),
        "Should find attractor 2 (cycle {{110, 111}})"
    );

    Ok(())
}

/// Test serialization/deserialization with XieBeerelAttractors that requires multiple iterations.
fn test_xie_beerel_serialization_multiple_steps_impl() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();

    // Create a generator
    let config = AttractorConfig::new(graph.clone());
    let initial_state = XieBeerelState::from(&graph);
    let mut generator = XieBeerelAttractors::configure(config.clone(), initial_state);

    // Get one attractor
    let mut first_attractor = None;
    for result in &mut generator {
        match result {
            Ok(attr) => {
                first_attractor = Some(attr);
                break;
            }
            Err(_) => {
                // Continue
            }
        }
    }

    // Serialize after the first attractor
    let json_step1 = serde_json::to_string(&generator)
        .expect("Failed to serialize XieBeerelAttractors Generator after step 1");

    // Deserialize
    let mut deserialized_step1: XieBeerelAttractors = serde_json::from_str(&json_step1)
        .expect("Failed to deserialize XieBeerelAttractors Generator after step 1");

    // Continue and get more attractors
    let mut collected_attractors = Vec::new();
    if let Some(attr) = first_attractor {
        collected_attractors.push(attr);
    }
    for result in &mut deserialized_step1 {
        match result {
            Ok(attr) => {
                collected_attractors.push(attr);
                break; // Get one more
            }
            Err(_) => {
                // Continue
            }
        }
    }

    // Serialize again after more steps
    let json_step2 = serde_json::to_string(&deserialized_step1)
        .expect("Failed to serialize XieBeerelAttractors Generator after step 2");

    // Deserialize again
    let deserialized_step2: XieBeerelAttractors = serde_json::from_str(&json_step2)
        .expect("Failed to deserialize XieBeerelAttractors Generator after step 2");

    // Collect all remaining attractors
    for result in deserialized_step2 {
        match result {
            Ok(attr) => collected_attractors.push(attr),
            Err(_) => {
                // Continue
            }
        }
    }

    // Compare with a fresh generator
    let fresh_initial_state = XieBeerelState::from(&graph);
    let fresh_generator = XieBeerelAttractors::configure(config, fresh_initial_state);
    let mut fresh_attractors = Vec::new();
    for result in fresh_generator {
        match result {
            Ok(attr) => fresh_attractors.push(attr),
            Err(_) => {
                // Continue
            }
        }
    }

    assert_eq!(
        collected_attractors.len(),
        fresh_attractors.len(),
        "XieBeerelAttractors generator serialized/deserialized multiple times should produce the same number of attractors"
    );

    // Verify we get the expected attractors
    let attractor_1 = mk_states(&graph, ATTRACTOR_1);
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);
    assert!(
        collected_attractors.iter().any(|attr| attr == &attractor_1),
        "Should find attractor 1 (fixed point {{000}})"
    );
    assert!(
        collected_attractors.iter().any(|attr| attr == &attractor_2),
        "Should find attractor 2 (cycle {{110, 111}})"
    );

    Ok(())
}

// ========== Tests for ITGR ==========

#[test]
fn test_itgr_serialization_roundtrip() -> Cancellable<()> {
    test_itgr_serialization_roundtrip_impl()
}

#[test]
fn test_itgr_serialization_multiple_steps() -> Cancellable<()> {
    test_itgr_serialization_multiple_steps_impl()
}

// ========== Tests for XieBeerelAttractors ==========

#[test]
fn test_xie_beerel_serialization_roundtrip() -> Cancellable<()> {
    test_xie_beerel_serialization_roundtrip_impl()
}

#[test]
fn test_xie_beerel_serialization_multiple_steps() -> Cancellable<()> {
    test_xie_beerel_serialization_multiple_steps_impl()
}
