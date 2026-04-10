use std::error::Error;

use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;
use thiserror::Error;

pub mod changelogs;
pub mod extensions;
pub mod release;
pub mod static_files;
pub mod version;

/// Represents a custom API result type.
/// Implements [`IntoResponse`] as long as T implements it.
pub type ZomApiResult<T> = Result<T, ApiError>;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("invalid query {msg}: {source}")]
    InvalidQuery {
        msg: String,
        source: Box<dyn std::error::Error>,
    },
    #[error("cannot find extension {0}")]
    NoExtension(String),
    #[error("cannot find any version for extension {0}")]
    NoExtensionVersion(String),
    #[error("cannot find version {1} for extension {0}")]
    NoExtensionVersionFound(String, String),
    #[error("cannot find any version for release {0}")]
    NoReleaseVersion(String),
    #[error("did not find any changelog for release {0}")]
    NoChangelog(String),
    #[error("cannot find any version for asset {0}")]
    NoAsset(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        log::error!(
            "got error while handling request: {self} (source: {:?})",
            self.source()
        );

        let response_status = match self {
            Self::Io(_)
            | Self::Serde(_)
            | Self::NoExtensionVersion(_)
            | Self::NoReleaseVersion(_)
            | Self::NoChangelog(_)
            | Self::NoExtensionVersionFound(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NoAsset(_) | Self::NoExtension(_) => StatusCode::NOT_FOUND,
            Self::InvalidQuery { .. } => StatusCode::BAD_REQUEST,
        };

        response_status.into_response()
    }
}
