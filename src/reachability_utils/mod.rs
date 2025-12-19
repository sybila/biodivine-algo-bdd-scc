#[cfg(test)]
mod test_llm_algorithm_naive;
#[cfg(test)]
mod test_llm_example_network;
mod algorithm_naive;
mod algorithm_saturation;

/// An "algorithm struct" which implements various symbolic reachability methods.
pub struct Reachability;
