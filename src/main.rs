mod config;
mod mirror;
mod model;
mod prune;
mod serve;
mod sync;
mod utils;

use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand, ValueHint};
use env_logger::Env;

use crate::config::Config;
use crate::prune::{PruneOpts, zom_prune};
use crate::serve::{ServeOpts, zom_serve};
use crate::sync::{SyncOpts, zom_sync};

#[derive(Subcommand)]
enum Command {
    /// Start the custom extension server.
    ///
    /// A sync must have been launched previously.
    Serve(ServeOpts),
    /// Start a synchronisation from Zed's upstream server.
    Sync(SyncOpts),
    /// Prune old versions of releases and extensions.
    Prune(PruneOpts),
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Opts {
    /// Path to zom configuration file.
    #[arg(short, long, value_hint = ValueHint::FilePath)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(Env::new().default_filter_or("info"));

    let opts = Opts::parse();

    let config = match opts.config {
        Some(config) => {
            log::info!("using configuration file, ignoring all other command line arguments");
            let config: Config = toml::from_str(&fs::read_to_string(config)?)?;
            log::debug!("configuration: {config:?}");
            Some(config)
        }
        None => None,
    };

    match opts.command {
        Command::Serve(opts) => zom_serve(config.map_or(opts, Into::into)).await,
        Command::Sync(opts) => zom_sync(config.map_or(opts, Into::into)).await,
        Command::Prune(opts) => zom_prune(config.map_or(opts, Into::into)).await,
    }
}
