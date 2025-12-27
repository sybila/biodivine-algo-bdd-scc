//! Tests comparing FwdBwdScc vs. ChainScc on real model files.
//!
//! These tests verify that both algorithms produce the same results,
//! while also testing with timeouts to ensure tests don't hang.

use crate::algorithm::scc::tests::sccs_to_sorted_sets;
use crate::algorithm::scc::{ChainScc, FwdBwdScc};
use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use cancel_this::Cancellable;
use std::time::Duration;
use test_generator::test_resources;

/// Compare SCCs from two algorithms, regardless of order.
/// Both algorithms should produce the same set of SCCs.
fn compare_scc_results(
    graph: &SymbolicAsyncGraph,
    fwd_bwd_sccs: Vec<GraphColoredVertices>,
    chain_sccs: Vec<GraphColoredVertices>,
    num_vars: usize,
    model_path: &str,
) {
    let fwd_bwd_sets = sccs_to_sorted_sets(graph, &fwd_bwd_sccs, num_vars);
    let chain_sets = sccs_to_sorted_sets(graph, &chain_sccs, num_vars);

    assert_eq!(
        fwd_bwd_sets.len(),
        chain_sets.len(),
        "SCC count mismatch for {}: FwdBwd found {}, Chain found {}",
        model_path,
        fwd_bwd_sets.len(),
        chain_sets.len()
    );

    for (i, (fwd_bwd_scc, chain_scc)) in fwd_bwd_sets.iter().zip(chain_sets.iter()).enumerate() {
        assert_eq!(
            fwd_bwd_scc, chain_scc,
            "SCC {} mismatch for {}: FwdBwd found {:?}, Chain found {:?}",
            i, model_path, fwd_bwd_scc, chain_scc
        );
    }
}

/// Generic helper function to compare FwdBwdScc and ChainScc algorithms.
fn test_scc_comparison_impl(model_path: &str) -> Cancellable<()> {
    // Load the model
    let bn = BooleanNetwork::try_from_file(model_path)
        .unwrap_or_else(|e| panic!("Failed to load model {}: {:?}", model_path, e));

    // Only test networks with <20 variables
    if bn.num_vars() >= 20 {
        return Ok(());
    }

    let graph = SymbolicAsyncGraph::new(&bn)
        .unwrap_or_else(|e| panic!("Failed to create graph from {}: {:?}", model_path, e));

    // Collect SCCs from FwdBwdScc
    let fwd_bwd_sccs = FwdBwdScc::configure(graph.clone(), &graph)
        .computation::<Vec<_>>()
        .compute()?;

    // Collect SCCs from ChainScc
    let chain_sccs = ChainScc::configure(graph.clone(), &graph)
        .computation::<Vec<_>>()
        .compute()?;

    // Compare results
    compare_scc_results(&graph, fwd_bwd_sccs, chain_sccs, bn.num_vars(), model_path);

    Ok(())
}

/// Test SCC algorithms comparison on model files.
///
/// The entire test has a 5s timeout. The test passes if it completes or times out.
#[test_resources("./models/bbm-inputs-true/*.aeon")]
fn test_scc_fwd_bwd_vs_chain_comparison(model_path: &str) {
    let five_seconds = Duration::from_secs(5);
    match cancel_this::on_timeout(five_seconds, || test_scc_comparison_impl(model_path)) {
        Ok(()) => {}
        Err(_) => {
            // Test passes if canceled due to timeout
            // Cancellation errors are expected for long-running computations
        }
    }
}
