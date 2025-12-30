use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReachabilityState {
    pub iteration: usize,
    pub set: GraphColoredVertices,
}

impl From<GraphColoredVertices> for ReachabilityState {
    fn from(value: GraphColoredVertices) -> Self {
        ReachabilityState {
            iteration: 0,
            set: value,
        }
    }
}

impl From<&GraphColoredVertices> for ReachabilityState {
    fn from(value: &GraphColoredVertices) -> Self {
        Self::from(value.clone())
    }
}

impl From<ReachabilityState> for GraphColoredVertices {
    fn from(value: ReachabilityState) -> Self {
        value.set
    }
}
