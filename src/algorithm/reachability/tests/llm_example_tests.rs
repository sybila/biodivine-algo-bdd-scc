//! Tests for reachability algorithms using the example test network.
//!
//! See `llm_example_network.rs` for the complete documentation of the test network structure.

use crate::algorithm::reachability::{
    BfsPredecessors, BfsSuccessors, IterativeUnion, ReachabilityComputation, ReachabilityConfig,
    SaturationPredecessors, SaturationSuccessors,
};
use crate::algorithm::test_utils::llm_example_network::create_test_network;
use crate::algorithm::test_utils::llm_example_network::sets::{
    ALL_STATES, ATTRACTOR_1, ATTRACTOR_2, CAN_REACH_ATTR1, CAN_REACH_ATTR2, SOURCE_STATES,
    STRONG_BASIN_ATTR1, STRONG_BASIN_ATTR2, WEAK_BASIN,
};
use crate::algorithm::test_utils::llm_example_network::states::*;
use crate::algorithm::test_utils::{init_logger, mk_state, mk_states};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use cancel_this::Cancellable;
use computation_process::{Algorithm, ComputationStep};

// ========== Parametrized test helpers ==========

/// Generic helper function for forward reachability tests.
/// This allows us to test both BFS and saturation variants with the same logic.
fn test_reach_forward_from_empty_set_impl<STEP>() -> Cancellable<()>
where
    STEP: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static,
{
    init_logger();
    let graph = create_test_network();
    let empty = graph.mk_empty_colored_vertices();

    let result = ReachabilityComputation::<STEP>::run(&graph, empty)?;

    assert!(
        result.is_empty(),
        "Forward reach from empty set should be empty"
    );
    Ok(())
}

fn test_reach_forward_from_fixed_point_impl<STEP>() -> Cancellable<()>
where
    STEP: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static,
{
    init_logger();
    let graph = create_test_network();
    let s000 = mk_state(&graph, S000);

    let result = ReachabilityComputation::<STEP>::run(&graph, s000.clone())?;

    // A fixed point can only reach itself
    assert_eq!(
        result, s000,
        "Forward reach from fixed point 000 should be just {{000}}"
    );
    Ok(())
}

fn test_reach_forward_from_attractor_2_impl<STEP>() -> Cancellable<()>
where
    STEP: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static,
{
    init_logger();
    let graph = create_test_network();
    let s110 = mk_state(&graph, S110);
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    let result = ReachabilityComputation::<STEP>::run(&graph, s110.clone())?;

    // From 110, we can reach 111 (and back), so we reach the whole attractor
    assert_eq!(
        result, attractor_2,
        "Forward reach from 110 should be {{110, 111}}"
    );
    Ok(())
}

fn test_reach_forward_from_strong_basin_of_attractor_1_impl<STEP>() -> Cancellable<()>
where
    STEP: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static,
{
    init_logger();
    let graph = create_test_network();
    let s001 = mk_state(&graph, S001);
    let attractor_1 = mk_states(&graph, ATTRACTOR_1);

    let result = ReachabilityComputation::<STEP>::run(&graph, s001.clone())?;

    // 001 → 000 (fixed point), so we reach {001, 000}
    let expected = s001.union(&attractor_1);
    assert_eq!(
        result, expected,
        "Forward reach from 001 should be {{000, 001}}"
    );
    Ok(())
}

fn test_reach_forward_from_strong_basin_of_attractor_2_impl<STEP>() -> Cancellable<()>
where
    STEP: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static,
{
    init_logger();
    let graph = create_test_network();
    let s101 = mk_state(&graph, S101);
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    let result = ReachabilityComputation::<STEP>::run(&graph, s101.clone())?;

    // 101 → 111 → 110 → 111 ..., so we reach {101, 110, 111}
    let expected = s101.union(&attractor_2);
    assert_eq!(
        result, expected,
        "Forward reach from 101 should be {{101, 110, 111}}"
    );
    Ok(())
}

fn test_reach_forward_from_weak_basin_reaches_both_attractors_impl<STEP>() -> Cancellable<()>
where
    STEP: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static,
{
    init_logger();
    let graph = create_test_network();
    let s011 = mk_state(&graph, S011);
    let attractor_1 = mk_states(&graph, ATTRACTOR_1);
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    let result = ReachabilityComputation::<STEP>::run(&graph, s011.clone())?;

    // 011 can reach both attractors:
    // - 011 → 010 → 000 (attractor 1)
    // - 011 → 111 → 110 → 111 ... (attractor 2)
    assert!(
        !result.intersect(&attractor_1).is_empty(),
        "Forward reach from 011 should include attractor 1 (000)"
    );
    assert!(
        !result.intersect(&attractor_2).is_empty(),
        "Forward reach from 011 should include attractor 2 (110, 111)"
    );
    Ok(())
}

fn test_reach_forward_from_weak_basin_100_impl<STEP>() -> Cancellable<()>
where
    STEP: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static,
{
    init_logger();
    let graph = create_test_network();
    let s100 = mk_state(&graph, S100);
    let attractor_1 = mk_states(&graph, ATTRACTOR_1);
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    let result = ReachabilityComputation::<STEP>::run(&graph, s100.clone())?;

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
    assert!(attractor_1.is_subset(&result));
    assert!(attractor_2.is_subset(&result));
    Ok(())
}

fn test_reach_forward_includes_initial_impl<STEP>() -> Cancellable<()>
where
    STEP: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static,
{
    init_logger();
    let graph = create_test_network();

    // For any starting set, forward reachability must include the initial set
    for state in ALL_STATES {
        let initial = mk_state(&graph, *state);
        let result = ReachabilityComputation::<STEP>::run(&graph, initial.clone())?;
        assert!(
            initial.is_subset(&result),
            "Forward reach from {:03b} must include the initial state",
            state
        );
    }
    Ok(())
}

/// Generic helper function for backward reachability tests.
fn test_reach_backward_from_empty_set_impl<STEP>() -> Cancellable<()>
where
    STEP: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static,
{
    init_logger();
    let graph = create_test_network();
    let empty = graph.mk_empty_colored_vertices();

    let result = ReachabilityComputation::<STEP>::run(&graph, empty)?;

    assert!(
        result.is_empty(),
        "Backward reach from empty set should be empty"
    );
    Ok(())
}

fn test_reach_backward_to_fixed_point_impl<STEP>() -> Cancellable<()>
where
    STEP: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static,
{
    init_logger();
    let graph = create_test_network();
    let s000 = mk_state(&graph, S000);

    let result = ReachabilityComputation::<STEP>::run(&graph, s000)?;

    // States that can reach 000: {000, 001, 010, 011, 100}
    let expected = mk_states(&graph, CAN_REACH_ATTR1);
    assert_eq!(
        result, expected,
        "Backward reach to 000 should be {{000, 001, 010, 011, 100}}"
    );
    Ok(())
}

fn test_reach_backward_to_attractor_2_impl<STEP>() -> Cancellable<()>
where
    STEP: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static,
{
    init_logger();
    let graph = create_test_network();
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    let result = ReachabilityComputation::<STEP>::run(&graph, attractor_2.clone())?;

    // States that can reach {110, 111}: {011, 100, 101, 110, 111}
    let expected = mk_states(&graph, CAN_REACH_ATTR2);
    assert_eq!(
        result, expected,
        "Backward reach to attractor 2 should be {{011, 100, 101, 110, 111}}"
    );
    Ok(())
}

fn test_reach_backward_from_single_state_in_cycle_impl<STEP>() -> Cancellable<()>
where
    STEP: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static,
{
    init_logger();
    let graph = create_test_network();
    let s110 = mk_state(&graph, S110);

    let result = ReachabilityComputation::<STEP>::run(&graph, s110)?;

    // Reaching just 110: 111 can reach 110 directly, then all states that can reach 111
    // Chain: 101 → 111 → 110, 100 → 110, 011 → 111 → 110
    // Expected: {110, 111, 101, 100, 011}
    let expected = mk_states(&graph, CAN_REACH_ATTR2);
    assert_eq!(
        result, expected,
        "Backward reach to 110 should be {{011, 100, 101, 110, 111}}"
    );
    Ok(())
}

fn test_reach_backward_includes_initial_impl<STEP>() -> Cancellable<()>
where
    STEP: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static,
{
    init_logger();
    let graph = create_test_network();

    // For any starting set, backward reachability must include the initial set
    for state in ALL_STATES {
        let initial = mk_state(&graph, *state);
        let result = ReachabilityComputation::<STEP>::run(&graph, initial.clone())?;
        assert!(
            initial.is_subset(&result),
            "Backward reach to {:03b} must include the initial state",
            state
        );
    }
    Ok(())
}

/// Generic helper for SCC tests that need both forward and backward algorithms.
fn test_scc_via_forward_backward_intersection_impl<F, B>() -> Cancellable<()>
where
    F: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static,
    B: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static,
{
    init_logger();
    let graph = create_test_network();

    // The SCC containing a state is: forward_reach(s) ∩ backward_reach(s)

    // Test for fixed point 000: SCC should be just {000}
    let s000 = mk_state(&graph, S000);
    let fwd_000 = ReachabilityComputation::<F>::run(&graph, s000.clone())?;
    let bwd_000 = ReachabilityComputation::<B>::run(&graph, s000.clone())?;
    let scc_000 = fwd_000.intersect(&bwd_000);
    assert_eq!(scc_000, s000, "SCC of 000 should be {{000}}");

    // Test for state in cycle: SCC should be {110, 111}
    let s110 = mk_state(&graph, S110);
    let fwd_110 = ReachabilityComputation::<F>::run(&graph, s110.clone())?;
    let bwd_110 = ReachabilityComputation::<B>::run(&graph, s110.clone())?;
    let scc_110 = fwd_110.intersect(&bwd_110);
    let expected_scc = mk_states(&graph, ATTRACTOR_2);
    assert_eq!(scc_110, expected_scc, "SCC of 110 should be {{110, 111}}");

    // Test for transient state: SCC should be just itself (trivial)
    let s001 = mk_state(&graph, S001);
    let fwd_001 = ReachabilityComputation::<F>::run(&graph, s001.clone())?;
    let bwd_001 = ReachabilityComputation::<B>::run(&graph, s001.clone())?;
    let scc_001 = fwd_001.intersect(&bwd_001);
    assert_eq!(scc_001, s001, "SCC of 001 should be {{001}} (trivial)");

    Ok(())
}

fn test_basin_separation_impl<STEP>() -> Cancellable<()>
where
    STEP: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static,
{
    init_logger();
    let graph = create_test_network();
    let attractor_1 = mk_states(&graph, ATTRACTOR_1);
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    // States in a strong basin of attractor 1 cannot reach attractor 2
    for state in STRONG_BASIN_ATTR1 {
        let s = mk_state(&graph, *state);
        let reachable = ReachabilityComputation::<STEP>::run(&graph, s)?;
        assert!(
            reachable.intersect(&attractor_2).is_empty(),
            "State {:03b} in strong basin of attr 1 should not reach attr 2",
            state
        );
        assert!(
            !reachable.intersect(&attractor_1).is_empty(),
            "State {:03b} in strong basin of attr 1 should reach attr 1",
            state
        );
    }

    // States in a strong basin of attractor 2 cannot reach attractor 1
    for state in STRONG_BASIN_ATTR2 {
        let s = mk_state(&graph, *state);
        let reachable = ReachabilityComputation::<STEP>::run(&graph, s)?;
        assert!(
            reachable.intersect(&attractor_1).is_empty(),
            "State {:03b} in strong basin of attr 2 should not reach attr 1",
            state
        );
        assert!(
            !reachable.intersect(&attractor_2).is_empty(),
            "State {:03b} in strong basin of attr 2 should reach attr 2",
            state
        );
    }

    // States in a weak basin can reach both attractors
    for state in WEAK_BASIN {
        let s = mk_state(&graph, *state);
        let reachable = ReachabilityComputation::<STEP>::run(&graph, s)?;
        assert!(
            !reachable.intersect(&attractor_1).is_empty(),
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

fn test_forward_reach_from_sources_covers_everything_impl<STEP>() -> Cancellable<()>
where
    STEP: ComputationStep<ReachabilityConfig, GraphColoredVertices, GraphColoredVertices> + 'static,
{
    init_logger();
    let graph = create_test_network();
    let sources = mk_states(&graph, SOURCE_STATES);
    let all = graph.mk_unit_colored_vertices();

    // Forward reachability from all source states should cover the entire state space
    let reachable = ReachabilityComputation::<STEP>::run(&graph, sources)?;

    assert_eq!(
        reachable, all,
        "Forward reach from all sources should cover all states"
    );

    Ok(())
}

// ========== Tests for BFS forward algorithms ==========
type ForwardReachabilityBfs = IterativeUnion<BfsSuccessors>;

#[test]
fn test_reach_forward_from_empty_set_bfs() -> Cancellable<()> {
    test_reach_forward_from_empty_set_impl::<ForwardReachabilityBfs>()
}

#[test]
fn test_reach_forward_from_fixed_point_bfs() -> Cancellable<()> {
    test_reach_forward_from_fixed_point_impl::<ForwardReachabilityBfs>()
}

#[test]
fn test_reach_forward_from_attractor_2_bfs() -> Cancellable<()> {
    test_reach_forward_from_attractor_2_impl::<ForwardReachabilityBfs>()
}

#[test]
fn test_reach_forward_from_strong_basin_of_attractor_1_bfs() -> Cancellable<()> {
    test_reach_forward_from_strong_basin_of_attractor_1_impl::<ForwardReachabilityBfs>()
}

#[test]
fn test_reach_forward_from_strong_basin_of_attractor_2_bfs() -> Cancellable<()> {
    test_reach_forward_from_strong_basin_of_attractor_2_impl::<ForwardReachabilityBfs>()
}

#[test]
fn test_reach_forward_from_weak_basin_reaches_both_attractors_bfs() -> Cancellable<()> {
    test_reach_forward_from_weak_basin_reaches_both_attractors_impl::<ForwardReachabilityBfs>()
}

#[test]
fn test_reach_forward_from_weak_basin_100_bfs() -> Cancellable<()> {
    test_reach_forward_from_weak_basin_100_impl::<ForwardReachabilityBfs>()
}

#[test]
fn test_reach_forward_includes_initial_bfs() -> Cancellable<()> {
    test_reach_forward_includes_initial_impl::<ForwardReachabilityBfs>()
}

#[test]
fn test_basin_separation_bfs() -> Cancellable<()> {
    test_basin_separation_impl::<ForwardReachabilityBfs>()
}

#[test]
fn test_forward_reach_from_sources_covers_everything_bfs() -> Cancellable<()> {
    test_forward_reach_from_sources_covers_everything_impl::<ForwardReachabilityBfs>()
}

// ========== Tests for saturation forward algorithms ==========
type ForwardReachability = IterativeUnion<SaturationSuccessors>;

#[test]
fn test_reach_forward_from_empty_set_sat() -> Cancellable<()> {
    test_reach_forward_from_empty_set_impl::<ForwardReachability>()
}

#[test]
fn test_reach_forward_from_fixed_point_sat() -> Cancellable<()> {
    test_reach_forward_from_fixed_point_impl::<ForwardReachability>()
}

#[test]
fn test_reach_forward_from_attractor_2_sat() -> Cancellable<()> {
    test_reach_forward_from_attractor_2_impl::<ForwardReachability>()
}

#[test]
fn test_reach_forward_from_strong_basin_of_attractor_1_sat() -> Cancellable<()> {
    test_reach_forward_from_strong_basin_of_attractor_1_impl::<ForwardReachability>()
}

#[test]
fn test_reach_forward_from_strong_basin_of_attractor_2_sat() -> Cancellable<()> {
    test_reach_forward_from_strong_basin_of_attractor_2_impl::<ForwardReachability>()
}

#[test]
fn test_reach_forward_from_weak_basin_reaches_both_attractors_sat() -> Cancellable<()> {
    test_reach_forward_from_weak_basin_reaches_both_attractors_impl::<ForwardReachability>()
}

#[test]
fn test_reach_forward_from_weak_basin_100_sat() -> Cancellable<()> {
    test_reach_forward_from_weak_basin_100_impl::<ForwardReachability>()
}

#[test]
fn test_reach_forward_includes_initial_sat() -> Cancellable<()> {
    test_reach_forward_includes_initial_impl::<ForwardReachability>()
}

#[test]
fn test_basin_separation_sat() -> Cancellable<()> {
    test_basin_separation_impl::<ForwardReachability>()
}

#[test]
fn test_forward_reach_from_sources_covers_everything_sat() -> Cancellable<()> {
    test_forward_reach_from_sources_covers_everything_impl::<ForwardReachability>()
}

// ========== Tests for BFS backward algorithms ==========
type BackwardReachabilityBfs = IterativeUnion<BfsPredecessors>;

#[test]
fn test_reach_backward_from_empty_set_bfs() -> Cancellable<()> {
    test_reach_backward_from_empty_set_impl::<BackwardReachabilityBfs>()
}

#[test]
fn test_reach_backward_to_fixed_point_bfs() -> Cancellable<()> {
    test_reach_backward_to_fixed_point_impl::<BackwardReachabilityBfs>()
}

#[test]
fn test_reach_backward_to_attractor_2_bfs() -> Cancellable<()> {
    test_reach_backward_to_attractor_2_impl::<BackwardReachabilityBfs>()
}

#[test]
fn test_reach_backward_from_single_state_in_cycle_bfs() -> Cancellable<()> {
    test_reach_backward_from_single_state_in_cycle_impl::<BackwardReachabilityBfs>()
}

#[test]
fn test_reach_backward_includes_initial_bfs() -> Cancellable<()> {
    test_reach_backward_includes_initial_impl::<BackwardReachabilityBfs>()
}

// ========== Tests for saturation backward algorithms ==========
type BackwardReachability = IterativeUnion<SaturationPredecessors>;

#[test]
fn test_reach_backward_from_empty_set_sat() -> Cancellable<()> {
    test_reach_backward_from_empty_set_impl::<BackwardReachability>()
}

#[test]
fn test_reach_backward_to_fixed_point_sat() -> Cancellable<()> {
    test_reach_backward_to_fixed_point_impl::<BackwardReachability>()
}

#[test]
fn test_reach_backward_to_attractor_2_sat() -> Cancellable<()> {
    test_reach_backward_to_attractor_2_impl::<BackwardReachability>()
}

#[test]
fn test_reach_backward_from_single_state_in_cycle_sat() -> Cancellable<()> {
    test_reach_backward_from_single_state_in_cycle_impl::<BackwardReachability>()
}

#[test]
fn test_reach_backward_includes_initial_sat() -> Cancellable<()> {
    test_reach_backward_includes_initial_impl::<BackwardReachability>()
}

// ========== Tests for SCC (forward + backward combinations) ==========

#[test]
fn test_scc_via_forward_backward_intersection_bfs_bfs() -> Cancellable<()> {
    test_scc_via_forward_backward_intersection_impl::<ForwardReachabilityBfs, BackwardReachabilityBfs>(
    )
}

#[test]
fn test_scc_via_forward_backward_intersection_sat_sat() -> Cancellable<()> {
    test_scc_via_forward_backward_intersection_impl::<ForwardReachability, BackwardReachability>()
}

#[test]
fn test_scc_via_forward_backward_intersection_bfs_sat() -> Cancellable<()> {
    test_scc_via_forward_backward_intersection_impl::<ForwardReachabilityBfs, BackwardReachability>(
    )
}

#[test]
fn test_scc_via_forward_backward_intersection_sat_bfs() -> Cancellable<()> {
    test_scc_via_forward_backward_intersection_impl::<ForwardReachability, BackwardReachabilityBfs>(
    )
}
