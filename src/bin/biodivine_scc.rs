use biodivine_algo_bdd_scc::scc::{ChainScc, FwdBwdScc, FwdBwdSccBfs, SccConfig};
use biodivine_algo_bdd_scc::trimming::TrimSetting;
use biodivine_lib_param_bn::BooleanNetwork;
use biodivine_lib_param_bn::symbolic_async_graph::{GraphColoredVertices, SymbolicAsyncGraph};
use cancel_this::Cancellable;
use clap::Parser;
use computation_process::Stateful;
use env_logger::Builder;
use log::LevelFilter;

#[derive(Parser)]
#[command(name = "biodivine_scc")]
#[command(about = "Enumerate strongly connected components in a Boolean network")]
struct Args {
    /// Path to a Boolean network file (.aeon, .bnet, etc.)
    #[arg(value_name = "FILE")]
    file: String,

    /// SCC detection algorithm
    #[arg(long, default_value = "fwd-bwd", require_equals = true)]
    algorithm: Algorithm,

    /// Pre-processing trimming strategy
    #[arg(long, default_value = "both", require_equals = true)]
    trim: TrimConfig,

    /// Number of SCCs to enumerate (0 = all)
    #[arg(long, default_value_t = 0, require_equals = true)]
    count: usize,

    /// Only report SCCs that cannot be escaped by a single variable update
    #[arg(long)]
    long_lived: bool,

    /// Disable constant propagation before analysis (may increase network size but preserves all SCCs)
    #[arg(long)]
    no_constant_propagation: bool,

    /// Logging verbosity (use -v for info, or -v=LEVEL for specific level)
    #[arg(long, short = 'v', value_name = "LEVEL", num_args = 0..=1, default_missing_value = "info", require_equals = true)]
    verbose: Option<Option<LogLevel>>,
}

#[derive(Clone, clap::ValueEnum)]
enum Algorithm {
    #[value(name = "fwd-bwd")]
    FwdBwd,
    #[value(name = "fwd-bwd-bfs")]
    FwdBwdBfs,
    #[value(name = "chain")]
    Chain,
}

#[derive(Clone, clap::ValueEnum)]
enum LogLevel {
    Trace,
    Debug,
    Info,
}

#[derive(Clone, clap::ValueEnum)]
enum TrimConfig {
    Both,
    Sources,
    Sinks,
    None,
}

impl From<TrimConfig> for TrimSetting {
    fn from(value: TrimConfig) -> Self {
        match value {
            TrimConfig::Both => TrimSetting::Both,
            TrimConfig::Sources => TrimSetting::Sources,
            TrimConfig::Sinks => TrimSetting::Sinks,
            TrimConfig::None => TrimSetting::None,
        }
    }
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

    // Note that constant inlining will not preserve all SCCs, but it is a relatively
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

    // Create SCC config
    let mut config = SccConfig::new(graph.clone());
    config.should_trim = args.trim.into();
    config.filter_long_lived = args.long_lived;

    // Helper function to enumerate SCCs from a generator
    fn enumerate_sccs<G>(generator: G, count: usize) -> usize
    where
        G: Iterator<Item = Cancellable<GraphColoredVertices>>,
    {
        let mut enumerated = 0;

        for result in generator {
            match result {
                Ok(scc) => {
                    // Only enumerate non-trivial SCCs (the generator already filters these)
                    if count == 0 || enumerated < count {
                        let cardinality = scc.exact_cardinality();
                        println!("SCC #{}: {} elements", enumerated + 1, cardinality);
                        enumerated += 1;
                    }

                    // Stop if we've reached the count limit
                    if count > 0 && enumerated >= count {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error during SCC computation: {}", e);
                    break;
                }
            }
        }

        enumerated
    }

    // Select algorithm and enumerate SCCs
    let enumerated = match args.algorithm {
        Algorithm::FwdBwd => enumerate_sccs(FwdBwdScc::configure(config, &graph), args.count),
        Algorithm::FwdBwdBfs => enumerate_sccs(FwdBwdSccBfs::configure(config, &graph), args.count),
        Algorithm::Chain => enumerate_sccs(ChainScc::configure(config, &graph), args.count),
    };

    if args.count == 0 {
        println!("Total SCCs enumerated: {}", enumerated);
    } else {
        println!("Enumerated first {} SCCs", enumerated);
    }
}
