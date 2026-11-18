use super::state::AppState;
use super::{auth, download_api, list_api, pages, pins_api, upload_api};
use axum::extract::DefaultBodyLimit;
use axum::middleware;
use axum::routing::{get, post};
use axum::Router;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(pages::page_routes())
        .merge(public_routes())
        .merge(protected_routes(state.clone()))
        .with_state(state)
}

/// Routes that work without login.
fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/api/auth", get(auth::auth_status))
        .route("/api/login", post(auth::login))
}

/// Routes that need the password (if one is set).
fn protected_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/api/list", get(list_api::list_folder))
        .route("/api/folder", post(list_api::create_folder))
        .route("/api/delete", post(list_api::delete_entry))
        .route(
            "/api/upload",
            post(upload_api::upload_file).layer(DefaultBodyLimit::disable()),
        )
        .route("/api/download", get(download_api::download_file))
        .route("/api/zip", post(download_api::download_zip))
        .route("/api/pins", get(pins_api::list_pins).post(pins_api::set_pin))
        .layer(middleware::from_fn_with_state(state, auth::require_login))
}
