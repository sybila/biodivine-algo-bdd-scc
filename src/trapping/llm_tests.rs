//! Tests for trapping algorithms using the example test network.
//!
//! See `llm_example_network.rs` for the complete documentation of the test network structure.
//!
//! Trapping algorithms compute the greatest forward-closed or backward-closed subset of the initial set.
//! A forward trap set is a set where all successors of states in the set are also in the set.
//! A backward trap set is a set where all predecessors of states in the set are also in the set.

use crate::test_utils::llm_example_network::create_test_network;
use crate::test_utils::llm_example_network::sets::{
    ALL_STATES, ATTRACTOR_1, ATTRACTOR_2, SOURCE_STATES, STRONG_BASIN_ATTR1, WEAK_BASIN,
};
use crate::test_utils::llm_example_network::states::*;
use crate::test_utils::{init_logger, mk_state, mk_states};
use crate::trapping::{BackwardTrap, ForwardTrap};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use cancel_this::Cancellable;
use computation_process::Algorithm;

// ========== ForwardTrap tests ==========

/// ForwardTrap computes the greatest forward-closed subset of the initial set.
/// A forward trap set S satisfies: for every state s in S, all successors of s are also in S.

#[test]
fn test_forward_trap_from_empty_set() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let empty = graph.mk_empty_colored_vertices();

    let result = ForwardTrap::run(&graph, empty.clone())?;

    assert!(
        result.is_empty(),
        "Forward trap from empty set should be empty"
    );
    Ok(())
}

#[test]
fn test_forward_trap_from_all_states() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let all = graph.mk_unit_colored_vertices();

    let result = ForwardTrap::run(&graph, all.clone())?;

    // All states together form a forward trap (all successors are in the set)
    assert_eq!(
        result, all,
        "Forward trap from all states should be all states"
    );
    Ok(())
}

#[test]
fn test_forward_trap_from_attractor_1() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractor_1 = mk_states(&graph, ATTRACTOR_1);

    let result = ForwardTrap::run(&graph, attractor_1.clone())?;

    // 000 has no successors, so {000} is forward-closed
    assert_eq!(
        result, attractor_1,
        "Forward trap from {{000}} should remain {{000}} (forward-closed)"
    );
    Ok(())
}

#[test]
fn test_forward_trap_from_attractor_2() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    let result = ForwardTrap::run(&graph, attractor_2.clone())?;

    // 110 → 111 (in set), 111 → 110 (in set), so {110, 111} is forward-closed
    assert_eq!(
        result, attractor_2,
        "Forward trap from {{110, 111}} should remain {{110, 111}} (forward-closed)"
    );
    Ok(())
}

#[test]
fn test_forward_trap_from_single_source_state() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let s011 = mk_state(&graph, S011);

    let result = ForwardTrap::run(&graph, s011.clone())?;

    // 011 → {001, 010, 111}, but none of these are in {011}, so 011 should be removed
    assert!(
        result.is_empty(),
        "Forward trap from {{011}} should be empty (011 has successors outside the set)"
    );
    Ok(())
}

#[test]
fn test_forward_trap_from_single_basin_state() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let s001 = mk_state(&graph, S001);

    let result = ForwardTrap::run(&graph, s001.clone())?;

    // 001 → 000, but 000 is not in {001}, so 001 should be removed
    assert!(
        result.is_empty(),
        "Forward trap from {{001}} should be empty (001 has successor 000 outside the set)"
    );
    Ok(())
}

#[test]
fn test_forward_trap_from_attractor_1_plus_basin() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractor_1 = mk_states(&graph, ATTRACTOR_1);
    let basin = mk_states(&graph, STRONG_BASIN_ATTR1);
    let initial = attractor_1.union(&basin);

    let result = ForwardTrap::run(&graph, initial.clone())?;

    // {000, 001, 010}: 001 → 000 (in set), 010 → 000 (in set), 000 has no successors
    // This is forward-closed, so should remain unchanged
    assert_eq!(
        result, initial,
        "Forward trap from {{000, 001, 010}} should remain unchanged (forward-closed)"
    );
    Ok(())
}

#[test]
fn test_forward_trap_from_attractor_2_plus_basin() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);
    let s101 = mk_state(&graph, S101);
    let initial = attractor_2.union(&s101);

    let result = ForwardTrap::run(&graph, initial.clone())?;

    // {101, 110, 111}: 101 → 111 (in set), 110 → 111 (in set), 111 → 110 (in set)
    // This is forward-closed, so should remain unchanged
    assert_eq!(
        result, initial,
        "Forward trap from {{101, 110, 111}} should remain unchanged (forward-closed)"
    );
    Ok(())
}

#[test]
fn test_forward_trap_from_mixed_set_with_gap() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    // {000, 001}: 001 → 000 (in set), 000 has no successors, so forward-closed
    let mixed = mk_states(&graph, &[S000, S001]);

    let result = ForwardTrap::run(&graph, mixed.clone())?;

    assert_eq!(
        result, mixed,
        "Forward trap from {{000, 001}} should remain unchanged (forward-closed)"
    );
    Ok(())
}

#[test]
fn test_forward_trap_from_mixed_set_with_leak() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    // {001, 010}: 001 → 000 (not in set), 010 → 000 (not in set)
    // Both states have successors outside the set, so both should be removed
    let mixed = mk_states(&graph, &[S001, S010]);

    let result = ForwardTrap::run(&graph, mixed.clone())?;

    assert!(
        result.is_empty(),
        "Forward trap from {{001, 010}} should be empty (both have successors outside the set)"
    );
    Ok(())
}

#[test]
fn test_forward_trap_from_weak_basin() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let weak_basin = mk_states(&graph, WEAK_BASIN);

    let result = ForwardTrap::run(&graph, weak_basin.clone())?;

    // {011, 100}: 011 → {001, 010, 111} (all outside), 100 → {000, 110} (all outside)
    // Both states have successors outside the set, so both should be removed
    assert!(
        result.is_empty(),
        "Forward trap from {{011, 100}} should be empty (both have successors outside the set)"
    );
    Ok(())
}

#[test]
fn test_forward_trap_from_source_states() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let sources = mk_states(&graph, SOURCE_STATES);

    let result = ForwardTrap::run(&graph, sources.clone())?;

    // {011, 100, 101}: All have successors outside the set
    // 011 → {001, 010, 111}, 100 → {000, 110}, 101 → 111
    // The algorithm iteratively removes states with successors outside
    // Eventually all should be removed
    assert!(
        result.is_empty(),
        "Forward trap from {{011, 100, 101}} should be empty (all have successors outside the set)"
    );
    Ok(())
}

#[test]
fn test_forward_trap_is_subset_of_initial() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();

    // Forward trap always returns a subset of the initial set
    for state in ALL_STATES {
        let initial = mk_state(&graph, *state);
        let result = ForwardTrap::run(&graph, initial.clone())?;
        assert!(
            result.is_subset(&initial),
            "Forward trap from {:03b} must return a subset of the initial set",
            state
        );
    }
    Ok(())
}

#[test]
fn test_forward_trap_from_cycle_with_one_outside() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    // {110, 111, 101}: 101 → 111 (in set), 110 → 111 (in set), 111 → 110 (in set)
    // This is forward-closed
    let cycle_plus = mk_states(&graph, &[S101, S110, S111]);

    let result = ForwardTrap::run(&graph, cycle_plus.clone())?;

    assert_eq!(
        result, cycle_plus,
        "Forward trap from {{101, 110, 111}} should remain unchanged (forward-closed)"
    );
    Ok(())
}

// ========== BackwardTrap tests ==========

/// BackwardTrap computes the greatest backward-closed subset of the initial set.
/// A backward trap set S satisfies: for every state s in S, all predecessors of s are also in S.

#[test]
fn test_backward_trap_from_empty_set() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let empty = graph.mk_empty_colored_vertices();

    let result = BackwardTrap::run(&graph, empty.clone())?;

    assert!(
        result.is_empty(),
        "Backward trap from empty set should be empty"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_all_states() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let all = graph.mk_unit_colored_vertices();

    let result = BackwardTrap::run(&graph, all.clone())?;

    // All states together form a backward trap (all predecessors are in the set)
    assert_eq!(
        result, all,
        "Backward trap from all states should be all states"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_attractor_1() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractor_1 = mk_states(&graph, ATTRACTOR_1);

    let result = BackwardTrap::run(&graph, attractor_1.clone())?;

    // 000 has predecessors {001, 010, 100}, but none of these are in {000}
    // So 000 should be removed
    assert!(
        result.is_empty(),
        "Backward trap from {{000}} should be empty (000 has predecessors outside the set)"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_attractor_2() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    let result = BackwardTrap::run(&graph, attractor_2.clone())?;

    // 110 has predecessors {100, 111} (111 is in set, but 100 is not)
    // 111 has predecessors {011, 101, 110} (110 is in set, but 011 and 101 are not)
    // So states with predecessors outside should be removed iteratively
    // Eventually all should be removed
    assert!(
        result.is_empty(),
        "Backward trap from {{110, 111}} should be empty (both have predecessors outside the set)"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_source_state() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let s011 = mk_state(&graph, S011);

    let result = BackwardTrap::run(&graph, s011.clone())?;

    // 011 has no predecessors (it's a source), so {011} is backward-closed
    assert_eq!(
        result, s011,
        "Backward trap from {{011}} should remain {{011}} (backward-closed, no predecessors)"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_all_source_states() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let sources = mk_states(&graph, SOURCE_STATES);

    let result = BackwardTrap::run(&graph, sources.clone())?;

    // All source states have no predecessors, so they form a backward trap
    assert_eq!(
        result, sources,
        "Backward trap from {{011, 100, 101}} should remain unchanged (all are sources, backward-closed)"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_attractor_1_plus_basin() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractor_1 = mk_states(&graph, ATTRACTOR_1);
    let basin = mk_states(&graph, STRONG_BASIN_ATTR1);
    let initial = attractor_1.union(&basin);

    let result = BackwardTrap::run(&graph, initial.clone())?;

    // {000, 001, 010}:
    // - 000 has predecessors {001, 010, 100} (001 and 010 are in set, but 100 is not)
    // - 001 has predecessor {011} (not in set)
    // - 010 has predecessor {011} (not in set)
    // So states with predecessors outside should be removed iteratively
    // Eventually all should be removed
    assert!(
        result.is_empty(),
        "Backward trap from {{000, 001, 010}} should be empty (all have predecessors outside the set)"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_attractor_2_plus_basin() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);
    let s101 = mk_state(&graph, S101);
    let initial = attractor_2.union(&s101);

    let result = BackwardTrap::run(&graph, initial.clone())?;

    // {101, 110, 111}:
    // - 101 has no predecessors (source), so it's backward-closed
    // - 110 has predecessors {100, 111} (111 is in set, but 100 is not)
    // - 111 has predecessors {011, 101, 110} (101 and 110 are in set, but 011 is not)
    // So 110 and 111 should be removed, leaving {101}
    let expected = s101;
    assert_eq!(
        result, expected,
        "Backward trap from {{101, 110, 111}} should be {{101}} (only 101 has no predecessors outside)"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_basin_state() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let s001 = mk_state(&graph, S001);

    let result = BackwardTrap::run(&graph, s001.clone())?;

    // 001 has predecessor {011}, but 011 is not in {001}, so 001 should be removed
    assert!(
        result.is_empty(),
        "Backward trap from {{001}} should be empty (001 has predecessor 011 outside the set)"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_weak_basin() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let weak_basin = mk_states(&graph, WEAK_BASIN);

    let result = BackwardTrap::run(&graph, weak_basin.clone())?;

    // {011, 100}: Both are sources (no predecessors), so backward-closed
    assert_eq!(
        result, weak_basin,
        "Backward trap from {{011, 100}} should remain unchanged (both are sources, backward-closed)"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_mixed_set_with_gap() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    // {011, 001}: 011 has no predecessors (source), 001 has predecessor {011} (in set)
    // This is backward-closed
    let mixed = mk_states(&graph, &[S011, S001]);

    let result = BackwardTrap::run(&graph, mixed.clone())?;

    assert_eq!(
        result, mixed,
        "Backward trap from {{011, 001}} should remain unchanged (backward-closed)"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_mixed_set_with_leak() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    // {000, 001}: 000 has predecessors {001, 010, 100} (001 is in set, but 010 and 100 are not)
    // 001 has predecessor {011} (not in set)
    // So states with predecessors outside should be removed iteratively
    // Eventually all should be removed
    let mixed = mk_states(&graph, &[S000, S001]);

    let result = BackwardTrap::run(&graph, mixed.clone())?;

    assert!(
        result.is_empty(),
        "Backward trap from {{000, 001}} should be empty (both have predecessors outside the set)"
    );
    Ok(())
}

#[test]
fn test_backward_trap_is_subset_of_initial() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();

    // Backward trap always returns a subset of the initial set
    for state in ALL_STATES {
        let initial = mk_state(&graph, *state);
        let result = BackwardTrap::run(&graph, initial.clone())?;
        assert!(
            result.is_subset(&initial),
            "Backward trap from {:03b} must return a subset of the initial set",
            state
        );
    }
    Ok(())
}

#[test]
fn test_backward_trap_from_complete_basin() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    // {011, 001, 010, 000}: Complete basin of attractor 1
    // - 011 has no predecessors (source)
    // - 001 has predecessor {011} (in set)
    // - 010 has predecessor {011} (in set)
    // - 000 has predecessors {001, 010, 100} (001 and 010 are in set, but 100 is not)
    // So 000 should be removed, leaving {011, 001, 010}
    let basin = mk_states(&graph, &[S011, S001, S010, S000]);

    let result = BackwardTrap::run(&graph, basin.clone())?;

    let expected = mk_states(&graph, &[S011, S001, S010]);
    assert_eq!(
        result, expected,
        "Backward trap from complete basin should be {{011, 001, 010}} (000 has predecessor 100 outside)"
    );
    Ok(())
}
