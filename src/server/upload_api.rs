use super::replies;
use super::state::AppState;
use crate::logic::safe_paths;
use axum::extract::{Multipart, Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use serde::Deserialize;
use std::path::Path;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;

#[derive(Deserialize)]
pub struct UploadQuery {
    path: String,
    #[serde(default)]
    offset: u64,
}

/// Receives one part of a file. Big files arrive in several parts.
/// The part with offset 0 creates the file, the next parts are appended.
pub async fn upload_file(
    State(state): State<AppState>,
    Query(query): Query<UploadQuery>,
    mut multipart: Multipart,
) -> Response {
    let Some((clean, target)) = safe_paths::locate(&state.storage_root(), &query.path) else {
        return replies::bad_path();
    };
    if clean.is_empty() {
        return replies::error(StatusCode::BAD_REQUEST, "Missing file name.");
    }
    match write_part(&target, query.offset, &mut multipart).await {
        Ok(()) => replies::ok(),
        Err(message) => {
            let _ = tokio::fs::remove_file(&target).await;
            replies::error(StatusCode::INTERNAL_SERVER_ERROR, &message)
        }
    }
}

async fn write_part(
    target: &Path,
    offset: u64,
    multipart: &mut Multipart,
) -> Result<(), String> {
    let mut field = multipart
        .next_field()
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "No file data received.".to_string())?;
    let mut file = open_target(target, offset).await?;
    while let Some(chunk) = field.chunk().await.map_err(|error| error.to_string())? {
        file.write_all(&chunk)
            .await
            .map_err(|error| error.to_string())?;
    }
    file.flush().await.map_err(|error| error.to_string())
}

async fn open_target(target: &Path, offset: u64) -> Result<File, String> {
    if offset == 0 {
        if let Some(parent) = target.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|error| error.to_string())?;
        }
        return File::create(target).await.map_err(|error| error.to_string());
    }
    let file = OpenOptions::new()
        .append(true)
        .open(target)
        .await
        .map_err(|error| error.to_string())?;
    let length = file.metadata().await.map_err(|error| error.to_string())?.len();
    if length != offset {
        return Err("The upload parts arrived in the wrong order.".to_string());
    }
    Ok(file)
}
