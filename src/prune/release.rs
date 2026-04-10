use std::path;

use anyhow::Result;

use crate::utils::list_dir_sorted_by_semver;

pub async fn prune_releases(
    releases_dir: &path::Path,
    keep_latest: u64,
    pin: Vec<String>,
) -> Result<()> {
    let versions = list_dir_sorted_by_semver(releases_dir).await?;
    let keep_latest = usize::try_from(keep_latest)?;

    log::info!("{} versions currently downloaded.", versions.len());
    if versions.len() <= keep_latest {
        log::info!("nothing to prune");
        return Ok(());
    }

    let mut success_count = 0;
    let mut error_count = 0;
    for version in versions.into_iter().skip(keep_latest) {
        let version_filename = version.file_name();
        let Some(version_str) = version_filename.to_str() else {
            log::warn!("unknown version {}", version_filename.display());
            continue;
        };

        if pin.iter().any(|s| s == version_str) {
            log::info!("skipping pinned version {version_str}");
            continue;
        }

        match tokio::fs::remove_dir_all(version.path()).await {
            Ok(()) => {
                log::info!("removed version {version_str}");
                success_count += 1;
            }
            Err(e) => {
                log::error!("could not remove version {version_str}: {e}");
                error_count += 1;
            }
        }
    }

    log::info!("removed {success_count} releases ({error_count} errors)");

    Ok(())
}
