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
