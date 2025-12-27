//! Tests for the long-lived SCC filtering feature.
//!
//! A long-lived SCC is one that cannot be escaped by updating a single variable.
//! Specifically, there is no variable such that ALL states in the SCC can escape
//! via that variable's update.
//!
//! A short-lived SCC has some variable where ALL states can transition outside the SCC
//! by updating that variable.

use crate::algorithm::scc::tests::sccs_to_sorted_sets;
use crate::algorithm::scc::{FwdBwdScc, SccConfig};
use crate::algorithm::test_utils::init_logger;
use crate::algorithm::test_utils::llm_transition_builder::from_transitions;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
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
    let found_sets = sccs_to_sorted_sets(&graph, &found_sccs, 3);

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
    let config = SccConfig::new(graph.clone()).filter_long_lived(true);
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
    let found_sets = sccs_to_sorted_sets(&graph, &found_sccs, 3);

    let expected_long_lived: HashSet<u32> = [0b000, 0b100].iter().copied().collect();

    assert_eq!(
        found_sets[0], expected_long_lived,
        "Expected long-lived SCC {{000, 100}}, found: {:?}",
        found_sets[0]
    );
}
