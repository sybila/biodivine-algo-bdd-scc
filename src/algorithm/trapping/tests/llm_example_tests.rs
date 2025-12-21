//! Tests for trapping algorithms using the example test network.
//!
//! See `llm_example_network.rs` for the complete documentation of the test network structure.
//!
//! A forward trap set is a subset of states where no state has an outgoing transition
//! leading outside the set. A backward trap set is a subset of states where no state
//! can be reached by anything outside the set.

use crate::Algorithm;
use crate::algorithm::test_utils::llm_example_network::sets::{
    ALL_STATES, ATTRACTOR_1, ATTRACTOR_2, CAN_REACH_ATTR1, CAN_REACH_ATTR2, STRONG_BASIN_ATTR1,
};
use crate::algorithm::test_utils::llm_example_network::states::*;
use crate::algorithm::test_utils::llm_example_network::{create_test_network, mk_state, mk_states};
use crate::algorithm::trapping::{TrappingBackward, TrappingForward};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use cancel_this::Cancellable;

/// Initialize env_logger for tests. Safe to call multiple times.
fn init_logger() {
    let _ = env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Trace)
        .is_test(true)
        .try_init();
}

/// Verify that a set is a forward trap set: all successors of states in the set
/// must also be in the set.
fn verify_forward_trap_set(
    graph: &biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph,
    trap_set: &biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices,
) -> bool {
    let post = graph.post(trap_set);
    post.is_subset(trap_set)
}

/// Verify that a set is a backward trap set: all predecessors of states in the set
/// must also be in the set.
fn verify_backward_trap_set(
    graph: &biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph,
    trap_set: &biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices,
) -> bool {
    let pre = graph.pre(trap_set);
    pre.is_subset(trap_set)
}

// ========== Forward trapping tests ==========

#[test]
fn test_forward_trap_from_empty_set() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let empty = graph.mk_empty_colored_vertices();

    let result = TrappingForward::compute((graph.clone(), empty.clone()))?;

    assert!(
        result.is_empty(),
        "Forward trap from empty set should be empty"
    );
    assert!(
        verify_forward_trap_set(&graph, &result),
        "Result should be a valid forward trap set"
    );
    Ok(())
}

#[test]
fn test_forward_trap_from_all_states() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let all = graph.mk_unit_colored_vertices();

    let result = TrappingForward::compute((graph.clone(), all.clone()))?;

    // The entire state space is always a forward trap set
    assert_eq!(
        result, all,
        "Forward trap from all states should be all states"
    );
    assert!(
        verify_forward_trap_set(&graph, &result),
        "Result should be a valid forward trap set"
    );
    Ok(())
}

#[test]
fn test_forward_trap_from_fixed_point() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let s000 = mk_state(&graph, S000);

    let result = TrappingForward::compute((graph.clone(), s000.clone()))?;

    // A fixed point is already a forward trap set (no outgoing transitions)
    assert_eq!(
        result, s000,
        "Forward trap from fixed point 000 should be just {{000}}"
    );
    assert!(
        verify_forward_trap_set(&graph, &result),
        "Result should be a valid forward trap set"
    );
    Ok(())
}

#[test]
fn test_forward_trap_from_attractor_2() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    let result = TrappingForward::compute((graph.clone(), attractor_2.clone()))?;

    // The cycle {110, 111} is already a forward trap set
    assert_eq!(
        result, attractor_2,
        "Forward trap from attractor 2 should be {{110, 111}}"
    );
    assert!(
        verify_forward_trap_set(&graph, &result),
        "Result should be a valid forward trap set"
    );
    Ok(())
}

#[test]
fn test_forward_trap_from_strong_basin_attr1() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let basin = mk_states(&graph, STRONG_BASIN_ATTR1);

    let result = TrappingForward::compute((graph.clone(), basin.clone()))?;

    // A forward trap Y ⊆ {001, 010} means: no state in Y can reach any state outside Y.
    // From 001, we can reach 000 (outside {001, 010}), so 001 cannot be in the trap.
    // From 010, we can reach 000 (outside {001, 010}), so 010 cannot be in the trap.
    // Therefore, the trap set must be empty.
    assert!(
        result.is_empty(),
        "Forward trap from {{001, 010}} should be empty (both can reach 000 outside)"
    );
    assert!(
        verify_forward_trap_set(&graph, &result),
        "Result should be a valid forward trap set"
    );
    Ok(())
}

#[test]
fn test_forward_trap_from_strong_basin_attr2() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let s101 = mk_state(&graph, S101);

    let result = TrappingForward::compute((graph.clone(), s101.clone()))?;

    // A forward trap Y ⊆ {101} means: no state in Y can reach any state outside Y.
    // From 101, we can reach 111 (outside {101}), so 101 cannot be in the trap.
    // Therefore, the trap set must be empty.
    assert!(
        result.is_empty(),
        "Forward trap from {{101}} should be empty (can reach 111 outside)"
    );
    assert!(
        verify_forward_trap_set(&graph, &result),
        "Result should be a valid forward trap set"
    );
    Ok(())
}

#[test]
fn test_forward_trap_from_weak_basin_011() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let s011 = mk_state(&graph, S011);

    let result = TrappingForward::compute((graph.clone(), s011.clone()))?;

    // From 011, we can reach {001, 010, 111}. The trap set should include
    // all states that can be reached and form a trap. Since 011 can reach
    // both attractors, the trap set should be empty (no forward trap exists
    // that contains 011 and is a subset of {011}).
    // Actually, wait - let me think: if we start with {011}, we need to find
    // the greatest forward trap subset. 011 can reach 001, 010, 111, 000, 110.
    // None of these form a trap that includes 011, so the result should be empty.
    assert!(
        result.is_empty(),
        "Forward trap from {{011}} should be empty (011 can reach outside any trap containing it)"
    );
    assert!(
        verify_forward_trap_set(&graph, &result),
        "Result should be a valid forward trap set"
    );
    Ok(())
}

#[test]
fn test_forward_trap_from_weak_basin_100() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let s100 = mk_state(&graph, S100);

    let result = TrappingForward::compute((graph.clone(), s100.clone()))?;

    // From 100, we can reach {000, 110}. The trap set should include
    // all states that can be reached and form a trap. Since 100 can reach
    // both attractors, the trap set should be empty.
    assert!(
        result.is_empty(),
        "Forward trap from {{100}} should be empty (100 can reach outside any trap containing it)"
    );
    assert!(
        verify_forward_trap_set(&graph, &result),
        "Result should be a valid forward trap set"
    );
    Ok(())
}

#[test]
fn test_forward_trap_is_subset_of_initial() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();

    // For any starting set, forward trap must be a subset of the initial set
    for state in ALL_STATES {
        let initial = mk_state(&graph, *state);
        let result = TrappingForward::compute((graph.clone(), initial.clone()))?;
        assert!(
            result.is_subset(&initial),
            "Forward trap from {:03b} must be a subset of the initial set",
            state
        );
        assert!(
            verify_forward_trap_set(&graph, &result),
            "Result should be a valid forward trap set"
        );
    }
    Ok(())
}

#[test]
fn test_forward_trap_from_attractor_1_plus_basin() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractor_1 = mk_states(&graph, ATTRACTOR_1);
    let basin = mk_states(&graph, STRONG_BASIN_ATTR1);
    let initial = attractor_1.union(&basin);

    let result = TrappingForward::compute((graph.clone(), initial.clone()))?;

    // The trap set should be {000, 001, 010} because:
    // - 001 → 000 (000 is in the set, so OK)
    // - 010 → 000 (000 is in the set, so OK)
    // - 000 has no outgoing transitions
    assert_eq!(
        result, initial,
        "Forward trap from {{000, 001, 010}} should be {{000, 001, 010}}"
    );
    assert!(
        verify_forward_trap_set(&graph, &result),
        "Result should be a valid forward trap set"
    );
    Ok(())
}

// ========== Backward trapping tests ==========

#[test]
fn test_backward_trap_from_empty_set() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let empty = graph.mk_empty_colored_vertices();

    let result = TrappingBackward::compute((graph.clone(), empty.clone()))?;

    assert!(
        result.is_empty(),
        "Backward trap from empty set should be empty"
    );
    assert!(
        verify_backward_trap_set(&graph, &result),
        "Result should be a valid backward trap set"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_all_states() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let all = graph.mk_unit_colored_vertices();

    let result = TrappingBackward::compute((graph.clone(), all.clone()))?;

    // The entire state space is always a backward trap set
    assert_eq!(
        result, all,
        "Backward trap from all states should be all states"
    );
    assert!(
        verify_backward_trap_set(&graph, &result),
        "Result should be a valid backward trap set"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_fixed_point() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let s000 = mk_state(&graph, S000);

    let result = TrappingBackward::compute((graph.clone(), s000.clone()))?;

    // A backward trap Y ⊆ {000} means: no state outside {000} can reach any state in Y.
    // Since states outside {000} (like 001, 010, 100) can reach 000, the trap set must be empty.
    assert!(
        result.is_empty(),
        "Backward trap from {{000}} should be empty (states outside can reach 000)"
    );
    assert!(
        verify_backward_trap_set(&graph, &result),
        "Result should be a valid backward trap set"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_attractor_2() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    let result = TrappingBackward::compute((graph.clone(), attractor_2.clone()))?;

    // A backward trap Y ⊆ {110, 111} means: no state outside {110, 111} can reach any state in Y.
    // Since states outside (like 011, 100, 101) can reach {110, 111}, the trap set must be empty.
    assert!(
        result.is_empty(),
        "Backward trap from attractor 2 should be empty (states outside can reach the attractor)"
    );
    assert!(
        verify_backward_trap_set(&graph, &result),
        "Result should be a valid backward trap set"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_source_state_011() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let s011 = mk_state(&graph, S011);

    let result = TrappingBackward::compute((graph.clone(), s011.clone()))?;

    // 011 is a source (no predecessors), so the backward trap should be just {011}
    assert_eq!(
        result, s011,
        "Backward trap from {{011}} should be {{011}} (source state)"
    );
    assert!(
        verify_backward_trap_set(&graph, &result),
        "Result should be a valid backward trap set"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_source_state_100() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let s100 = mk_state(&graph, S100);

    let result = TrappingBackward::compute((graph.clone(), s100.clone()))?;

    // 100 is a source (no predecessors), so the backward trap should be just {100}
    assert_eq!(
        result, s100,
        "Backward trap from {{100}} should be {{100}} (source state)"
    );
    assert!(
        verify_backward_trap_set(&graph, &result),
        "Result should be a valid backward trap set"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_source_state_101() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let s101 = mk_state(&graph, S101);

    let result = TrappingBackward::compute((graph.clone(), s101.clone()))?;

    // 101 is a source (no predecessors), so the backward trap should be just {101}
    assert_eq!(
        result, s101,
        "Backward trap from {{101}} should be {{101}} (source state)"
    );
    assert!(
        verify_backward_trap_set(&graph, &result),
        "Result should be a valid backward trap set"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_transient_state_001() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let s001 = mk_state(&graph, S001);

    let result = TrappingBackward::compute((graph.clone(), s001.clone()))?;

    // A backward trap Y ⊆ {001} means: no state outside {001} can reach any state in Y.
    // Since state 011 (outside {001}) can reach 001, the trap set must be empty.
    assert!(
        result.is_empty(),
        "Backward trap from {{001}} should be empty (state 011 outside can reach 001)"
    );
    assert!(
        verify_backward_trap_set(&graph, &result),
        "Result should be a valid backward trap set"
    );
    Ok(())
}

#[test]
fn test_backward_trap_is_subset_of_initial() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();

    // For any starting set, backward trap must be a subset of the initial set
    for state in ALL_STATES {
        let initial = mk_state(&graph, *state);
        let result = TrappingBackward::compute((graph.clone(), initial.clone()))?;
        assert!(
            result.is_subset(&initial),
            "Backward trap from {:03b} must be a subset of the initial set",
            state
        );
        assert!(
            verify_backward_trap_set(&graph, &result),
            "Result should be a valid backward trap set"
        );
    }
    Ok(())
}

#[test]
fn test_backward_trap_from_can_reach_attr1() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let can_reach_attr1 = mk_states(&graph, CAN_REACH_ATTR1);

    let result = TrappingBackward::compute((graph.clone(), can_reach_attr1.clone()))?;

    // If we start with all states that can reach attractor 1, the backward trap
    // should be the same set (it's already a backward trap)
    assert_eq!(
        result, can_reach_attr1,
        "Backward trap from CAN_REACH_ATTR1 should be itself"
    );
    assert!(
        verify_backward_trap_set(&graph, &result),
        "Result should be a valid backward trap set"
    );
    Ok(())
}

#[test]
fn test_backward_trap_from_can_reach_attr2() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let can_reach_attr2 = mk_states(&graph, CAN_REACH_ATTR2);

    let result = TrappingBackward::compute((graph.clone(), can_reach_attr2.clone()))?;

    // If we start with all states that can reach attractor 2, the backward trap
    // should be the same set (it's already a backward trap)
    assert_eq!(
        result, can_reach_attr2,
        "Backward trap from CAN_REACH_ATTR2 should be itself"
    );
    assert!(
        verify_backward_trap_set(&graph, &result),
        "Result should be a valid backward trap set"
    );
    Ok(())
}
