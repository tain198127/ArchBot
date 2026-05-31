use std::path::PathBuf;

use crate::trace_fmt;

use serde::{Deserialize, Serialize};

use crate::secret::SecretManager;

// ── Data structures ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProvider {
    pub id: String,
    pub name: String,
    pub protocol: String,
    pub base_url: String,
    #[serde(default)]
    pub models: Vec<String>,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub is_builtin: bool,
    /// Whether the API key has been configured in SecretManager
    #[serde(default)]
    pub has_api_key: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct AIProvidersConfig {
    providers: Vec<AIProvider>,
}

// ── Built-in providers ──

fn builtin_providers() -> Vec<AIProvider> {
    vec![
        AIProvider {
            id: "anthropic".into(),
            name: "Anthropic".into(),
            protocol: "anthropic".into(),
            base_url: "https://api.anthropic.com".into(),
            models: vec![
                "claude-opus-4-7".into(),
                "claude-sonnet-4-6".into(),
                "claude-haiku-4-5".into(),
            ],
            default_model: Some("claude-sonnet-4-6".into()),
            is_builtin: true,
            has_api_key: false,
        },
        AIProvider {
            id: "openai".into(),
            name: "OpenAI".into(),
            protocol: "openai".into(),
            base_url: "https://api.openai.com/v1".into(),
            models: vec!["gpt-5.2".into(), "gpt-4.1".into(), "gpt-4.1-mini".into()],
            default_model: Some("gpt-5.2".into()),
            is_builtin: true,
            has_api_key: false,
        },
        AIProvider {
            id: "deepseek".into(),
            name: "DeepSeek".into(),
            protocol: "openai".into(),
            base_url: "https://api.deepseek.com/v1".into(),
            models: vec!["deepseek-chat".into(), "deepseek-reasoner".into()],
            default_model: Some("deepseek-chat".into()),
            is_builtin: true,
            has_api_key: false,
        },
        AIProvider {
            id: "groq".into(),
            name: "Groq".into(),
            protocol: "openai".into(),
            base_url: "https://api.groq.com/openai/v1".into(),
            models: vec![
                "llama-4-scout-17b-16e-instruct".into(),
                "mixtral-8x7b-32768".into(),
            ],
            default_model: Some("llama-4-scout-17b-16e-instruct".into()),
            is_builtin: true,
            has_api_key: false,
        },
        AIProvider {
            id: "openrouter".into(),
            name: "OpenRouter".into(),
            protocol: "openai".into(),
            base_url: "https://openrouter.ai/api/v1".into(),
            models: vec!["anthropic/claude-sonnet-4".into(), "openai/gpt-5.2".into()],
            default_model: Some("anthropic/claude-sonnet-4".into()),
            is_builtin: true,
            has_api_key: false,
        },
        AIProvider {
            id: "mistral".into(),
            name: "Mistral AI".into(),
            protocol: "openai".into(),
            base_url: "https://api.mistral.ai/v1".into(),
            models: vec![
                "mistral-large-latest".into(),
                "mistral-small-latest".into(),
                "codestral-latest".into(),
            ],
            default_model: Some("mistral-large-latest".into()),
            is_builtin: true,
            has_api_key: false,
        },
        AIProvider {
            id: "together".into(),
            name: "Together AI".into(),
            protocol: "openai".into(),
            base_url: "https://api.together.xyz/v1".into(),
            models: vec![
                "meta-llama/Llama-4-Maverick-17B-128E-Instruct-FP8".into(),
                "deepseek-ai/DeepSeek-R1".into(),
            ],
            default_model: Some("meta-llama/Llama-4-Maverick-17B-128E-Instruct-FP8".into()),
            is_builtin: true,
            has_api_key: false,
        },
        AIProvider {
            id: "fireworks".into(),
            name: "Fireworks AI".into(),
            protocol: "openai".into(),
            base_url: "https://api.fireworks.ai/inference/v1".into(),
            models: vec![
                "accounts/fireworks/models/llama4-maverick-instruct-basic".into(),
                "accounts/fireworks/models/deepseek-r1".into(),
            ],
            default_model: Some("accounts/fireworks/models/llama4-maverick-instruct-basic".into()),
            is_builtin: true,
            has_api_key: false,
        },
        AIProvider {
            id: "google".into(),
            name: "Google Gen AI".into(),
            protocol: "openai".into(),
            base_url: "https://generativelanguage.googleapis.com/v1beta/openai".into(),
            models: vec!["gemini-2.5-pro".into(), "gemini-2.5-flash".into()],
            default_model: Some("gemini-2.5-pro".into()),
            is_builtin: true,
            has_api_key: false,
        },
    ]
}

// ── File paths ──

fn providers_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".archbot")
        .join("config")
        .join("ai_providers.yml")
}

// ── Load / Save ──

/// Load all AI providers (built-in + user-configured) — public for use by turn_executor.
pub fn load_providers_raw() -> Result<Vec<AIProvider>, String> {
    let path = providers_config_path();
    let saved: Vec<AIProvider> = if path.exists() {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {:?}: {}", path, e))?;
        let config: AIProvidersConfig = serde_yml::from_str(&content)
            .map_err(|e| format!("Failed to parse ai_providers.yml: {}", e))?;
        config.providers
    } else {
        vec![]
    };

    // Start from saved state (user modifications are the source of truth).
    // Then add any builtins that don't exist in the save file yet
    // (e.g. new builtins added in a future app version).
    let mut providers = saved;
    for builtin in builtin_providers() {
        if !providers.iter().any(|p| p.id == builtin.id) {
            providers.push(builtin);
        } else {
            // Ensure builtin flag stays correct
            if let Some(p) = providers.iter_mut().find(|p| p.id == builtin.id) {
                p.is_builtin = true;
            }
        }
    }

    Ok(providers)
}

fn save_providers(providers: &[AIProvider]) -> Result<(), String> {
    let path = providers_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    // Save all providers so modifications to builtins (models, default, base_url) persist.
    let config = AIProvidersConfig {
        providers: providers.to_vec(),
    };
    let yml = serde_yml::to_string(&config).map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(&path, yml).map_err(|e| format!("Failed to write {:?}: {}", path, e))?;
    Ok(())
}

fn machine_id() -> String {
    hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

// ── Tauri Commands ──

#[tauri::command]
pub fn ai_list_providers() -> Result<Vec<AIProvider>, String> {
    let mut providers = load_providers_raw()?;
    let sm = SecretManager::new(&machine_id())?;
    for p in &mut providers {
        p.has_api_key = sm.exists(&p.id, "api_token");
    }
    Ok(providers)
}

#[tauri::command]
pub fn ai_save_provider(
    id: String,
    name: String,
    protocol: String,
    base_url: String,
    models: Vec<String>,
    default_model: Option<String>,
) -> Result<(), String> {
    eprintln!(
        "[ai_config] save_provider id={} models={:?} default={:?}",
        id, models, default_model
    );
    let mut providers = load_providers_raw()?;

    // Validate protocol
    if protocol != "anthropic" && protocol != "openai" {
        return Err("Protocol must be 'anthropic' or 'openai'".into());
    }

    // Validate URL
    let parsed = url::Url::parse(&base_url).map_err(|e| format!("Invalid URL: {}", e))?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err("Base URL must use http:// or https://".into());
    }
    if parsed.host_str().is_none() {
        return Err("Base URL must include a host".into());
    }

    // Validate default_model is actually in the models list
    if let Some(ref dm) = default_model {
        if !models.contains(dm) {
            return Err(format!("Default model '{}' is not in the models list", dm));
        }
    }

    if let Some(existing) = providers.iter_mut().find(|p| p.id == id) {
        if existing.is_builtin {
            existing.base_url = base_url;
            existing.models = models;
            existing.default_model = default_model;
        } else {
            existing.name = name;
            existing.protocol = protocol;
            existing.base_url = base_url;
            existing.models = models;
            existing.default_model = default_model;
        }
    } else {
        providers.push(AIProvider {
            id: id.clone(),
            name,
            protocol,
            base_url,
            models,
            default_model,
            is_builtin: false,
            has_api_key: false,
        });
    }

    save_providers(&providers)
}

#[tauri::command]
pub fn ai_delete_provider(id: String) -> Result<(), String> {
    let providers = load_providers_raw()?;
    let provider = providers
        .iter()
        .find(|p| p.id == id)
        .ok_or_else(|| format!("Provider not found: {}", id))?;

    if provider.is_builtin {
        return Err("Cannot delete built-in provider".into());
    }

    let remaining: Vec<AIProvider> = providers.into_iter().filter(|p| p.id != id).collect();
    save_providers(&remaining)
}

#[tauri::command]
pub fn ai_save_provider_secret(id: String, key: String) -> Result<(), String> {
    let sm = SecretManager::new(&machine_id())?;
    sm.store(&id, "api_token", &key)
}

/// Validate the host portion of a URL against SSRF targets.
///
/// Allows localhost and private IPs (needed for Ollama, vLLM, self-hosted LLMs)
/// but blocks cloud metadata, unspecified, and link-local addresses.
/// For hostnames, resolves DNS and validates each IP.
async fn validate_host_allowed(host: &str) -> Result<(), String> {
    // Try parsing as a raw IP address first
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        return validate_ip_allowed(&ip);
    }

    // Handle bracketed IPv6 like [::1]
    if host.starts_with('[') && host.ends_with(']') {
        let inner = &host[1..host.len() - 1];
        if let Ok(ip) = inner.parse::<std::net::IpAddr>() {
            return validate_ip_allowed(&ip);
        }
    }

    // Resolve hostname → check all resolved IPs
    // Append :0 to satisfy the SocketAddr parser in lookup_host
    let lookup = format!("{}:0", host);
    let lookup_str = lookup; // extend lifetime
    match tokio::net::lookup_host(lookup_str).await {
        Ok(addrs) => {
            for addr in addrs {
                validate_ip_allowed(&addr.ip())?;
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to resolve host '{}': {}", host, e)),
    }
}

fn validate_ip_allowed(ip: &std::net::IpAddr) -> Result<(), String> {
    match ip {
        std::net::IpAddr::V4(v4) => {
            let octets = v4.octets();
            if v4.is_unspecified() {
                return Err("Requests to 0.0.0.0 are blocked".into());
            }
            if v4.is_loopback() {
                return Ok(()); // allowed: Ollama, local dev servers
            }
            if v4.is_link_local() {
                // 169.254.0.0/16 — block (cloud metadata)
                return Err("Requests to link-local addresses are blocked".into());
            }
            if v4.is_private() {
                return Ok(()); // allowed: self-hosted LLMs on LAN/VPN
            }
            // Special case: exact cloud metadata IP (already covered by link-local,
            // but explicit check for clarity)
            if octets == [169, 254, 169, 254] {
                return Err("Requests to cloud metadata endpoints are blocked".into());
            }
            Ok(())
        }
        std::net::IpAddr::V6(v6) => {
            if v6.is_unspecified() {
                return Err("Requests to the unspecified IPv6 address are blocked".into());
            }
            if v6.is_loopback() {
                return Ok(()); // allowed: ::1
            }
            if v6.is_multicast() {
                return Err("Requests to multicast addresses are blocked".into());
            }
            // Check IPv4-mapped IPv6 (::ffff:x.x.x.x)
            if let Some(v4) = v6.to_ipv4_mapped() {
                return validate_ip_allowed(&std::net::IpAddr::V4(v4));
            }
            Ok(())
        }
    }
}

#[tauri::command]
pub async fn ai_validate_provider(
    id: String,
    protocol: String,
    base_url: String,
    model: String,
) -> Result<ValidateResult, String> {
    trace_fmt!("ai:validate", "START — provider={} protocol={} base_url={}", id, protocol, base_url);

    let sm = SecretManager::new(&machine_id())?;
    let api_key = sm.get(&id, "api_token").unwrap_or_default();

    if api_key.is_empty() {
        trace_fmt!("ai:validate", "FAIL — no API key configured for provider={}", id);
        return Ok(ValidateResult {
            ok: false,
            response: None,
            remote_models: None,
            error: Some("API key not configured. Please save your API key first.".into()),
        });
    }
    trace_fmt!("ai:validate", "API key loaded for provider={}", id);

    let model = if model.is_empty() {
        if protocol == "anthropic" {
            "claude-haiku-4-5"
        } else {
            "gpt-4.1-mini"
        }
        .into()
    } else {
        model
    };

    // Build chat request body per protocol
    let (endpoint, body) = if protocol == "anthropic" {
        let endpoint = format!("{}/v1/messages", base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": model,
            "max_tokens": 256,
            "messages": [{"role": "user", "content": "你是谁？请用一句话简短回答。"}]
        });
        (endpoint, body)
    } else {
        let endpoint = format!("{}/chat/completions", base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": model,
            "max_tokens": 256,
            "messages": [{"role": "user", "content": "你是谁？请用一句话简短回答。"}]
        });
        (endpoint, body)
    };

    // SSRF protection
    let parsed = url::Url::parse(&endpoint).map_err(|e| format!("Invalid URL: {}", e))?;
    let host = parsed.host_str().ok_or("URL has no host")?;
    trace_fmt!("ai:validate", "SSRF check — host={}", host);
    validate_host_allowed(host).await?;
    trace_fmt!("ai:validate", "SSRF check passed");

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    trace_fmt!("ai:validate", "Sending HTTP {} to {}", if protocol == "anthropic" { "POST (anthropic)" } else { "POST (openai)" }, endpoint);
    let req = if protocol == "anthropic" {
        client
            .post(&endpoint)
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
    } else {
        client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
    };

    match req.send().await {
        Ok(resp) => {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_default();
            trace_fmt!("ai:validate", "HTTP response — status={} body_len={}", status.as_u16(), body_text.len());

            if status.is_success() {
                let reply = parse_chat_reply(&protocol, &body_text);
                let remote_models =
                    fetch_remote_models(&protocol, &base_url, &api_key, &client).await;
                trace_fmt!("ai:validate", "SUCCESS — reply={:?} remote_models_count={}", reply, remote_models.len());
                Ok(ValidateResult {
                    ok: true,
                    response: Some(reply),
                    remote_models: Some(remote_models),
                    error: None,
                })
            } else {
                let short = if body_text.len() > 500 {
                    format!("{}...", &body_text[..500])
                } else {
                    body_text
                };
                trace_fmt!("ai:validate", "FAIL — HTTP {} body={}", status.as_u16(), short);
                Ok(ValidateResult {
                    ok: false,
                    response: None,
                    remote_models: None,
                    error: Some(format!("HTTP {} — {}", status.as_u16(), short)),
                })
            }
        }
        Err(e) => {
            trace_fmt!("ai:validate", "FAIL — connection error: {}", e);
            Ok(ValidateResult {
                ok: false,
                response: None,
                remote_models: None,
                error: Some(format!("Connection failed: {}", e)),
            })
        },
    }
}

/// Fetch the list of available model IDs from the provider's models endpoint.
async fn fetch_remote_models(
    protocol: &str,
    base_url: &str,
    api_key: &str,
    client: &reqwest::Client,
) -> Vec<String> {
    let url = if protocol == "anthropic" {
        format!("{}/v1/models", base_url.trim_end_matches('/'))
    } else {
        format!("{}/models", base_url.trim_end_matches('/'))
    };

    let req = if protocol == "anthropic" {
        client
            .get(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
    } else {
        client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
    };

    match req.send().await {
        Ok(resp) => {
            let body = resp.text().await.unwrap_or_default();
            parse_model_ids(protocol, &body)
        }
        Err(_) => vec![],
    }
}

/// Parse model IDs from a /models API response.
fn parse_model_ids(protocol: &str, body: &str) -> Vec<String> {
    let v: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    v["data"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["id"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Extract the assistant's text reply from the chat API response.
fn parse_chat_reply(protocol: &str, body: &str) -> String {
    let v: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return body.to_string(),
    };

    if protocol == "anthropic" {
        v["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|c| c["text"].as_str())
            .unwrap_or(body)
            .to_string()
    } else {
        v["choices"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|c| c["message"]["content"].as_str())
            .unwrap_or(body)
            .to_string()
    }
}

#[derive(Serialize)]
pub struct ValidateResult {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_models: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
