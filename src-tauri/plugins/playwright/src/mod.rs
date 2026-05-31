use tauri::plugin::{Builder, TauriPlugin};

mod bridge;
mod http;

/// Initialize the playwright plugin.
///
/// Stores the AppHandle for use by the eval bridge (HTTP → WebView JS injection).
pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new("playwright")
        .setup(|app_handle, _api| {
            bridge::init_bridge(app_handle.clone());
            Ok(())
        })
        .build()
}

/// Returns the Axum router for playwright HTTP endpoints.
///
/// Call this from the main HTTP server setup to register the
/// `/api/playwright/*` routes.
pub fn http_router() -> axum::Router {
    http::routes()
}
