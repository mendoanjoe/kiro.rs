//! Admin UI router configuration

use axum::{
    Router,
    body::Body,
    http::{Response, StatusCode, Uri, header},
    response::IntoResponse,
    routing::get,
};
use rust_embed::Embed;

/// Embeds frontend build artifacts
#[derive(Embed)]
#[folder = "admin-ui/dist"]
struct Asset;

/// Create Admin UI router
pub fn create_admin_ui_router() -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/{*file}", get(static_handler))
}

/// Handle index page request
async fn index_handler() -> impl IntoResponse {
    serve_index()
}

/// Handle static file request
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // Security check: reject paths containing ..
    if path.contains("..") {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Invalid path"))
            .expect("Failed to build response");
    }

    // Try to get the requested file
    if let Some(content) = Asset::get(path) {
        let mime = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        // Set different cache strategies based on file type
        let cache_control = get_cache_control(path);

        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime)
            .header(header::CACHE_CONTROL, cache_control)
            .body(Body::from(content.data.into_owned()))
            .expect("Failed to build response");
    }

    // SPA fallback: if the file does not exist and is not an asset file, return index.html
    if !is_asset_path(path) {
        return serve_index();
    }

    // 404
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not found"))
        .expect("Failed to build response")
}

/// Serve index.html
fn serve_index() -> Response<Body> {
    match Asset::get("index.html") {
        Some(content) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(Body::from(content.data.into_owned()))
            .expect("Failed to build response"),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(
                "Admin UI not built. Run 'pnpm build' in admin-ui directory.",
            ))
            .expect("Failed to build response"),
    }
}

/// Return appropriate cache strategy based on file type
fn get_cache_control(path: &str) -> &'static str {
    if path.ends_with(".html") {
        // HTML files are not cached to ensure users get the latest version
        "no-cache"
    } else if path.starts_with("assets/") {
        // Files under assets/ have content hashes and can be cached long-term
        "public, max-age=31536000, immutable"
    } else {
        // Other files (such as favicon) use a shorter cache duration
        "public, max-age=3600"
    }
}

/// Determine if a path is an asset file path (files with extensions)
fn is_asset_path(path: &str) -> bool {
    // Check if the last path segment contains an extension
    path.rsplit('/')
        .next()
        .map(|filename| filename.contains('.'))
        .unwrap_or(false)
}
