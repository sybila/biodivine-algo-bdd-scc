use crate::algorithm::log_set;
use crate::algorithm::reachability::{BackwardReachability, ForwardReachability};
use crate::algorithm::trimming::TrimSinksAndSources;
use biodivine_lib_param_bn::biodivine_std::traits::Set;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use num_bigint::BigUint;

pub fn slice(graph: &SymbolicAsyncGraph, set: GraphColoredVertices) -> Vec<GraphColoredVertices> {
    let mut result = vec![set];
    let threshold = BigUint::from(1u32 << 30);
    for var in graph.variables().rev() {
        let mut sliced_result = Vec::new();
        let len = result.len();
        for (i, mut set) in result.into_iter().enumerate() {
            println!("[{}][{}/{}]", var, i + 1, len);
            if set.exact_cardinality() < threshold {
                if set.is_empty() {
                    continue;
                }
                sliced_result.push(set);
                continue;
            }

            loop {
                let var_can_post = graph.var_can_post(var, &set);
                let var_can_pre = graph.var_can_pre(var, &set);

                let backward =
                    BackwardReachability::run(graph.restrict(&set), var_can_post.clone()).unwrap();

                let orphans = var_can_pre.minus(&backward);

                if orphans.is_empty() {
                    println!("[{}] Stopping forward: {}", var, log_set(&set));
                    break;
                } else {
                    let forward =
                        ForwardReachability::run(graph.restrict(&set), orphans.clone()).unwrap();
                    set = set.minus(&forward);
                    println!("[{}] Forward: {:?}", var, log_set(&forward));
                    if forward.exact_cardinality() < threshold {
                        let forward = TrimSinksAndSources::run(graph, forward).unwrap();
                        println!(
                            "[{}] Isolated and trimmed {} | remaining {}",
                            var,
                            log_set(&forward),
                            log_set(&set)
                        );
                        if !forward.is_empty() {
                            sliced_result.push(forward);
                        }
                    } else {
                        println!(
                            "[{}] Isolated {} | remaining {}",
                            var,
                            log_set(&forward),
                            log_set(&set)
                        );
                        sliced_result.push(forward);
                    }
                }
            }

            loop {
                let var_can_post = graph.var_can_post(var, &set);
                let var_can_pre = graph.var_can_pre(var, &set);

                let forward =
                    ForwardReachability::run(graph.restrict(&set), var_can_pre.clone()).unwrap();

                let orphans = var_can_post.minus(&forward);

                if orphans.is_empty() {
                    println!("[{}] Remaining: {}", var, log_set(&set));
                    sliced_result.push(set);
                    break;
                } else {
                    let backward =
                        BackwardReachability::run(graph.restrict(&set), orphans.clone()).unwrap();
                    set = set.minus(&backward);
                    println!("[{}] Backward: {:?}", var, log_set(&forward));
                    if backward.exact_cardinality() < threshold {
                        let backward = TrimSinksAndSources::run(graph, backward).unwrap();
                        println!(
                            "[{}] Isolated and trimmed {} | remaining {}",
                            var,
                            log_set(&backward),
                            log_set(&set)
                        );
                        if !backward.is_empty() {
                            sliced_result.push(backward);
                        }
                    } else {
                        println!(
                            "[{}] Isolated {} | remaining {}",
                            var,
                            log_set(&backward),
                            log_set(&set)
                        );
                        sliced_result.push(backward);
                    }
                }
            }
        }

        result = sliced_result;
        println!("{} sets so far", result.len());
    }

    result
}
