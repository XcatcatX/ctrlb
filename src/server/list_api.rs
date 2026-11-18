use super::replies;
use super::state::AppState;
use crate::logic::folder_listing::{self, EntryInfo};
use crate::logic::{file_ops, safe_paths};
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct PathQuery {
    #[serde(default)]
    pub path: String,
}

#[derive(Serialize)]
struct ListResponse {
    path: String,
    entries: Vec<EntryInfo>,
}

pub async fn list_folder(
    State(state): State<AppState>,
    Query(query): Query<PathQuery>,
) -> Response {
    let Some((clean, full)) = safe_paths::locate(&state.storage_root(), &query.path) else {
        return replies::bad_path();
    };
    let relative = clean.clone();
    let result =
        tokio::task::spawn_blocking(move || folder_listing::list_folder(&full, &relative)).await;
    match result {
        Ok(Ok(entries)) => Json(ListResponse { path: clean, entries }).into_response(),
        _ => replies::error(StatusCode::NOT_FOUND, "Folder not found."),
    }
}

#[derive(Deserialize)]
pub struct CreateFolderRequest {
    path: String,
    name: String,
}

pub async fn create_folder(
    State(state): State<AppState>,
    Json(body): Json<CreateFolderRequest>,
) -> Response {
    let Some((_, parent)) = safe_paths::locate(&state.storage_root(), &body.path) else {
        return replies::bad_path();
    };
    match file_ops::create_folder(&parent, &body.name) {
        Ok(()) => replies::ok(),
        Err(message) => replies::error(StatusCode::BAD_REQUEST, &message),
    }
}

#[derive(Deserialize)]
pub struct DeleteRequest {
    path: String,
}

pub async fn delete_entry(
    State(state): State<AppState>,
    Json(body): Json<DeleteRequest>,
) -> Response {
    let root = state.storage_root();
    let Some((clean, _)) = safe_paths::locate(&root, &body.path) else {
        return replies::bad_path();
    };
    if clean.is_empty() {
        return replies::error(StatusCode::BAD_REQUEST, "The main folder cannot be deleted.");
    }
    let (parent_path, name) = clean.rsplit_once('/').unwrap_or(("", clean.as_str()));
    let Some((_, parent)) = safe_paths::locate(&root, parent_path) else {
        return replies::bad_path();
    };
    let name = name.to_string();
    let result = tokio::task::spawn_blocking(move || file_ops::delete_entry(&parent, &name)).await;
    if !matches!(result, Ok(Ok(()))) {
        return replies::error(StatusCode::INTERNAL_SERVER_ERROR, "Could not delete.");
    }
    let _ = state.pins.lock().unwrap().set_pinned(&clean, false);
    replies::ok()
}
