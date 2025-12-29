[![Crates.io](https://img.shields.io/crates/v/biodivine-algo-bdd-scc?style=flat-square)](https://crates.io/crates/biodivine-algo-bdd-scc)
[![Api Docs](https://img.shields.io/badge/docs-api-yellowgreen?style=flat-square)](https://docs.rs/biodivine-algo-bdd-scc/)
[![Continuous integration](https://img.shields.io/github/actions/workflow/status/sybila/biodivine-algo-bdd-scc/build.yml?branch=main&style=flat-square)](https://github.com/sybila/biodivine-algo-bdd-scc/actions/workflows/build.yml)
[![Coverage](https://img.shields.io/codecov/c/github/sybila/biodivine-algo-bdd-scc?style=flat-square)](https://codecov.io/gh/sybila/biodivine-algo-bdd-scc)
[![GitHub issues](https://img.shields.io/github/issues/sybila/biodivine-algo-bdd-scc?style=flat-square)](https://github.com/sybila/biodivine-algo-bdd-scc/issues)
[![GitHub last commit](https://img.shields.io/github/last-commit/sybila/biodivine-algo-bdd-scc?style=flat-square)](https://github.com/sybila/biodivine-algo-bdd-scc/commits/main)
[![Crates.io](https://img.shields.io/crates/l/biodivine-algo-bdd-scc?style=flat-square)](https://github.com/sybila/biodivine-algo-bdd-scc/blob/main/LICENSE)

# Biodivine BDD-based SCC detection algorithms

BDD-based algorithms for symbolic strongly connected component (SCC) detection 
in Boolean networks.

This crate provides efficient symbolic algorithms for computing SCCs and 
reachability in asynchronous Boolean networks, using Binary Decision Diagrams 
(BDDs) as the underlying representation. It builds on top of 
[`biodivine-lib-param-bn`](https://crates.io/crates/biodivine-lib-param-bn) for 
Boolean network modeling and symbolic graph operations.

## Features

- **Symbolic SCC detection**: Find all non-trivial strongly connected components
  using forward-backward or chain-based algorithms
- **Reachability analysis**: Compute forward and backward reachable sets using
  BFS or saturation-based strategies  
- **Trimming**: Remove trivial sink/source states before SCC computation
- **Long-lived filtering**: Optionally filter SCCs that can be escaped via a
  single variable update
- **Cancellable computations**: All algorithms support cooperative cancellation
  via the `cancel-this` crate

## Quick Start

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
biodivine-algo-bdd-scc = "0.1"
biodivine-lib-param-bn = "0.6"
```

### Basic SCC Enumeration

```rust
use biodivine_algo_bdd_scc::scc::{FwdBwdScc, SccConfig};
use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use computation_process::Stateful;

fn example() {
    // Load a Boolean network from a file
    let bn = BooleanNetwork::try_from_file("model.aeon").unwrap();
    let graph = SymbolicAsyncGraph::new(&bn).unwrap();
  
    // Create an SCC configuration with default settings
    let config = SccConfig::new(graph.clone());
  
    // Enumerate all non-trivial SCCs
    for scc in FwdBwdScc::configure(config, &graph) {
      let scc = scc.unwrap();
      println!("Found SCC with {} states", scc.exact_cardinality());
    } 
}
```

### Reachability Analysis

```rust
use biodivine_algo_bdd_scc::reachability::ForwardReachability;
use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::symbolic_async_graph::SymbolicAsyncGraph;
use computation_process::Algorithm;

fn example() {
    let bn = BooleanNetwork::try_from_file("model.aeon").unwrap();
    let graph = SymbolicAsyncGraph::new(&bn).unwrap();
  
    // Compute all states reachable from a given initial set
    let initial = graph.mk_unit_colored_vertices().pick_vertex();
    let reachable = ForwardReachability::run(&graph, initial).unwrap();
  
    println!("Reachable states: {}", reachable.exact_cardinality()); 
}
```

## Available Algorithms

### SCC Detection

| Algorithm      | Description                                                      |
|----------------|------------------------------------------------------------------|
| `FwdBwdScc`    | Classic forward-backward algorithm using saturation reachability |
| `FwdBwdSccBfs` | Forward-backward with BFS reachability (useful for benchmarks)   |
| `ChainScc`     | Chain-based algorithm; can handle some larger networks           |

### Reachability

| Algorithm                 | Description                            |
|---------------------------|----------------------------------------|
| `ForwardReachability`     | Forward reachability using saturation  |
| `BackwardReachability`    | Backward reachability using saturation |
| `ForwardReachabilityBfs`  | Forward reachability using BFS         |
| `BackwardReachabilityBfs` | Backward reachability using BFS        |

### Trimming

| Algorithm             | Description                        |
|-----------------------|------------------------------------|
| `TrimSinks`           | Remove states with no successors   |
| `TrimSources`         | Remove states with no predecessors |
| `TrimSinksAndSources` | Remove both sinks and sources      |

## Command-Line Tool

The crate includes a binary for SCC enumeration. Build it with:

```bash
cargo build --release --features build-binary
```

Usage:

```bash
# Enumerate all SCCs using the default forward-backward algorithm
./target/release/biodivine_scc model.aeon

# Use the chain algorithm with verbose logging
./target/release/biodivine_scc model.aeon --algorithm=chain -v

# Enumerate only long-lived SCCs (cannot be escaped by one variable update)
./target/release/biodivine_scc model.aeon --long-lived

# Enumerate only the first 5 SCCs
./target/release/biodivine_scc model.aeon --count=5
```

## Testing

Run the test suite:

```bash
# Basic tests (can take a few minutes)
cargo test
```

## License

This project is licensed under the MIT License.

## References

The general idea of SCC decomposition for colored graphs (i.e., with logical parameters) was presented here:

> Beneš, Nikola, Luboš Brim, Samuel Pastva, and David Šafránek. "Symbolic coloured SCC decomposition." In 
> International Conference on Tools and Algorithms for the Construction and Analysis of Systems, pp. 64-83. 
> Cham: Springer International Publishing, 2021.
> [DOI](https://doi.org/10.1007/978-3-030-72013-1_4)

The chain algorithm is also loosely based on:

> Larsen, Casper Abild, Simon Meldahl Schmidt, Jesper Steensgaard, Anna Blume Jakobsen, Jaco van de Pol, 
> and Andreas Pavlogiannis. "A truly symbolic linear-time algorithm for SCC decomposition." In International 
> Conference on Tools and Algorithms for the Construction and Analysis of Systems, pp. 353-371. Cham: Springer 
> Nature Switzerland, 2023.
> [DOI](https://doi.org/10.1007/978-3-031-30820-8_22)