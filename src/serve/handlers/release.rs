use std::ffi;
use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, header},
    response::{AppendHeaders, IntoResponse},
};
use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    model::Asset,
    serve::{
        compute_basepath_from_headers,
        handlers::{ApiError, ZomApiResult},
        state::AppState,
    },
    utils::SafeJoin,
};

#[derive(Debug, Deserialize)]
pub struct ReleaseAssetQuery {
    os: String,
    arch: String,
    asset: String,
}

pub async fn get_release_manifest(
    Path(version_query): Path<String>,
    Query(params): Query<ReleaseAssetQuery>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> ZomApiResult<impl IntoResponse> {
    let asset = Asset {
        name: params.asset,
        os: params.os,
        arch: params.arch,
    };

    let version =
        state
            .dir
            .version_dir(&version_query)
            .await
            .map_err(|e| ApiError::InvalidQuery {
                msg: version_query.clone(),
                source: e.into(),
            })?;

    let version_str = version
        .file_name()
        .and_then(ffi::OsStr::to_str)
        .ok_or(ApiError::NoReleaseVersion(version_query))?;

    let asset_path = version
        .safe_join(asset.filename())
        .map_err(|e| ApiError::InvalidQuery {
            msg: asset.to_string(),
            source: e.into(),
        })?;

    if !asset_path.exists() {
        return Err(ApiError::NoAsset(asset.to_string()));
    }

    let url = format!(
        "{}/releases/stable/{version_str}/download?asset={}&os={}&arch={}",
        state.base_url.as_ref().map_or_else(
            || compute_basepath_from_headers(&headers),
            std::borrow::ToOwned::to_owned
        ),
        asset.name,
        asset.os,
        asset.arch,
    );

    Ok(Json(
        serde_json::json!({"version": version_str, "url": url}),
    ))
}

pub async fn download_asset(
    Path(version_query): Path<String>,
    Query(params): Query<ReleaseAssetQuery>,
    State(state): State<Arc<AppState>>,
) -> ZomApiResult<impl IntoResponse> {
    let asset = Asset {
        name: params.asset,
        os: params.os,
        arch: params.arch,
    };

    let version =
        state
            .dir
            .version_dir(&version_query)
            .await
            .map_err(|e| ApiError::InvalidQuery {
                msg: version_query.clone(),
                source: e.into(),
            })?;

    let asset_path = version
        .safe_join(asset.filename())
        .map_err(|e| ApiError::InvalidQuery {
            msg: asset.to_string(),
            source: e.into(),
        })?;

    let file = tokio::fs::File::open(&asset_path)
        .await
        .map_err(|_| ApiError::NoAsset(asset.to_string()))?;
    let body = Body::from_stream(tokio_util::io::ReaderStream::new(file));

    let headers = AppendHeaders([
        (header::CONTENT_TYPE, "application/gzip".to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{asset}\""),
        ),
    ]);

    Ok((StatusCode::OK, headers, body))
}
