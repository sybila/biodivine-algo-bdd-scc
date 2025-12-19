//! Tests for naive reachability algorithms using the canonical test network.
//!
//! See `test_network.rs` for the complete documentation of the test network structure.

use super::llm_test_network::sets::{
    ALL_STATES, ATTRACTOR_1, ATTRACTOR_2, CAN_REACH_ATTR1, CAN_REACH_ATTR2, SOURCE_STATES,
    STRONG_BASIN_ATTR1, STRONG_BASIN_ATTR2, WEAK_BASIN,
};
use super::llm_test_network::states::*;
use super::llm_test_network::{create_test_network, mk_state, mk_states};
use crate::Reachability;
use biodivine_lib_param_bn::biodivine_std::traits::Set;

// ========== Tests for reach_forward_naive ==========

#[test]
fn test_reach_forward_from_empty_set() {
    let graph = create_test_network();
    let empty = graph.mk_empty_colored_vertices();

    let result = Reachability::reach_forward_naive(&graph, empty).unwrap();

    assert!(
        result.is_empty(),
        "Forward reach from empty set should be empty"
    );
}

#[test]
fn test_reach_forward_from_fixed_point() {
    let graph = create_test_network();
    let s000 = mk_state(&graph, S000);

    let result = Reachability::reach_forward_naive(&graph, s000.clone()).unwrap();

    // A fixed point can only reach itself
    assert_eq!(
        result, s000,
        "Forward reach from fixed point 000 should be just {{000}}"
    );
}

#[test]
fn test_reach_forward_from_attractor_2() {
    let graph = create_test_network();
    let s110 = mk_state(&graph, S110);
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    let result = Reachability::reach_forward_naive(&graph, s110).unwrap();

    // From 110, we can reach 111 (and back), so we reach the whole attractor
    assert_eq!(
        result, attractor_2,
        "Forward reach from 110 should be {{110, 111}}"
    );
}

#[test]
fn test_reach_forward_from_strong_basin_of_attractor_1() {
    let graph = create_test_network();
    let s001 = mk_state(&graph, S001);
    let s000 = mk_state(&graph, S000);

    let result = Reachability::reach_forward_naive(&graph, s001.clone()).unwrap();

    // 001 → 000 (fixed point), so we reach {001, 000}
    let expected = s001.union(&s000);
    assert_eq!(
        result, expected,
        "Forward reach from 001 should be {{000, 001}}"
    );
}

#[test]
fn test_reach_forward_from_strong_basin_of_attractor_2() {
    let graph = create_test_network();
    let s101 = mk_state(&graph, S101);
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    let result = Reachability::reach_forward_naive(&graph, s101.clone()).unwrap();

    // 101 → 111 → 110 → 111 ..., so we reach {101, 110, 111}
    let expected = s101.union(&attractor_2);
    assert_eq!(
        result, expected,
        "Forward reach from 101 should be {{101, 110, 111}}"
    );
}

#[test]
fn test_reach_forward_from_weak_basin_reaches_both_attractors() {
    let graph = create_test_network();
    let s011 = mk_state(&graph, S011);
    let s000 = mk_state(&graph, S000);
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    let result = Reachability::reach_forward_naive(&graph, s011).unwrap();

    // 011 can reach both attractors:
    // - 011 → 010 → 000 (attractor 1)
    // - 011 → 111 → 110 → 111 ... (attractor 2)
    assert!(
        !result.intersect(&s000).is_empty(),
        "Forward reach from 011 should include attractor 1 (000)"
    );
    assert!(
        !result.intersect(&attractor_2).is_empty(),
        "Forward reach from 011 should include attractor 2 (110, 111)"
    );
}

#[test]
fn test_reach_forward_from_weak_basin_100() {
    let graph = create_test_network();
    let s100 = mk_state(&graph, S100);
    let s000 = mk_state(&graph, S000);
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    let result = Reachability::reach_forward_naive(&graph, s100).unwrap();

    // 100 can reach both attractors:
    // - 100 → 000 (attractor 1)
    // - 100 → 110 → 111 → 110 ... (attractor 2)
    // Expected: {100, 000, 110, 111}
    let expected = mk_states(&graph, &[S100, S000, S110, S111]);
    assert_eq!(
        result, expected,
        "Forward reach from 100 should be {{000, 100, 110, 111}}"
    );

    // Verify it reaches both attractors
    assert!(!result.intersect(&s000).is_empty());
    assert!(!result.intersect(&attractor_2).is_empty());
}

#[test]
fn test_reach_forward_includes_initial() {
    let graph = create_test_network();

    // For any starting set, forward reachability must include the initial set
    for state in ALL_STATES {
        let initial = mk_state(&graph, *state);
        let result = Reachability::reach_forward_naive(&graph, initial.clone()).unwrap();
        assert!(
            initial.is_subset(&result),
            "Forward reach from {:03b} must include the initial state",
            state
        );
    }
}

// ========== Tests for reach_backward_naive ==========

#[test]
fn test_reach_backward_from_empty_set() {
    let graph = create_test_network();
    let empty = graph.mk_empty_colored_vertices();

    let result = Reachability::reach_backward_naive(&graph, empty).unwrap();

    assert!(
        result.is_empty(),
        "Backward reach from empty set should be empty"
    );
}

#[test]
fn test_reach_backward_to_fixed_point() {
    let graph = create_test_network();
    let s000 = mk_state(&graph, S000);

    let result = Reachability::reach_backward_naive(&graph, s000).unwrap();

    // States that can reach 000: {000, 001, 010, 011, 100}
    let expected = mk_states(&graph, CAN_REACH_ATTR1);
    assert_eq!(
        result, expected,
        "Backward reach to 000 should be {{000, 001, 010, 011, 100}}"
    );
}

#[test]
fn test_reach_backward_to_attractor_2() {
    let graph = create_test_network();
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    let result = Reachability::reach_backward_naive(&graph, attractor_2).unwrap();

    // States that can reach {110, 111}: {011, 100, 101, 110, 111}
    let expected = mk_states(&graph, CAN_REACH_ATTR2);
    assert_eq!(
        result, expected,
        "Backward reach to attractor 2 should be {{011, 100, 101, 110, 111}}"
    );
}

#[test]
fn test_reach_backward_from_single_state_in_cycle() {
    let graph = create_test_network();
    let s110 = mk_state(&graph, S110);

    let result = Reachability::reach_backward_naive(&graph, s110).unwrap();

    // Reaching just 110: 111 can reach 110 directly, then all states that can reach 111
    // Chain: 101 → 111 → 110, 100 → 110, 011 → 111 → 110
    // Expected: {110, 111, 101, 100, 011}
    let expected = mk_states(&graph, CAN_REACH_ATTR2);
    assert_eq!(
        result, expected,
        "Backward reach to 110 should be {{011, 100, 101, 110, 111}}"
    );
}

#[test]
fn test_reach_backward_strong_basin_states_only_reach_one_attractor() {
    let graph = create_test_network();

    // State 001 is in the strong basin of attractor 1 - it can ONLY reach 000
    let s001 = mk_state(&graph, S001);
    let s000 = mk_state(&graph, S000);
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    // Forward from 001 should not reach attractor 2
    let forward = Reachability::reach_forward_naive(&graph, s001).unwrap();
    assert!(
        forward.intersect(&attractor_2).is_empty(),
        "001 should not be able to reach attractor 2"
    );
    assert!(
        !forward.intersect(&s000).is_empty(),
        "001 should be able to reach attractor 1"
    );
}

#[test]
fn test_reach_backward_includes_initial() {
    let graph = create_test_network();

    // For any starting set, backward reachability must include the initial set
    for state in ALL_STATES {
        let initial = mk_state(&graph, *state);
        let result = Reachability::reach_backward_naive(&graph, initial.clone()).unwrap();
        assert!(
            initial.is_subset(&result),
            "Backward reach to {:03b} must include the initial state",
            state
        );
    }
}

// ========== Tests for trap_forward_naive ==========

#[test]
fn test_trap_forward_from_empty_set() {
    let graph = create_test_network();
    let empty = graph.mk_empty_colored_vertices();

    let result = Reachability::trap_forward_naive(&graph, empty).unwrap();

    assert!(
        result.is_empty(),
        "Forward trap of empty set should be empty"
    );
}

#[test]
fn test_trap_forward_of_all_states_is_all_states() {
    let graph = create_test_network();
    let all = graph.mk_unit_colored_vertices();

    let result = Reachability::trap_forward_naive(&graph, all.clone()).unwrap();

    // When the input is ALL states, the forward trap is trivially ALL states
    // (no state can escape to "outside" because outside is empty)
    assert_eq!(
        result, all,
        "Forward trap of all states should be all states (nothing to escape to)"
    );
}

#[test]
fn test_trap_forward_fixed_point_is_trap() {
    let graph = create_test_network();
    let s000 = mk_state(&graph, S000);

    let result = Reachability::trap_forward_naive(&graph, s000.clone()).unwrap();

    // A fixed point is its own forward trap
    assert_eq!(
        result, s000,
        "Fixed point 000 should be its own forward trap"
    );
}

#[test]
fn test_trap_forward_cycle_is_trap() {
    let graph = create_test_network();
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    let result = Reachability::trap_forward_naive(&graph, attractor_2.clone()).unwrap();

    // The cycle {110, 111} is a forward trap (transitions stay within the set)
    assert_eq!(
        result, attractor_2,
        "Cycle {{110, 111}} should be a forward trap"
    );
}

#[test]
fn test_trap_forward_removes_non_trapped_states() {
    let graph = create_test_network();
    // Start with attractor 2 plus its strong basin: {101, 110, 111}
    let initial = mk_states(&graph, &[S101, S110, S111]);

    let result = Reachability::trap_forward_naive(&graph, initial).unwrap();

    let expected = mk_states(&graph, &[S101, S110, S111]);
    assert_eq!(
        result, expected,
        "{{101, 110, 111}} should be a forward trap"
    );
}

#[test]
fn test_trap_forward_strong_basin_plus_attractor() {
    let graph = create_test_network();
    // Strong basin of attractor 1 plus attractor 1: {000, 001, 010}
    let initial = mk_states(&graph, &[S000, S001, S010]);

    let result = Reachability::trap_forward_naive(&graph, initial).unwrap();

    // 001 → 000 (in set), 010 → 000 (in set), 000 has no transitions
    // So {000, 001, 010} is a forward trap
    let expected = mk_states(&graph, &[S000, S001, S010]);
    assert_eq!(
        result, expected,
        "{{000, 001, 010}} should be a forward trap"
    );
}

#[test]
fn test_trap_forward_weak_basin_escapes() {
    let graph = create_test_network();
    // Just the weak basin: {011, 100}
    let initial = mk_states(&graph, &[S011, S100]);

    let result = Reachability::trap_forward_naive(&graph, initial).unwrap();

    // 011 → 010 (not in set) and 011 → 111 (not in set) - both escape!
    // 100 → 000 (not in set) and 100 → 110 (not in set) - both escape!
    // So the forward trap should be empty
    assert!(
        result.is_empty(),
        "Weak basin {{011, 100}} has no forward trap (all states escape)"
    );
}

#[test]
fn test_trap_forward_result_is_subset_of_input() {
    let graph = create_test_network();

    // For any starting set, the forward trap must be a subset
    for state in ALL_STATES {
        let initial = mk_state(&graph, *state);
        let result = Reachability::trap_forward_naive(&graph, initial.clone()).unwrap();
        assert!(
            result.is_subset(&initial),
            "Forward trap of {:03b} must be a subset of input",
            state
        );
    }
}

#[test]
fn test_trap_forward_result_has_no_escaping_transitions() {
    let graph = create_test_network();
    let all = graph.mk_unit_colored_vertices();

    let trap = Reachability::trap_forward_naive(&graph, all).unwrap();

    // Verify no state in the trap can escape
    let can_escape = graph.can_post_out(&trap);
    assert!(
        can_escape.is_empty(),
        "Forward trap should have no escaping transitions"
    );
}

// ========== Tests for trap_backward_naive ==========

#[test]
fn test_trap_backward_from_empty_set() {
    let graph = create_test_network();
    let empty = graph.mk_empty_colored_vertices();

    let result = Reachability::trap_backward_naive(&graph, empty).unwrap();

    assert!(
        result.is_empty(),
        "Backward trap of empty set should be empty"
    );
}

#[test]
fn test_trap_backward_of_all_states_is_all_states() {
    let graph = create_test_network();
    let all = graph.mk_unit_colored_vertices();

    let result = Reachability::trap_backward_naive(&graph, all.clone()).unwrap();

    // When the input is ALL states, the backward trap is trivially ALL states
    // (no state can have predecessors from "outside" because outside is empty)
    assert_eq!(
        result, all,
        "Backward trap of all states should be all states (nothing can enter from outside)"
    );
}

#[test]
fn test_trap_backward_fixed_point_may_not_be_trap() {
    let graph = create_test_network();
    let s000 = mk_state(&graph, S000);

    let result = Reachability::trap_backward_naive(&graph, s000).unwrap();

    // 000 has predecessors (001, 010, 100), so it's NOT a backward trap by itself
    // The backward trap of {000} is empty (since 000 can be entered from outside)
    assert!(
        result.is_empty(),
        "Fixed point 000 is not a backward trap (has predecessors)"
    );
}

#[test]
fn test_trap_backward_source_state_is_trap() {
    let graph = create_test_network();
    // 011 is a SOURCE state (has no predecessors)
    let s011 = mk_state(&graph, S011);

    let result = Reachability::trap_backward_naive(&graph, s011.clone()).unwrap();

    // 011 has no predecessors, so it's a backward trap by itself
    assert_eq!(
        result, s011,
        "Source state 011 should be its own backward trap"
    );
}

#[test]
fn test_trap_backward_cycle_not_a_trap() {
    let graph = create_test_network();
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    let result = Reachability::trap_backward_naive(&graph, attractor_2).unwrap();

    // The cycle {110, 111} has predecessors from outside:
    // - 110: predecessor 100 (outside)
    // - 111: predecessors 011, 101 (outside)
    // So {110, 111} is NOT a backward trap
    assert!(
        result.is_empty(),
        "Cycle {{110, 111}} is not a backward trap (has external predecessors)"
    );
}

#[test]
fn test_trap_backward_result_is_subset_of_input() {
    let graph = create_test_network();

    // For any starting set, the backward trap must be a subset
    for state in ALL_STATES {
        let initial = mk_state(&graph, *state);
        let result = Reachability::trap_backward_naive(&graph, initial.clone()).unwrap();
        assert!(
            result.is_subset(&initial),
            "Backward trap of {:03b} must be a subset of input",
            state
        );
    }
}

#[test]
fn test_trap_backward_result_has_no_incoming_transitions() {
    let graph = create_test_network();
    let all = graph.mk_unit_colored_vertices();

    let trap = Reachability::trap_backward_naive(&graph, all).unwrap();

    // Verify no state outside the trap can reach into it
    let can_enter = graph.can_pre_out(&trap);
    assert!(
        can_enter.is_empty(),
        "Backward trap should have no incoming transitions from outside"
    );
}

// ========== Integration / SCC-related tests ==========

#[test]
fn test_scc_via_forward_backward_intersection() {
    let graph = create_test_network();

    // The SCC containing a state is: forward_reach(s) ∩ backward_reach(s)

    // Test for fixed point 000: SCC should be just {000}
    let s000 = mk_state(&graph, S000);
    let fwd_000 = Reachability::reach_forward_naive(&graph, s000.clone()).unwrap();
    let bwd_000 = Reachability::reach_backward_naive(&graph, s000.clone()).unwrap();
    let scc_000 = fwd_000.intersect(&bwd_000);
    assert_eq!(scc_000, s000, "SCC of 000 should be {{000}}");

    // Test for state in cycle: SCC should be {110, 111}
    let s110 = mk_state(&graph, S110);
    let fwd_110 = Reachability::reach_forward_naive(&graph, s110.clone()).unwrap();
    let bwd_110 = Reachability::reach_backward_naive(&graph, s110).unwrap();
    let scc_110 = fwd_110.intersect(&bwd_110);
    let expected_scc = mk_states(&graph, ATTRACTOR_2);
    assert_eq!(scc_110, expected_scc, "SCC of 110 should be {{110, 111}}");

    // Test for transient state: SCC should be just itself (trivial)
    let s001 = mk_state(&graph, S001);
    let fwd_001 = Reachability::reach_forward_naive(&graph, s001.clone()).unwrap();
    let bwd_001 = Reachability::reach_backward_naive(&graph, s001.clone()).unwrap();
    let scc_001 = fwd_001.intersect(&bwd_001);
    assert_eq!(scc_001, s001, "SCC of 001 should be {{001}} (trivial)");
}

#[test]
fn test_basin_separation() {
    let graph = create_test_network();
    let s000 = mk_state(&graph, S000);
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    // States in strong basin of attractor 1 cannot reach attractor 2
    for state in STRONG_BASIN_ATTR1 {
        let s = mk_state(&graph, *state);
        let reachable = Reachability::reach_forward_naive(&graph, s).unwrap();
        assert!(
            reachable.intersect(&attractor_2).is_empty(),
            "State {:03b} in strong basin of attr 1 should not reach attr 2",
            state
        );
        assert!(
            !reachable.intersect(&s000).is_empty(),
            "State {:03b} in strong basin of attr 1 should reach attr 1",
            state
        );
    }

    // States in strong basin of attractor 2 cannot reach attractor 1
    for state in STRONG_BASIN_ATTR2 {
        let s = mk_state(&graph, *state);
        let reachable = Reachability::reach_forward_naive(&graph, s).unwrap();
        assert!(
            reachable.intersect(&s000).is_empty(),
            "State {:03b} in strong basin of attr 2 should not reach attr 1",
            state
        );
        assert!(
            !reachable.intersect(&attractor_2).is_empty(),
            "State {:03b} in strong basin of attr 2 should reach attr 2",
            state
        );
    }

    // States in weak basin can reach both attractors
    for state in WEAK_BASIN {
        let s = mk_state(&graph, *state);
        let reachable = Reachability::reach_forward_naive(&graph, s).unwrap();
        assert!(
            !reachable.intersect(&s000).is_empty(),
            "State {:03b} in weak basin should reach attr 1",
            state
        );
        assert!(
            !reachable.intersect(&attractor_2).is_empty(),
            "State {:03b} in weak basin should reach attr 2",
            state
        );
    }
}

#[test]
fn test_source_states_are_backward_traps() {
    let graph = create_test_network();
    let sources = mk_states(&graph, SOURCE_STATES);

    // Source states have no predecessors, so they form a backward trap
    let backward_trap = Reachability::trap_backward_naive(&graph, sources.clone()).unwrap();
    assert_eq!(
        backward_trap, sources,
        "Source states should form a backward trap"
    );
}

#[test]
fn test_forward_reach_from_sources_covers_all() {
    let graph = create_test_network();
    let sources = mk_states(&graph, SOURCE_STATES);
    let all = graph.mk_unit_colored_vertices();

    // Forward reachability from all source states should cover the entire state space
    let reachable = Reachability::reach_forward_naive(&graph, sources).unwrap();
    assert_eq!(
        reachable, all,
        "Forward reach from all sources should cover all states"
    );
}
