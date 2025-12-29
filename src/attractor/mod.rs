mod attractor_config;
mod itgr;
mod xie_beerel;

#[cfg(test)]
mod tests;

pub use attractor_config::AttractorConfig;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use computation_process::{Computation, Generator};
pub use itgr::{ItgrState, ItgrStep};
pub use xie_beerel::{XieBeerelState, XieBeerelStep};

pub type XieBeerelAttractors =
    Generator<AttractorConfig, XieBeerelState, GraphColoredVertices, XieBeerelStep>;

pub type InterleavedTransitionGuidedReduction =
    Computation<AttractorConfig, ItgrState, GraphColoredVertices, ItgrStep>;
