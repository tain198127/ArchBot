//! 远程向量数据库后端
//!
//! 通过 REST API 实现 [`VectorBackend`] trait。
//!
//! ## REST API 约定
//! | 操作 | Method | URL | Body |
//! |------|--------|-----|------|
//! | create_table | POST | `{base}/vector/{name}` | `{ dimension }` |
//! | insert | POST | `{base}/vector/{table}/insert` | `{ id, vector }` |
//! | search | POST | `{base}/vector/{table}/search` | `{ query, top_k }` |
//! | delete | DELETE | `{base}/vector/{table}/{id}` | — |
//! | list_tables | GET | `{base}/vector/tables` | — |
//! | table_info | GET | `{base}/vector/{table}` | — |

use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;

use super::{SearchResult, TableInfo, VectorBackend};

/// 远程向量数据库后端
///
/// 通过 REST API 操作远程向量数据库。
/// SSRF 防护：禁用重定向 + 校验 host。
pub struct RemoteVectorDb {
    base_url: String,
    base_host: String,
    token: Option<String>,
    client: reqwest::Client,
}

impl RemoteVectorDb {
    /// 创建新的远程向量数据库后端
    pub fn new(base_url: &str, token: Option<&str>) -> Self {
        let base = base_url.trim_end_matches('/').to_string();
        let host = reqwest::Url::parse(&base)
            .ok()
            .and_then(|u| u.host_str().map(String::from))
            .unwrap_or_default();
        Self {
            base_url: base,
            base_host: host,
            token: token.map(String::from),
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    fn url(&self, path: &str) -> Result<String, String> {
        let full = format!("{}{path}", self.base_url);
        let parsed = reqwest::Url::parse(&full).map_err(|e| format!("URL 无效: {e}"))?;
        if parsed.host_str() != Some(&self.base_host) {
            return Err("URL 主机不匹配".into());
        }
        Ok(parsed.to_string())
    }

    fn auth(&self) -> Option<String> {
        self.token.as_ref().map(|t| format!("Bearer {t}"))
    }

    async fn get(&self, path: &str) -> Result<reqwest::Response, String> {
        let url = self.url(path)?;
        let mut req = self.client.get(url);
        if let Some(ref a) = self.auth() {
            req = req.header("Authorization", a);
        }
        req.send()
            .await
            .map_err(|e| format!("请求失败: {e}"))?
            .error_for_status()
            .map_err(|e| format!("HTTP 错误: {e}"))
    }

    async fn post(&self, path: &str, body: &Value) -> Result<reqwest::Response, String> {
        let url = self.url(path)?;
        let mut req = self.client.post(url).json(body);
        if let Some(ref a) = self.auth() {
            req = req.header("Authorization", a);
        }
        req.send()
            .await
            .map_err(|e| format!("请求失败: {e}"))?
            .error_for_status()
            .map_err(|e| format!("HTTP 错误: {e}"))
    }

    async fn delete_req(&self, path: &str) -> Result<reqwest::Response, String> {
        let url = self.url(path)?;
        let mut req = self.client.delete(url);
        if let Some(ref a) = self.auth() {
            req = req.header("Authorization", a);
        }
        req.send()
            .await
            .map_err(|e| format!("请求失败: {e}"))?
            .error_for_status()
            .map_err(|e| format!("HTTP 错误: {e}"))
    }
}

fn enc(s: &str) -> String {
    s.replace('%', "%25")
        .replace('/', "%2F")
        .replace('\\', "%5C")
        .replace(' ', "%20")
        .replace('&', "%26")
        .replace('=', "%3D")
        .replace('?', "%3F")
        .replace('#', "%23")
        .replace('@', "%40")
}

#[async_trait]
impl VectorBackend for RemoteVectorDb {
    async fn create_table(&self, name: &str, dimension: u32) -> Result<(), String> {
        #[derive(Serialize)]
        struct Body {
            dimension: u32,
        }
        self.post(
            &format!("/vector/{}", enc(name)),
            &serde_json::to_value(&Body { dimension }).unwrap(),
        )
        .await?;
        Ok(())
    }

    async fn insert(&self, table: &str, id: &str, vector: Vec<f32>) -> Result<(), String> {
        #[derive(Serialize)]
        struct Body<'a> {
            id: &'a str,
            vector: Vec<f32>,
        }
        self.post(
            &format!("/vector/{}/insert", enc(table)),
            &serde_json::to_value(&Body { id, vector }).unwrap(),
        )
        .await?;
        Ok(())
    }

    async fn search(
        &self,
        table: &str,
        query: Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, String> {
        #[derive(Serialize)]
        struct Body {
            query: Vec<f32>,
            top_k: usize,
        }
        let resp = self
            .post(
                &format!("/vector/{}/search", enc(table)),
                &serde_json::to_value(&Body { query, top_k }).unwrap(),
            )
            .await?;
        resp.json().await.map_err(|e| format!("解析响应失败: {e}"))
    }

    async fn delete(&self, table: &str, id: &str) -> Result<(), String> {
        self.delete_req(&format!("/vector/{}/{}", enc(table), enc(id)))
            .await?;
        Ok(())
    }

    async fn list_tables(&self) -> Result<Vec<String>, String> {
        let resp = self.get("/vector/tables").await?;
        resp.json().await.map_err(|e| format!("解析响应失败: {e}"))
    }

    async fn table_info(&self, name: &str) -> Result<TableInfo, String> {
        let resp = self.get(&format!("/vector/{}", enc(name))).await?;
        resp.json().await.map_err(|e| format!("解析响应失败: {e}"))
    }
}
