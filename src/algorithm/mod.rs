use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;

#[cfg(test)]
mod test_utils;

pub mod reachability;
pub mod scc;

/// A utility method for printing useful metadata of symbolic sets.
fn log_set(set: &GraphColoredVertices) -> String {
    format!(
        "elements={}; BDD nodes={}",
        set.exact_cardinality(),
        set.symbolic_size()
    )
}

fn simple_type_name<T>() -> &'static str {
    std::any::type_name::<T>().split("::").last().unwrap_or("?")
}
