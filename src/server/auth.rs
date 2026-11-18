use super::state::AppState;
use axum::Json;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const COOKIE_NAME: &str = "datei_session";

/// Lets a request pass if no password is set or the cookie is valid.
pub async fn require_login(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if is_allowed(&state, request.headers()) {
        next.run(request).await
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

#[derive(Serialize)]
pub struct AuthStatus {
    password_required: bool,
    logged_in: bool,
}

pub async fn auth_status(State(state): State<AppState>, headers: HeaderMap) -> Json<AuthStatus> {
    Json(AuthStatus {
        password_required: !state.password().is_empty(),
        logged_in: is_allowed(&state, &headers),
    })
}

#[derive(Deserialize)]
pub struct LoginRequest {
    password: String,
}

pub async fn login(State(state): State<AppState>, Json(body): Json<LoginRequest>) -> Response {
    if body.password != state.password() {
        tokio::time::sleep(Duration::from_millis(500)).await;
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let cookie = format!(
        "{COOKIE_NAME}={}; Path=/; HttpOnly; SameSite=Strict",
        state.session_token()
    );
    ([(header::SET_COOKIE, cookie)], StatusCode::OK).into_response()
}

fn is_allowed(state: &AppState, headers: &HeaderMap) -> bool {
    state.password().is_empty() || has_valid_cookie(state, headers)
}

fn has_valid_cookie(state: &AppState, headers: &HeaderMap) -> bool {
    read_cookie(headers, COOKIE_NAME)
        .map(|token| state.is_valid_token(&token))
        .unwrap_or(false)
}

fn read_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    let text = headers.get(header::COOKIE)?.to_str().ok()?;
    text.split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.to_string())
}
