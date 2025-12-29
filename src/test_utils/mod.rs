pub mod llm_example_network;
pub mod llm_transition_builder;

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

/// Initialize env_logger for tests. Safe to call multiple times.
pub fn init_logger() {
    let _ = env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Trace)
        .is_test(true)
        .try_init();
}

/// Creates a singleton state from a state number.
///
/// The state number is interpreted as binary encoding (the most significant bit = variable 0).
/// The number of variables is inferred from the graph.
///
/// # Arguments
///
/// * `graph` - The symbolic async graph
/// * `state` - The state number (binary encoding: x0*2^(n-1) + x1*2^(n-2) + ... + x(n-1)*2^0)
///
/// # Example
///
/// For a 3-variable graph:
/// - `mk_state(graph, 0)` creates state `000`
/// - `mk_state(graph, 5)` creates state `101`
/// - `mk_state(graph, 7)` creates state `111`
pub fn mk_state(graph: &SymbolicAsyncGraph, state: u32) -> GraphColoredVertices {
    let vars: Vec<_> = graph.variables().collect();
    let num_vars = vars.len();

    assert!(
        state < (1u32 << num_vars),
        "State {} out of range for {} variables (max: {})",
        state,
        num_vars,
        (1u32 << num_vars) - 1
    );

    let mut assignments = Vec::new();
    for i in 0..num_vars {
        let shift = num_vars - 1 - i;
        let value = (state >> shift) & 1 == 1;
        assignments.push((vars[i], value));
    }

    graph.mk_subspace(&assignments)
}

/// Creates a set of states from a list of state numbers.
///
/// # Example
///
/// `mk_states(graph, &[0, 5, 7])` creates the set `{000, 101, 111}`.
pub fn mk_states(graph: &SymbolicAsyncGraph, states: &[u32]) -> GraphColoredVertices {
    let mut result = graph.mk_empty_colored_vertices();
    for &s in states {
        result = result.union(&mk_state(graph, s));
    }
    result
}

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

/// Convert a slice of GraphColoredVertices sets to sorted sets of state numbers.
/// This is useful for comparing sets from different algorithms, as it normalizes
/// the representation and sorts them consistently.
///
/// # Arguments
///
/// * `graph` - The symbolic async graph
/// * `sets` - A slice of sets represented as GraphColoredVertices
/// * `num_vars` - The number of variables in the graph
///
/// # Returns
///
/// A vector of HashSets containing state numbers, sorted by size and then by sorted state numbers.
pub fn symbolic_sets_to_sorted_sets(
    graph: &SymbolicAsyncGraph,
    sets: &[GraphColoredVertices],
    num_vars: usize,
) -> Vec<std::collections::HashSet<u32>> {
    use std::collections::HashSet;

    let mut result: Vec<HashSet<u32>> = sets
        .iter()
        .map(|set| {
            collect_state_numbers(graph, set, num_vars)
                .into_iter()
                .collect()
        })
        .collect();

    // Sort by size, then by sorted state numbers for consistent ordering
    result.sort_by_cached_key(|s| {
        let mut v: Vec<u32> = s.iter().copied().collect();
        v.sort();
        (v.len(), v)
    });

    result
}
