use biodivine_lib_param_bn::symbolic_async_graph::GraphColoredVertices;

#[cfg(test)]
mod test_utils;

pub mod reachability;
pub mod scc;
pub mod trimming;

/// A utility method for printing useful metadata of symbolic sets.
fn log_set(set: &GraphColoredVertices) -> String {
    format!(
        "elements={}; BDD nodes={}",
        set.exact_cardinality(),
        set.symbolic_size()
    )
}

/// Extract the "simple name" of a type argument at compile time.
///
/// In the future, this should be a `const fn`, but `type_name` and `unwrap_or` are not
/// yet stabilized as `const` functions (even thought they probably are).
fn simple_type_name<T>() -> &'static str {
    std::any::type_name::<T>().split("::").last().unwrap_or("?")
}
