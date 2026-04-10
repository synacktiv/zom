mod extensions;
mod opts;
mod release;

use std::str;

use anyhow::Result;
use serde::Deserialize;

use crate::{
    mirror::MirrorDirectory,
    prune::{extensions::prune_extensions, release::prune_releases},
};

pub use opts::PruneOpts;

#[derive(Debug, Clone)]
pub struct ExtensionVersion {
    pub extension_id: String,
    pub version: String,
}

impl str::FromStr for ExtensionVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.split_once('=') {
            None => Err(anyhow::anyhow!(
                "Expected extension_id=version, but character `=` was not found"
            )),
            Some((extension_id, version)) => Ok(Self {
                extension_id: extension_id.to_string(),
                version: version.to_string(),
            }),
        }
    }
}

impl<'de> Deserialize<'de> for ExtensionVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        str::FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

pub async fn zom_prune(opts: PruneOpts) -> Result<()> {
    log::debug!("running prune with options {opts:?}");
    log::info!("prunning directory '{}'", opts.mirror_directory.display());

    let dir = MirrorDirectory::new(&opts.mirror_directory);
    dir.check_valid()?;

    prune_releases(
        dir.releases_dir(),
        opts.keep_latest_releases,
        opts.pin_releases,
    )
    .await?;
    prune_extensions(
        dir.extensions_dir(),
        opts.keep_latest_extensions,
        opts.pin_extensions,
    )
    .await?;

    Ok(())
}
