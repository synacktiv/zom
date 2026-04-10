mod changelog;
mod extensions;
mod opts;
mod release;
mod static_files;

use anyhow::Result;

pub use opts::SyncOpts;

use crate::mirror::MirrorDirectory;
use crate::sync::changelog::{SyncChangelogOptions, sync_changelogs};
use crate::sync::extensions::{SyncExtensionsOptions, sync_extensions};
use crate::sync::release::{SyncReleaseOptions, sync_release};
use crate::sync::static_files::{SyncStaticFilesOptions, sync_static_files};

pub async fn zom_sync(opts: SyncOpts) -> Result<()> {
    log::debug!("running sync with options {opts:?}");
    log::info!("syncing directory '{}'", opts.mirror_directory.display());

    let dir = MirrorDirectory::new(&opts.mirror_directory);
    dir.create_dir()?;
    dir.check_valid()?;

    sync_release(
        opts.assets,
        SyncReleaseOptions {
            release_dir: dir.releases_dir().to_path_buf(),
            upstream_url: opts.upstream_cloud_url,
        },
    )
    .await?;

    sync_changelogs(SyncChangelogOptions {
        upstream_url: opts.upstream_zed_url.clone(),
        release_dir: dir.releases_dir().to_path_buf(),
    })
    .await?;

    sync_extensions(SyncExtensionsOptions {
        extension_dir: dir.extensions_dir().to_path_buf(),
        max_schema_version: opts.max_schema_version,
        max_wasm_api_version: opts.max_wasm_api_version,
        min_schema_version: opts.min_schema_version,
        min_wasm_api_version: opts.min_wasm_api_version,
        upstream_url: opts.upstream_api_url,
    })
    .await?;

    sync_static_files(SyncStaticFilesOptions {
        static_files_dir: dir.static_files_dir().to_path_buf(),
        upstream_zed_url: opts.upstream_zed_url,
    })
    .await?;

    Ok(())
}
