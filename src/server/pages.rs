use super::state::AppState;
use axum::Router;
use axum::http::header;
use axum::routing::{MethodRouter, get};

const HTML: &str = "text/html; charset=utf-8";
const CSS: &str = "text/css; charset=utf-8";
const JS: &str = "text/javascript; charset=utf-8";

/// Routes for the website files. They are built into the program.
pub fn page_routes() -> Router<AppState> {
    Router::new()
        .route("/", file_route(HTML, include_str!("../../web/index.html")))
        .route("/style.css", file_route(CSS, include_str!("../../web/style.css")))
        .route("/script.js", file_route(JS, include_str!("../../web/script.js")))
        .route("/api.js", file_route(JS, include_str!("../../web/api.js")))
        .route("/dom.js", file_route(JS, include_str!("../../web/dom.js")))
        .route("/paths.js", file_route(JS, include_str!("../../web/paths.js")))
        .route("/list.js", file_route(JS, include_str!("../../web/list.js")))
        .route("/sidebar.js", file_route(JS, include_str!("../../web/sidebar.js")))
        .route("/upload.js", file_route(JS, include_str!("../../web/upload.js")))
        .route("/popups.js", file_route(JS, include_str!("../../web/popups.js")))
}

fn file_route(content_type: &'static str, content: &'static str) -> MethodRouter<AppState> {
    get(move || async move {
        (
            [
                (header::CONTENT_TYPE, content_type),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            content,
        )
    })
}
