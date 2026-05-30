use serde::Deserialize;
use tower_http::cors::CorsLayer;
use tower_http::cors::Any;

/// Configuration for the embedded HTTP server, deserialized from settings.json.
#[derive(Deserialize)]
pub struct HttpConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_port")]
    pub port: u16,
    /// When true, bind to `0.0.0.0` for LAN access instead of `127.0.0.1`.
    #[serde(default)]
    pub bind_lan: bool,
}

fn default_port() -> u16 { 1421 }

impl Default for HttpConfig {
    fn default() -> Self {
        Self { enabled: false, port: 1421, bind_lan: false }
    }
}

/// Start the Axum HTTP server in a background tokio task.
///
/// Security notes:
/// - `db_execute_raw` and `fs_configure_local` are excluded from the HTTP API.
/// - CORS applies `Any` only when `bind_lan: true` (user explicitly enables LAN access).
///   When localhost-only, CORS restricts to the Vite dev server origin.
pub async fn start(config: HttpConfig) -> tokio::task::JoinHandle<()> {
    let bind_address = if config.bind_lan { "0.0.0.0" } else { "127.0.0.1" };

    let cors = if config.bind_lan {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        CorsLayer::new()
            .allow_origin("http://localhost:1420".parse::<axum::http::HeaderValue>().unwrap())
            .allow_methods(Any)
            .allow_headers(Any)
    };

    let app = crate::handlers::router().layer(cors);
    let addr = format!("{bind_address}:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind HTTP server address");

    println!("[ArchBot] HTTP API server listening on http://{addr}");

    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("[ArchBot] HTTP server error: {e}");
        }
    })
}
