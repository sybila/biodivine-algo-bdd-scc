//! Comprehensive tests for the forward-backward SCC detection algorithm.
//!
//! These tests verify that the algorithm correctly identifies all non-trivial SCCs
//! (SCCs with more than one state) and that no extra SCCs are reported.

use crate::algorithm::scc::FwdBwdScc;
use crate::algorithm::test_utils::llm_example_network::create_test_network;
use crate::algorithm::test_utils::llm_example_network::sets::ATTRACTOR_2;
use crate::algorithm::test_utils::llm_transition_builder::from_transitions;
use crate::algorithm::test_utils::mk_state;
use crate::algorithm::test_utils::{init_logger, mk_states};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use std::collections::HashSet;

/// Collect all state numbers from a GraphColoredVertices set.
/// Returns a sorted vector of state numbers for comparison.
fn collect_state_numbers(
    graph: &SymbolicAsyncGraph,
    set: &GraphColoredVertices,
    num_vars: usize,
) -> Vec<u32> {
    let mut states = Vec::new();
    let max_state = (1u32 << num_vars) - 1;
    for state in 0..=max_state {
        let state_set = mk_state(graph, state);
        if !state_set.intersect(set).is_empty() {
            states.push(state);
        }
    }
    states.sort();
    states
}

/// Verify that the SCCs found match the expected SCCs exactly.
/// This handles the fact that SCCs can be returned in arbitrary order.
fn verify_sccs(
    graph: &SymbolicAsyncGraph,
    found_sccs: Vec<GraphColoredVertices>,
    expected_sccs: &[&[u32]],
    num_vars: usize,
) {
    // Convert expected SCCs to sets of state numbers for comparison
    let mut expected_sets: Vec<HashSet<u32>> = expected_sccs
        .iter()
        .map(|scc| scc.iter().copied().collect())
        .collect();

    // Convert found SCCs to sets of state numbers
    let mut found_sets: Vec<HashSet<u32>> = found_sccs
        .iter()
        .map(|scc| {
            collect_state_numbers(graph, scc, num_vars)
                .into_iter()
                .collect()
        })
        .collect();

    // Sort both for easier comparison (by first element, then size)
    expected_sets.sort_by_cached_key(|s| {
        let mut v: Vec<u32> = s.iter().copied().collect();
        v.sort();
        (v.len(), v)
    });
    found_sets.sort_by_cached_key(|s| {
        let mut v: Vec<u32> = s.iter().copied().collect();
        v.sort();
        (v.len(), v)
    });

    assert_eq!(
        found_sets.len(),
        expected_sets.len(),
        "Expected {} SCCs, but found {}",
        expected_sets.len(),
        found_sets.len()
    );

    for (i, (found, expected)) in found_sets.iter().zip(expected_sets.iter()).enumerate() {
        assert_eq!(
            found, expected,
            "SCC {} mismatch: expected {:?}, found {:?}",
            i, expected, found
        );
    }
}

/// Test case: Single 2-cycle (simplest non-trivial SCC).
#[test]
fn test_single_2_cycle() {
    init_logger();
    // Create a 2-variable network with a single 2-cycle: 00 ↔ 10
    // 00 → 10 (x0 flips)
    // 10 → 00 (x0 flips)
    let transitions = vec![(0b00, 0b10), (0b10, 0b00)];

    let bn = from_transitions(2, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");

    let mut generator = FwdBwdScc::configure(graph.clone(), &graph);
    let mut found_sccs = Vec::new();

    while let Some(result) = generator.next() {
        found_sccs.push(result.unwrap());
    }

    verify_sccs(&graph, found_sccs, &[&[0b00, 0b10]], 2);
}

/// Test case: Single 3-cycle.
#[test]
fn test_single_3_cycle() {
    init_logger();
    // Create a 3-variable network with a cycle containing 6 states
    // 000 → 100 → 110 → 111 → 011 → 001 → 000
    let transitions = vec![
        (0b000, 0b100), // x0 flips
        (0b100, 0b110), // x1 flips
        (0b110, 0b111), // x2 flips
        (0b111, 0b011), // x0 flips
        (0b011, 0b001), // x1 flips
        (0b001, 0b000), // x2 flips
    ];

    let bn = from_transitions(3, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");

    let mut generator = FwdBwdScc::configure(graph.clone(), &graph);
    let mut found_sccs = Vec::new();

    while let Some(result) = generator.next() {
        found_sccs.push(result.unwrap());
    }

    // All 6 states form one SCC
    verify_sccs(
        &graph,
        found_sccs,
        &[&[0b000, 0b001, 0b011, 0b100, 0b110, 0b111]],
        3,
    );
}

/// Test case: Two disjoint 2-cycles.
#[test]
fn test_two_disjoint_2_cycles() {
    init_logger();
    // Create a 3-variable network with two disjoint 2-cycles:
    // Cycle 1: 000 ↔ 100
    // Cycle 2: 011 ↔ 111
    // States 001, 010, 101, 110 are fixed points (trivial SCCs; should not be returned)
    let transitions = vec![
        (0b000, 0b100), // x0 flips
        (0b100, 0b000), // x0 flips
        (0b011, 0b111), // x0 flips
        (0b111, 0b011), // x0 flips
    ];

    let bn = from_transitions(3, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");

    let mut generator = FwdBwdScc::configure(graph.clone(), &graph);
    let mut found_sccs = Vec::new();

    while let Some(result) = generator.next() {
        found_sccs.push(result.unwrap());
    }

    // Should find exactly two SCCs
    verify_sccs(&graph, found_sccs, &[&[0b000, 0b100], &[0b011, 0b111]], 3);
}

/// Test case: Multiple SCCs with different sizes.
#[test]
fn test_multiple_sccs_different_sizes() {
    init_logger();
    // Create a 3-variable network with:
    // - A 2-cycle: 000 ↔ 100
    // - A 4-cycle: 001 → 101 → 111 → 011 → 001
    // - Fixed points: 010, 110 (trivial, should not be returned)
    let transitions = vec![
        (0b000, 0b100), // x0 flips
        (0b100, 0b000), // x0 flips
        (0b001, 0b101), // x0 flips
        (0b101, 0b111), // x1 flips
        (0b111, 0b011), // x0 flips
        (0b011, 0b001), // x1 flips
    ];

    let bn = from_transitions(3, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");

    let mut generator = FwdBwdScc::configure(graph.clone(), &graph);
    let mut found_sccs = Vec::new();

    while let Some(result) = generator.next() {
        found_sccs.push(result.unwrap());
    }

    // Should find exactly two SCCs
    verify_sccs(
        &graph,
        found_sccs,
        &[&[0b000, 0b100], &[0b001, 0b011, 0b101, 0b111]],
        3,
    );
}

/// Test case: SCC with branching paths (more complex structure).
#[test]
fn test_scc_with_branching() {
    init_logger();
    // Create a network where multiple paths converge into a cycle:
    // 000 → 100 → 110 ↔ 111
    // 001 → 101 → 111
    // So {110, 111} is the SCC, and 000, 100, 001, 101 are in the basin
    let transitions = vec![
        (0b000, 0b100), // x0 flips
        (0b100, 0b110), // x1 flips
        (0b110, 0b111), // x2 flips
        (0b111, 0b110), // x2 flips back
        (0b001, 0b101), // x1 flips
        (0b101, 0b111), // x2 flips
    ];

    let bn = from_transitions(3, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");

    let mut generator = FwdBwdScc::configure(graph.clone(), &graph);
    let mut found_sccs = Vec::new();

    while let Some(result) = generator.next() {
        found_sccs.push(result.unwrap());
    }

    // Should find exactly one SCC: {110, 111}
    verify_sccs(&graph, found_sccs, &[&[0b110, 0b111]], 3);
}

/// Test case: Network with no non-trivial SCCs (only fixed points).
#[test]
fn test_only_trivial_sccs() {
    init_logger();
    // Create a network where all states transition to fixed points (no cycles)
    // 000 is fixed, 001 → 000 (x2 flips from 1 to 0)
    let transitions = vec![
        (0b001, 0b000), // x2 flips: 001 → 000
    ];

    let bn = from_transitions(3, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");

    let mut generator = FwdBwdScc::configure(graph.clone(), &graph);
    let mut found_sccs = Vec::new();

    while let Some(result) = generator.next() {
        found_sccs.push(result.unwrap());
    }

    // Should find no non-trivial SCCs
    assert_eq!(
        found_sccs.len(),
        0,
        "Expected no non-trivial SCCs, but found {}",
        found_sccs.len()
    );
}

/// Test case: Large SCC (4-cycle).
#[test]
fn test_4_cycle() {
    init_logger();
    // Create a 2-variable network with a 4-cycle: 00 → 10 → 11 → 01 → 00
    let transitions = vec![
        (0b00, 0b10), // x0 flips
        (0b10, 0b11), // x1 flips
        (0b11, 0b01), // x0 flips
        (0b01, 0b00), // x1 flips
    ];

    let bn = from_transitions(2, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");

    let mut generator = FwdBwdScc::configure(graph.clone(), &graph);
    let mut found_sccs = Vec::new();

    while let Some(result) = generator.next() {
        found_sccs.push(result.unwrap());
    }

    // All 4 states form one SCC
    verify_sccs(&graph, found_sccs, &[&[0b00, 0b01, 0b10, 0b11]], 2);
}

/// Test case: SCC with multiple paths.
#[test]
fn test_scc_with_multiple_paths() {
    init_logger();
    // Create a network where one state in an SCC has multiple outgoing edges:
    // 000 ↔ 100, and 100 can also go to 110, but 110 goes back to 100
    // So {000, 100, 110} forms one SCC
    let transitions = vec![
        (0b000, 0b100), // x0 flips
        (0b100, 0b000), // x0 flips back
        (0b100, 0b110), // x1 flips
        (0b110, 0b100), // x1 flips back
    ];

    let bn = from_transitions(3, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");

    let mut generator = FwdBwdScc::configure(graph.clone(), &graph);
    let mut found_sccs = Vec::new();

    while let Some(result) = generator.next() {
        found_sccs.push(result.unwrap());
    }

    // Should find one SCC containing all three states
    verify_sccs(&graph, found_sccs, &[&[0b000, 0b100, 0b110]], 3);
}

/// Test case: Verify the example network from llm_example_network.
#[test]
fn test_llm_example_network() {
    init_logger();
    let graph = create_test_network();
    let mut generator = FwdBwdScc::configure(graph.clone(), &graph);
    let mut found_sccs = Vec::new();

    while let Some(result) = generator.next() {
        found_sccs.push(result.unwrap());
    }

    // Should find exactly one SCC: {110, 111}
    assert_eq!(found_sccs.len(), 1);
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);
    assert_eq!(attractor_2, found_sccs[0]);
}

/// Test case: Complex network with multiple SCCs and basins.
#[test]
fn test_complex_network() {
    init_logger();
    // Create a 4-variable network with:
    // - SCC 1: {0000, 1000} (2-cycle)
    // - SCC 2: {0001, 0011, 1001, 1011} (4-cycle)
    // - Fixed points: all other states (trivial, should not be returned)
    let transitions = vec![
        // SCC 1: 0000 ↔ 1000
        (0b0000, 0b1000), // x0 flips
        (0b1000, 0b0000), // x0 flips
        // SCC 2: 0001 → 1001 → 1011 → 0011 → 0001 (4-cycle)
        (0b0001, 0b1001), // x0 flips
        (0b1001, 0b1011), // x2 flips
        (0b1011, 0b0011), // x0 flips
        (0b0011, 0b0001), // x2 flips
    ];

    let bn = from_transitions(4, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");

    let mut generator = FwdBwdScc::configure(graph.clone(), &graph);
    let mut found_sccs = Vec::new();

    while let Some(result) = generator.next() {
        found_sccs.push(result.unwrap());
    }

    // Should find exactly two SCCs
    verify_sccs(
        &graph,
        found_sccs,
        &[&[0b0000, 0b1000], &[0b0001, 0b0011, 0b1001, 0b1011]],
        4,
    );
}
