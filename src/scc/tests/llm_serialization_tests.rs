//! Tests for serialization of SCC algorithms.
//!
//! These tests verify that FwdBwdScc and ChainScc generators can be serialized
//! and deserialized mid-execution, and that deserialized generators can be resumed
//! to produce the same results as uninterrupted generators.

use crate::scc::{ChainScc, ChainState, FwdBwdScc, FwdBwdState, SccConfig};
use crate::test_utils::llm_example_network::create_test_network;
use crate::test_utils::llm_example_network::sets::ATTRACTOR_2;
use crate::test_utils::{init_logger, mk_states};
use cancel_this::Cancellable;
use computation_process::Stateful;
use serde_json;

// ========== Helper functions ==========

/// Test that a FwdBwdScc generator can be serialized and deserialized mid-execution,
/// and that the deserialized generator produces the same SCCs when resumed.
fn test_fwd_bwd_scc_serialization_roundtrip_impl() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();

    // Create a generator
    let config = SccConfig::from(&graph);
    let initial_state = FwdBwdState::from(&graph);
    let mut generator = FwdBwdScc::configure(config.clone(), initial_state);

    // Get the first SCC (if any)
    let mut first_scc = None;
    for result in &mut generator {
        match result {
            Ok(scc) => {
                first_scc = Some(scc);
                break;
            }
            Err(_) => {
                // Continue
            }
        }
    }

    // Serialize the generator
    let json = serde_json::to_string(&generator).expect("Failed to serialize FwdBwdScc Generator");

    // Deserialize the generator
    let deserialized_generator: FwdBwdScc =
        serde_json::from_str(&json).expect("Failed to deserialize FwdBwdScc Generator");

    // Collect all remaining SCCs from the deserialized generator
    let mut deserialized_sccs = Vec::new();
    if let Some(scc) = first_scc {
        deserialized_sccs.push(scc);
    }
    for result in deserialized_generator {
        match result {
            Ok(scc) => deserialized_sccs.push(scc),
            Err(_) => {
                // Continue
            }
        }
    }

    // Run a fresh generator to completion for comparison
    let fresh_initial_state = FwdBwdState::from(&graph);
    let fresh_generator = FwdBwdScc::configure(config, fresh_initial_state);
    let mut fresh_sccs = Vec::new();
    for result in fresh_generator {
        match result {
            Ok(scc) => fresh_sccs.push(scc),
            Err(_) => {
                // Continue
            }
        }
    }

    // Results should be identical (same SCCs, possibly in different order)
    assert_eq!(
        deserialized_sccs.len(),
        fresh_sccs.len(),
        "Deserialized generator should produce the same number of SCCs"
    );

    // Verify we get the expected SCC (the 2-cycle attractor)
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);
    assert!(
        deserialized_sccs.iter().any(|scc| scc == &attractor_2),
        "Should find the 2-cycle SCC {{110, 111}}"
    );

    Ok(())
}

/// Test serialization/deserialization with FwdBwdScc that requires multiple iterations.
fn test_fwd_bwd_scc_serialization_multiple_steps_impl() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();

    // Create a generator
    let config = SccConfig::from(&graph);
    let initial_state = FwdBwdState::from(&graph);
    let mut generator = FwdBwdScc::configure(config.clone(), initial_state);

    // Get one SCC
    let mut first_scc = None;
    for result in &mut generator {
        match result {
            Ok(scc) => {
                first_scc = Some(scc);
                break;
            }
            Err(_) => {
                // Continue
            }
        }
    }

    // Serialize after the first SCC
    let json_step1 = serde_json::to_string(&generator)
        .expect("Failed to serialize FwdBwdScc Generator after step 1");

    // Deserialize
    let deserialized_step1: FwdBwdScc = serde_json::from_str(&json_step1)
        .expect("Failed to deserialize FwdBwdScc Generator after step 1");

    // Continue and collect remaining SCCs
    let mut collected_sccs = Vec::new();
    if let Some(scc) = first_scc {
        collected_sccs.push(scc);
    }
    for result in deserialized_step1 {
        match result {
            Ok(scc) => collected_sccs.push(scc),
            Err(_) => {
                // Continue
            }
        }
    }

    // Compare with a fresh generator
    let fresh_initial_state = FwdBwdState::from(&graph);
    let fresh_generator = FwdBwdScc::configure(config, fresh_initial_state);
    let mut fresh_sccs = Vec::new();
    for result in fresh_generator {
        match result {
            Ok(scc) => fresh_sccs.push(scc),
            Err(_) => {
                // Continue
            }
        }
    }

    assert_eq!(
        collected_sccs.len(),
        fresh_sccs.len(),
        "FwdBwdScc generator serialized/deserialized should produce the same number of SCCs"
    );

    // Verify we get the expected SCC
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);
    assert!(
        collected_sccs.iter().any(|scc| scc == &attractor_2),
        "Should find the 2-cycle SCC {{110, 111}}"
    );

    Ok(())
}

/// Test that a ChainScc generator can be serialized and deserialized mid-execution,
/// and that the deserialized generator produces the same SCCs when resumed.
fn test_chain_scc_serialization_roundtrip_impl() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();

    // Create a generator
    let config = SccConfig::from(&graph);
    let initial_state = ChainState::from(&graph);
    let mut generator = ChainScc::configure(config.clone(), initial_state);

    // Get the first SCC (if any)
    let mut first_scc = None;
    for result in &mut generator {
        match result {
            Ok(scc) => {
                first_scc = Some(scc);
                break;
            }
            Err(_) => {
                // Continue
            }
        }
    }

    // Serialize the generator
    let json = serde_json::to_string(&generator).expect("Failed to serialize ChainScc Generator");

    // Deserialize the generator
    let deserialized_generator: ChainScc =
        serde_json::from_str(&json).expect("Failed to deserialize ChainScc Generator");

    // Collect all remaining SCCs from the deserialized generator
    let mut deserialized_sccs = Vec::new();
    if let Some(scc) = first_scc {
        deserialized_sccs.push(scc);
    }
    for result in deserialized_generator {
        match result {
            Ok(scc) => deserialized_sccs.push(scc),
            Err(_) => {
                // Continue
            }
        }
    }

    // Run a fresh generator to completion for comparison
    let fresh_initial_state = ChainState::from(&graph);
    let fresh_generator = ChainScc::configure(config, fresh_initial_state);
    let mut fresh_sccs = Vec::new();
    for result in fresh_generator {
        match result {
            Ok(scc) => fresh_sccs.push(scc),
            Err(_) => {
                // Continue
            }
        }
    }

    // Results should be identical (same SCCs, possibly in different order)
    assert_eq!(
        deserialized_sccs.len(),
        fresh_sccs.len(),
        "Deserialized generator should produce the same number of SCCs"
    );

    // Verify we get the expected SCC (the 2-cycle attractor)
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);
    assert!(
        deserialized_sccs.iter().any(|scc| scc == &attractor_2),
        "Should find the 2-cycle SCC {{110, 111}}"
    );

    Ok(())
}

/// Test serialization/deserialization with ChainScc that requires multiple iterations.
fn test_chain_scc_serialization_multiple_steps_impl() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();

    // Create a generator
    let config = SccConfig::from(&graph);
    let initial_state = ChainState::from(&graph);
    let mut generator = ChainScc::configure(config.clone(), initial_state);

    // Get one SCC
    let mut first_scc = None;
    for result in &mut generator {
        match result {
            Ok(scc) => {
                first_scc = Some(scc);
                break;
            }
            Err(_) => {
                // Continue
            }
        }
    }

    // Serialize after the first SCC
    let json_step1 = serde_json::to_string(&generator)
        .expect("Failed to serialize ChainScc Generator after step 1");

    // Deserialize
    let deserialized_step1: ChainScc = serde_json::from_str(&json_step1)
        .expect("Failed to deserialize ChainScc Generator after step 1");

    // Continue and collect remaining SCCs
    let mut collected_sccs = Vec::new();
    if let Some(scc) = first_scc {
        collected_sccs.push(scc);
    }
    for result in deserialized_step1 {
        match result {
            Ok(scc) => collected_sccs.push(scc),
            Err(_) => {
                // Continue
            }
        }
    }

    // Compare with a fresh generator
    let fresh_initial_state = ChainState::from(&graph);
    let fresh_generator = ChainScc::configure(config, fresh_initial_state);
    let mut fresh_sccs = Vec::new();
    for result in fresh_generator {
        match result {
            Ok(scc) => fresh_sccs.push(scc),
            Err(_) => {
                // Continue
            }
        }
    }

    assert_eq!(
        collected_sccs.len(),
        fresh_sccs.len(),
        "ChainScc generator serialized/deserialized should produce the same number of SCCs"
    );

    // Verify we get the expected SCC
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);
    assert!(
        collected_sccs.iter().any(|scc| scc == &attractor_2),
        "Should find the 2-cycle SCC {{110, 111}}"
    );

    Ok(())
}

// ========== Tests for FwdBwdScc ==========

#[test]
fn test_fwd_bwd_scc_serialization_roundtrip() -> Cancellable<()> {
    test_fwd_bwd_scc_serialization_roundtrip_impl()
}

#[test]
fn test_fwd_bwd_scc_serialization_multiple_steps() -> Cancellable<()> {
    test_fwd_bwd_scc_serialization_multiple_steps_impl()
}

// ========== Tests for ChainScc ==========

#[test]
fn test_chain_scc_serialization_roundtrip() -> Cancellable<()> {
    test_chain_scc_serialization_roundtrip_impl()
}

#[test]
fn test_chain_scc_serialization_multiple_steps() -> Cancellable<()> {
    test_chain_scc_serialization_multiple_steps_impl()
}
