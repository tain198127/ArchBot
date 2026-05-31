use tauri::plugin::{Builder, TauriPlugin};

mod bridge;
mod http;

/// Initialize the playwright plugin.
///
/// This sets up the eval bridge (AppHandle → WebView JS injection)
/// and exposes the HTTP routes for test control.
pub fn init() -> TauriPlugin<tauri::Wry> {
    Builder::new("playwright")
        .invoke_handler(tauri::generate_handler![bridge::callback])
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
