//! 远程文件系统后端
//!
//! 封装 `reqwest` HTTP 客户端，通过 REST API 操作远程文件。
//! 所有请求的 URL 基于配置的 `base_url` 拼接，并校验最终 host 防止 SSRF。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{FileEntry, FsBackend};

/// 远程文件系统后端
///
/// 通过 REST API 操作远程文件。URL 构建规则：
/// - `GET    {base_url}/fs/read?path=...`
/// - `POST   {base_url}/fs/write`  (body: {path, content})
/// - `GET    {base_url}/fs/list?path=...`
/// - `DELETE {base_url}/fs/delete?path=...`
/// - `GET    {base_url}/fs/exists?path=...`
/// - `POST   {base_url}/fs/mkdir`  (body: {path})
///
/// 安全措施：禁止 HTTP 重定向 + 校验 URL host 与配置一致。
pub struct RemoteFs {
    /// REST API 根地址（不含尾部 /）
    base_url: String,
    /// 从 base_url 解析的主机名，用于 SSRF 防护
    base_host: String,
    /// 可选 Bearer token
    token: Option<String>,
    /// HTTP 客户端（禁用重定向）
    client: reqwest::Client,
}

impl RemoteFs {
    /// 创建新的远程后端实例
    ///
    /// `base_url`: REST API 根地址
    /// `token`: 可选的 Bearer token，用于 Authorization 头
    ///
    /// 初始化时解析并缓存主机名用于后续 SSRF 校验。
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
                // 禁用重定向以防止通过 30x 跳转到内网地址
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    /// 安全构建请求 URL
    ///
    /// 业务逻辑：
    /// 1. 将 path 拼接到 base_url
    /// 2. 用 reqwest::Url::parse 解析完整 URL
    /// 3. 校验解析后的 host 与初始缓存的一致
    ///
    /// 第 3 步是关键安全措施——攻击者可能通过 path 中的 `@evil.com`
    /// 或类似技巧改变请求目标，host 校验可防止此类 SSRF 攻击。
    fn url(&self, path: &str) -> Result<String, String> {
        let full = format!("{}{path}", self.base_url);
        let parsed = reqwest::Url::parse(&full).map_err(|e| format!("URL 无效: {e}"))?;
        if parsed.host_str() != Some(&self.base_host) {
            return Err("URL 主机不匹配".into());
        }
        Ok(parsed.to_string())
    }

    /// 发送 GET 请求
    ///
    /// 自动附带 Bearer token（如有配置），失败时返回中文错误。
    async fn get(&self, path: &str) -> Result<reqwest::Response, String> {
        let url = self.url(path)?;
        let mut req = self.client.get(url);
        if let Some(ref t) = self.token {
            req = req.header("Authorization", format!("Bearer {t}"));
        }
        req.send()
            .await
            .map_err(|e| format!("请求失败: {e}"))?
            .error_for_status()
            .map_err(|e| format!("HTTP 错误: {e}"))
    }

    /// 发送 POST 请求（JSON body）
    async fn post<T: Serialize + ?Sized>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<reqwest::Response, String> {
        let url = self.url(path)?;
        let mut req = self.client.post(url).json(body);
        if let Some(ref t) = self.token {
            req = req.header("Authorization", format!("Bearer {t}"));
        }
        req.send()
            .await
            .map_err(|e| format!("请求失败: {e}"))?
            .error_for_status()
            .map_err(|e| format!("HTTP 错误: {e}"))
    }

    /// 发送 DELETE 请求
    async fn delete_req(&self, path: &str) -> Result<reqwest::Response, String> {
        let url = self.url(path)?;
        let mut req = self.client.delete(url);
        if let Some(ref t) = self.token {
            req = req.header("Authorization", format!("Bearer {t}"));
        }
        req.send()
            .await
            .map_err(|e| format!("请求失败: {e}"))?
            .error_for_status()
            .map_err(|e| format!("HTTP 错误: {e}"))
    }
}

#[async_trait]
impl FsBackend for RemoteFs {
    async fn read_file(&self, path: &str) -> Result<String, String> {
        let resp = self
            .get(&format!("/fs/read?path={}", urlencoding(path)))
            .await?;
        resp.text().await.map_err(|e| format!("读取响应失败: {e}"))
    }

    async fn write_file(&self, path: &str, content: &str) -> Result<(), String> {
        #[derive(Serialize)]
        struct WriteBody<'a> {
            path: &'a str,
            content: &'a str,
        }
        self.post("/fs/write", &WriteBody { path, content }).await?;
        Ok(())
    }

    async fn list_dir(&self, path: &str) -> Result<Vec<FileEntry>, String> {
        let resp = self
            .get(&format!("/fs/list?path={}", urlencoding(path)))
            .await?;
        resp.json::<Vec<FileEntry>>()
            .await
            .map_err(|e| format!("解析响应失败: {e}"))
    }

    async fn delete(&self, path: &str) -> Result<(), String> {
        self.delete_req(&format!("/fs/delete?path={}", urlencoding(path)))
            .await?;
        Ok(())
    }

    async fn exists(&self, path: &str) -> Result<bool, String> {
        #[derive(Deserialize)]
        struct ExistsResp {
            exists: bool,
        }
        let resp = self
            .get(&format!("/fs/exists?path={}", urlencoding(path)))
            .await?;
        let body: ExistsResp = resp
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {e}"))?;
        Ok(body.exists)
    }

    async fn create_dir(&self, path: &str) -> Result<(), String> {
        #[derive(Serialize)]
        struct MkdirBody<'a> {
            path: &'a str,
        }
        self.post("/fs/mkdir", &MkdirBody { path }).await?;
        Ok(())
    }
}

/// 简易 URL 编码
///
/// 对 path 中的特殊字符进行百分号编码，防止注入。
/// 注意：`@` 被编码为 `%40`，防止 URL userinfo 注入。
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
