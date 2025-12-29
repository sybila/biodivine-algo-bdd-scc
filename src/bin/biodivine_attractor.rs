use biodivine_algo_bdd_scc::attractor::{AttractorConfig, XieBeerelState, XieBeerelStep};
use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use clap::Parser;
use computation_process::{Generator, Stateful};
use env_logger::Builder;
use log::LevelFilter;

#[derive(Parser)]
#[command(name = "biodivine_attractor")]
#[command(about = "Enumerate attractors in a Boolean network")]
struct Args {
    /// Path to a Boolean network file (.aeon, .bnet, etc.)
    #[arg(value_name = "FILE")]
    file: String,

    /// Attractor detection algorithm
    #[arg(long, default_value = "xie-beerel", require_equals = true)]
    algorithm: Algorithm,

    /// Number of attractors to enumerate (0 = all)
    #[arg(long, default_value_t = 0, require_equals = true)]
    count: usize,

    /// Disable constant propagation before analysis (may increase network size but preserves all attractors)
    #[arg(long)]
    no_constant_propagation: bool,

    /// Logging verbosity (use -v for info, or -v=LEVEL for a specific level)
    #[arg(long, short = 'v', value_name = "LEVEL", num_args = 0..=1, default_missing_value = "info", require_equals = true)]
    verbose: Option<Option<LogLevel>>,
}

#[derive(Clone, clap::ValueEnum)]
enum Algorithm {
    #[value(name = "xie-beerel")]
    XieBeerel,
}

#[derive(Clone, clap::ValueEnum)]
enum LogLevel {
    Trace,
    Debug,
    Info,
}

impl From<LogLevel> for LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
        }
    }
}

/// Wrap XieBeerelAttractors in a Generator to enable iteration
type XieBeerelAttractorGenerator =
    Generator<AttractorConfig, XieBeerelState, GraphColoredVertices, XieBeerelStep>;

fn main() {
    let args = Args::parse();

    // Configure logging:
    // Handle verbose flag: None = not specified, Some(None) = specified without value (defaults to info), Some(Some(level)) = specified with value
    let log_level = match args.verbose {
        None => LevelFilter::Off,
        Some(None) => LevelFilter::Info, // --verbose or -v without value
        Some(Some(level)) => level.into(), // --verbose=level or -v level
    };
    Builder::from_default_env().filter_level(log_level).init();

    // Load BN file
    let bn = BooleanNetwork::try_from_file(&args.file).unwrap_or_else(|e| {
        eprintln!("Failed to load BN file {}: {}", args.file, e);
        std::process::exit(1);
    });

    println!("Loaded BN with {} variables.", bn.num_vars());

    // Note that constant inlining will not preserve all attractors, but it is a relatively
    // "fair" way of reducing the problem size for benchmarking.
    let bn = if !args.no_constant_propagation {
        let bn = bn.inline_constants(true, true);
        println!(
            "After constant propagation, BN has {} variables.",
            bn.num_vars()
        );
        bn
    } else {
        bn
    };

    if bn.num_vars() == 0 {
        println!("Network is fully determined by constants.");
        return;
    }

    let graph = SymbolicAsyncGraph::new(&bn).unwrap_or_else(|e| {
        eprintln!("Failed to create symbolic async graph: {}", e);
        std::process::exit(1);
    });

    // Select algorithm and enumerate attractors
    let enumerated = match args.algorithm {
        Algorithm::XieBeerel => {
            let config = AttractorConfig::new(graph.clone());
            let initial_state = XieBeerelState::from(&graph);
            let generator = XieBeerelAttractorGenerator::configure(config, initial_state);
            let mut enumerated = 0;

            for result in generator {
                match result {
                    Ok(attractor) => {
                        if args.count == 0 || enumerated < args.count {
                            let cardinality = attractor.exact_cardinality();
                            println!("Attractor #{}: {} elements", enumerated + 1, cardinality);
                            enumerated += 1;
                        }

                        // Stop if we've reached the count limit
                        if args.count > 0 && enumerated >= args.count {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error during attractor computation: {}", e);
                        break;
                    }
                }
            }

            enumerated
        }
    };

    if args.count == 0 {
        println!("Total attractors enumerated: {}", enumerated);
    } else {
        println!("Enumerated first {} attractors", enumerated);
    }
}
