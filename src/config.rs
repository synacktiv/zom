use std::path;

use serde::Deserialize;

use crate::{prune::PruneOpts, serve::ServeOpts, sync::SyncOpts};

#[derive(Debug, Deserialize)]
pub struct Config {
    directory: path::PathBuf,
    serve: ServeOpts,
    sync: SyncOpts,
    prune: PruneOpts,
}

impl From<Config> for SyncOpts {
    fn from(val: Config) -> Self {
        let mut opts = val.sync;
        opts.mirror_directory = val.directory;
        opts
    }
}

impl From<Config> for ServeOpts {
    fn from(val: Config) -> Self {
        let mut opts = val.serve;
        opts.mirror_directory = val.directory;
        opts
    }
}

impl From<Config> for PruneOpts {
    fn from(val: Config) -> Self {
        let mut opts = val.prune;
        opts.mirror_directory = val.directory;
        opts
    }
}
