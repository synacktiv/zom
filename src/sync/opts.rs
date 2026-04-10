use std::path::PathBuf;

use clap::{Parser, ValueHint};
use serde::Deserialize;

use crate::model::{Asset, DEFAULT_ASSETS};

#[derive(Parser, Debug, Deserialize)]
pub struct SyncOpts {
    /// Sync folder
    #[serde(skip)] // handled in top level configuration
    #[arg(short = 'd', long, env = "ZOM_MIRROR_DIRECTORY", value_hint = ValueHint::FilePath, default_value = "dist")]
    pub mirror_directory: PathBuf,

    // TODO: document why would someone change this
    /// Minimum extension schema version.
    #[arg(long, default_value_t = 0)]
    pub min_schema_version: u16,

    /// Maximum extension schema version.
    ///
    /// As of 2026-01-20, the maximum is 2. If unsure and some extensions are
    /// missing, use a large value.
    #[arg(long, default_value_t = 2)]
    pub max_schema_version: u16,

    // TODO: document why would someone change this
    /// Minimum supported version for the WASM api.
    #[arg(long, default_value = "0.0.0")]
    pub min_wasm_api_version: String,

    // TODO: document why would someone change this
    /// Maximum supported version for the WASM api.
    ///
    /// As of 2026-01-20, max version used in extensions is 0.7.0.
    #[arg(long, default_value = "0.7.0")]
    pub max_wasm_api_version: String,

    /// URL to an existing zed api server.
    ///
    /// This is where releases are downloaded.
    #[arg(long, env = "ZOM_UPSTREAM_ZED_CLOUD", value_hint = ValueHint::Url, default_value = "https://cloud.zed.dev")]
    pub upstream_cloud_url: String,

    /// URL to an existing zed api server.
    ///
    /// This is where extensions are downloaded.
    #[arg(long, env = "ZOM_UPSTREAM_ZED_API", value_hint = ValueHint::Url, default_value = "https://api.zed.dev")]
    pub upstream_api_url: String,

    /// URL to an existing zed server.
    ///
    /// This is where static resources are downloaded.
    #[arg(long, env = "ZOM_UPSTREAM_ZED", value_hint = ValueHint::Url, default_value = "https://zed.dev")]
    pub upstream_zed_url: String,

    /// Which assets should be synced for zed releases.
    ///
    /// By default, all zed releases and zed-remote-server are downloaded.
    /// Assets are given in form <name>-<os>-<arch>
    #[arg(short, long, default_values = DEFAULT_ASSETS)]
    pub assets: Vec<Asset>,
    // TODO: force (re)download extension/asset
}
