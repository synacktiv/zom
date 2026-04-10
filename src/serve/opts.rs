use std::{net::SocketAddrV4, path::PathBuf};

use clap::{Parser, ValueHint};
use serde::Deserialize;

#[derive(Parser, Debug, Deserialize)]
pub struct ServeOpts {
    /// Server listen address
    #[clap(
        short = 'l',
        long,
        env = "ZOM_LISTEN_ADDR",
        default_value = "0.0.0.0:8080"
    )]
    pub listen_addr: SocketAddrV4,

    /// Path to mirror directory
    #[serde(skip)] // handled in top level configuration
    #[arg(short = 'd', long, env = "ZOM_MIRROR_DIRECTORY", value_hint = ValueHint::FilePath, default_value = "dist")]
    pub mirror_directory: PathBuf,

    /// URL of the root of the server.
    #[arg(long, env = "ZOM_BASE_URL", value_hint = ValueHint::Url)]
    pub base_url: Option<String>,
}
