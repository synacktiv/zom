use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use tokio::io::AsyncReadExt;

use crate::{
    serve::{
        handlers::{ApiError, ZomApiResult},
        state::AppState,
    },
    utils::SafeJoin,
};

pub async fn serve_changelogs(
    Path(version): Path<String>,
    State(state): State<Arc<AppState>>,
) -> ZomApiResult<impl IntoResponse> {
    let (fixed_version, _) = version
        .split_once('+')
        .ok_or_else(|| ApiError::NoReleaseVersion(format!("version {version} is not valid")))?;

    let changelog_path = state
        .dir
        .releases_dir()
        .safe_join(fixed_version)?
        .safe_join("changelog.json")?;

    let mut file = tokio::fs::File::open(&changelog_path)
        .await
        .map_err(|_| ApiError::NoChangelog(version))?;

    let mut changelog_content = String::new();
    file.read_to_string(&mut changelog_content).await?;

    Ok(changelog_content)
}
