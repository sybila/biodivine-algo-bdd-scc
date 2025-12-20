//! Tests for reachability algorithms using the canonical test network.
//!
//! See `test_network.rs` for the complete documentation of the test network structure.

use super::llm_example_network::sets::{
    ALL_STATES, ATTRACTOR_2, CAN_REACH_ATTR1, CAN_REACH_ATTR2, SOURCE_STATES, STRONG_BASIN_ATTR1,
    STRONG_BASIN_ATTR2, WEAK_BASIN,
};
use super::llm_example_network::states::*;
use super::llm_example_network::{create_test_network, mk_state, mk_states};
use crate::Algorithm;
use crate::algorithm::reachability::{
    BackwardReachabilityBFS, ForwardReachabilityBFS, ReachabilityState,
};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use cancel_this::Cancellable;

// ========== Tests for reach_forward_naive ==========

#[test]
fn test_reach_forward_from_empty_set() {
    let graph = create_test_network();
    let empty = graph.mk_empty_colored_vertices();

    let state = ReachabilityState::initial(graph, empty);
    let result = ForwardReachabilityBFS::compute(state).unwrap();

    assert!(
        result.is_empty(),
        "Forward reach from empty set should be empty"
    );
}

#[test]
fn test_reach_forward_from_fixed_point() {
    let graph = create_test_network();
    let s000 = mk_state(&graph, S000);

    let state = ReachabilityState::initial(graph, s000.clone());
    let result = ForwardReachabilityBFS::compute(state).unwrap();

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

    let state = ReachabilityState::initial(graph, s110.clone());
    let result = ForwardReachabilityBFS::compute(state).unwrap();

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

    let state = ReachabilityState::initial(graph, s001.clone());
    let result = ForwardReachabilityBFS::compute(state).unwrap();

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

    let state = ReachabilityState::initial(graph, s101.clone());
    let result = ForwardReachabilityBFS::compute(state).unwrap();

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

    let state = ReachabilityState::initial(graph, s011.clone());
    let result = ForwardReachabilityBFS::compute(state).unwrap();

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

    let state = ReachabilityState::initial(graph.clone(), s100.clone());
    let result = ForwardReachabilityBFS::compute(state).unwrap();

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
    assert!(s000.is_subset(&result));
    assert!(attractor_2.is_subset(&result));
}

#[test]
fn test_reach_forward_includes_initial() {
    let graph = create_test_network();

    // For any starting set, forward reachability must include the initial set
    for state in ALL_STATES {
        let initial = mk_state(&graph, *state);
        let algo_state = ReachabilityState::initial(graph.clone(), initial.clone());
        let result = ForwardReachabilityBFS::compute(algo_state).unwrap();
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

    let state = ReachabilityState::initial(graph, empty);
    let result = BackwardReachabilityBFS::compute(state).unwrap();

    assert!(
        result.is_empty(),
        "Backward reach from empty set should be empty"
    );
}

#[test]
fn test_reach_backward_to_fixed_point() {
    let graph = create_test_network();
    let s000 = mk_state(&graph, S000);

    let state = ReachabilityState::initial(graph.clone(), s000);
    let result = BackwardReachabilityBFS::compute(state).unwrap();

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

    let state = ReachabilityState::initial(graph.clone(), attractor_2.clone());
    let result = BackwardReachabilityBFS::compute(state).unwrap();

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

    let state = ReachabilityState::initial(graph.clone(), s110);
    let result = BackwardReachabilityBFS::compute(state).unwrap();

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
fn test_reach_backward_includes_initial() {
    let graph = create_test_network();

    // For any starting set, backward reachability must include the initial set
    for state in ALL_STATES {
        let initial = mk_state(&graph, *state);
        let algo_state = ReachabilityState::initial(graph.clone(), initial.clone());
        let result = BackwardReachabilityBFS::compute(algo_state).unwrap();
        assert!(
            initial.is_subset(&result),
            "Backward reach to {:03b} must include the initial state",
            state
        );
    }
}

// ========== Integration / SCC-related tests ==========

#[test]
fn test_scc_via_forward_backward_intersection() -> Cancellable<()> {
    let graph = create_test_network();

    // The SCC containing a state is: forward_reach(s) ∩ backward_reach(s)

    // Test for fixed point 000: SCC should be just {000}
    let s000 = mk_state(&graph, S000);
    let state = ReachabilityState::initial(graph.clone(), s000.clone());
    let fwd_000 = ForwardReachabilityBFS::compute(state.clone())?;
    let bwd_000 = BackwardReachabilityBFS::compute(state)?;
    let scc_000 = fwd_000.intersect(&bwd_000);
    assert_eq!(scc_000, s000, "SCC of 000 should be {{000}}");

    // Test for state in cycle: SCC should be {110, 111}
    let s110 = mk_state(&graph, S110);
    let state = ReachabilityState::initial(graph.clone(), s110.clone());
    let fwd_110 = ForwardReachabilityBFS::compute(state.clone())?;
    let bwd_110 = BackwardReachabilityBFS::compute(state)?;
    let scc_110 = fwd_110.intersect(&bwd_110);
    let expected_scc = mk_states(&graph, ATTRACTOR_2);
    assert_eq!(scc_110, expected_scc, "SCC of 110 should be {{110, 111}}");

    // Test for transient state: SCC should be just itself (trivial)
    let s001 = mk_state(&graph, S001);
    let state = ReachabilityState::initial(graph.clone(), s001.clone());
    let fwd_001 = ForwardReachabilityBFS::compute(state.clone())?;
    let bwd_001 = BackwardReachabilityBFS::compute(state)?;
    let scc_001 = fwd_001.intersect(&bwd_001);
    assert_eq!(scc_001, s001, "SCC of 001 should be {{001}} (trivial)");

    Ok(())
}

#[test]
fn test_basin_separation() -> Cancellable<()> {
    let graph = create_test_network();
    let s000 = mk_state(&graph, S000);
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    // States in strong basin of attractor 1 cannot reach attractor 2
    for state in STRONG_BASIN_ATTR1 {
        let s = mk_state(&graph, *state);
        let reachable =
            ForwardReachabilityBFS::compute(ReachabilityState::initial(graph.clone(), s))?;
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
        let reachable =
            ForwardReachabilityBFS::compute(ReachabilityState::initial(graph.clone(), s))?;
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
        let reachable =
            ForwardReachabilityBFS::compute(ReachabilityState::initial(graph.clone(), s))?;
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

    Ok(())
}

#[test]
fn test_forward_reach_from_sources_covers_everything() -> Cancellable<()> {
    let graph = create_test_network();
    let sources = mk_states(&graph, SOURCE_STATES);
    let all = graph.mk_unit_colored_vertices();

    // Forward reachability from all source states should cover the entire state space
    let reachable = ForwardReachabilityBFS::compute(ReachabilityState::initial(graph, sources))?;

    assert_eq!(
        reachable, all,
        "Forward reach from all sources should cover all states"
    );

    Ok(())
}
