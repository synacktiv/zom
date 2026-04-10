use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    model::ExtensionManifest,
    serve::{handlers::ApiError, state::AppState},
    utils::{SafeJoin, list_dir_sorted_by_semver},
};
use crate::{serve::handlers::ZomApiResult, utils::list_dir};

#[derive(Debug, Deserialize)]
pub struct ListExtensionQuery {
    pub max_schema_version: u16,
    pub provides: String,
    pub filter: Option<String>,
}

pub async fn list_extensions(
    query: Query<ListExtensionQuery>,
    State(state): State<Arc<AppState>>,
) -> ZomApiResult<impl IntoResponse> {
    log::trace!("got list extensions query {query:?}");

    let filter = query
        .filter
        .as_deref()
        .map(|f| f.trim().replace(' ', "").to_lowercase());

    let mut extensions_list = vec![];
    for extension_dir in list_dir(state.dir.extensions_dir()).await? {
        let versions = list_dir_sorted_by_semver(&extension_dir.path()).await?;
        let latest_version = versions.first().ok_or_else(|| {
            ApiError::NoExtensionVersion(extension_dir.file_name().to_string_lossy().to_string())
        })?;

        let manifest_path = latest_version.path().join("manifest.json");
        let manifest: ExtensionManifest =
            serde_json::from_slice(&tokio::fs::read(&manifest_path).await?)?;

        if manifest.schema_version > query.max_schema_version {
            continue;
        }
        if !query.provides.is_empty() && !manifest.provides.contains(&query.provides) {
            continue;
        }
        if let Some(filter) = &filter
            && !manifest.match_filter(filter)
        {
            continue;
        }

        extensions_list.push(manifest);
    }
    extensions_list.sort_by_key(|extension| std::cmp::Reverse(extension.download_count));

    Ok(Json(serde_json::json!({"data": extensions_list})))
}

#[derive(Debug, Deserialize)]
pub struct UpdateExtensionQuery {
    pub min_schema_version: u16,
    pub max_schema_version: u16,
    pub min_wasm_api_version: semver::Version,
    pub max_wasm_api_version: semver::Version,
    pub ids: String,
}

pub async fn update_extension(
    query: Query<UpdateExtensionQuery>,
    State(state): State<Arc<AppState>>,
) -> ZomApiResult<impl IntoResponse> {
    log::trace!("got update extension query {query:?}");
    let mut extensions_list = vec![];

    for extension_id in query.ids.split(',') {
        let extension_path = match state.dir.extensions_dir().safe_join(extension_id) {
            Ok(path) => path,
            Err(e) => {
                log::warn!("Invalid extension_id '{extension_id}' in update_extension: {e}");
                continue;
            }
        };
        let Ok(versions) = list_dir_sorted_by_semver(extension_path).await else {
            continue;
        };
        for version in versions {
            let manifest_path = version.path().join("manifest.json");
            let manifest: ExtensionManifest =
                serde_json::from_slice(&tokio::fs::read(&manifest_path).await?)?;

            if manifest.check_schema_version(query.min_schema_version, query.max_schema_version)
                && manifest.check_wasm_api_version(
                    &query.min_wasm_api_version,
                    &query.max_wasm_api_version,
                )
            {
                extensions_list.push(manifest);
                break;
            }
        }
    }

    Ok(Json(serde_json::json!({"data": extensions_list})))
}

#[derive(Debug, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct DownloadExtensionQuery {
    pub min_schema_version: u16,
    pub max_schema_version: u16,
    pub min_wasm_api_version: semver::Version,
    pub max_wasm_api_version: semver::Version,
}

pub async fn download_extension(
    query: Query<DownloadExtensionQuery>,
    Path(extension_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> ZomApiResult<impl IntoResponse> {
    log::trace!("got download extension query for {extension_id} {query:?}",);

    let extension_path = match state.dir.extensions_dir().safe_join(&extension_id) {
        Ok(path) => path,
        Err(e) => {
            return Err(ApiError::InvalidQuery {
                msg: extension_id,
                source: e.into(),
            });
        }
    };
    let versions = list_dir_sorted_by_semver(extension_path).await?;
    for version in versions {
        let manifest_path = version.path().join("manifest.json");
        let manifest: ExtensionManifest =
            serde_json::from_slice(&tokio::fs::read(&manifest_path).await?)?;

        if manifest.check_schema_version(query.min_schema_version, query.max_schema_version)
            && manifest
                .check_wasm_api_version(&query.min_wasm_api_version, &query.max_wasm_api_version)
        {
            let body = get_streamed_extension_archive(version.path()).await?;

            return Ok((StatusCode::OK, body));
        }
    }

    Err(ApiError::NoExtension(extension_id))
}

pub async fn download_extension_version(
    Path((extension_id, version)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> ZomApiResult<impl IntoResponse> {
    log::trace!("got download extension for {extension_id} version {version}");

    let extension_path = match state.dir.extensions_dir().safe_join(&extension_id) {
        Ok(path) => path,
        Err(e) => {
            log::warn!("Invalid extension_id '{extension_id}' in download_extension_version: {e}");
            return Err(ApiError::NoExtension(extension_id));
        }
    };

    let extension_version_path = match extension_path.safe_join(&version) {
        Ok(path) => path,
        Err(e) => {
            log::warn!(
                "Invalid version {version} for extension '{extension_id}' in download_extension_version: {e}"
            );
            return Err(ApiError::NoExtensionVersionFound(extension_id, version));
        }
    };

    let body = get_streamed_extension_archive(extension_version_path).await?;

    Ok((StatusCode::OK, body))
}

async fn get_streamed_extension_archive<P: AsRef<std::path::Path>>(
    extension_version_path: P,
) -> ZomApiResult<Body> {
    let archive_path = extension_version_path.as_ref().join("archive.tar.gz");
    let file = tokio::fs::File::open(&archive_path).await.map_err(|_| {
        ApiError::NoExtensionVersion(
            extension_version_path
                .as_ref()
                .to_string_lossy()
                .to_string(),
        )
    })?;

    Ok(Body::from_stream(tokio_util::io::ReaderStream::new(file)))
}
