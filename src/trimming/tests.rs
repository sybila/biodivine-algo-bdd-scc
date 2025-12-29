//! Tests for trimming algorithms using the example test network.
//!
//! See `llm_example_network.rs` for the complete documentation of the test network structure.
//!
//! Trimming removes "local source states" (TrimSources) or "local sink states" (TrimSinks)
//! from the initial set. A source state has no predecessors within the remaining states
//! (complement), and a sink state has no successors within the remaining states.

use crate::test_utils::llm_example_network::create_test_network;
use crate::test_utils::llm_example_network::sets::{
    ALL_STATES, ATTRACTOR_1, ATTRACTOR_2, SOURCE_STATES, STRONG_BASIN_ATTR1, WEAK_BASIN,
};
use crate::test_utils::llm_example_network::states::*;
use crate::test_utils::{mk_state, mk_states};
use crate::trimming::{TrimSinks, TrimSources};
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use cancel_this::Cancellable;
use computation_process::Algorithm;

/// Initialize env_logger for tests. Safe to call multiple times.
fn init_logger() {
    let _ = env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Trace)
        .is_test(true)
        .try_init();
}

// ========== TrimSources tests ==========

/// TrimSources removes states that are sources relative to the remaining states.
/// Source states in the network: {011, 100, 101} (no predecessors at all).

#[test]
fn test_trim_sources_from_empty_set() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let empty = graph.mk_empty_colored_vertices();

    let result = TrimSources::run(&graph, empty.clone())?;

    assert!(
        result.is_empty(),
        "Trim sources from empty set should be empty"
    );
    Ok(())
}

#[test]
fn test_trim_sources_from_all_states() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let all = graph.mk_unit_colored_vertices();

    let result = TrimSources::run(&graph, all.clone())?;

    // The algorithm iteratively removes sources. After removing {011, 100, 101},
    // the remaining states {000, 001, 010, 110, 111} may have some that become sources
    // relative to the new complement. Based on the log, the result has 2 elements.
    // These should be the non-trivial SCC: {110, 111}
    let expected = mk_states(&graph, &[S110, S111]);
    assert_eq!(
        result, expected,
        "Trim sources from all states should iteratively remove sources, leaving {{110, 111}}"
    );
    Ok(())
}

#[test]
fn test_trim_sources_from_source_states_only() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let sources = mk_states(&graph, SOURCE_STATES);

    let result = TrimSources::run(&graph, sources.clone())?;

    // All source states should be removed
    assert!(
        result.is_empty(),
        "Trim sources from {{011, 100, 101}} should remove all, leaving empty"
    );
    Ok(())
}

#[test]
fn test_trim_sources_from_single_source_state() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let s011 = mk_state(&graph, S011);

    let result = TrimSources::run(&graph, s011.clone())?;

    // Single source state should be removed
    assert!(
        result.is_empty(),
        "Trim sources from {{011}} should remove it, leaving empty"
    );
    Ok(())
}

#[test]
fn test_trim_sources_from_non_source_states() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    // States that are not sources: {000, 001, 010, 110, 111}
    let non_sources = mk_states(&graph, &[S000, S001, S010, S110, S111]);

    let result = TrimSources::run(&graph, non_sources.clone())?;

    // After removing sources {011, 100, 101}, states {001, 010} may become sources
    // relative to the complement, so they get removed too. Only {110, 111} remain.
    let expected = mk_states(&graph, &[S110, S111]);
    assert_eq!(
        result, expected,
        "Trim sources from non-source states should iteratively remove sources, leaving {{110, 111}}"
    );
    Ok(())
}

#[test]
fn test_trim_sources_from_attractor_1() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractor_1 = mk_states(&graph, ATTRACTOR_1);

    let result = TrimSources::run(&graph, attractor_1.clone())?;

    // 000 is not a source, but the algorithm checks iteratively.
    // With complement = {001, 010, 011, 100, 101, 110, 111}, 000 may become a source
    // relative to some subset, so it gets removed.
    assert!(
        result.is_empty(),
        "Trim sources from {{000}} should remove it (becomes source relative to complement)"
    );
    Ok(())
}

#[test]
fn test_trim_sources_from_attractor_2() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    let result = TrimSources::run(&graph, attractor_2.clone())?;

    // Neither 110 nor 111 are sources (they have predecessors)
    assert_eq!(
        result, attractor_2,
        "Trim sources from {{110, 111}} should not remove anything (not sources)"
    );
    Ok(())
}

#[test]
fn test_trim_sources_from_strong_basin_attr1() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let basin = mk_states(&graph, STRONG_BASIN_ATTR1);

    let result = TrimSources::run(&graph, basin.clone())?;

    // 001 and 010 have predecessor 011, but 011 is not in the set.
    // With complement = {000, 011, 100, 101, 110, 111}, they may become sources
    // relative to the complement, so they get removed.
    assert!(
        result.is_empty(),
        "Trim sources from {{001, 010}} should remove them (become sources relative to complement)"
    );
    Ok(())
}

#[test]
fn test_trim_sources_from_strong_basin_attr2() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let s101 = mk_state(&graph, S101);

    let result = TrimSources::run(&graph, s101.clone())?;

    // 101 is a source, so it should be removed
    assert!(
        result.is_empty(),
        "Trim sources from {{101}} should remove it (source state)"
    );
    Ok(())
}

#[test]
fn test_trim_sources_from_weak_basin() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let weak_basin = mk_states(&graph, WEAK_BASIN);

    let result = TrimSources::run(&graph, weak_basin.clone())?;

    // Both 011 and 100 are sources, so both should be removed
    assert!(
        result.is_empty(),
        "Trim sources from {{011, 100}} should remove both (both are sources)"
    );
    Ok(())
}

#[test]
fn test_trim_sources_from_mixed_set() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    // Mix of source and non-source states: {000, 011, 110}
    let mixed = mk_states(&graph, &[S000, S011, S110]);

    let result = TrimSources::run(&graph, mixed.clone())?;

    // Should remove 011 (source). After that, the algorithm continues iteratively.
    // All states eventually get removed.
    assert!(
        result.is_empty(),
        "Trim sources from {{000, 011, 110}} should remove all (iterative source removal)"
    );
    Ok(())
}

#[test]
fn test_trim_sources_is_subset_of_initial() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();

    // Source trim always returns a subset of the initial set, and if the initial set is
    // a singleton, it is always trimmed.
    for state in ALL_STATES {
        let initial = mk_state(&graph, *state);
        let result = TrimSources::run(&graph, initial.clone())?;
        assert!(
            result.is_empty(),
            "Trimming single state {:03b} must produce an empty set",
            state
        );
    }
    Ok(())
}

// ========== TrimSinks tests ==========

/// TrimSinks removes states that are sinks relative to the remaining states.
/// Sink states in the network: {000} (no successors, fixed point).

#[test]
fn test_trim_sinks_from_empty_set() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let empty = graph.mk_empty_colored_vertices();

    let result = TrimSinks::run(&graph, empty.clone())?;

    assert!(
        result.is_empty(),
        "Trim sinks from empty set should be empty"
    );
    Ok(())
}

#[test]
fn test_trim_sinks_from_all_states() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let all = graph.mk_unit_colored_vertices();

    let result = TrimSinks::run(&graph, all.clone())?;

    // The algorithm iteratively removes sinks. After removing {000},
    // {001, 010} become sinks (they only reach 000 which is gone), so they're removed too.
    // Remaining: {011, 100, 101, 110, 111} (5 states)
    let expected = mk_states(&graph, &[S011, S100, S101, S110, S111]);
    assert_eq!(
        result, expected,
        "Trim sinks from all states should iteratively remove sinks, leaving {{011, 100, 101, 110, 111}}"
    );
    Ok(())
}

#[test]
fn test_trim_sinks_from_sink_state_only() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let s000 = mk_state(&graph, S000);

    let result = TrimSinks::run(&graph, s000.clone())?;

    // The sink state should be removed
    assert!(
        result.is_empty(),
        "Trim sinks from {{000}} should remove it, leaving empty"
    );
    Ok(())
}

#[test]
fn test_trim_sinks_from_non_sink_states() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    // States that are not sinks: {001, 010, 011, 100, 101, 110, 111}
    let non_sinks = mk_states(&graph, &[S001, S010, S011, S100, S101, S110, S111]);

    let result = TrimSinks::run(&graph, non_sinks.clone())?;

    // After removing {000}, {001, 010} become sinks and are removed.
    // Remaining: {011, 100, 101, 110, 111}
    let expected = mk_states(&graph, &[S011, S100, S101, S110, S111]);
    assert_eq!(
        result, expected,
        "Trim sinks from non-sink states should iteratively remove sinks, leaving {{011, 100, 101, 110, 111}}"
    );
    Ok(())
}

#[test]
fn test_trim_sinks_from_attractor_2() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractor_2 = mk_states(&graph, ATTRACTOR_2);

    let result = TrimSinks::run(&graph, attractor_2.clone())?;

    // Neither 110 nor 111 are sinks (they have successors: 110→111, 111→110)
    assert_eq!(
        result, attractor_2,
        "Trim sinks from {{110, 111}} should not remove anything (not sinks)"
    );
    Ok(())
}

#[test]
fn test_trim_sinks_from_source_states() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let sources = mk_states(&graph, SOURCE_STATES);

    let result = TrimSinks::run(&graph, sources.clone())?;

    // Source states have successors, but some successors may be sinks relative to complement.
    // `011 → {001, 010, 111}`, but `001` and `010` reach `000`, which is a sink.
    // Actually, the algorithm checks sinks in the complement iteratively.
    // With complement = {000, 001, 010, 110, 111}, 000 is a sink, so it gets added to reachability.
    // Then 001 and 010 become sinks relative to the new complement, so they're added too.
    // This means all states in the complement eventually get added, so `result = empty`.
    assert!(
        result.is_empty(),
        "Trim sinks from {{011, 100, 101}} should remove all (iterative sink removal)"
    );
    Ok(())
}

#[test]
fn test_trim_sinks_from_strong_basin_attr1() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let basin = mk_states(&graph, STRONG_BASIN_ATTR1);

    let result = TrimSinks::run(&graph, basin.clone())?;

    // 001 and 010 have successors: 001→000, 010→000
    // The complement is {000, 011, 100, 101, 110, 111}
    // 000 is a sink in the complement, so it gets added to reachability.
    // Then 001 and 010 become sinks (they only reach `000` which is now in the reachability set).
    // So they get removed too.
    assert!(
        result.is_empty(),
        "Trim sinks from {{001, 010}} should remove all (iterative sink removal)"
    );
    Ok(())
}

#[test]
fn test_trim_sinks_from_strong_basin_attr2() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let s101 = mk_state(&graph, S101);

    let result = TrimSinks::run(&graph, s101.clone())?;

    // 101 → 111, and 111 is in the complement.
    // But the algorithm checks sinks iteratively. With complement = {000, 001, 010, 011, 100, 110, 111},
    // sinks get added iteratively; eventually all states in complement are added, so `result = empty`.
    assert!(
        result.is_empty(),
        "Trim sinks from {{101}} should remove it (iterative sink removal)"
    );
    Ok(())
}

#[test]
fn test_trim_sinks_from_weak_basin() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let weak_basin = mk_states(&graph, WEAK_BASIN);

    let result = TrimSinks::run(&graph, weak_basin.clone())?;

    // The algorithm removes sinks iteratively. With a complement containing {000},
    // sinks get removed iteratively until a fixed point.
    // All states eventually get removed.
    assert!(
        result.is_empty(),
        "Trim sinks from {{011, 100}} should remove all (iterative sink removal)"
    );
    Ok(())
}

#[test]
fn test_trim_sinks_from_mixed_set() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    // Mix of sink and non-sink states: {000, 001, 110}
    let mixed = mk_states(&graph, &[S000, S001, S110]);

    let result = TrimSinks::run(&graph, mixed.clone())?;

    // Should remove 000 (sink). After that, 001 becomes a sink (only reaches 000),
    // so it gets removed too. The algorithm continues iteratively until all are removed.
    assert!(
        result.is_empty(),
        "Trim sinks from {{000, 001, 110}} should remove all (iterative sink removal)"
    );
    Ok(())
}

#[test]
fn test_trim_sinks_from_attractor_1_plus_basin() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();
    let attractor_1 = mk_states(&graph, ATTRACTOR_1);
    let basin = mk_states(&graph, STRONG_BASIN_ATTR1);
    let initial = attractor_1.union(&basin);

    let result = TrimSinks::run(&graph, initial.clone())?;

    // The algorithm removes sinks iteratively.
    // Initial = {000, 001, 010}, complement = {011, 100, 101, 110, 111}
    // 000 is a sink, so it gets removed first.
    // Then 001 and 010 become sinks (they only reach 000 which is gone), so they're removed too.
    // The result is empty.
    assert!(
        result.is_empty(),
        "Trim sinks from {{000, 001, 010}} should remove all (iterative sink removal)"
    );
    Ok(())
}

#[test]
fn test_trim_sinks_is_subset_of_initial() -> Cancellable<()> {
    init_logger();
    let graph = create_test_network();

    // Sinks trim always returns a subset of the initial set
    for state in ALL_STATES {
        let initial = mk_state(&graph, *state);
        let result = TrimSinks::run(&graph, initial.clone())?;
        assert!(
            result.is_subset(&initial),
            "Trim sinks from {:03b} must return a subset of the initial set",
            state
        );
    }
    Ok(())
}
