//! Comprehensive tests for SCC detection algorithms.
//!
//! These tests verify that algorithms correctly identify all non-trivial SCCs
//! (SCCs with more than one state) and that no extra SCCs are reported.
//!
//! The tests are generic and can be used to test any algorithm that implements
//! the `SccAlgorithm` trait.

use crate::scc::tests::sccs_to_sorted_sets;
use crate::scc::{ChainScc, ChainState, FwdBwdScc, FwdBwdSccBfs, FwdBwdState, SccAlgorithm};
use crate::test_utils::llm_example_network::create_test_network;
use crate::test_utils::llm_example_network::sets::ATTRACTOR_2;
use crate::test_utils::llm_transition_builder::from_transitions;
use crate::test_utils::{init_logger, mk_states};
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use std::collections::HashSet;

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

    // Convert found SCCs to sets of state numbers using the shared helper
    let found_sets = sccs_to_sorted_sets(graph, &found_sccs, num_vars);

    // Sort expected sets for easier comparison (by size, then sorted state numbers)
    expected_sets.sort_by_cached_key(|s| {
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

// ========== Parametrized test helpers ==========

/// Generic helper function for testing single 2-cycle detection.
fn test_single_2_cycle_impl<STATE, ALG>()
where
    ALG: SccAlgorithm<STATE>,
    STATE: for<'a> From<&'a SymbolicAsyncGraph>,
{
    init_logger();
    // Create a 2-variable network with a single 2-cycle: 00 ↔ 10
    // 00 → 10 (x0 flips)
    // 10 → 00 (x0 flips)
    let transitions = vec![(0b00, 0b10), (0b10, 0b00)];

    let bn = from_transitions(2, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");

    let mut generator = ALG::configure(graph.clone(), &graph);
    let mut found_sccs = Vec::new();

    while let Some(result) = generator.next() {
        found_sccs.push(result.unwrap());
    }

    verify_sccs(&graph, found_sccs, &[&[0b00, 0b10]], 2);
}

/// Generic helper function for testing single 3-cycle detection.
fn test_single_3_cycle_impl<STATE, ALG>()
where
    ALG: SccAlgorithm<STATE>,
    STATE: for<'a> From<&'a SymbolicAsyncGraph>,
{
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

    let mut generator = ALG::configure(graph.clone(), &graph);
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

/// Generic helper function for testing two disjoint 2-cycles detection.
fn test_two_disjoint_2_cycles_impl<STATE, ALG>()
where
    ALG: SccAlgorithm<STATE>,
    STATE: for<'a> From<&'a SymbolicAsyncGraph>,
{
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

    let mut generator = ALG::configure(graph.clone(), &graph);
    let mut found_sccs = Vec::new();

    while let Some(result) = generator.next() {
        found_sccs.push(result.unwrap());
    }

    // Should find exactly two SCCs
    verify_sccs(&graph, found_sccs, &[&[0b000, 0b100], &[0b011, 0b111]], 3);
}

/// Generic helper function for testing multiple SCCs with different sizes.
fn test_multiple_sccs_different_sizes_impl<STATE, ALG>()
where
    ALG: SccAlgorithm<STATE>,
    STATE: for<'a> From<&'a SymbolicAsyncGraph>,
{
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

    let mut generator = ALG::configure(graph.clone(), &graph);
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

/// Generic helper function for testing SCC with branching paths.
fn test_scc_with_branching_impl<STATE, ALG>()
where
    ALG: SccAlgorithm<STATE>,
    STATE: for<'a> From<&'a SymbolicAsyncGraph>,
{
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

    let mut generator = ALG::configure(graph.clone(), &graph);
    let mut found_sccs = Vec::new();

    while let Some(result) = generator.next() {
        found_sccs.push(result.unwrap());
    }

    // Should find exactly one SCC: {110, 111}
    verify_sccs(&graph, found_sccs, &[&[0b110, 0b111]], 3);
}

/// Generic helper function for testing network with no non-trivial SCCs.
fn test_only_trivial_sccs_impl<STATE, ALG>()
where
    ALG: SccAlgorithm<STATE>,
    STATE: for<'a> From<&'a SymbolicAsyncGraph>,
{
    init_logger();
    // Create a network where all states transition to fixed points (no cycles)
    // 000 is fixed, 001 → 000 (x2 flips from 1 to 0)
    let transitions = vec![
        (0b001, 0b000), // x2 flips: 001 → 000
    ];

    let bn = from_transitions(3, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");

    let mut generator = ALG::configure(graph.clone(), &graph);
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

/// Generic helper function for testing 4-cycle detection.
fn test_4_cycle_impl<STATE, ALG>()
where
    ALG: SccAlgorithm<STATE>,
    STATE: for<'a> From<&'a SymbolicAsyncGraph>,
{
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

    let mut generator = ALG::configure(graph.clone(), &graph);
    let mut found_sccs = Vec::new();

    while let Some(result) = generator.next() {
        found_sccs.push(result.unwrap());
    }

    // All 4 states form one SCC
    verify_sccs(&graph, found_sccs, &[&[0b00, 0b01, 0b10, 0b11]], 2);
}

/// Generic helper function for testing SCC with multiple paths.
fn test_scc_with_multiple_paths_impl<STATE, ALG>()
where
    ALG: SccAlgorithm<STATE>,
    STATE: for<'a> From<&'a SymbolicAsyncGraph>,
{
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

    let mut generator = ALG::configure(graph.clone(), &graph);
    let mut found_sccs = Vec::new();

    while let Some(result) = generator.next() {
        found_sccs.push(result.unwrap());
    }

    // Should find one SCC containing all three states
    verify_sccs(&graph, found_sccs, &[&[0b000, 0b100, 0b110]], 3);
}

/// Generic helper function for testing the example network from llm_example_network.
fn test_llm_example_network_impl<STATE, ALG>()
where
    ALG: SccAlgorithm<STATE>,
    STATE: for<'a> From<&'a SymbolicAsyncGraph>,
{
    init_logger();
    let graph = create_test_network();
    let mut generator = ALG::configure(graph.clone(), &graph);
    let mut found_sccs = Vec::new();

    while let Some(result) = generator.next() {
        found_sccs.push(result.unwrap());
    }

    // Should find exactly one SCC: {110, 111}
    assert_eq!(found_sccs.len(), 1);
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);
    assert_eq!(attractor_2, found_sccs[0]);
}

/// Generic helper function for testing a complex network with multiple SCCs.
fn test_complex_network_impl<STATE, ALG>()
where
    ALG: SccAlgorithm<STATE>,
    STATE: for<'a> From<&'a SymbolicAsyncGraph>,
{
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

    let mut generator = ALG::configure(graph.clone(), &graph);
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

// ========== Tests for FwdBwdScc (saturation) ==========

#[test]
fn test_single_2_cycle_fwd_bwd() {
    test_single_2_cycle_impl::<FwdBwdState, FwdBwdScc>()
}

#[test]
fn test_single_3_cycle_fwd_bwd() {
    test_single_3_cycle_impl::<FwdBwdState, FwdBwdScc>()
}

#[test]
fn test_two_disjoint_2_cycles_fwd_bwd() {
    test_two_disjoint_2_cycles_impl::<FwdBwdState, FwdBwdScc>()
}

#[test]
fn test_multiple_sccs_different_sizes_fwd_bwd() {
    test_multiple_sccs_different_sizes_impl::<FwdBwdState, FwdBwdScc>()
}

#[test]
fn test_scc_with_branching_fwd_bwd() {
    test_scc_with_branching_impl::<FwdBwdState, FwdBwdScc>()
}

#[test]
fn test_only_trivial_sccs_fwd_bwd() {
    test_only_trivial_sccs_impl::<FwdBwdState, FwdBwdScc>()
}

#[test]
fn test_4_cycle_fwd_bwd() {
    test_4_cycle_impl::<FwdBwdState, FwdBwdScc>()
}

#[test]
fn test_scc_with_multiple_paths_fwd_bwd() {
    test_scc_with_multiple_paths_impl::<FwdBwdState, FwdBwdScc>()
}

#[test]
fn test_llm_example_network_fwd_bwd() {
    test_llm_example_network_impl::<FwdBwdState, FwdBwdScc>()
}

#[test]
fn test_complex_network_fwd_bwd() {
    test_complex_network_impl::<FwdBwdState, FwdBwdScc>()
}

// ========== Tests for FwdBwdSccBfs ==========

#[test]
fn test_single_2_cycle_fwd_bwd_bfs() {
    test_single_2_cycle_impl::<FwdBwdState, FwdBwdSccBfs>()
}

#[test]
fn test_single_3_cycle_fwd_bwd_bfs() {
    test_single_3_cycle_impl::<FwdBwdState, FwdBwdSccBfs>()
}

#[test]
fn test_two_disjoint_2_cycles_fwd_bwd_bfs() {
    test_two_disjoint_2_cycles_impl::<FwdBwdState, FwdBwdSccBfs>()
}

#[test]
fn test_multiple_sccs_different_sizes_fwd_bwd_bfs() {
    test_multiple_sccs_different_sizes_impl::<FwdBwdState, FwdBwdSccBfs>()
}

#[test]
fn test_scc_with_branching_fwd_bwd_bfs() {
    test_scc_with_branching_impl::<FwdBwdState, FwdBwdSccBfs>()
}

#[test]
fn test_only_trivial_sccs_fwd_bwd_bfs() {
    test_only_trivial_sccs_impl::<FwdBwdState, FwdBwdSccBfs>()
}

#[test]
fn test_4_cycle_fwd_bwd_bfs() {
    test_4_cycle_impl::<FwdBwdState, FwdBwdSccBfs>()
}

#[test]
fn test_scc_with_multiple_paths_fwd_bwd_bfs() {
    test_scc_with_multiple_paths_impl::<FwdBwdState, FwdBwdSccBfs>()
}

#[test]
fn test_llm_example_network_fwd_bwd_bfs() {
    test_llm_example_network_impl::<FwdBwdState, FwdBwdSccBfs>()
}

#[test]
fn test_complex_network_fwd_bwd_bfs() {
    test_complex_network_impl::<FwdBwdState, FwdBwdSccBfs>()
}

// ========== Tests for ChainScc ==========

#[test]
fn test_single_2_cycle_chain() {
    test_single_2_cycle_impl::<ChainState, ChainScc>()
}

#[test]
fn test_single_3_cycle_chain() {
    test_single_3_cycle_impl::<ChainState, ChainScc>()
}

#[test]
fn test_two_disjoint_2_cycles_chain() {
    test_two_disjoint_2_cycles_impl::<ChainState, ChainScc>()
}

#[test]
fn test_multiple_sccs_different_sizes_chain() {
    test_multiple_sccs_different_sizes_impl::<ChainState, ChainScc>()
}

#[test]
fn test_scc_with_branching_chain() {
    test_scc_with_branching_impl::<ChainState, ChainScc>()
}

#[test]
fn test_only_trivial_sccs_chain() {
    test_only_trivial_sccs_impl::<ChainState, ChainScc>()
}

#[test]
fn test_4_cycle_chain() {
    test_4_cycle_impl::<ChainState, ChainScc>()
}

#[test]
fn test_scc_with_multiple_paths_chain() {
    test_scc_with_multiple_paths_impl::<ChainState, ChainScc>()
}

#[test]
fn test_llm_example_network_chain() {
    test_llm_example_network_impl::<ChainState, ChainScc>()
}

#[test]
fn test_complex_network_chain() {
    test_complex_network_impl::<ChainState, ChainScc>()
}
