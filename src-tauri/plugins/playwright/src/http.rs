use axum::extract::Query;
use axum::response::IntoResponse;
use axum::{Json, Router, routing::{get, post}};

use serde::{Deserialize, Serialize};

use super::bridge;

// ── API envelope (matches the project's handlers::ApiResponse pattern) ──

#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    success: bool,
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn api_ok<T: Serialize>(data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        success: true,
        data: Some(data),
        error: None,
    })
}

fn api_err<T: Serialize>(error: impl Into<String>) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        success: false,
        data: None,
        error: Some(error.into()),
    })
}

fn api_ok_empty() -> Json<ApiResponse<()>> {
    Json(ApiResponse {
        success: true,
        data: None,
        error: None,
    })
}

// ── Router ──

pub fn routes() -> Router {
    Router::new()
        .route("/playwright/eval", get(eval_get).post(eval_post))
        .route("/playwright/click", get(click_get).post(click_post))
        .route("/playwright/fill", post(fill))
        .route("/playwright/text", get(text))
        .route("/playwright/attribute", get(attribute))
        .route("/playwright/screenshot", get(screenshot))
        .route("/playwright/count", get(count))
        .route("/playwright/wait", get(wait))
        .route("/playwright/visible", get(visible))
        .route("/playwright/checked", get(checked))
        .route("/playwright/info", get(info))
        .route("/playwright/hover", get(hover_get).post(hover_post))
}

// ── eval ──

#[derive(Deserialize)]
struct EvalQuery {
    js: Option<String>,
}

#[derive(Deserialize)]
struct EvalBody {
    js: String,
}

async fn eval_get(Query(q): Query<EvalQuery>) -> impl IntoResponse {
    let js = match q.js {
        Some(s) => s,
        None => return api_err::<serde_json::Value>("missing 'js' query param").into_response(),
    };
    match bridge::eval_js(&js).await {
        Ok(v) => api_ok(v).into_response(),
        Err(e) => api_err::<serde_json::Value>(e).into_response(),
    }
}

async fn eval_post(Json(body): Json<EvalBody>) -> impl IntoResponse {
    match bridge::eval_js(&body.js).await {
        Ok(v) => api_ok(v).into_response(),
        Err(e) => api_err::<serde_json::Value>(e).into_response(),
    }
}

// ── click ──

#[derive(Deserialize)]
struct SelectorQuery {
    selector: String,
}

#[derive(Deserialize)]
struct SelectorBody {
    selector: String,
}

async fn click_get(Query(q): Query<SelectorQuery>) -> impl IntoResponse {
    let js = format!(
        "var el=document.querySelector('{}'); if(el){{ el.click(); return true; }} return false;",
        escape_js_str(&q.selector)
    );
    match bridge::eval_js(&js).await {
        Ok(v) => api_ok(v).into_response(),
        Err(e) => api_err::<serde_json::Value>(e).into_response(),
    }
}

async fn click_post(Json(body): Json<SelectorBody>) -> impl IntoResponse {
    let js = format!(
        "var el=document.querySelector('{}'); if(el){{ el.click(); return true; }} return false;",
        escape_js_str(&body.selector)
    );
    match bridge::eval_js(&js).await {
        Ok(v) => api_ok(v).into_response(),
        Err(e) => api_err::<serde_json::Value>(e).into_response(),
    }
}

// ── fill ──

#[derive(Deserialize)]
struct FillBody {
    selector: String,
    value: String,
}

async fn fill(Json(body): Json<FillBody>) -> impl IntoResponse {
    let js = format!(
        r#"var el=document.querySelector('{}');
if(!el) return false;
var nativeSetter=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value')?.set||Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype,'value')?.set;
if(nativeSetter){{ nativeSetter.call(el,'{}'); }} else {{ el.value='{}'; }}
el.dispatchEvent(new Event('input',{{bubbles:true}}));
el.dispatchEvent(new Event('change',{{bubbles:true}}));
return true;"#,
        escape_js_str(&body.selector),
        escape_js_str(&body.value),
        escape_js_str(&body.value)
    );
    match bridge::eval_js(&js).await {
        Ok(v) => api_ok(v).into_response(),
        Err(e) => api_err::<serde_json::Value>(e).into_response(),
    }
}

// ── text ──

async fn text(Query(q): Query<SelectorQuery>) -> impl IntoResponse {
    let js = format!(
        "var el=document.querySelector('{}'); return el ? el.textContent : null;",
        escape_js_str(&q.selector)
    );
    match bridge::eval_js(&js).await {
        Ok(v) => api_ok(v).into_response(),
        Err(e) => api_err::<serde_json::Value>(e).into_response(),
    }
}

// ── attribute ──

#[derive(Deserialize)]
struct AttributeQuery {
    selector: String,
    name: String,
}

async fn attribute(Query(q): Query<AttributeQuery>) -> impl IntoResponse {
    let js = format!(
        "var el=document.querySelector('{}'); return el ? el.getAttribute('{}') : null;",
        escape_js_str(&q.selector),
        escape_js_str(&q.name)
    );
    match bridge::eval_js(&js).await {
        Ok(v) => api_ok(v).into_response(),
        Err(e) => api_err::<serde_json::Value>(e).into_response(),
    }
}

// ── count ──

async fn count(Query(q): Query<SelectorQuery>) -> impl IntoResponse {
    let js = format!(
        "return document.querySelectorAll('{}').length;",
        escape_js_str(&q.selector)
    );
    match bridge::eval_js(&js).await {
        Ok(v) => api_ok(v).into_response(),
        Err(e) => api_err::<serde_json::Value>(e).into_response(),
    }
}

// ── wait ──

#[derive(Deserialize)]
struct WaitQuery {
    selector: String,
    #[serde(default = "default_timeout")]
    timeout: u64,
}

fn default_timeout() -> u64 {
    10000
}

async fn wait(Query(q): Query<WaitQuery>) -> impl IntoResponse {
    let js = format!(
        "return document.querySelector('{}') !== null;",
        escape_js_str(&q.selector)
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(q.timeout);
    loop {
        match bridge::eval_js(&js).await {
            Ok(v) if v.as_bool() == Some(true) => return api_ok(()).into_response(),
            Err(_) => { /* retry */ }
            _ => { /* not found yet */ }
        }
        if std::time::Instant::now() > deadline {
            return api_err::<()>(format!(
                "timeout waiting for selector '{}' after {}ms",
                q.selector, q.timeout
            ))
            .into_response();
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
}

// ── visible ──

async fn visible(Query(q): Query<SelectorQuery>) -> impl IntoResponse {
    let js = format!(
        r#"var el=document.querySelector('{}');
if(!el) return false;
var style=window.getComputedStyle(el);
return style.display!=='none' && style.visibility!=='hidden' && (el.offsetParent!==null || el===document.body || el===document.documentElement);"#,
        escape_js_str(&q.selector)
    );
    match bridge::eval_js(&js).await {
        Ok(v) => api_ok(v).into_response(),
        Err(e) => api_err::<serde_json::Value>(e).into_response(),
    }
}

// ── checked ──

async fn checked(Query(q): Query<SelectorQuery>) -> impl IntoResponse {
    let js = format!(
        "var el=document.querySelector('{}'); return el ? !!el.checked : null;",
        escape_js_str(&q.selector)
    );
    match bridge::eval_js(&js).await {
        Ok(v) => api_ok(v).into_response(),
        Err(e) => api_err::<serde_json::Value>(e).into_response(),
    }
}

// ── info ──

#[derive(Serialize)]
struct WindowInfo {
    title: String,
    width: u32,
    height: u32,
    focused: bool,
}

async fn info() -> impl IntoResponse {
    // Window info requires the bridge to be alive
    let info_js = "return { title: document.title, width: window.innerWidth, height: window.innerHeight, focused: document.hasFocus() };";
    match bridge::eval_js(info_js).await {
        Ok(v) => {
            if let Some(obj) = v.as_object() {
                let info = WindowInfo {
                    title: obj
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    width: obj.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    height: obj.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    focused: obj.get("focused").and_then(|v| v.as_bool()).unwrap_or(false),
                };
                api_ok(info).into_response()
            } else {
                api_err::<WindowInfo>("unexpected response format").into_response()
            }
        }
        Err(e) => api_err::<WindowInfo>(e).into_response(),
    }
}

// ── hover ──

async fn hover_get(Query(q): Query<SelectorQuery>) -> impl IntoResponse {
    let js = format!(
        r#"var el=document.querySelector('{}');
if(!el) return false;
el.dispatchEvent(new MouseEvent('mouseenter',{{bubbles:true,cancelable:true}}));
el.dispatchEvent(new MouseEvent('mouseover',{{bubbles:true,cancelable:true}}));
return true;"#,
        escape_js_str(&q.selector)
    );
    match bridge::eval_js(&js).await {
        Ok(v) => api_ok(v).into_response(),
        Err(e) => api_err::<serde_json::Value>(e).into_response(),
    }
}

async fn hover_post(Json(body): Json<SelectorBody>) -> impl IntoResponse {
    let js = format!(
        r#"var el=document.querySelector('{}');
if(!el) return false;
el.dispatchEvent(new MouseEvent('mouseenter',{{bubbles:true,cancelable:true}}));
el.dispatchEvent(new MouseEvent('mouseover',{{bubbles:true,cancelable:true}}));
return true;"#,
        escape_js_str(&body.selector)
    );
    match bridge::eval_js(&js).await {
        Ok(v) => api_ok(v).into_response(),
        Err(e) => api_err::<serde_json::Value>(e).into_response(),
    }
}

// ── screenshot ──

async fn screenshot() -> impl IntoResponse {
    // Use html2canvas-style approach: draw DOM to a canvas via CSS painting
    // For a real implementation, we could use the native window capture.
    // For now, return a lightweight DOM snapshot as JSON.
    let js = r#"
return { url: window.location.href, title: document.title, ready: true };"#;
    match bridge::eval_js(js).await {
        Ok(v) => api_ok(v).into_response(),
        Err(e) => api_err::<serde_json::Value>(format!("screenshot failed: {}", e)).into_response(),
    }
}

// ── helpers ──

fn escape_js_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}
