//! Web UI configuration for serving embedded frontend assets

use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::IntoResponse,
};
use tracing::{info, debug, warn};
use crate::config::AppConfig;

/// Get content type based on file extension
fn get_content_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else if path.ends_with(".woff") {
        "font/woff"
    } else if path.ends_with(".ttf") {
        "font/ttf"
    } else {
        "application/octet-stream"
    }
}

/// Check if a path looks like a file (has extension)
fn is_file_path(path: &str) -> bool {
    path.contains('.')
}

/// Handler for index.html - the main entry point
pub async fn index() -> impl IntoResponse {
    info!("[WEB] Serving index.html (root path)");
    handle_embedded_file("index.html", false).await
}

/// Handler for serving embedded assets
pub async fn serve_asset(Path(path): Path<String>) -> impl IntoResponse {
    // The *path in /assets/*path only captures the part after /assets/
    // (e.g., "index-DTU9k91d.js"), but embedded files are stored with
    // the full path including "assets/" prefix
    let embedded_path = format!("assets/{}", path.trim_start_matches('/'));
    info!("[WEB] Asset requested: path=\"/{}\"", path);
    handle_embedded_file(&embedded_path, true).await
}

/// Handle embedded file serving
async fn handle_embedded_file(path: &str, is_resource_request: bool) -> impl IntoResponse {
    info!(
        "[WEB] Looking up embedded file: path=\"{}\", is_resource_request={}",
        path, is_resource_request
    );

    // Try to get the file from embedded assets
    match domain_agent_management_web_dist_wrap::get_embedded_file(path) {
        Some(content) => {
            let content_type = get_content_type(path);
            let data_len = content.data.len();

            info!(
                "[WEB] Serving file: path=\"{}\", content_type=\"{}\", size={} bytes",
                path, content_type, data_len
            );

            let headers = [
                (header::CONTENT_TYPE, content_type),
                (header::CACHE_CONTROL, "max-age=604800, public"),
            ];

            (StatusCode::OK, headers, content.data.into_owned()).into_response()
        }
        None => {
            // If this is a resource request (has file extension), return 404
            if is_resource_request || is_file_path(path) {
                warn!(
                    "[WEB] File not found (404): path=\"{}\", is_resource_request={}",
                    path, is_resource_request
                );
                return (StatusCode::NOT_FOUND, "404 Not Found").into_response();
            }

            // For SPA routing: if path doesn't look like a file, try index.html
            info!(
                "[WEB] Path not found, attempting SPA fallback: path=\"{}\"",
                path
            );

            if let Some(content) =
                domain_agent_management_web_dist_wrap::get_embedded_file("index.html")
            {
                let content_type = "text/html";
                let data_len = content.data.len();

                info!(
                    "[WEB] Serving index.html for SPA route: path=\"{}\", size={} bytes",
                    path, data_len
                );

                let headers = [
                    (header::CONTENT_TYPE, content_type),
                    (header::CACHE_CONTROL, "no-cache"),
                ];
                return (StatusCode::OK, headers, content.data.into_owned()).into_response();
            }

            warn!("[WEB] Both file and index.html not found: path=\"{}\"", path);
            (StatusCode::NOT_FOUND, "404 Not Found").into_response()
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.ws_port, 8081);
        assert_eq!(config.grpc.port, 50051);
        assert_eq!(config.rest.port, 8080);
    }
}
