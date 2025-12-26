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

use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;

use super::llm_transition_builder::from_transitions;

/// Creates the canonical test network as a `SymbolicAsyncGraph`.
///
/// See the module documentation for a complete description of the network structure.
///
/// The network is automatically generated from the list of transitions defined below.
pub fn create_test_network() -> SymbolicAsyncGraph {
    // Define the transitions as documented in the module comments
    // States are encoded as binary: x0*4 + x1*2 + x2*1
    let transitions = vec![
        (0b001, 0b000), // x2 updates: f_x2(001) = 0 ≠ 1
        (0b010, 0b000), // x1 updates: f_x1(010) = 0 ≠ 1
        (0b011, 0b001), // x1 updates: f_x1(011) = 0 ≠ 1
        (0b011, 0b010), // x2 updates: f_x2(011) = 0 ≠ 1
        (0b011, 0b111), // x0 updates: f_x0(011) = 1 ≠ 0
        (0b100, 0b000), // x0 updates: f_x0(100) = 0 ≠ 1
        (0b100, 0b110), // x1 updates: f_x1(100) = 1 ≠ 0
        (0b101, 0b111), // x1 updates: f_x1(101) = 1 ≠ 0
        (0b110, 0b111), // x2 updates: f_x2(110) = 1 ≠ 0
        (0b111, 0b110), // x2 updates: f_x2(111) = 0 ≠ 1
    ];

    let bn = from_transitions(3, &transitions).expect("Failed to create network from transitions");
    SymbolicAsyncGraph::new(&bn).expect("Failed to create symbolic graph")
}

/// State constants for readability in tests.
pub mod states {
    /// State `000` - Fixed point, Attractor 1.
    pub const S000: u32 = 0b000;
    /// State `001` - Strong basin of Attractor 1.
    pub const S001: u32 = 0b001;
    /// State `010` - Strong basin of Attractor 1.
    pub const S010: u32 = 0b010;
    /// State `011` - Weak basin (can reach both attractors), SOURCE (no predecessors).
    pub const S011: u32 = 0b011;
    /// State `100` - Weak basin (can reach both attractors), SOURCE (no predecessors).
    pub const S100: u32 = 0b100;
    /// State `101` - Strong basin of Attractor 2, SOURCE (no predecessors).
    pub const S101: u32 = 0b101;
    /// State `110` - Part of Attractor 2 (non-trivial SCC).
    pub const S110: u32 = 0b110;
    /// State `111` - Part of Attractor 2 (non-trivial SCC).
    pub const S111: u32 = 0b111;
}

/// Predefined sets for common test scenarios.
pub mod sets {
    use super::states::*;

    /// Attractor 1: the fixed point `{000}`.
    pub const ATTRACTOR_1: &[u32] = &[S000];

    /// Attractor 2: the cycle `{110, 111}`.
    pub const ATTRACTOR_2: &[u32] = &[S110, S111];

    /// Strong basin of Attractor 1 (excluding the attractor itself): `{001, 010}`.
    pub const STRONG_BASIN_ATTR1: &[u32] = &[S001, S010];

    /// Strong basin of Attractor 2 (excluding the attractor itself): `{101}`.
    pub const STRONG_BASIN_ATTR2: &[u32] = &[S101];

    /// Weak basin (can reach both attractors): `{011, 100}`.
    pub const WEAK_BASIN: &[u32] = &[S011, S100];

    /// Source states (no predecessors): `{011, 100, 101}`.
    pub const SOURCE_STATES: &[u32] = &[S011, S100, S101];

    /// All states that can reach Attractor 1: `{000, 001, 010, 011, 100}`.
    pub const CAN_REACH_ATTR1: &[u32] = &[S000, S001, S010, S011, S100];

    /// All states that can reach Attractor 2: `{011, 100, 101, 110, 111}`.
    pub const CAN_REACH_ATTR2: &[u32] = &[S011, S100, S101, S110, S111];

    /// All 8 states in the network.
    pub const ALL_STATES: &[u32] = &[S000, S001, S010, S011, S100, S101, S110, S111];
}

#[cfg(test)]
mod tests {
    use super::states::*;
    use super::*;
    use crate::algorithm_2::test_utils::{mk_state, mk_states};

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
        assert_eq!(
            post_110, s111,
            "State 110 should have exactly successor 111"
        );

        // 111 should have exactly one successor: 110
        let post_111 = graph.post(&s111);
        assert_eq!(
            post_111, s110,
            "State 111 should have exactly successor 110"
        );
    }

    /// Verify the complete edge structure of the transition graph.
    #[test]
    fn test_complete_transition_structure() {
        let graph = create_test_network();

        // Expected successors for each state (as documented)
        let expected_successors: [(u32, &[u32]); 8] = [
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

            assert_eq!(
                post, expected_set,
                "State {:03b} should have successors {:?}, but got different result",
                state, expected
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
