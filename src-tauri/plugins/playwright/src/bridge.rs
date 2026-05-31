use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;

use tauri::Manager;
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

type PendingMap = HashMap<Uuid, oneshot::Sender<serde_json::Value>>;

static BRIDGE: OnceLock<BridgeState> = OnceLock::new();

struct BridgeState {
    pending: Mutex<PendingMap>,
    app_handle: tauri::AppHandle,
}

pub fn init_bridge(app_handle: tauri::AppHandle) {
    let state = BridgeState {
        pending: Mutex::new(HashMap::new()),
        app_handle,
    };
    BRIDGE.set(state).ok();
}

fn get_bridge() -> Result<&'static BridgeState, String> {
    BRIDGE
        .get()
        .ok_or_else(|| "playwright bridge not initialized".to_string())
}

/// Retrieve the pending queries map for the callback command.
pub fn pending_map() -> &'static Mutex<PendingMap> {
    &BRIDGE.get().expect("bridge not initialized").pending
}

/// Evaluate JavaScript in the WebView and return the result.
///
/// For fire-and-forget operations (click, fill), use `eval_void()` instead.
/// This function injects JS and waits for the callback (30s timeout).
pub async fn eval_js(js: &str) -> Result<serde_json::Value, String> {
    let bridge = get_bridge()?;
    let id = Uuid::new_v4();
    let (tx, rx) = oneshot::channel();

    {
        let mut pending = bridge.pending.lock().await;
        pending.insert(id, tx);
    }

    // Build the callback JS: wrap the user's JS in a try/catch, call invoke back
    let escaped_js = js.replace('\\', "\\\\").replace('\'', "\\'");
    let invoke_js = format!(
        r#"(function(){{
  try {{
    var __r = (function(){{ return {} }})();
    window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke
      ? window.__TAURI_INTERNALS__.invoke('plugin:playwright|callback', {{ id: '{}', ok: true, result: __r }})
      : window.__TAURI__.invoke('plugin:playwright|callback', {{ id: '{}', ok: true, result: __r }});
  }} catch(e) {{
    var invokeFn = (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke)
      ? window.__TAURI_INTERNALS__.invoke
      : window.__TAURI__.invoke;
    invokeFn('plugin:playwright|callback', {{ id: '{}', ok: false, error: e.message || String(e) }});
  }}
}})()"#,
        escaped_js, id, id, id
    );

    // Execute in the main WebView
    if let Some(window) = bridge.app_handle.get_webview_window("main") {
        window
            .eval(&invoke_js)
            .map_err(|e| format!("eval failed: {}", e))?;
    } else {
        return Err("main window not found".to_string());
    }

    // Wait for the callback (with 30s timeout)
    match tokio::time::timeout(Duration::from_secs(30), rx).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(_)) => Err("callback sender dropped".to_string()),
        Err(_) => {
            // Clean up the pending entry on timeout
            let mut pending = bridge.pending.lock().await;
            pending.remove(&id);
            Err("eval timed out after 30s".to_string())
        }
    }
}

/// Evaluate JavaScript in the WebView without waiting for a return value.
///
/// Use this for fire-and-forget operations (click, fill, scroll).
pub fn eval_js_void(js: &str) -> Result<(), String> {
    let bridge = get_bridge()?;
    if let Some(window) = bridge.app_handle.get_webview_window("main") {
        window
            .eval(js)
            .map_err(|e| format!("eval failed: {}", e))
    } else {
        Err("main window not found".to_string())
    }
}

/// The callback Tauri command — called from JS after eval completes.
#[tauri::command]
pub async fn callback(
    id: String,
    ok: bool,
    result: Option<serde_json::Value>,
    error: Option<String>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|_| format!("invalid callback id: {}", id))?;
    let mut pending = pending_map().lock().await;
    if let Some(sender) = pending.remove(&uuid) {
        let value = if ok {
            result.unwrap_or(serde_json::Value::Null)
        } else {
            serde_json::json!({ "__error": error.unwrap_or_else(|| "unknown JS error".into()) })
        };
        let _ = sender.send(value);
    }
    Ok(())
}
