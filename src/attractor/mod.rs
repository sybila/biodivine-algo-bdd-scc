mod attractor_config;
mod xie_beerel;

#[cfg(test)]
mod tests;

pub use attractor_config::AttractorConfig;
use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;
use computation_process::Computation;
pub use xie_beerel::{XieBeerelState, XieBeerelStep};

pub type XieBeerelAttractors =
    Computation<AttractorConfig, XieBeerelState, GraphColoredVertices, XieBeerelStep>;
