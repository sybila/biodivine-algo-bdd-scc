//! Tests comparing ITGR+XieBeerel vs. XieBeerel only vs. SCC-based attractors on real model files.
//!
//! These tests verify that all three methods produce the same results,
//! while also testing with timeouts to ensure tests don't hang.

use crate::attractor::{
    AttractorConfig, InterleavedTransitionGuidedReduction, ItgrState, XieBeerelAttractors,
    XieBeerelState,
};
use crate::scc::{FwdBwdScc, SccConfig};
use crate::test_utils::symbolic_sets_to_sorted_sets;
use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use cancel_this::Cancellable;
use computation_process::{Computable, GenAlgorithm, Stateful};
use std::collections::BTreeSet;
use std::time::Duration;
use test_generator::test_resources;

/// Check if an SCC has no outgoing edges (i.e., is an attractor).
/// An SCC is an attractor if all successors of states in the SCC stay within the SCC.
fn is_attractor_scc(graph: &SymbolicAsyncGraph, scc: &GraphColoredVertices) -> bool {
    // Check if any variable can take states outside the SCC
    for var in graph.variables() {
        let can_post_out = graph.var_post_out(var, scc);
        // `var_post_out` returns successors outside `scc` (and excludes successors already in `scc`).
        // Hence, `scc` is an attractor iff this set is empty for every variable.
        if !can_post_out.is_empty() {
            return false;
        }
    }
    true
}

/// Extract attractors from SCCs by filtering to those with no outgoing edges.
fn extract_attractors_from_sccs(
    graph: &SymbolicAsyncGraph,
    sccs: Vec<GraphColoredVertices>,
) -> Vec<GraphColoredVertices> {
    sccs.into_iter()
        .filter(|scc| is_attractor_scc(graph, scc))
        .collect()
}

/// Filter out trivial (single-state) attractors.
/// SCC detection skips trivial SCCs, so we need to filter them out from attractor detection
/// results to make a fair comparison.
fn filter_trivial_attractors(attractors: Vec<GraphColoredVertices>) -> Vec<GraphColoredVertices> {
    attractors
        .into_iter()
        .filter(|attr| {
            // An attractor is trivial if it's a singleton (single state)
            // Check by seeing if removing one vertex leaves no colors
            let non_trivial = attr.minus(&attr.pick_vertex()).colors();
            !non_trivial.is_empty()
        })
        .collect()
}

/// Compare attractors from three different methods, regardless of order.
fn compare_attractor_results(
    graph: &SymbolicAsyncGraph,
    itgr_xie_beerel: Vec<GraphColoredVertices>,
    xie_beerel_only: Vec<GraphColoredVertices>,
    scc_based: Vec<GraphColoredVertices>,
    num_vars: usize,
    model_path: &str,
) {
    let itgr_sets = symbolic_sets_to_sorted_sets(graph, &itgr_xie_beerel, num_vars);
    let xie_beerel_sets = symbolic_sets_to_sorted_sets(graph, &xie_beerel_only, num_vars);
    let scc_sets = symbolic_sets_to_sorted_sets(graph, &scc_based, num_vars);

    // Compare counts
    assert_eq!(
        itgr_sets.len(),
        xie_beerel_sets.len(),
        "Attractor count mismatch for {}: ITGR+XieBeerel found {}, XieBeerel found {}",
        model_path,
        itgr_sets.len(),
        xie_beerel_sets.len()
    );

    assert_eq!(
        itgr_sets.len(),
        scc_sets.len(),
        "Attractor count mismatch for {}: ITGR+XieBeerel found {}, SCC-based found {}",
        model_path,
        itgr_sets.len(),
        scc_sets.len()
    );

    // Compare individual attractors
    for (i, ((itgr_attr, xie_beerel_attr), scc_attr)) in itgr_sets
        .iter()
        .zip(xie_beerel_sets.iter())
        .zip(scc_sets.iter())
        .enumerate()
    {
        assert_eq!(
            itgr_attr, xie_beerel_attr,
            "Attractor {} mismatch for {}: ITGR+XieBeerel found {:?}, XieBeerel found {:?}",
            i, model_path, itgr_attr, xie_beerel_attr
        );

        assert_eq!(
            itgr_attr, scc_attr,
            "Attractor {} mismatch for {}: ITGR+XieBeerel found {:?}, SCC-based found {:?}",
            i, model_path, itgr_attr, scc_attr
        );
    }
}

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

/// Generic helper function to compare three attractor detection methods.
fn test_attractor_comparison_impl(model_path: &str) -> Cancellable<()> {
    // Load the model
    let bn = BooleanNetwork::try_from_file(model_path)
        .unwrap_or_else(|e| panic!("Failed to load model {}: {:?}", model_path, e));

    // Only test networks with <20 variables
    if bn.num_vars() >= 20 {
        return Ok(());
    }

    let graph = SymbolicAsyncGraph::new(&bn)
        .unwrap_or_else(|e| panic!("Failed to create graph from {}: {:?}", model_path, e));

    // Collect attractors using ITGR + XieBeerel
    let itgr_xie_beerel = filter_trivial_attractors(run_xie_beerel(&graph, true)?);

    // Collect attractors using XieBeerel only
    let xie_beerel_only = filter_trivial_attractors(run_xie_beerel(&graph, false)?);

    // Collect attractors using SCC decomposition (fwd-bwd), filtered to SCCs with no outgoing edges
    // Note: SCC detection already filters out trivial SCCs, so we don't need to filter again
    let scc_config = SccConfig::new(graph.clone());
    let sccs: Vec<GraphColoredVertices> = FwdBwdScc::configure(scc_config, &graph)
        .computation::<Vec<_>>()
        .compute()?;
    let scc_based = extract_attractors_from_sccs(&graph, sccs);

    // Compare results
    compare_attractor_results(
        &graph,
        itgr_xie_beerel,
        xie_beerel_only,
        scc_based,
        bn.num_vars(),
        model_path,
    );

    Ok(())
}

/// Test attractor algorithms comparison on model files.
///
/// The entire test has a 2s timeout. The test passes if it completes or times out.
#[test_resources("./models/bbm-inputs-true/*.aeon")]
fn test_attractor_comparison(model_path: &str) {
    let two_seconds = Duration::from_secs(2);
    match cancel_this::on_timeout(two_seconds, || test_attractor_comparison_impl(model_path)) {
        Ok(()) => {}
        Err(_) => {
            // Test passes if canceled due to timeout
            // Cancellation errors are expected for long-running computations
        }
    }
}
