//! 远程 MySQL 数据库后端
//!
//! 通过 REST API 实现 [`DbBackend`] trait。
//! 所有操作转换为 HTTP 请求，由远程服务处理实际的数据库操作。
//!
//! ## REST API 约定
//! | 操作 | Method | URL | Body |
//! |------|--------|-----|------|
//! | find_by_id | GET | `{base}/db/{table}/{id}` | — |
//! | find_all | POST | `{base}/db/{table}/query` | QueryParams JSON |
//! | insert | POST | `{base}/db/{table}` | data JSON |
//! | update | PUT | `{base}/db/{table}/{id}` | data JSON |
//! | delete | DELETE | `{base}/db/{table}/{id}` | — |
//! | execute_raw | POST | `{base}/db/execute` | { sql } JSON |

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{DbBackend, DbRow, QueryParams, QueryResult};

/// 远程 MySQL 后端
///
/// 封装 reqwest HTTP 客户端，通过 REST API 操作远程数据库。
/// SSRF 防护：禁用重定向 + 校验 host。
pub struct RemoteMySqlDb {
    base_url: String,
    base_host: String,
    token: Option<String>,
    client: reqwest::Client,
}

impl RemoteMySqlDb {
    /// 创建新的远程数据库后端实例
    ///
    /// `base_url`: REST API 根地址
    /// `token`: 可选的 Bearer token
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

    /// 安全构建请求 URL，校验 host 防止 SSRF
    fn url(&self, path: &str) -> Result<String, String> {
        let full = format!("{}{path}", self.base_url);
        let parsed = reqwest::Url::parse(&full).map_err(|e| format!("URL 无效: {e}"))?;
        if parsed.host_str() != Some(&self.base_host) {
            return Err("URL 主机不匹配".into());
        }
        Ok(parsed.to_string())
    }

    /// 添加认证头
    fn auth_header(&self) -> Option<String> {
        self.token.as_ref().map(|t| format!("Bearer {}", t))
    }

    async fn get(&self, path: &str) -> Result<reqwest::Response, String> {
        let url = self.url(path)?;
        let mut req = self.client.get(url);
        if let Some(ref auth) = self.auth_header() {
            req = req.header("Authorization", auth);
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
        if let Some(ref auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        req.send()
            .await
            .map_err(|e| format!("请求失败: {e}"))?
            .error_for_status()
            .map_err(|e| format!("HTTP 错误: {e}"))
    }

    async fn put(&self, path: &str, body: &Value) -> Result<reqwest::Response, String> {
        let url = self.url(path)?;
        let mut req = self.client.put(url).json(body);
        if let Some(ref auth) = self.auth_header() {
            req = req.header("Authorization", auth);
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
        if let Some(ref auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        req.send()
            .await
            .map_err(|e| format!("请求失败: {e}"))?
            .error_for_status()
            .map_err(|e| format!("HTTP 错误: {e}"))
    }
}

#[async_trait]
impl DbBackend for RemoteMySqlDb {
    async fn find_by_id(&self, table: &str, id: &str) -> Result<Option<DbRow>, String> {
        #[derive(Deserialize)]
        struct FindByIdResp {
            data: Option<DbRow>,
        }
        let resp = self
            .get(&format!("/db/{}/{}", urlencoding(table), urlencoding(id)))
            .await?;
        let body: FindByIdResp = resp
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {e}"))?;
        Ok(body.data)
    }

    async fn find_all(&self, table: &str, params: QueryParams) -> Result<QueryResult, String> {
        let resp = self
            .post(
                &format!("/db/{}/query", urlencoding(table)),
                &serde_json::to_value(&params).map_err(|e| format!("序列化失败: {e}"))?,
            )
            .await?;
        resp.json::<QueryResult>()
            .await
            .map_err(|e| format!("解析响应失败: {e}"))
    }

    async fn insert(&self, table: &str, data: DbRow) -> Result<String, String> {
        #[derive(Deserialize)]
        struct InsertResp {
            id: String,
        }
        let resp = self
            .post(
                &format!("/db/{}", urlencoding(table)),
                &serde_json::to_value(&data).map_err(|e| format!("序列化失败: {e}"))?,
            )
            .await?;
        let body: InsertResp = resp
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {e}"))?;
        Ok(body.id)
    }

    async fn update(&self, table: &str, id: &str, data: DbRow) -> Result<(), String> {
        self.put(
            &format!("/db/{}/{}", urlencoding(table), urlencoding(id)),
            &serde_json::to_value(&data).map_err(|e| format!("序列化失败: {e}"))?,
        )
        .await?;
        Ok(())
    }

    async fn delete(&self, table: &str, id: &str) -> Result<(), String> {
        self.delete_req(&format!("/db/{}/{}", urlencoding(table), urlencoding(id)))
            .await?;
        Ok(())
    }

    async fn execute_raw(&self, sql: &str) -> Result<QueryResult, String> {
        #[derive(Serialize)]
        struct ExecuteBody {
            sql: String,
        }
        let resp = self
            .post(
                "/db/execute",
                &serde_json::to_value(&ExecuteBody {
                    sql: sql.to_string(),
                })
                .map_err(|e| format!("序列化失败: {e}"))?,
            )
            .await?;
        resp.json::<QueryResult>()
            .await
            .map_err(|e| format!("解析响应失败: {e}"))
    }
}

/// 简易 URL 编码
fn urlencoding(s: &str) -> String {
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
