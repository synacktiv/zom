use std::path::PathBuf;

use clap::{Parser, ValueHint};
use serde::Deserialize;

use crate::prune::ExtensionVersion;

#[derive(Parser, Debug, Deserialize)]
pub struct PruneOpts {
    /// Path to mirror directory
    #[serde(skip)] // handled in top level configuration
    #[arg(short = 'd', long, env = "ZOM_MIRROR_DIRECTORY", value_hint = ValueHint::FilePath, default_value = "dist")]
    pub mirror_directory: PathBuf,

    /// Number of latest release versions to keep.
    #[arg(long, default_value_t = 1)]
    pub keep_latest_releases: u64,

    /// Number of latest extension versions to keep.
    #[arg(long, default_value_t = 1)]
    pub keep_latest_extensions: u64,

    /// A list of release versions that will not be removed.
    #[arg(long)]
    pub pin_releases: Vec<String>,

    /// A list of extensions and versions that will not be removed.
    ///
    /// Format: <extension>=<version>
    #[arg(long)]
    pub pin_extensions: Vec<ExtensionVersion>,
}
