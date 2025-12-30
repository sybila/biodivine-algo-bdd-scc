//! Tests for the long-lived SCC filtering feature.
//!
//! A long-lived SCC is one that cannot be escaped by updating a single variable.
//! Specifically, there is no variable such that ALL states in the SCC can escape
//! via that variable's update.
//!
//! A short-lived SCC has some variable where ALL states can transition outside the SCC
//! by updating that variable.

use crate::scc::retain_long_lived;
use crate::scc::{FwdBwdScc, SccConfig};
use crate::test_utils::llm_transition_builder::from_transitions;
use crate::test_utils::mk_states;
use crate::test_utils::{init_logger, symbolic_sets_to_sorted_sets};
use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use computation_process::Stateful;
use std::collections::HashSet;

/// Create a test network with two non-trivial SCCs:
/// - Long-lived SCC: {000, 100} - a 2-cycle with no escape
/// - Short-lived SCC: {011, 111} - a 2-cycle where ALL states can escape via x1
///   (011 → 001 and 111 → 101)
///
/// States 001 and 101 are fixed points (sinks).
fn create_long_lived_test_network() -> SymbolicAsyncGraph {
    let transitions = vec![
        // Long-lived SCC: {000, 100} - no escape
        (0b000, 0b100), // 000 → 100 (x0 flips)
        (0b100, 0b000), // 100 → 000 (x0 flips)
        // Short-lived SCC: {011, 111} - ALL states can escape via x1
        (0b011, 0b111), // 011 → 111 (x0 flips) - cycle
        (0b111, 0b011), // 111 → 011 (x0 flips) - cycle
        (0b011, 0b001), // 011 → 001 (x1 flips) - ESCAPE via x1
        (0b111, 0b101), // 111 → 101 (x1 flips) - ESCAPE via x1
    ];

    let bn = from_transitions(3, &transitions).expect("Failed to create network");
    SymbolicAsyncGraph::new(&bn).expect("Failed to create graph")
}

/// Test that without long-lived filtering, both SCCs are reported.
#[test]
fn test_without_long_lived_filter_reports_both_sccs() {
    init_logger();
    let graph = create_long_lived_test_network();

    // Default config: filter_long_lived = false
    let config = SccConfig::new(graph.clone());
    assert!(!config.filter_long_lived, "Default should be false");

    let mut generator = FwdBwdScc::configure(config, &graph);
    let mut found_sccs = Vec::new();

    while let Some(result) = generator.next() {
        found_sccs.push(result.unwrap());
    }

    // Should find exactly two SCCs
    assert_eq!(
        found_sccs.len(),
        2,
        "Expected 2 SCCs without long-lived filtering, but found {}",
        found_sccs.len()
    );

    // Verify the SCCs are the expected ones
    let found_sets = symbolic_sets_to_sorted_sets(&graph, &found_sccs, 3);

    let expected_long_lived: HashSet<u32> = [0b000, 0b100].iter().copied().collect();
    let expected_short_lived: HashSet<u32> = [0b011, 0b111].iter().copied().collect();

    assert!(
        found_sets.contains(&expected_long_lived),
        "Expected to find long-lived SCC {{000, 100}}, found: {:?}",
        found_sets
    );
    assert!(
        found_sets.contains(&expected_short_lived),
        "Expected to find short-lived SCC {{011, 111}}, found: {:?}",
        found_sets
    );
}

/// Test that with long-lived filtering, only the long-lived SCC is reported.
#[test]
fn test_with_long_lived_filter_reports_only_long_lived_scc() {
    init_logger();
    let graph = create_long_lived_test_network();

    // Enable long-lived filtering
    let mut config = SccConfig::new(graph.clone());
    config.filter_long_lived = true;
    assert!(config.filter_long_lived, "Should be enabled");

    let mut generator = FwdBwdScc::configure(config, &graph);
    let mut found_sccs = Vec::new();

    while let Some(result) = generator.next() {
        found_sccs.push(result.unwrap());
    }

    // Should find exactly one SCC (the long-lived one)
    assert_eq!(
        found_sccs.len(),
        1,
        "Expected 1 SCC with long-lived filtering, but found {}",
        found_sccs.len()
    );

    // Verify it's the long-lived SCC
    let found_sets = symbolic_sets_to_sorted_sets(&graph, &found_sccs, 3);

    let expected_long_lived: HashSet<u32> = [0b000, 0b100].iter().copied().collect();

    assert_eq!(
        found_sets[0], expected_long_lived,
        "Expected long-lived SCC {{000, 100}}, found: {:?}",
        found_sets[0]
    );
}

/// Create a parameterized 2-variable network that tests the multicolor logic in `retain_long_lived`.
///
/// The network has one parameter `p` that switches between two behaviors:
/// - When p=false (Network 1, SHORT-LIVED): A' = !A, B' = B
/// - When p=true (Network 2, LONG-LIVED): A' = A ^ B, B' = A ^ B
///
/// For the test set S = {00, 11}:
/// - Network 1 (p=false): ALL states can escape via A → short-lived; should be filtered
/// - Network 2 (p=true): State 00 cannot escape via any variable → long-lived; should be kept
fn create_parameterized_long_lived_network() -> SymbolicAsyncGraph {
    // AEON model with parameter p:
    // When p=false: A' = !A, B' = B (Network 1 - short-lived)
    // When p=true: A' = A^B, B' = A^B (Network 2 - long-lived)
    //
    // We use observable edges (-?) to avoid monotonicity constraints.
    let aeon_model = r#"
        A -? A
        B -? A
        A -? B
        B -? B
        $A: (p & (A ^ B)) | (!p & !A)
        $B: (p & (A ^ B)) | (!p & B)
    "#;

    let bn = BooleanNetwork::try_from(aeon_model).expect("Failed to parse AEON model");
    SymbolicAsyncGraph::new(&bn).expect("Failed to create graph")
}

/// Test that `retain_long_lived` correctly filters colors using intersection logic.
///
/// This test directly calls `retain_long_lived` with a multicolor set and verifies
/// that only the long-lived color is retained.
///
/// - Start: safe_colors = {p=true, p=false}
/// - After checking A: safe_colors = {p=true} (Network 1 has all states escaping via A)
/// - After checking B: safe_colors = {p=true} ∩ {p=true, p=false} = {p=true} (CORRECT!)
#[test]
fn test_retain_long_lived_multi_color_uses_intersection() {
    init_logger();
    let graph = create_parameterized_long_lived_network();

    // Verify we have 2 colors (p=true and p=false)
    let all_colors = graph.mk_unit_colored_vertices().colors();
    assert_eq!(
        all_colors.exact_cardinality(),
        2u32.into(),
        "Expected 2 colors (p=true and p=false)"
    );

    // Create the test set S = {00, 11} with all colors
    // State encoding: A is variable 0 (MSB), B is variable 1 (LSB)
    // 00 = state 0b00 = 0
    // 11 = state 0b11 = 3
    let test_set = mk_states(&graph, &[0b00, 0b11]);

    // Verify the test set has 2 states × 2 colors = 4 state-color pairs
    assert_eq!(
        test_set.exact_cardinality(),
        4u32.into(),
        "Expected 4 state-color pairs (2 states × 2 colors)"
    );

    // Call retain_long_lived
    let result = retain_long_lived(&graph, &test_set);

    // The result should only contain color p=true (the long-lived network),
    // which means 2 states × 1 color = 2 state-color pairs
    assert_eq!(
        result.exact_cardinality(),
        2u32.into(),
        "Expected 2 state-color pairs (2 states × 1 color). \
         If 4 pairs remain, the bug is that union was used instead of intersection."
    );

    // Verify the result contains only 1 color
    let result_colors = result.colors();
    assert_eq!(
        result_colors.exact_cardinality(),
        1u32.into(),
        "Expected exactly 1 color to be retained (p=true). \
         If 2 colors remain, the bug is that union was used instead of intersection."
    );

    // Verify the remaining states are still {00, 11}
    let remaining_vertices = result.vertices();
    let expected_vertices = test_set.vertices();
    assert_eq!(
        remaining_vertices, expected_vertices,
        "The retained states should still be {{00, 11}}"
    );
}
