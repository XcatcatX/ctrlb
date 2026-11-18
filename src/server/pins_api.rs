use super::replies;
use super::state::AppState;
use crate::logic::folder_listing::{self, EntryInfo};
use crate::logic::safe_paths;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use std::path::Path;

/// Returns all pinned items that still exist.
pub async fn list_pins(State(state): State<AppState>) -> Response {
    let pinned = state.pins.lock().unwrap().items().to_vec();
    let root = state.storage_root();
    let entries = tokio::task::spawn_blocking(move || existing_entries(&root, &pinned))
        .await
        .unwrap_or_default();
    Json(entries).into_response()
}

#[derive(Deserialize)]
pub struct PinRequest {
    path: String,
    pinned: bool,
}

pub async fn set_pin(State(state): State<AppState>, Json(body): Json<PinRequest>) -> Response {
    let Some((clean, full)) = safe_paths::locate(&state.storage_root(), &body.path) else {
        return replies::bad_path();
    };
    if body.pinned && (clean.is_empty() || !full.exists()) {
        return replies::error(StatusCode::NOT_FOUND, "Item not found.");
    }
    let result = state.pins.lock().unwrap().set_pinned(&clean, body.pinned);
    match result {
        Ok(()) => replies::ok(),
        Err(_) => replies::error(StatusCode::INTERNAL_SERVER_ERROR, "Could not save the pins."),
    }
}

fn existing_entries(root: &Path, pinned: &[String]) -> Vec<EntryInfo> {
    pinned
        .iter()
        .filter_map(|relative| {
            let (clean, full) = safe_paths::locate(root, relative)?;
            folder_listing::entry_for(&full, &clean)
        })
        .collect()
}
