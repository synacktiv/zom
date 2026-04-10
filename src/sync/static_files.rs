use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use tokio::{fs, io::AsyncWriteExt};

use crate::utils::SafeJoin;

pub struct SyncStaticFilesOptions {
    pub upstream_zed_url: String,
    pub static_files_dir: PathBuf,
}

/// Fetch given remote resource pointed to by `resource`.
/// Always re-fetches / re-writes without checking if file has been modified.
async fn fetch_remote_resource<S: AsRef<str>>(
    client: reqwest::Client,
    opts: Arc<SyncStaticFilesOptions>,
    resource: S,
) -> Result<()> {
    let remote_resource = client
        .get(format!("{}/{}", opts.upstream_zed_url, resource.as_ref()))
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let mut resource_backing_file =
        fs::File::create(opts.static_files_dir.safe_join(resource.as_ref())?).await?;
    resource_backing_file.write_all(&remote_resource).await?;

    log::info!("downloading static file {}", resource.as_ref());

    Ok(())
}

pub async fn sync_static_files(opts: SyncStaticFilesOptions) -> Result<()> {
    let http_client = reqwest::Client::new();
    let opts = Arc::new(opts);

    // Currently only fetching a single element, but it can easily be changed to handle multiple simultaneous downloads.
    let mut fetch_tasks = tokio::task::JoinSet::new();
    let resource = "install.sh";
    fetch_tasks.spawn({
        let opts = Arc::clone(&opts);
        let http_client = http_client.clone();

        async move {
            fetch_remote_resource(http_client, opts, resource)
                .await
                .with_context(|| resource.to_string())
        }
    });

    let mut error_count = 0;
    while let Some(res) = fetch_tasks.join_next().await {
        if let Err(e) = res? {
            error_count += 1;
            log::error!("error downloading static file {e}: {}", e.root_cause());
        }
    }

    log::info!("static files sync complete ({error_count} errors)");

    Ok(())
}
