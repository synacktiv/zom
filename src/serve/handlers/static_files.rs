use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, HeaderValue, Response},
    response::IntoResponse,
};
use reqwest::header::CONTENT_TYPE;
use tokio::fs;

use crate::{
    serve::{compute_basepath_from_headers, handlers::ZomApiResult, state::AppState},
    utils::SafeJoin,
};

static ZED_CLOUD_ADDR: &str = "https://cloud.zed.dev";

pub async fn serve_installation_script(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> ZomApiResult<impl IntoResponse> {
    let zed_install_script_content =
        fs::read_to_string(state.dir.static_files_dir().safe_join("install.sh")?).await?;

    let base_url: String = state.base_url.as_ref().map_or_else(
        || compute_basepath_from_headers(&headers),
        std::string::ToString::to_string,
    );

    // We need to replace upstream Zed domain name by ours to ensure that the script will load correctly.
    Ok(Response::new(Body::new(
        zed_install_script_content.replace(ZED_CLOUD_ADDR, &base_url),
    )))
}

pub async fn serve_index(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> ZomApiResult<impl IntoResponse> {
    let url = state.base_url.as_ref().map_or_else(
        || compute_basepath_from_headers(&headers),
        std::borrow::ToOwned::to_owned,
    );

    let index = include_str!("../static/index.html").replace("http://localhost:8080", &url);

    Ok(axum::response::Html(index))
}

pub async fn serve_css() -> ZomApiResult<impl IntoResponse> {
    let stylesheet = include_str!("../static/style.css");

    let mut response = Response::new(stylesheet.to_string());
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("text/css"));

    Ok(response)
}
