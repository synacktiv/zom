mod handlers;
mod opts;
mod state;

use std::{sync::Arc, time::Duration};

use anyhow::Result;
use axum::{
    Router,
    body::{Body, HttpBody},
    http::{HeaderMap, Request, Response, uri::PathAndQuery},
    routing::get,
};
use reqwest::{
    StatusCode,
    header::{CONTENT_TYPE, USER_AGENT},
};
use tokio::net::TcpListener;
use tower_http::{
    classify::{SharedClassifier, StatusInRangeAsFailures},
    trace::{DefaultMakeSpan, OnRequest, OnResponse, TraceLayer},
};
use tracing::Span;

pub use crate::serve::opts::ServeOpts;
use crate::{
    mirror::MirrorDirectory,
    serve::{
        handlers::{changelogs, extensions, release, static_files, version},
        state::AppState,
    },
};

fn compute_basepath_from_headers(headers: &HeaderMap) -> String {
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("http");
    let host = headers
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();
    format!("{proto}://{host}")
}

/// Create a new Layer that logs at INFO level for every request / response received.
fn trace_layer() -> TraceLayer<
    SharedClassifier<StatusInRangeAsFailures>,
    DefaultMakeSpan,
    impl OnRequest<Body> + Clone,
    impl OnResponse<Body> + Clone,
> {
    // classify all status in 400..599 as errors for logs
    let classifier = StatusInRangeAsFailures::new(StatusCode::BAD_REQUEST.as_u16()..=599);

    TraceLayer::new(classifier.into_make_classifier())
        .on_request(|request: &Request<Body>, _span: &Span| {
            log::info!(
                "[{}] {} http_version={:?} user_agent={} request_size={}",
                request.method(),
                request
                    .uri()
                    .path_and_query()
                    .map(PathAndQuery::as_str)
                    .unwrap_or_default(),
                request.version(),
                request
                    .headers()
                    .get(USER_AGENT)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or_default(),
                request.body().size_hint().exact().unwrap_or_default()
            );
        })
        .on_response(
            |response: &Response<Body>, latency: Duration, _span: &Span| {
                log::info!(
                    "[{}] latency={}ms content_type={} response_size={}",
                    response.status(),
                    latency.as_millis(),
                    response
                        .headers()
                        .get(CONTENT_TYPE)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or_default(),
                    response.body().size_hint().exact().unwrap_or_default()
                );
            },
        )
}

pub async fn zom_serve(opts: ServeOpts) -> Result<()> {
    log::debug!("running sync with options {opts:?}");

    log::info!("Starting server on http://{} ...", opts.listen_addr);
    log::info!("Version: {}", env!("CARGO_PKG_VERSION"));

    let app_state = AppState {
        base_url: opts.base_url,
        dir: MirrorDirectory::new(opts.mirror_directory),
    };

    let extensions_router = Router::new()
        .route("/", get(extensions::list_extensions))
        .route("/updates", get(extensions::update_extension))
        .route(
            "/{extension_id}/download",
            get(extensions::download_extension),
        )
        .route(
            "/{extension_id}/{version}/download",
            get(extensions::download_extension_version),
        );

    let app = Router::new()
        .route("/", get(static_files::serve_index))
        .route("/index", get(static_files::serve_index))
        .route("/index.html", get(static_files::serve_index))
        .route("/style.css", get(static_files::serve_css))
        .route("/version", get(version::version_handler))
        .route("/install.sh", get(static_files::serve_installation_script))
        .route(
            "/releases/stable/{version}/asset",
            get(release::get_release_manifest),
        )
        .route(
            "/releases/stable/{version}/download",
            get(release::download_asset),
        )
        .route(
            "/api/release_notes/v2/stable/{version}",
            get(changelogs::serve_changelogs),
        )
        .nest("/extensions", extensions_router)
        .layer(trace_layer())
        .with_state(Arc::new(app_state));

    let listener = TcpListener::bind(opts.listen_addr).await?;
    Ok(axum::serve(listener, app).await?)
}
