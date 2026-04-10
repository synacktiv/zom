use std::collections::HashMap;
use std::path;

use anyhow::Result;

use crate::{
    prune::ExtensionVersion,
    utils::{list_dir, list_dir_sorted_by_semver},
};

pub async fn prune_extensions(
    extensions_dir: &path::Path,
    keep_latest: u64,
    pin: Vec<ExtensionVersion>,
) -> Result<()> {
    let keep_latest = usize::try_from(keep_latest)?;
    let mut success_count = 0;
    let mut error_count = 0;

    let pin: HashMap<String, Vec<String>> = pin.into_iter().fold(HashMap::new(), |mut acc, p| {
        acc.entry(p.extension_id).or_default().push(p.version);
        acc
    });

    log::info!("pruning extensions");
    for extension in list_dir(extensions_dir).await? {
        let extension_filename = extension.file_name();
        let Some(extension_str) = extension_filename.to_str() else {
            log::warn!("unknown extension {}", extension_filename.display());
            continue;
        };
        let versions = list_dir_sorted_by_semver(extension.path()).await?;
        if versions.len() <= keep_latest {
            continue;
        }
        let pinned_version = pin.get(extension_str);
        for version in versions.into_iter().skip(keep_latest) {
            let version_filename = version.file_name();
            let Some(version_str) = version_filename.to_str() else {
                log::warn!(
                    "({}) unknown version {}",
                    extension_str,
                    version_filename.display()
                );
                continue;
            };

            if let Some(pinned) = pinned_version
                && pinned.iter().any(|s| s == version_str)
            {
                log::info!("({extension_str}) skipping pinned version {version_str}");
                continue;
            }

            match tokio::fs::remove_dir_all(version.path()).await {
                Ok(()) => {
                    log::debug!("({extension_str}) removed version {version_str}",);
                    success_count += 1;
                }
                Err(e) => {
                    log::error!("({extension_str}) could not remove version {version_str}: {e}",);
                    error_count += 1;
                }
            }
        }

        if list_dir(extension.path()).await?.is_empty() {
            tokio::fs::remove_dir(extension.path()).await?;
        }
    }

    log::info!("removed {success_count} extensions ({error_count} errors)");

    Ok(())
}
