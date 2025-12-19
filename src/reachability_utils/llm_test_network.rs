//! A well-documented test network for unit testing reachability algorithms.
//!
//! This module provides a single, carefully designed 3-variable Boolean network
//! that serves as a canonical test case for reachability and SCC algorithms.
//!
//! # Network Design
//!
//! The network uses three Boolean variables: `x0`, `x1`, `x2`, giving 8 possible states.
//! States are denoted as binary strings `x0x1x2` (e.g., `101` means x0=1, x1=0, x2=1).
//!
//! ## Update Functions
//!
//! The update functions are:
//! - `f_x0 = (x0 ∧ x1) ∨ (x1 ∧ x2) ∨ (x0 ∧ x2)` (majority function - true when ≥2 variables are true)
//! - `f_x1 = x0`
//! - `f_x2 = x0 ∧ (x1 ⊕ x2)` (x0 AND (x1 XOR x2))
//!
//! ## Asynchronous State Transition Graph
//!
//! In asynchronous semantics, from state `s`, we can transition to state `s'` if exactly one
//! variable `i` differs between `s` and `s'`, and `f_i(s) ≠ s[i]`.
//!
//! The resulting transition graph is:
//!
//! ```text
//!       (011) ─┬─► (001) ─┐
//!              │          ├─► (000) ◄── (100) ─┐
//!              └─► (010) ─┘                    │
//!              │                               │
//!              └───────────────────────────┐   │
//!                                          │   │
//!       (101) ──────────────────────────┐  │   │
//!                                       │  │   │
//!                                       ▼  ▼   ▼
//!                                     (111) ⇄ (110)
//! ```
//!
//! ### Edge List (explicit transitions):
//! - `001 → 000` (x2 updates: f_x2(001) = 0 ≠ 1)
//! - `010 → 000` (x1 updates: f_x1(010) = 0 ≠ 1)
//! - `011 → 001` (x1 updates: f_x1(011) = 0 ≠ 1)
//! - `011 → 010` (x2 updates: f_x2(011) = 0 ≠ 1)
//! - `011 → 111` (x0 updates: f_x0(011) = 1 ≠ 0)
//! - `100 → 000` (x0 updates: f_x0(100) = 0 ≠ 1)
//! - `100 → 110` (x1 updates: f_x1(100) = 1 ≠ 0)
//! - `101 → 111` (x1 updates: f_x1(101) = 1 ≠ 0)
//! - `110 → 111` (x2 updates: f_x2(110) = 1 ≠ 0)
//! - `111 → 110` (x2 updates: f_x2(111) = 0 ≠ 1)
//!
//! ### Structure Summary:
//!
//! | State | Successors     | Predecessors    | Description                                |
//! |-------|----------------|-----------------|--------------------------------------------|
//! | 000   | (none)         | 001, 010, 100   | Fixed point - Attractor 1 (trivial SCC)    |
//! | 001   | 000            | 011             | Strong basin of Attractor 1                |
//! | 010   | 000            | 011             | Strong basin of Attractor 1                |
//! | 011   | 001, 010, 111  | (none) - SOURCE | Weak basin (can reach both attractors)     |
//! | 100   | 000, 110       | (none) - SOURCE | Weak basin (can reach both attractors)     |
//! | 101   | 111            | (none) - SOURCE | Strong basin of Attractor 2                |
//! | 110   | 111            | 100, 111        | Part of Attractor 2 (non-trivial SCC)      |
//! | 111   | 110            | 011, 101, 110   | Part of Attractor 2 (non-trivial SCC)      |
//!
//! ## Basins and SCCs
//!
//! **Attractors:**
//! - Attractor 1: `{000}` - trivial SCC (fixed point)
//! - Attractor 2: `{110, 111}` - non-trivial SCC (2-cycle)
//!
//! **Basins:**
//! - Strong basin of Attractor 1: `{001, 010}` - states that can ONLY reach `{000}`
//! - Strong basin of Attractor 2: `{101}` - states that can ONLY reach `{110, 111}`
//! - Weak basin (shared): `{011, 100}` - states that can reach BOTH attractors
//!
//! **Source states (no predecessors):** `{011, 100, 101}`
//!
//! **Forward reachability examples:**
//! - From `001`: reaches `{001, 000}`
//! - From `010`: reaches `{010, 000}`
//! - From `011`: reaches `{011, 001, 010, 111, 000, 110}` (all except 100, 101)
//! - From `100`: reaches `{100, 000, 110, 111}` (all except 001, 010, 011, 101)
//! - From `101`: reaches `{101, 111, 110}`
//!
//! **Backward reachability examples:**
//! - To `000`: from `{000, 001, 010, 011, 100}` (all states that can reach attractor 1)
//! - To `110`: from `{110, 111, 100, 101, 011}` (attractor 2 plus its basin)
//! - To `111`: from `{111, 110, 101, 011, 100}` (same as 110, since they form a cycle)

use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};

/// Creates the canonical test network as a `SymbolicAsyncGraph`.
///
/// See the module documentation for a complete description of the network structure.
pub fn create_test_network() -> SymbolicAsyncGraph {
    // The network in AEON format.
    // Variables: x0, x1, x2
    // Update functions:
    //   f_x0 = (x0 & x1) | (x1 & x2) | (x0 & x2)  -- majority (all positive influences)
    //   f_x1 = x0                                  -- simple copy (positive influence)
    //   f_x2 = x0 & ((x1 & !x2) | (!x1 & x2))     -- x0 & (x1 XOR x2)
    //
    // For f_x0: all influences are positive (majority is monotone in all arguments)
    // For f_x1: x0 is positive
    // For f_x2: x0 is positive (AND), but x1 and x2 are non-monotonic (XOR)
    //
    // Edge notation:
    //   -> means activating (positive monotone)
    //   -? means observable (non-monotonic, can be either positive or negative)
    let aeon_model = r#"
        x0 -> x0
        x1 -> x0
        x2 -> x0
        x0 -> x1
        x0 -> x2
        x1 -? x2
        x2 -? x2
        $x0: (x0 & x1) | (x1 & x2) | (x0 & x2)
        $x1: x0
        $x2: x0 & ((x1 & !x2) | (!x1 & x2))
    "#;

    let bn = BooleanNetwork::try_from(aeon_model).expect("Invalid AEON model");
    SymbolicAsyncGraph::new(&bn).expect("Failed to create symbolic graph")
}

/// Returns the variable order for the test network: [x0, x1, x2].
///
/// This is useful for creating specific states using `mk_subspace_with_assignments`.
pub fn variable_order(graph: &SymbolicAsyncGraph) -> Vec<biodivine_lib_param_bn::VariableId> {
    graph.variables().collect()
}

/// Creates a singleton state from a binary representation (0-7).
///
/// The state number corresponds to the binary encoding x0*4 + x1*2 + x2*1.
/// For example:
/// - `mk_state(graph, 0)` creates state `000`
/// - `mk_state(graph, 5)` creates state `101`
/// - `mk_state(graph, 7)` creates state `111`
pub fn mk_state(graph: &SymbolicAsyncGraph, state: u8) -> GraphColoredVertices {
    assert!(state < 8, "State must be in range 0-7");
    let vars = variable_order(graph);
    let x0 = (state >> 2) & 1 == 1;
    let x1 = (state >> 1) & 1 == 1;
    let x2 = state & 1 == 1;
    graph.mk_subspace(&[(vars[0], x0), (vars[1], x1), (vars[2], x2)])
}

/// Creates a set of states from a list of binary representations.
///
/// For example, `mk_states(graph, &[0, 5, 7])` creates the set `{000, 101, 111}`.
pub fn mk_states(graph: &SymbolicAsyncGraph, states: &[u8]) -> GraphColoredVertices {
    let mut result = graph.mk_empty_colored_vertices();
    for &s in states {
        result = result.union(&mk_state(graph, s));
    }
    result
}

/// State constants for readability in tests.
pub mod states {
    /// State `000` - Fixed point, Attractor 1.
    pub const S000: u8 = 0b000;
    /// State `001` - Strong basin of Attractor 1.
    pub const S001: u8 = 0b001;
    /// State `010` - Strong basin of Attractor 1.
    pub const S010: u8 = 0b010;
    /// State `011` - Weak basin (can reach both attractors), SOURCE (no predecessors).
    pub const S011: u8 = 0b011;
    /// State `100` - Weak basin (can reach both attractors), SOURCE (no predecessors).
    pub const S100: u8 = 0b100;
    /// State `101` - Strong basin of Attractor 2, SOURCE (no predecessors).
    pub const S101: u8 = 0b101;
    /// State `110` - Part of Attractor 2 (non-trivial SCC).
    pub const S110: u8 = 0b110;
    /// State `111` - Part of Attractor 2 (non-trivial SCC).
    pub const S111: u8 = 0b111;
}

/// Predefined sets for common test scenarios.
pub mod sets {
    use super::states::*;

    /// Attractor 1: the fixed point `{000}`.
    pub const ATTRACTOR_1: &[u8] = &[S000];

    /// Attractor 2: the cycle `{110, 111}`.
    pub const ATTRACTOR_2: &[u8] = &[S110, S111];

    /// Strong basin of Attractor 1 (excluding the attractor itself): `{001, 010}`.
    pub const STRONG_BASIN_ATTR1: &[u8] = &[S001, S010];

    /// Strong basin of Attractor 2 (excluding the attractor itself): `{101}`.
    pub const STRONG_BASIN_ATTR2: &[u8] = &[S101];

    /// Weak basin (can reach both attractors): `{011, 100}`.
    pub const WEAK_BASIN: &[u8] = &[S011, S100];

    /// Source states (no predecessors): `{011, 100, 101}`.
    pub const SOURCE_STATES: &[u8] = &[S011, S100, S101];

    /// All states that can reach Attractor 1: `{000, 001, 010, 011, 100}`.
    pub const CAN_REACH_ATTR1: &[u8] = &[S000, S001, S010, S011, S100];

    /// All states that can reach Attractor 2: `{011, 100, 101, 110, 111}`.
    pub const CAN_REACH_ATTR2: &[u8] = &[S011, S100, S101, S110, S111];

    /// All 8 states in the network.
    pub const ALL_STATES: &[u8] = &[S000, S001, S010, S011, S100, S101, S110, S111];
}

#[cfg(test)]
mod tests {
    use super::states::*;
    use super::*;

    /// Verify that the test network has exactly 8 states (no parameters).
    #[test]
    fn test_network_has_8_states() {
        let graph = create_test_network();
        let all = graph.mk_unit_colored_vertices();
        assert_eq!(all.exact_cardinality(), 8u32.into());
    }

    /// Verify that state 000 is a fixed point (no outgoing transitions).
    #[test]
    fn test_state_000_is_fixed_point() {
        let graph = create_test_network();
        let s000 = mk_state(&graph, S000);

        let post = graph.post(&s000);
        assert!(
            post.is_empty(),
            "State 000 should have no successors (fixed point)"
        );
    }

    /// Verify that states 110 and 111 form a 2-cycle.
    #[test]
    fn test_attractor_2_is_cycle() {
        let graph = create_test_network();
        let s110 = mk_state(&graph, S110);
        let s111 = mk_state(&graph, S111);

        // 110 should have exactly one successor: 111
        let post_110 = graph.post(&s110);
        assert!(
            post_110.is_subset(&s111) && s111.is_subset(&post_110),
            "State 110 should have exactly successor 111"
        );

        // 111 should have exactly one successor: 110
        let post_111 = graph.post(&s111);
        assert!(
            post_111.is_subset(&s110) && s110.is_subset(&post_111),
            "State 111 should have exactly successor 110"
        );
    }

    /// Verify the complete edge structure of the transition graph.
    #[test]
    fn test_complete_transition_structure() {
        let graph = create_test_network();

        // Expected successors for each state (as documented)
        let expected_successors: [(u8, &[u8]); 8] = [
            (S000, &[]),                 // Fixed point
            (S001, &[S000]),             // → 000
            (S010, &[S000]),             // → 000
            (S011, &[S001, S010, S111]), // → 001, 010, 111 (three successors!)
            (S100, &[S000, S110]),       // → 000, 110 (non-deterministic)
            (S101, &[S111]),             // → 111
            (S110, &[S111]),             // → 111
            (S111, &[S110]),             // → 110
        ];

        for (state, expected) in expected_successors {
            let s = mk_state(&graph, state);
            let post = graph.post(&s);
            let expected_set = mk_states(&graph, expected);

            assert!(
                post.is_subset(&expected_set) && expected_set.is_subset(&post),
                "State {:03b} should have successors {:?}, but got different result",
                state,
                expected
            );
        }
    }

    /// Verify that weak basin states have multiple successors (nondeterminism).
    #[test]
    fn test_weak_basin_has_nondeterminism() {
        let graph = create_test_network();

        // State 011 should have 3 successors (001, 010, 111)
        let s011 = mk_state(&graph, S011);
        let post_011 = graph.post(&s011);
        assert_eq!(
            post_011.exact_cardinality(),
            3u32.into(),
            "State 011 should have exactly 3 successors"
        );

        // State 100 should have 2 successors (000, 110)
        let s100 = mk_state(&graph, S100);
        let post_100 = graph.post(&s100);
        assert_eq!(
            post_100.exact_cardinality(),
            2u32.into(),
            "State 100 should have exactly 2 successors"
        );
    }

    /// Verify the source states (no predecessors).
    #[test]
    fn test_source_states() {
        let graph = create_test_network();

        // Source states should have no predecessors
        for state in [S011, S100, S101] {
            let s = mk_state(&graph, state);
            let pre = graph.pre(&s);
            assert!(
                pre.is_empty(),
                "State {:03b} should be a source (no predecessors)",
                state
            );
        }

        // Non-source states should have predecessors
        for state in [S000, S001, S010, S110, S111] {
            let s = mk_state(&graph, state);
            let pre = graph.pre(&s);
            assert!(
                !pre.is_empty(),
                "State {:03b} should have predecessors",
                state
            );
        }
    }
}
