//! Generate Boolean Networks from lists of state transitions.
//!
//! This module provides utilities to construct Boolean Networks by specifying
//! the asynchronous state transition graph directly as a list of edges.
//!
//! # Overview
//!
//! In asynchronous Boolean networks, a transition `s → s'` means exactly one
//! variable `i` changes: `s[i] ≠ s'[i]` and `s[j] = s'[j]` for all `j ≠ i`.
//! When variable `i` updates from state `s` to `s'`, we know that `f_i(s) = s'[i]`.
//!
//! This module derives update functions using Disjunctive Normal Form (DNF):
//! for each variable `i`, it collects all states `s` where `f_i(s) = 1` and
//! expresses them as a DNF formula.
//!
//! # Example
//!
//! ```ignore
//! // This module is only available during testing.
//! use crate::test_utils::llm_transition_builder::from_transitions;
//!
//! // Define transitions for a 2-variable network:
//! // 00 → 10 (variable 0 flips to 1)
//! // 10 → 11 (variable 1 flips to 1)
//! // 11 → 01 (variable 0 flips to 0)
//! // 01 → 00 (variable 1 flips to 0)
//! let transitions = vec![
//!     (0b00, 0b10),  // 00 → 10
//!     (0b10, 0b11),  // 10 → 11
//!     (0b11, 0b01),  // 11 → 01
//!     (0b01, 0b00),  // 01 → 00
//! ];
//!
//! let bn = from_transitions(2, &transitions).expect("Failed to create network");
//! ```

use biodivine_lib_param_bn::BooleanNetwork;
use std::collections::{HashMap, HashSet};

/// Represents a transition from one state to another.
/// States are represented as integers where the binary encoding corresponds
/// to the variable values (the most significant bit = variable 0).
pub type Transition = (u32, u32);

/// Error type for transition-based network construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionError {
    /// A transition has invalid state indices (out of range for the number of variables).
    InvalidState { state: u32, num_vars: usize },
    /// A transition changes more than one variable (invalid for asynchronous semantics).
    MultipleVariablesChanged { from: u32, to: u32 },
    /// A transition changes no variables (self-loop without an update).
    NoVariableChanged { state: u32 },
    /// Failed to parse the generated AEON model.
    ParseError(String),
}

impl std::fmt::Display for TransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransitionError::InvalidState { state, num_vars } => {
                write!(
                    f,
                    "State {} is invalid for {} variables (max state: {})",
                    state,
                    num_vars,
                    (1u32 << num_vars) - 1
                )
            }
            TransitionError::MultipleVariablesChanged { from, to } => {
                write!(
                    f,
                    "Transition {} → {} changes multiple variables (asynchronous semantics requires exactly one)",
                    from, to
                )
            }
            TransitionError::NoVariableChanged { state } => {
                write!(f, "Transition {} → {} changes no variables", state, state)
            }
            TransitionError::ParseError(msg) => {
                write!(f, "Failed to parse generated AEON model: {}", msg)
            }
        }
    }
}

impl std::error::Error for TransitionError {}

/// Extract the value of variable `i` from state `s`.
/// Variable 0 is the most significant bit.
fn get_variable_value(state: u32, var_idx: usize, num_vars: usize) -> bool {
    let shift = num_vars - 1 - var_idx;
    (state >> shift) & 1 == 1
}

/// Count the number of differing bits between two states.
fn hamming_distance(a: u32, b: u32) -> u32 {
    (a ^ b).count_ones()
}

/// Find which variable changed in a transition or return [`TransitionError`] if invalid.
fn find_changed_variable(from: u32, to: u32, num_vars: usize) -> Result<usize, TransitionError> {
    let distance = hamming_distance(from, to);

    if distance == 0 {
        return Err(TransitionError::NoVariableChanged { state: from });
    }

    if distance > 1 {
        return Err(TransitionError::MultipleVariablesChanged { from, to });
    }

    // Find the differing bit position
    let diff = from ^ to;
    for i in 0..num_vars {
        let shift = num_vars - 1 - i;
        if (diff >> shift) & 1 == 1 {
            return Ok(i);
        }
    }

    unreachable!("Hamming distance was 1 but no differing bit found")
}

/// Convert a set of states to a DNF formula, simplified to remove redundant variables.
/// Returns "0" if the set is empty (function is always false), "1" if all states are true.
fn states_to_dnf(states: &HashSet<u32>, num_vars: usize, var_names: &[String]) -> String {
    if states.is_empty() {
        return "0".to_string(); // Always false
    }

    let all_states = 1u32 << num_vars;
    if states.len() == all_states as usize {
        return "1".to_string(); // Always true
    }

    // Find which variables are actually essential (affect the function value)
    let mut essential_vars = Vec::new();
    for j in 0..num_vars {
        let mut is_essential = false;
        let flip_mask = 1u32 << (num_vars - 1 - j);

        for state_a in 0..all_states {
            let state_b = state_a ^ flip_mask;
            let f_a = states.contains(&state_a);
            let f_b = states.contains(&state_b);

            if f_a != f_b {
                is_essential = true;
                break;
            }
        }

        if is_essential {
            essential_vars.push(j);
        }
    }

    // If no essential variables, the function is constant (shouldn't happen here)
    if essential_vars.is_empty() {
        return if states.contains(&0) {
            "1".to_string()
        } else {
            "0".to_string()
        };
    }

    // Build simplified DNF using only essential variables
    let mut simplified_terms = HashSet::new();
    for &state in states {
        let mut term_parts = Vec::new();
        for &j in &essential_vars {
            let value = get_variable_value(state, j, num_vars);
            if value {
                term_parts.push(var_names[j].clone());
            } else {
                term_parts.push(format!("!{}", var_names[j]));
            }
        }
        simplified_terms.insert(format!("({})", term_parts.join(" & ")));
    }

    let mut terms: Vec<String> = simplified_terms.into_iter().collect();
    terms.sort(); // For deterministic output
    terms.join(" | ")
}

/// Generate variable names for a network with `num_vars` variables.
fn generate_var_names(num_vars: usize) -> Vec<String> {
    (0..num_vars).map(|i| format!("x{}", i)).collect()
}

/// Determine which variables appear in the simplified DNF representation of a function.
fn find_variables_in_dnf(function_true_states: &HashSet<u32>, num_vars: usize) -> HashSet<usize> {
    if function_true_states.is_empty() || function_true_states.len() == (1u32 << num_vars) as usize
    {
        // Constant function: return an empty set (self-loop will be handled separately)
        HashSet::new()
    } else {
        // Find essential variables (same logic as in states_to_dnf)
        let mut essential = HashSet::new();
        for j in 0..num_vars {
            let mut is_essential = false;
            let flip_mask = 1u32 << (num_vars - 1 - j);

            for state_a in 0..(1u32 << num_vars) {
                let state_b = state_a ^ flip_mask;
                let f_a = function_true_states.contains(&state_a);
                let f_b = function_true_states.contains(&state_b);

                if f_a != f_b {
                    is_essential = true;
                    break;
                }
            }

            if is_essential {
                essential.insert(j);
            }
        }
        essential
    }
}

/// Create a Boolean Network from a list of transitions.
///
/// # Arguments
///
/// * `num_vars` - The number of variables in the network
/// * `transitions` - A list of `(from_state, to_state)` pairs. States are represented
///   as integers where the binary encoding corresponds to variable values
///   (the most significant bit = variable 0).
///
/// # Returns
///
/// A `BooleanNetwork` constructed from the transitions, or an error if:
/// - Any state is out of range (must be < 2^num_vars)
/// - Any transition changes more than one variable (invalid for asynchronous semantics)
/// - Any transition is a self-loop (no variable changes)
///
/// # Algorithm
///
/// 1. For each transition `s → s'`, determine which variable `i` changed
/// 2. Set `f_i(s) = s'[i]` (the new value of variable `i` in state `s'`)
/// 3. For states with no outgoing transitions, `f_i(s) = s[i]` (fixed point)
/// 4. Convert each function `f_i` to DNF by collecting all states where `f_i(s) = 1`
/// 5. Generate an AEON format string and parse it into a `BooleanNetwork`
///
/// # Example
///
/// ```ignore
/// // This module is only available during testing.
/// use crate::test_utils::llm_transition_builder::from_transitions;
///
/// // Create a simple 2-variable cycle: 00 → 10 → 11 → 01 → 00
/// let transitions = vec![
///     (0b00, 0b10),  // x0 flips: f_0(00) = 1
///     (0b10, 0b11),  // x1 flips: f_1(10) = 1
///     (0b11, 0b01),  // x0 flips: f_0(11) = 0
///     (0b01, 0b00),  // x1 flips: f_1(01) = 0
/// ];
///
/// let bn = from_transitions(2, &transitions).expect("Failed to create network");
/// assert_eq!(bn.num_vars(), 2);
/// ```
pub fn from_transitions(
    num_vars: usize,
    transitions: &[Transition],
) -> Result<BooleanNetwork, TransitionError> {
    let max_state = (1u32 << num_vars) - 1;

    // Validate all states are in range
    for &(from, to) in transitions {
        if from > max_state {
            return Err(TransitionError::InvalidState {
                state: from,
                num_vars,
            });
        }
        if to > max_state {
            return Err(TransitionError::InvalidState {
                state: to,
                num_vars,
            });
        }
    }

    // Track which states have outgoing transitions
    let mut states_with_transitions: HashSet<u32> = HashSet::new();

    // For each variable, collect states where f_i(s) = 1
    let mut function_true_states: Vec<HashSet<u32>> = vec![HashSet::new(); num_vars];

    // Build a map of which variables can update from each state
    let mut variables_that_update: HashMap<u32, HashSet<usize>> = HashMap::new();

    // Process each transition to determine which variables can update
    for &(from, to) in transitions {
        states_with_transitions.insert(from);

        // Find which variable changed
        let var_idx = find_changed_variable(from, to, num_vars)?;

        // Record that this variable can update from this state
        variables_that_update
            .entry(from)
            .or_insert_with(HashSet::new)
            .insert(var_idx);

        // Determine the new value of the changed variable
        let new_value = get_variable_value(to, var_idx, num_vars);

        // Set f_var_idx(from) = new_value (this variable will change)
        if new_value {
            function_true_states[var_idx].insert(from);
        } else {
            // Explicitly remove from a true set (in case it was added before)
            function_true_states[var_idx].remove(&from);
        }
    }

    // For states with transitions, set functions for variables that DON'T update
    // to their current values (so they don't change)
    for (&state, &ref updating_vars) in &variables_that_update {
        for j in 0..num_vars {
            if !updating_vars.contains(&j) {
                // This variable doesn't update from this state, so f_j(state) = state[j]
                let current_value = get_variable_value(state, j, num_vars);
                if current_value {
                    function_true_states[j].insert(state);
                } else {
                    function_true_states[j].remove(&state);
                }
            }
        }
    }

    // For states without outgoing transitions (fixed points),
    // set f_i(s) = s[i]
    for state in 0..=max_state {
        if !states_with_transitions.contains(&state) {
            for i in 0..num_vars {
                if get_variable_value(state, i, num_vars) {
                    function_true_states[i].insert(state);
                }
            }
        }
    }

    // Generate variable names
    let var_names = generate_var_names(num_vars);

    // Build AEON format string
    let mut aeon_lines = Vec::new();

    // Add update functions in DNF first (we need them to determine dependencies)
    let mut dnf_functions = Vec::new();
    for i in 0..num_vars {
        let dnf = states_to_dnf(&function_true_states[i], num_vars, &var_names);
        dnf_functions.push(dnf.clone());
    }

    // Add edges: for each variable, declare edges from variables that appear in its DNF.
    // We use "observable" (-?) edges since we don't know `monotonicity` from transitions alone.
    for i in 0..num_vars {
        let dnf = &dnf_functions[i];
        let is_constant = dnf == "0" || dnf == "1";

        if is_constant {
            // For constant functions, only declare self-loop (required by AEON)
            aeon_lines.push(format!("{} -? {}", var_names[i], var_names[i]));
        } else {
            // For non-constant functions, declare all variables that appear in DNF
            // Since our DNF terms include all variables, declare all edges
            let vars_in_dnf = find_variables_in_dnf(&function_true_states[i], num_vars);
            for &j in &vars_in_dnf {
                aeon_lines.push(format!("{} -? {}", var_names[j], var_names[i]));
            }
        }
    }

    // Add update functions
    for i in 0..num_vars {
        aeon_lines.push(format!("${}: {}", var_names[i], dnf_functions[i]));
    }

    let aeon_model = aeon_lines.join("\n");

    // Parse and return the Boolean Network
    BooleanNetwork::try_from(aeon_model.as_str())
        .and_then(|it| it.infer_valid_graph())
        .map_err(|e| TransitionError::ParseError(format!("{:?}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{collect_state_numbers, mk_state};
    use biodivine_lib_param_bn::biodivine_std::traits::Set;
    use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;

    /// Verify that all declared transitions exist in the graph.
    fn verify_transitions(graph: &SymbolicAsyncGraph, transitions: &[Transition], num_vars: usize) {
        for &(from_state, to_state) in transitions {
            let from = mk_state(graph, from_state);
            let to = mk_state(graph, to_state);

            let successors = graph.post(&from);

            assert!(
                to.is_subset(&successors),
                "Transition {} → {} not found in graph. Successors of {}: {:?}",
                from_state,
                to_state,
                from_state,
                collect_state_numbers(graph, &successors, num_vars)
            );
        }
    }

    /// Verify that states with transitions have ONLY the declared transitions (no extra ones).
    fn verify_no_unwanted_transitions(
        graph: &SymbolicAsyncGraph,
        transitions: &[Transition],
        num_vars: usize,
    ) {
        let states_with_transitions: HashSet<u32> =
            transitions.iter().map(|(from, _)| *from).collect();

        // Build a map of declared transitions: from_state -> set of to_states
        let mut declared_transitions: HashMap<u32, HashSet<u32>> = HashMap::new();
        for &(from, to) in transitions {
            declared_transitions
                .entry(from)
                .or_insert_with(HashSet::new)
                .insert(to);
        }

        // Check each state that should have transitions
        for &from_state in &states_with_transitions {
            let from = mk_state(graph, from_state);
            let successors = graph.post(&from);
            let actual_successors: HashSet<u32> =
                collect_state_numbers(graph, &successors, num_vars)
                    .into_iter()
                    .collect();

            let declared_successors = declared_transitions
                .get(&from_state)
                .expect("State should be in declared transitions map");

            // Check for unwanted transitions (transitions that exist but weren't declared)
            let unwanted: Vec<u32> = actual_successors
                .difference(declared_successors)
                .copied()
                .collect();

            assert!(
                unwanted.is_empty(),
                "State {} has {} unwanted transition(s): {:?}. Declared transitions: {:?}, Actual transitions: {:?}",
                from_state,
                unwanted.len(),
                unwanted,
                declared_successors,
                actual_successors
            );
        }
    }

    /// Verify that the graph contains exactly the declared transitions (and fixed points).
    /// Fixed points are states with no outgoing transitions.
    fn verify_exact_transitions(
        graph: &SymbolicAsyncGraph,
        transitions: &[Transition],
        num_vars: usize,
    ) {
        let max_state = (1u32 << num_vars) - 1;
        let states_with_transitions: HashSet<u32> =
            transitions.iter().map(|(from, _)| *from).collect();

        // First, explicitly verify no unwanted transitions exist
        verify_no_unwanted_transitions(graph, transitions, num_vars);

        // Then verify all states have the correct transitions
        for state in 0..=max_state {
            let from = mk_state(graph, state);
            let successors = graph.post(&from);

            if states_with_transitions.contains(&state) {
                // This state should have exactly the declared transitions
                let declared_successors: HashSet<u32> = transitions
                    .iter()
                    .filter(|(from, _)| *from == state)
                    .map(|(_, to)| *to)
                    .collect();

                let actual_successors: HashSet<u32> =
                    collect_state_numbers(graph, &successors, num_vars)
                        .into_iter()
                        .collect();

                // Verify all declared transitions exist
                for &declared_to in &declared_successors {
                    assert!(
                        actual_successors.contains(&declared_to),
                        "State {} should have transition to {}, but it's missing. Actual successors: {:?}",
                        state,
                        declared_to,
                        actual_successors
                    );
                }

                // Verify no extra transitions exist (this is also checked by verify_no_unwanted_transitions,
                // but we check again here for completeness)
                assert_eq!(
                    actual_successors, declared_successors,
                    "State {} has unexpected transitions. Expected exactly: {:?}, Got: {:?}",
                    state, declared_successors, actual_successors
                );
            } else {
                // This state is a fixed point (no outgoing transitions)
                assert!(
                    successors.is_empty(),
                    "State {} should be a fixed point (no transitions), but has {} successor(s): {:?}",
                    state,
                    successors.exact_cardinality(),
                    collect_state_numbers(graph, &successors, num_vars)
                );
            }
        }
    }

    #[test]
    fn test_simple_2var_cycle() {
        // Create a 2-variable cycle: 00 → 10 → 11 → 01 → 00
        let transitions = vec![
            (0b00, 0b10), // x0 flips: f_0(00) = 1
            (0b10, 0b11), // x1 flips: f_1(10) = 1
            (0b11, 0b01), // x0 flips: f_0(11) = 0
            (0b01, 0b00), // x1 flips: f_1(01) = 0
        ];

        let bn = from_transitions(2, &transitions).expect("Failed to create network");
        assert_eq!(bn.num_vars(), 2);

        // Verify we can create a graph from it
        let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");
        assert_eq!(
            graph.mk_unit_colored_vertices().exact_cardinality(),
            4u32.into()
        );

        // Verify all declared transitions exist
        verify_transitions(&graph, &transitions, 2);

        // Verify exact transition structure
        verify_exact_transitions(&graph, &transitions, 2);
    }

    #[test]
    fn test_fixed_point() {
        // Create a network where 00 is a fixed point
        // 00 → 00 (no transition, so f_i(00) = 00[i] = 0 for all i)
        // 01 → 00 (x1 flips: f_1(01) = 0)
        let transitions = vec![
            (0b01, 0b00), // x1 flips: f_1(01) = 0
        ];

        let bn = from_transitions(2, &transitions).expect("Failed to create network");
        let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");

        // Verify all declared transitions exist
        verify_transitions(&graph, &transitions, 2);

        // Verify the exact transition structure (including fixed points)
        verify_exact_transitions(&graph, &transitions, 2);

        // Specifically, verify that state 00 is a fixed point
        let s00 = mk_state(&graph, 0b00);
        let successors = graph.post(&s00);
        assert!(successors.is_empty(), "State 00 should be a fixed point");
    }

    #[test]
    fn test_invalid_multiple_variables() {
        // Transition that changes multiple variables should fail
        let transitions = vec![
            (0b00, 0b11), // Changes both x0 and x1 - invalid!
        ];

        let result = from_transitions(2, &transitions);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransitionError::MultipleVariablesChanged { .. } => {}
            _ => panic!("Expected MultipleVariablesChanged error"),
        }
    }

    #[test]
    fn test_invalid_state_range() {
        // State out of range should fail
        let transitions = vec![
            (0b100, 0b101), // State 4 is invalid for 2 variables (max is 3)
        ];

        let result = from_transitions(2, &transitions);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransitionError::InvalidState { .. } => {}
            _ => panic!("Expected InvalidState error"),
        }
    }

    #[test]
    fn test_self_loop() {
        // Self-loop (no change) should fail
        let transitions = vec![
            (0b00, 0b00), // No variable changes - invalid!
        ];

        let result = from_transitions(2, &transitions);
        assert!(result.is_err());
        match result.unwrap_err() {
            TransitionError::NoVariableChanged { .. } => {}
            _ => panic!("Expected NoVariableChanged error"),
        }
    }

    #[test]
    fn test_3var_example() {
        // Test with 3 variables
        // Simple transitions: 000 → 100 → 110 → 111
        let transitions = vec![
            (0b000, 0b100), // x0 flips: f_0(000) = 1
            (0b100, 0b110), // x1 flips: f_1(100) = 1
            (0b110, 0b111), // x2 flips: f_2(110) = 1
        ];

        let bn = from_transitions(3, &transitions).expect("Failed to create network");
        assert_eq!(bn.num_vars(), 3);

        let graph = SymbolicAsyncGraph::new(&bn).expect("Failed to create graph");
        assert_eq!(
            graph.mk_unit_colored_vertices().exact_cardinality(),
            8u32.into()
        );

        // Verify all declared transitions exist
        verify_transitions(&graph, &transitions, 3);

        // Verify exact transition structure
        verify_exact_transitions(&graph, &transitions, 3);
    }
}
