use super::download_headers::attachment_value;
use super::list_api::PathQuery;
use super::replies;
use super::state::AppState;
use super::temp_reader::TempFileReader;
use crate::logic::{safe_paths, zip_builder};
use axum::Form;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use std::path::PathBuf;
use tokio_util::io::ReaderStream;

pub async fn download_file(
    State(state): State<AppState>,
    Query(query): Query<PathQuery>,
) -> Response {
    let Some((clean, full)) = safe_paths::locate(&state.storage_root(), &query.path) else {
        return replies::bad_path();
    };
    let Ok(file) = tokio::fs::File::open(&full).await else {
        return replies::error(StatusCode::NOT_FOUND, "File not found.");
    };
    let Ok(metadata) = file.metadata().await else {
        return replies::error(StatusCode::NOT_FOUND, "File not found.");
    };
    if !metadata.is_file() {
        return replies::error(StatusCode::NOT_FOUND, "File not found.");
    }
    let body = Body::from_stream(ReaderStream::new(file));
    attachment_response(body, metadata.len(), safe_paths::last_segment(&clean))
}

#[derive(Deserialize)]
pub struct ZipForm {
    paths: String,
}

/// Builds a ZIP of the given paths in a temp file and sends it.
pub async fn download_zip(State(state): State<AppState>, Form(form): Form<ZipForm>) -> Response {
    let Ok(requested) = serde_json::from_str::<Vec<String>>(&form.paths) else {
        return replies::error(StatusCode::BAD_REQUEST, "Invalid request.");
    };
    let mut cleaned = Vec::new();
    for relative in &requested {
        match safe_paths::clean_relative(relative) {
            Some(clean) => cleaned.push(clean),
            None => return replies::bad_path(),
        }
    }
    if cleaned.is_empty() {
        return replies::error(StatusCode::BAD_REQUEST, "Nothing selected.");
    }
    let zip_name = zip_file_name(&cleaned);
    let temp_path = zip_builder::new_temp_zip_path();
    let root = state.storage_root();
    let target = temp_path.clone();
    let built = tokio::task::spawn_blocking(move || {
        let items = zip_builder::items_for_paths(&root, &cleaned);
        zip_builder::build_zip(&items, &target)
    })
    .await;
    if !matches!(built, Ok(Ok(()))) {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return replies::error(StatusCode::INTERNAL_SERVER_ERROR, "Could not create the ZIP file.");
    }
    zip_response(temp_path, &zip_name).await
}

async fn zip_response(temp_path: PathBuf, zip_name: &str) -> Response {
    let Ok(file) = tokio::fs::File::open(&temp_path).await else {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return replies::error(StatusCode::INTERNAL_SERVER_ERROR, "Could not read the ZIP file.");
    };
    let length = file.metadata().await.map(|data| data.len()).unwrap_or(0);
    let reader = TempFileReader::new(file, temp_path);
    attachment_response(Body::from_stream(ReaderStream::new(reader)), length, zip_name)
}

fn zip_file_name(cleaned: &[String]) -> String {
    match cleaned {
        [only] if !only.is_empty() => format!("{}.zip", safe_paths::last_segment(only)),
        _ => "datei-browser.zip".to_string(),
    }
}

fn attachment_response(body: Body, length: u64, file_name: &str) -> Response {
    let headers = [
        (header::CONTENT_TYPE, "application/octet-stream".to_string()),
        (header::CONTENT_LENGTH, length.to_string()),
        (header::CONTENT_DISPOSITION, attachment_value(file_name)),
    ];
    (headers, body).into_response()
}
