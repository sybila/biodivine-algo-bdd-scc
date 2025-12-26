mod fwd_bwd;

use crate::algorithm_trait_2::Generator;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
pub use fwd_bwd::{FwdBwdIterationState, FwdBwdState, FwdBwdStep};

///
pub type FwdBwdScc = Generator<SymbolicAsyncGraph, FwdBwdState, GraphColoredVertices, FwdBwdStep>;

#[cfg(test)]
mod tests {
    use crate::algorithm_2::scc::FwdBwdScc;
    use crate::algorithm_2::test_utils::init_logger;
    use crate::algorithm_2::test_utils::llm_example_network::sets::ATTRACTOR_2;
    use crate::algorithm_2::test_utils::llm_example_network::{create_test_network, mk_states};

    #[test]
    fn simple_scc() {
        init_logger();
        let graph = create_test_network();
        let mut generator = FwdBwdScc::configure(graph.clone(), &graph);
        let scc = generator.next().unwrap().unwrap();

        // The first SCC is the oscillating attractor:
        let attractor_2 = mk_states(&graph, ATTRACTOR_2);
        assert_eq!(attractor_2, scc);

        // There is only one SCC
        assert!(generator.next().is_none());
    }
}
