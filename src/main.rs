use clap::Parser;
use log::LevelFilter;
use std::fs::File;
use std::io::prelude::*;
use std::io::Write;

mod cli;
mod exclude;
mod graph;
mod model;
mod network;

#[derive(Parser, Debug)]
#[clap(author, about, version)]
struct Opts {
    // TODO: go back to long_about to fix line breaks
    /// Exclude protocol/machines/ports/services
    ///
    /// Syntax:
    ///
    ///     -e [host|IP][%dev][:port][/protocol][#process]
    ///
    /// Examples:
    ///
    ///     -e %lo                     - exclude loopback devices
    ///
    ///     -e host123                 - exclude host <host123>
    ///
    ///     -e #autossh                - exclude `autossh` processes
    ///
    ///     -e host123/udp             - exclude UDP connection to/from host <host123>
    ///
    ///     -e :53/udp                 - exclude DNS (UDP on port 53)
    ///
    ///     -e localhost%lo:22/tcp     - exclude local ssh connections
    ///
    ///     -e :22-25,:80/tcp,:8080    - exclude multiple ports
    #[clap(
        short,
        long,
        // long_about = r"Exclude protocol/machines/ports/services
// Syntax:
    // -e [host|IP][%dev][:port][/protocol][#process]
// Examples:
    // -e %lo                     - exclude loopback devices
    // -e host123                 - exclude host <host123>
    // -e #autossh                - exclude `autossh` processes
    // -e host123/udp             - exclude UDP connection to/from host <host123>
    // -e :53/udp                 - exclude DNS (UDP on port 53)
    // -e localhost%lo:22/tcp     - exclude local ssh connections
    // -e :22-25,:80/tcp,:8080    - exclude multiple ports
// "
    )]
    excludes: Option<String>,
    /// Networks to be included - using CIR format
    ///
    /// Examples:
    ///
    ///     -s 192.168.1.1/24          - privat Class C network range
    ///
    ///     -s 10.0.0.0/8              - privat Class A network range
    ///
    ///     -s fc00::/7                - address block = RFC 4193 Unique Local Addresses (ULA)
    ///
    ///     -s fc00::/7,10.0.0.0/8     - multiple networks
    #[clap(
        short,
        long,
        // long_about = r"Networks to be included - using CIR format
// Examples:
    // -s 192.168.1.1/24          - privat Class C network range
    // -s 10.0.0.0/8              - privat Class A network range
    // -s fc00::/7                - address block = RFC 4193 Unique Local Addresses (ULA)
    // -s fc00::/7,10.0.0.0/8     - multiple networks
// "
    )]
    networks: String,
    /// Verbosity level 1 (-v) up to 2 (-vv) or level 0 otherwise
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    /// Output dot file (graphviz) or stdout otherwise
    #[clap(short, long)]
    output: Option<String>,
    /// Continuous mode interval [30s .. 24h] or run once otherwise
    #[clap(short, long)]
    continue_timeout: Option<String>,
    /// List of initial hosts
    #[clap(name = "HOST")]
    hosts: Vec<String>,
    /// only load from file
    #[clap(long)]
    offline: bool,
}

fn init_logging(verbosity: i32) {
    let level = match verbosity {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };
    let config = simplelog::ConfigBuilder::new()
        .set_time_offset_to_local()
        .expect("no locale found")
        .set_thread_mode(simplelog::ThreadLogMode::Both)
        .build();
    simplelog::TermLogger::init(
        level,
        config,
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .expect("cannot create a logger");
    log::debug!("Initialized logger!");
}

fn main() {
    let opts: Opts = Opts::parse();
    init_logging(opts.verbose);
    log::debug!("CLI paramters: {:?}", opts);

    let mut model = model::Model::new();
    if opts.offline {
        let mut file = File::open("model.json").unwrap();
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).unwrap();
        model = serde_json::from_str(&buffer).unwrap();
    } else {
        for host in opts.hosts {
            model.add_machine(&host, &opts.excludes, &opts.networks);
        }
        let serialized = serde_json::to_string(&model).unwrap();
        let mut file = File::create("model.json").unwrap();
        file.write_all(serialized.as_bytes()).unwrap();
    }
    log::debug!("Model: {:?}", model);
    model.generate(&opts.output);
}
