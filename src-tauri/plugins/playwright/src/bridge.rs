use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use tauri::Manager;
use tokio::sync::oneshot;

/// ── Bridge State ──
///
/// The eval bridge uses Tauri 2's native `eval_with_callback` which
/// executes JS in the WebView and invokes a Rust closure with the result.
/// This avoids the manual `invoke` callback pattern that doesn't work
/// reliably across Tauri 2 IPC changes.

static BRIDGE: OnceLock<tauri::AppHandle> = OnceLock::new();

pub fn init_bridge(app_handle: tauri::AppHandle) {
    BRIDGE.set(app_handle).ok();
}

fn get_app_handle() -> Result<&'static tauri::AppHandle, String> {
    BRIDGE
        .get()
        .ok_or_else(|| "playwright bridge not initialized".to_string())
}

/// Evaluate JavaScript in the WebView and return the parsed result.
///
/// The JS must return a JSON-stringifiable value. We wrap it in a
/// try/catch that always returns a JSON envelope: `{ok, data?, error?}`.
pub async fn eval_js(js: &str) -> Result<serde_json::Value, String> {
    let (tx, rx) = oneshot::channel();
    let tx = Arc::new(Mutex::new(Some(tx)));

    // Only escape backslashes. Single quotes and newlines are fine since
    // the JS runs inside a function body (delimited by {}), not a string literal.
    // Newlines are valid JS whitespace.
    let escaped_js = js.replace('\\', "\\\\");

    // Return a plain object — eval_with_callback will JSON-stringify it once.
    // If we JSON.stringify here, it gets double-stringified.
    let wrapped = format!(
        "(function(){{ try {{ var __r = (function(){{ {} }})(); return {{ok:true,data:__r===undefined?null:__r}}; }} catch(e) {{ return {{ok:false,error:e.message||String(e)}}; }} }})()",
        escaped_js
    );

    let app_handle = get_app_handle()?;

    if let Some(window) = app_handle.get_webview_window("main") {
        let tx_clone = tx.clone();
        window
            .eval_with_callback(&wrapped, move |result: String| {
                // eprintln!("[playwright:eval] raw result (len={}): {}", result.len(), &result[..result.len().min(200)]);
                let parsed: serde_json::Value = serde_json::from_str(&result)
                    .unwrap_or(serde_json::json!({"ok": true, "data": result}));
                if let Ok(mut guard) = tx_clone.lock() {
                    if let Some(sender) = guard.take() {
                        let _ = sender.send(parsed);
                    }
                }
            })
            .map_err(|e| format!("eval_with_callback failed: {}", e))?;
    } else {
        return Err("main window not found".to_string());
    }

    match tokio::time::timeout(Duration::from_secs(30), rx).await {
        Ok(Ok(envelope)) => {
            // eprintln!("[playwright:eval] envelope type: {:?}", envelope);
            if let Some(obj) = envelope.as_object() {
                let ok = obj.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                // eprintln!("[playwright:eval] ok={}, keys={:?}", ok, obj.keys().collect::<Vec<_>>());
                if ok {
                    let data = obj.get("data").cloned().unwrap_or(serde_json::Value::Null);
                    // eprintln!("[playwright:eval] returning data type: {:?}", data);
                    return Ok(data);
                }
                return Err(
                    obj.get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown JS error")
                        .to_string(),
                );
            }
            // eprintln!("[playwright:eval] envelope is not an object, returning as-is");
            Ok(envelope)
        }
        Ok(Err(_)) => Err("eval callback dropped".to_string()),
        Err(_) => Err("eval timed out after 30s".to_string()),
    }
}

/// Evaluate JavaScript in the WebView without waiting for a return value.
pub fn eval_js_void(js: &str) -> Result<(), String> {
    let app_handle = get_app_handle()?;
    if let Some(window) = app_handle.get_webview_window("main") {
        window
            .eval(js)
            .map_err(|e| format!("eval failed: {}", e))
    } else {
        Err("main window not found".to_string())
    }
}
