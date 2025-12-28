use crate::algorithm::test_utils::mk_state;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

#[cfg(test)]
mod llm_test_long_lived;
#[cfg(test)]
mod llm_test_model_comparison;
#[cfg(test)]
mod llm_tests;

/// Collect all state numbers from a GraphColoredVertices set.
/// Returns a sorted vector of state numbers for comparison.
///
/// At the moment, this only supports up to 20 variables.
///
/// # Arguments
///
/// * `graph` - The symbolic async graph
/// * `set` - The set of colored vertices to extract state numbers from
/// * `num_vars` - The number of variables in the graph
///
/// # Example
///
/// For a 3-variable graph, if `set` contains states `{000, 101, 111}`, this returns `[0, 5, 7]`.
pub fn collect_state_numbers(
    graph: &SymbolicAsyncGraph,
    set: &GraphColoredVertices,
    num_vars: usize,
) -> Vec<u32> {
    assert!(num_vars <= 20);
    let mut states = Vec::new();
    let max_state = (1u32 << num_vars) - 1;
    for state in 0..=max_state {
        let state_set = mk_state(graph, state);
        if !state_set.intersect(set).is_empty() {
            states.push(state);
        }
    }
    states
}

/// Convert a slice of SCCs (as GraphColoredVertices) to sorted sets of state numbers.
/// This is useful for comparing SCCs from different algorithms, as it normalizes
/// the representation and sorts them consistently.
///
/// # Arguments
///
/// * `graph` - The symbolic async graph
/// * `sccs` - A slice of SCCs represented as GraphColoredVertices
/// * `num_vars` - The number of variables in the graph
///
/// # Returns
///
/// A vector of HashSets containing state numbers, sorted by size and then by sorted state numbers.
pub fn sccs_to_sorted_sets(
    graph: &SymbolicAsyncGraph,
    sccs: &[GraphColoredVertices],
    num_vars: usize,
) -> Vec<std::collections::HashSet<u32>> {
    use std::collections::HashSet;

    let mut sets: Vec<HashSet<u32>> = sccs
        .iter()
        .map(|scc| {
            collect_state_numbers(graph, scc, num_vars)
                .into_iter()
                .collect()
        })
        .collect();

    // Sort by size, then by sorted state numbers for consistent ordering
    sets.sort_by_cached_key(|s| {
        let mut v: Vec<u32> = s.iter().copied().collect();
        v.sort();
        (v.len(), v)
    });

    sets
}
