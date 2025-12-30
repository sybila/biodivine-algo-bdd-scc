//! Tests for attractor detection algorithms using the example test network.
//!
//! See `llm_example_network.rs` for the complete documentation of the test network structure.

use crate::attractor::{
    AttractorConfig, InterleavedTransitionGuidedReduction, ItgrState, XieBeerelAttractors,
    XieBeerelState,
};
use crate::test_utils::llm_example_network::create_test_network;
use crate::test_utils::llm_example_network::sets::{ATTRACTOR_1, ATTRACTOR_2};
use crate::test_utils::llm_transition_builder::from_transitions;
use crate::test_utils::{init_logger, mk_states};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use cancel_this::Cancellable;
use computation_process::{Computable, Stateful};
use std::collections::BTreeSet;

/// Verify that the attractors found match the expected attractors exactly.
/// This handles the fact that attractors can be returned in arbitrary order.
fn verify_attractors(
    graph: &SymbolicAsyncGraph,
    found_attractors: Vec<GraphColoredVertices>,
    expected_attractors: &[&[u32]],
) {
    use std::collections::HashSet;

    // Convert expected attractors to sets of state numbers for comparison
    let mut expected_sets: Vec<HashSet<u32>> = expected_attractors
        .iter()
        .map(|attr| attr.iter().copied().collect())
        .collect();

    // Convert found attractors to sets of state numbers
    let mut found_sets: Vec<HashSet<u32>> = found_attractors
        .iter()
        .map(|attr| {
            let mut states = Vec::new();
            let num_vars = graph.num_vars();
            let max_state = (1u32 << num_vars) - 1;
            for state in 0..=max_state {
                let state_set = crate::test_utils::mk_state(graph, state);
                if !state_set.intersect(attr).is_empty() {
                    states.push(state);
                }
            }
            states.into_iter().collect()
        })
        .collect();

    // Sort both for easier comparison (by size, then sorted state numbers)
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
        "Expected {} attractors, but found {}",
        expected_sets.len(),
        found_sets.len()
    );

    for (i, (found, expected)) in found_sets.iter().zip(expected_sets.iter()).enumerate() {
        assert_eq!(
            found, expected,
            "Attractor {} mismatch: expected {:?}, found {:?}",
            i, expected, found
        );
    }
}

// ========== Helper functions ==========

/// Run XieBeerel algorithm on a graph (with optional ITGR reduction).
fn run_xie_beerel(
    graph: &SymbolicAsyncGraph,
    use_itgr: bool,
) -> Cancellable<Vec<GraphColoredVertices>> {
    let mut config = AttractorConfig::new(graph.clone());
    let (config, initial_state) = if use_itgr {
        // First, run ITGR to reduce the state space
        let itgr_state = ItgrState::new(graph, &graph.mk_unit_colored_vertices());
        let mut itgr = InterleavedTransitionGuidedReduction::configure(config.clone(), itgr_state);
        let reduced = itgr.compute()?;

        let active_variables = itgr.state().active_variables().collect::<BTreeSet<_>>();
        config.active_variables = active_variables;
        let initial_state = XieBeerelState::from(&reduced);
        (config, initial_state)
    } else {
        let initial_state = XieBeerelState::from(graph);
        (config, initial_state)
    };

    let generator = XieBeerelAttractors::configure(config, initial_state);
    let mut attractors = Vec::new();
    for result in generator {
        attractors.push(result?);
    }
    Ok(attractors)
}

// ========== Test implementations ==========

/// Test that we can find both attractors in the example network.
fn test_find_all_attractors_impl(use_itgr: bool) -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractors = run_xie_beerel(&graph, use_itgr)?;

    // Should find exactly two attractors: {000} and {110, 111}
    verify_attractors(&graph, attractors, &[ATTRACTOR_1, ATTRACTOR_2]);
    Ok(())
}

/// Test that we find the fixed point attractor.
fn test_find_fixed_point_attractor_impl(use_itgr: bool) -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractors = run_xie_beerel(&graph, use_itgr)?;

    // Should find the fixed point {000}
    let attractor_1 = mk_states(&graph, ATTRACTOR_1);
    assert!(
        attractors.iter().any(|attr| attr == &attractor_1),
        "Should find attractor 1 (fixed point {{000}})"
    );
    Ok(())
}

/// Test that we find the cycle attractor.
fn test_find_cycle_attractor_impl(use_itgr: bool) -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractors = run_xie_beerel(&graph, use_itgr)?;

    // Should find the cycle {110, 111}
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);
    assert!(
        attractors.iter().any(|attr| attr == &attractor_2),
        "Should find attractor 2 (cycle {{110, 111}})"
    );
    Ok(())
}

/// Test that attractors are non-empty (except for trivial cases).
fn test_attractors_are_non_empty_impl(use_itgr: bool) -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractors = run_xie_beerel(&graph, use_itgr)?;

    // All attractors should be non-empty
    for (i, attr) in attractors.iter().enumerate() {
        assert!(!attr.is_empty(), "Attractor {} should be non-empty", i);
    }
    Ok(())
}

/// Test that attractors are disjoint (each state appears in at most one attractor).
fn test_attractors_are_disjoint_impl(use_itgr: bool) -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractors = run_xie_beerel(&graph, use_itgr)?;

    // Check that attractors are pairwise disjoint
    for i in 0..attractors.len() {
        for j in (i + 1)..attractors.len() {
            let intersection = attractors[i].intersect(&attractors[j]);
            assert!(
                intersection.is_empty(),
                "Attractors {} and {} should be disjoint, but share {} states",
                i,
                j,
                intersection.exact_cardinality()
            );
        }
    }
    Ok(())
}

/// Test that attractors cover all states that can reach them.
fn test_attractors_cover_reachable_states_impl(use_itgr: bool) -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractors = run_xie_beerel(&graph, use_itgr)?;

    // The union of all attractors should be a subset of all states
    let mut union = graph.mk_empty_colored_vertices();
    for attr in &attractors {
        union = union.union(attr);
    }

    // All attractors together should be a subset of the full state space
    let all_states = graph.mk_unit_colored_vertices();
    assert!(
        union.is_subset(&all_states),
        "Union of attractors should be a subset of all states"
    );
    Ok(())
}

// ========== Tests with toy networks ==========

/// Test a network with a single fixed point attractor.
/// All states eventually converge to state 00.
fn test_single_fixed_point_impl(use_itgr: bool) -> Cancellable<()> {
    init_logger();
    // Create a 2-variable network where all states converge to 00:
    // 01 → 00 (x1 flips)
    // 10 → 00 (x0 flips)
    // 11 → 10 → 00 (x0 flips, then x0 flips again)
    let transitions = vec![
        (0b01, 0b00), // x1 flips: 01 → 00
        (0b10, 0b00), // x0 flips: 10 → 00
        (0b11, 0b10), // x0 flips: 11 → 10
    ];

    let bn = from_transitions(2, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");
    let attractors = run_xie_beerel(&graph, use_itgr)?;

    // Should find exactly one attractor: {00}
    verify_attractors(&graph, attractors, &[&[0b00]]);
    Ok(())
}

/// Test a network with a single 2-cycle attractor.
/// States 00 and 10 form a cycle, and state 01 converges to it.
fn test_single_2_cycle_impl(use_itgr: bool) -> Cancellable<()> {
    init_logger();
    // Create a 2-variable network with a 2-cycle: 00 ↔ 10
    // 00 → 10 (x0 flips)
    // 10 → 00 (x0 flips)
    // 01 → 00 (x1 flips)
    let transitions = vec![
        (0b00, 0b10), // x0 flips: 00 → 10
        (0b10, 0b00), // x0 flips: 10 → 00
        (0b01, 0b00), // x1 flips: 01 → 00
        (0b11, 0b01), // x0 flips: 11 → 01
    ];

    let bn = from_transitions(2, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");
    let attractors = run_xie_beerel(&graph, use_itgr)?;

    // Should find exactly one attractor: {00, 10}
    verify_attractors(&graph, attractors, &[&[0b00, 0b10]]);
    Ok(())
}

/// Test a network with a single 4-cycle attractor.
fn test_single_4_cycle_impl(use_itgr: bool) -> Cancellable<()> {
    init_logger();
    // Create a 2-variable network with a 4-cycle: 00 → 10 → 11 → 01 → 00
    let transitions = vec![
        (0b00, 0b10), // x0 flips: 00 → 10
        (0b10, 0b11), // x1 flips: 10 → 11
        (0b11, 0b01), // x0 flips: 11 → 01
        (0b01, 0b00), // x1 flips: 01 → 00
    ];

    let bn = from_transitions(2, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");
    let attractors = run_xie_beerel(&graph, use_itgr)?;

    // Should find exactly one attractor: {00, 01, 10, 11}
    verify_attractors(&graph, attractors, &[&[0b00, 0b01, 0b10, 0b11]]);
    Ok(())
}

/// Test a network with two disjoint attractors: one fixed point and one cycle.
fn test_two_disjoint_attractors_impl(use_itgr: bool) -> Cancellable<()> {
    init_logger();
    // Create a 3-variable network with:
    // - Fixed point: {000}
    // - 2-cycle: {110, 111}
    // - Transient states: {001, 010, 011, 100, 101}
    let transitions = vec![
        // Transient states converging to fixed point 000
        (0b001, 0b000), // x2 flips: 001 → 000
        (0b010, 0b000), // x1 flips: 010 → 000
        (0b011, 0b001), // x1 flips: 011 → 001
        (0b100, 0b000), // x0 flips: 100 → 000
        // Transient states converging to cycle
        (0b101, 0b111), // x1 flips: 101 → 111
        // Cycle: 110 ↔ 111
        (0b110, 0b111), // x2 flips: 110 → 111
        (0b111, 0b110), // x2 flips: 111 → 110
    ];

    let bn = from_transitions(3, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");
    let attractors = run_xie_beerel(&graph, use_itgr)?;

    // Should find exactly two attractors: {000} and {110, 111}
    verify_attractors(&graph, attractors, &[&[0b000], &[0b110, 0b111]]);
    Ok(())
}

/// Test a network with two disjoint cycles.
fn test_two_disjoint_cycles_impl(use_itgr: bool) -> Cancellable<()> {
    init_logger();
    // Create a 3-variable network with two disjoint 2-cycles:
    // - Cycle 1: {000, 100}
    // - Cycle 2: {011, 111}
    // - Fixed points: {001, 010, 101, 110} (trivial attractors should not be found)
    let transitions = vec![
        // Cycle 1: 000 ↔ 100
        (0b000, 0b100), // x0 flips: 000 → 100
        (0b100, 0b000), // x0 flips: 100 → 000
        // Cycle 2: 011 ↔ 111
        (0b011, 0b111), // x0 flips: 011 → 111
        (0b111, 0b011), // x0 flips: 111 → 011
        // Remaining states fall into one of the attractors:
        (0b001, 0b000), // x2 flips: 001 → 000
        (0b010, 0b000), // x1 flips: 010 → 000
        (0b101, 0b111), // x1 flips: 101 → 111
        (0b110, 0b111), // x2 flips: 110 → 111
    ];

    let bn = from_transitions(3, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");
    let attractors = run_xie_beerel(&graph, use_itgr)?;

    // Should find exactly two attractors: {000, 100} and {011, 111}
    verify_attractors(&graph, attractors, &[&[0b000, 0b100], &[0b011, 0b111]]);
    Ok(())
}

/// Test a network with multiple fixed points.
fn test_multiple_fixed_points_impl(use_itgr: bool) -> Cancellable<()> {
    init_logger();
    // Create a 2-variable network with two fixed points:
    // - Fixed point: {00}
    // - Fixed point: {11}
    // - Transient states: {01, 10} converge to {00}
    let transitions = vec![
        (0b01, 0b00), // x1 flips: 01 → 00
        (0b10, 0b00), // x0 flips: 10 → 00
    ];

    let bn = from_transitions(2, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");
    let attractors = run_xie_beerel(&graph, use_itgr)?;

    // Should find exactly two attractors: {00} and {11}
    verify_attractors(&graph, attractors, &[&[0b00], &[0b11]]);
    Ok(())
}

/// Test a network with a 4-cycle attractor (using 3 variables).
fn test_single_4_cycle_3vars_impl(use_itgr: bool) -> Cancellable<()> {
    init_logger();
    // Create a 3-variable network with a 4-cycle: 000 → 100 → 101 → 001 → 000
    let transitions = vec![
        (0b000, 0b100), // x0 flips: 000 → 100
        (0b100, 0b101), // x2 flips: 100 → 101
        (0b101, 0b001), // x0 flips: 101 → 001
        (0b001, 0b000), // x2 flips: 001 → 000
        // Remaining states fall into the attractor:
        (0b010, 0b000), // x1 flips: 010 → 000
        (0b011, 0b001), // x1 flips: 011 → 001
        (0b110, 0b100), // x1 flips: 110 → 100
        (0b111, 0b101), // x1 flips: 111 → 101
    ];

    let bn = from_transitions(3, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");
    let attractors = run_xie_beerel(&graph, use_itgr)?;

    // Should find exactly one attractor: {000, 001, 100, 101}
    verify_attractors(&graph, attractors, &[&[0b000, 0b001, 0b100, 0b101]]);
    Ok(())
}

/// Test a network with a larger cycle and transient states.
fn test_cycle_with_transient_states_impl(use_itgr: bool) -> Cancellable<()> {
    init_logger();
    // Create a 3-variable network with:
    // - 2-cycle: {110, 111}
    // - Transient states: {000, 001, 010, 011, 100, 101} that converge to the cycle
    let transitions = vec![
        // Paths converging to the cycle
        (0b000, 0b100), // x0 flips: 000 → 100
        (0b001, 0b101), // x0 flips: 001 → 101
        (0b010, 0b110), // x0 flips: 010 → 110
        (0b011, 0b111), // x0 flips: 011 → 111
        (0b100, 0b110), // x1 flips: 100 → 110
        (0b101, 0b111), // x1 flips: 101 → 111
        // Cycle: 110 ↔ 111
        (0b110, 0b111), // x2 flips: 110 → 111
        (0b111, 0b110), // x2 flips: 111 → 110
    ];

    let bn = from_transitions(3, &transitions).expect("Failed to create network");
    let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");
    let attractors = run_xie_beerel(&graph, use_itgr)?;

    // Should find exactly one attractor: {110, 111}
    verify_attractors(&graph, attractors, &[&[0b110, 0b111]]);
    Ok(())
}

// ========== Tests ==========

#[test]
fn test_find_all_attractors() -> Cancellable<()> {
    test_find_all_attractors_impl(false)
}

#[test]
fn test_find_all_attractors_with_itgr() -> Cancellable<()> {
    test_find_all_attractors_impl(true)
}

#[test]
fn test_find_fixed_point_attractor() -> Cancellable<()> {
    test_find_fixed_point_attractor_impl(false)
}

#[test]
fn test_find_fixed_point_attractor_with_itgr() -> Cancellable<()> {
    test_find_fixed_point_attractor_impl(true)
}

#[test]
fn test_find_cycle_attractor() -> Cancellable<()> {
    test_find_cycle_attractor_impl(false)
}

#[test]
fn test_find_cycle_attractor_with_itgr() -> Cancellable<()> {
    test_find_cycle_attractor_impl(true)
}

#[test]
fn test_attractors_are_non_empty() -> Cancellable<()> {
    test_attractors_are_non_empty_impl(false)
}

#[test]
fn test_attractors_are_non_empty_with_itgr() -> Cancellable<()> {
    test_attractors_are_non_empty_impl(true)
}

#[test]
fn test_attractors_are_disjoint() -> Cancellable<()> {
    test_attractors_are_disjoint_impl(false)
}

#[test]
fn test_attractors_are_disjoint_with_itgr() -> Cancellable<()> {
    test_attractors_are_disjoint_impl(true)
}

#[test]
fn test_attractors_cover_reachable_states() -> Cancellable<()> {
    test_attractors_cover_reachable_states_impl(false)
}

#[test]
fn test_attractors_cover_reachable_states_with_itgr() -> Cancellable<()> {
    test_attractors_cover_reachable_states_impl(true)
}

#[test]
fn test_single_fixed_point() -> Cancellable<()> {
    test_single_fixed_point_impl(false)
}

#[test]
fn test_single_fixed_point_with_itgr() -> Cancellable<()> {
    test_single_fixed_point_impl(true)
}

#[test]
fn test_single_2_cycle() -> Cancellable<()> {
    test_single_2_cycle_impl(false)
}

#[test]
fn test_single_2_cycle_with_itgr() -> Cancellable<()> {
    test_single_2_cycle_impl(true)
}

#[test]
fn test_single_4_cycle() -> Cancellable<()> {
    test_single_4_cycle_impl(false)
}

#[test]
fn test_single_4_cycle_with_itgr() -> Cancellable<()> {
    test_single_4_cycle_impl(true)
}

#[test]
fn test_two_disjoint_attractors() -> Cancellable<()> {
    test_two_disjoint_attractors_impl(false)
}

#[test]
fn test_two_disjoint_attractors_with_itgr() -> Cancellable<()> {
    test_two_disjoint_attractors_impl(true)
}

#[test]
fn test_two_disjoint_cycles() -> Cancellable<()> {
    test_two_disjoint_cycles_impl(false)
}

#[test]
fn test_two_disjoint_cycles_with_itgr() -> Cancellable<()> {
    test_two_disjoint_cycles_impl(true)
}

#[test]
fn test_multiple_fixed_points() -> Cancellable<()> {
    test_multiple_fixed_points_impl(false)
}

#[test]
fn test_multiple_fixed_points_with_itgr() -> Cancellable<()> {
    test_multiple_fixed_points_impl(true)
}

#[test]
fn test_single_4_cycle_3vars() -> Cancellable<()> {
    test_single_4_cycle_3vars_impl(false)
}

#[test]
fn test_single_4_cycle_3vars_with_itgr() -> Cancellable<()> {
    test_single_4_cycle_3vars_impl(true)
}

#[test]
fn test_cycle_with_transient_states() -> Cancellable<()> {
    test_cycle_with_transient_states_impl(false)
}

#[test]
fn test_cycle_with_transient_states_with_itgr() -> Cancellable<()> {
    test_cycle_with_transient_states_impl(true)
}
