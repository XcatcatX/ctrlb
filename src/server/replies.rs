use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub fn ok() -> Response {
    StatusCode::OK.into_response()
}

pub fn error(status: StatusCode, message: &str) -> Response {
    (status, message.to_string()).into_response()
}

pub fn bad_path() -> Response {
    error(StatusCode::BAD_REQUEST, "Invalid path.")
}
