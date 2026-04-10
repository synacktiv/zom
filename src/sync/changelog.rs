use anyhow::{Context, Result};
use std::{path, sync::Arc};
use tokio::{
    fs::{self, DirEntry},
    io::AsyncWriteExt,
};

use crate::utils::{SafeJoin, list_dir};

pub struct SyncChangelogOptions {
    pub upstream_url: String,
    pub release_dir: path::PathBuf,
}

async fn fetch_single_changelog(
    http_client: reqwest::Client,
    opts: Arc<SyncChangelogOptions>,
    release_directory: &DirEntry,
) -> Result<()> {
    let changelog_path = release_directory.path().safe_join("changelog.json")?;
    let release_version = release_directory.file_name();

    if changelog_path.exists() {
        log::debug!(
            "not syncing {}, already exists",
            release_version.to_string_lossy()
        );
        return Ok(());
    }

    let changelog = http_client
        .get(format!(
            "{}/api/release_notes/v2/{}",
            opts.upstream_url,
            release_version.to_string_lossy()
        ))
        .send()
        .await?
        .text()
        .await?;

    let mut changelog_file = fs::File::create(changelog_path).await?;
    changelog_file.write_all(changelog.as_bytes()).await?;

    Ok(())
}

/// Iterates over all downloaded releases and fetches its associated changelog
pub async fn sync_changelogs(opts: SyncChangelogOptions) -> Result<()> {
    let http_client = reqwest::Client::new();
    let opts = Arc::new(opts);

    let mut fetch_tasks = tokio::task::JoinSet::new();
    for release_directory in list_dir(&opts.release_dir).await? {
        fetch_tasks.spawn({
            let http_client = http_client.clone();
            let opts = Arc::clone(&opts);

            async move {
                fetch_single_changelog(http_client, opts, &release_directory)
                    .await
                    .with_context(|| release_directory.file_name().to_string_lossy().to_string())
            }
        });
    }

    let mut error_count = 0;
    while let Some(res) = fetch_tasks.join_next().await {
        if let Err(e) = res? {
            error_count += 1;
            log::error!("error downloading changelog {e}: {}", e.root_cause());
        }
    }

    log::info!("changelog sync complete ({error_count} errors)");

    Ok(())
}
