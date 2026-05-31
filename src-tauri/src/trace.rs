/// Execution trace bridge — sends trace events to both stderr and the frontend log panel.
///
/// Initialize with `trace_init(app_handle)` at startup, then use `trace_event()` anywhere.
/// Trace events appear in:
///   1. stderr (visible in `cargo tauri dev` terminal)
///   2. Frontend bottom panel → log tab (via Tauri `archbot:trace` event)
use std::sync::OnceLock;
use tauri::Emitter;

static APP: OnceLock<tauri::AppHandle> = OnceLock::new();

/// Call once during app setup.
pub fn trace_init(app_handle: tauri::AppHandle) {
    APP.set(app_handle).ok();
}

/// Emit a trace event to stderr and the frontend log panel.
pub fn trace_event(category: &str, message: &str) {
    let ts = chrono::Utc::now().format("%H:%M:%S%.3f").to_string();
    let line = format!("[TRACE {}] [{}] {}", ts, category, message);

    // 1. Terminal visibility
    eprintln!("{}", line);

    // 2. Frontend log panel
    if let Some(app) = APP.get() {
        let payload = serde_json::json!({
            "timestamp": ts,
            "category": category,
            "message": message,
        });
        let _ = app.emit("archbot:trace", payload);
    }
}

/// Convenience: `trace_fmt!(category, format!(...))` without needing to import format.
/// Use as: `trace_fmt!("cat", "msg with {} args", value);`
#[macro_export]
macro_rules! trace_fmt {
    ($cat:expr, $($arg:tt)*) => {
        $crate::trace::trace_event($cat, &format!($($arg)*))
    };
}
