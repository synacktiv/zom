use std::path;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::fs;

use crate::model::ExtensionManifest;
use crate::utils::{SafeJoin, download_to_file};

pub struct SyncExtensionsOptions {
    pub upstream_url: String,
    pub extension_dir: path::PathBuf,
    pub min_schema_version: u16,
    pub max_schema_version: u16,
    pub min_wasm_api_version: String,
    pub max_wasm_api_version: String,
}

pub async fn sync_extensions(opts: SyncExtensionsOptions) -> Result<()> {
    let http_client = reqwest::Client::new();
    let opts = Arc::new(opts);

    let extension_manifests = fetch_extensions_info(http_client.clone(), Arc::clone(&opts)).await?;
    log::info!("{} currently listed extensions", extension_manifests.len());

    let mut fetch_tasks = tokio::task::JoinSet::new();
    for manifest in extension_manifests {
        fetch_tasks.spawn({
            let opts = Arc::clone(&opts);
            let http_client = http_client.clone();
            let extension_string = manifest.to_string();
            async move {
                fetch_single_extension(manifest, http_client, opts)
                    .await
                    .with_context(|| extension_string)
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

    log::info!("extensions sync complete ({error_count} errors)");

    Ok(())
}

#[derive(Deserialize)]
struct ExtensionsIndexResponse {
    data: Vec<ExtensionManifest>,
}

async fn fetch_extensions_info(
    http_client: reqwest::Client,
    opts: Arc<SyncExtensionsOptions>,
) -> Result<Vec<ExtensionManifest>> {
    let res = http_client
        .get(format!("{}/extensions", opts.upstream_url))
        // TODO: zedex also uses include_native=true, check if its usefull
        .query(&[("max_schema_version", &opts.max_schema_version)])
        .send()
        .await?
        .error_for_status()?;
    let raw_json_body = res.bytes().await?;

    let extensions: ExtensionsIndexResponse = serde_json::from_slice(&raw_json_body)?;
    log::info!("{} currently listed extensions", extensions.data.len());

    Ok(extensions.data)
}

async fn fetch_single_extension(
    manifest: ExtensionManifest,
    http_client: reqwest::Client,
    opts: Arc<SyncExtensionsOptions>,
) -> anyhow::Result<ExtensionManifest> {
    let target_dir = opts
        .extension_dir
        .safe_join(&manifest.id)?
        .safe_join(manifest.version.to_string())?;
    if target_dir.exists() {
        log::debug!("skipping {manifest}: already downloaded");
        return Ok(manifest);
    }

    log::debug!("new version for {manifest}");
    fs::create_dir_all(&target_dir).await?;

    let manifest_f = std::fs::File::create_new(target_dir.join("manifest.json"))?;
    serde_json::to_writer(manifest_f, &manifest)?;

    let req = http_client
        .get(format!(
            "{}/extensions/{}/download",
            &opts.upstream_url, manifest.id
        ))
        .query(&[
            ("min_schema_version", &opts.min_schema_version.to_string()),
            ("max_schema_version", &opts.max_schema_version.to_string()),
            ("min_wasm_api_version", &opts.min_wasm_api_version),
            ("max_wasm_api_version", &opts.max_wasm_api_version),
        ]);
    download_to_file(req, target_dir.join("archive.tar.gz")).await?;

    log::info!("download complete {manifest}");
    Ok::<ExtensionManifest, anyhow::Error>(manifest)
}
