use std::path;
use std::str;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::fs;

use crate::model::Asset;
use crate::utils::SafeJoin;
use crate::utils::download_to_file;

pub struct SyncReleaseOptions {
    pub upstream_url: String,
    pub release_dir: path::PathBuf,
}

pub async fn sync_release(assets: Vec<Asset>, opts: SyncReleaseOptions) -> Result<()> {
    let http_client = reqwest::Client::new();
    let opts = Arc::new(opts);

    let mut fetch_tasks = tokio::task::JoinSet::new();
    for asset in assets {
        fetch_tasks.spawn({
            let http_client = http_client.clone();
            let opts = Arc::clone(&opts);
            let asset_string = asset.to_string();
            async move {
                fetch_single_asset(asset, http_client, opts)
                    .await
                    .with_context(|| asset_string)
            }
        });
    }

    let mut error_count = 0;
    while let Some(res) = fetch_tasks.join_next().await {
        if let Err(e) = res? {
            error_count += 1;
            log::error!("error downloading {e}: {}", e.root_cause());
        }
    }

    log::info!("release sync complete ({error_count} errors)");

    Ok(())
}

#[derive(Deserialize)]
struct ReleaseAssetResponse {
    version: String,
    url: String,
}

async fn fetch_single_asset(
    asset: Asset,
    http_client: reqwest::Client,
    opts: Arc<SyncReleaseOptions>,
) -> Result<Asset> {
    let res = http_client
        .get(format!(
            "{}/releases/stable/latest/asset",
            opts.upstream_url
        ))
        .query(&[
            ("asset", &asset.name),
            ("os", &asset.os),
            ("arch", &asset.arch),
        ])
        .send()
        .await?
        .error_for_status()?;
    let raw_json_body = res.bytes().await?;

    let manifest: ReleaseAssetResponse = serde_json::from_slice(&raw_json_body)?;

    let version_dir = opts.release_dir.safe_join(&manifest.version)?;
    fs::create_dir_all(&version_dir).await?;

    let target_file = version_dir.safe_join(asset.filename())?;
    if target_file.exists() {
        log::debug!("skipping {asset}: already downloaded");
        return Ok(asset);
    }
    log::info!("downloading {asset} version {}", manifest.version);

    download_to_file(http_client.get(manifest.url), target_file).await?;

    log::info!("download complete {asset} version {}", manifest.version);
    Ok::<Asset, anyhow::Error>(asset)
}
