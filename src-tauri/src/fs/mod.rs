//! 文件系统抽象模块
//!
//! 提供统一的文件操作接口：
//! - [`FsBackend`] trait — 6 个异步方法，Local / Remote 两种实现
//! - 通用命令（`fs_read` / `fs_write` 等）通过 `fs_type` 参数切换后端
//! - 业务命令（项目创建、配置读写等）直接操作本地文件系统
//!
//! ## 架构
//! ```text
//! 前端 invoke()
//!   ├── fs_read / fs_write / fs_list / fs_delete / fs_exists / fs_mkdir
//!   │     └── dispatch! macro → LocalFs 或 RemoteFs（由 fs_type 决定）
//!   ├── read_local_file / load_settings / save_settings / create_project / open_project
//!   │     └── 直接使用 std::fs（不受 base_dir 限制）
//!   └── fetch_remote
//!         └── 直接使用 reqwest（通用 HTTP GET）
//! ```

pub mod local;
pub mod remote;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use tokio::sync::Mutex;

use local::LocalFs;
use remote::RemoteFs;

// ─── Types ────────────────────────────────────────────────────

/// 文件系统后端抽象 trait
///
/// 定义了 6 个统一的文件操作方法。每种操作有 Local 和 Remote 两种实现：
/// - `LocalFs` 封装 `std::fs`
/// - `RemoteFs` 封装 `reqwest` HTTP 调用
///
/// 所有方法均为 async，内部由 `tokio` 运行时驱动。
#[async_trait]
pub trait FsBackend: Send + Sync {
    /// 读取文件内容（UTF-8 字符串）
    async fn read_file(&self, path: &str) -> Result<String, String>;
    /// 写入文件内容，自动创建父目录
    async fn write_file(&self, path: &str, content: &str) -> Result<(), String>;
    /// 列出目录下的所有文件/子目录条目
    async fn list_dir(&self, path: &str) -> Result<Vec<FileEntry>, String>;
    /// 删除文件或目录
    async fn delete(&self, path: &str) -> Result<(), String>;
    /// 检查路径是否存在
    async fn exists(&self, path: &str) -> Result<bool, String>;
    /// 递归创建目录
    async fn create_dir(&self, path: &str) -> Result<(), String>;
}

/// 文件/目录条目
///
/// 前端文件树展示用的最小信息单元。
#[derive(Serialize, Deserialize, Clone)]
pub struct FileEntry {
    /// 文件/目录名（不含路径）
    pub name: String,
    /// 完整路径
    pub path: String,
    /// 是否为目录
    pub is_dir: bool,
    /// 文件大小（字节）
    pub size: u64,
    /// 最后修改时间（Unix 时间戳字符串）
    pub modified: String,
}

/// 本地文件读取结果
///
/// 封装文件名和内容，供前端 Tab 编辑器展示使用。
#[derive(Serialize)]
pub struct FileContent {
    pub name: String,
    pub content: String,
}

/// 远程 HTTP 请求响应结果
///
/// 封装 HTTP 状态码和响应体文本。
#[derive(Serialize)]
pub struct RemoteResponse {
    /// HTTP 状态码（如 200、404）
    pub status: u16,
    /// 响应体文本
    pub body: String,
}

/// .ab 项目文件结构
///
/// ArchBot 的项目文件采用 YAML 格式存储，此结构定义了文件内容。
/// 通过 `serde_yml` 进行序列化和反序列化。
#[derive(Serialize, Deserialize)]
pub struct AbProject {
    pub name: String,
    pub version: String,
    pub description: String,
    /// ISO 8601 格式的创建时间
    pub created_at: String,
}

// ─── Global backend instances ──────────────────────────────────

/// LocalFs 全局单例（tokio::sync::Mutex 保证跨 await Send 安全）
static LOCAL: OnceLock<Mutex<LocalFs>> = OnceLock::new();
/// RemoteFs 全局单例（Option 表示可能未配置）
static REMOTE: OnceLock<Mutex<Option<RemoteFs>>> = OnceLock::new();

/// 获取或初始化 LocalFs 单例
///
/// 首次调用时以用户主目录作为 base_dir 创建实例。
/// 后续可通过 `fs_configure_local` 重新配置。
fn local_cell() -> &'static Mutex<LocalFs> {
    LOCAL.get_or_init(|| {
        let home = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/"))
            .to_string_lossy()
            .to_string();
        Mutex::new(LocalFs::new(&home))
    })
}

/// 获取或初始化 RemoteFs 单例容器
///
/// 首次调用时内部为 None，需先调用 `fs_configure_remote` 配置。
fn remote_cell() -> &'static Mutex<Option<RemoteFs>> {
    REMOTE.get_or_init(|| Mutex::new(None))
}

// ─── Dispatch helper ──────────────────────────────────────────

/// 根据 `fs_type` 派发到正确的后端并调用指定方法
///
/// `fs_type` 为 `"local"` 时走 LocalFs，`"remote"` 时走 RemoteFs。
/// Remote 后端未配置时返回中文错误提示。
macro_rules! dispatch {
    ($fs_type:expr, $method:ident, $($arg:expr),*) => {{
        match $fs_type.as_str() {
            "local" => {
                let guard = local_cell().lock().await;
                guard.$method($($arg),*).await
            }
            "remote" => {
                let guard = remote_cell().lock().await;
                let backend = guard.as_ref().ok_or("远程后端未配置，请先调用 fs_configure_remote")?;
                backend.$method($($arg),*).await
            }
            _ => Err(format!("未知的 fs_type: {}", $fs_type))
        }
    }};
}

// ─── Settings path ────────────────────────────────────────────

/// 获取系统配置文件路径：`~/.ArchBot/settings.json`
fn get_settings_path() -> Result<std::path::PathBuf, String> {
    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    Ok(home.join(".ArchBot").join("settings.json"))
}

/// 获取当前 ISO 8601 格式时间戳
fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

// ─── ArchBot project directory ──────────────────────────────────

/// 获取 `.archbot` 目录路径
///
/// `project_path` 为 `.ab` 文件的完整路径，`.archbot/` 位于其同级目录。
fn get_archbot_dir(project_path: &str) -> Result<std::path::PathBuf, String> {
    let ab_path = std::path::Path::new(project_path);
    let project_dir = ab_path.parent().ok_or("无法获取项目目录")?;
    Ok(project_dir.join(".archbot"))
}

/// `.archbot/` 骨架目录列表
const ARCHBOT_SKELETON_DIRS: &[&str] = &[
    "digital-employees",
    "business-flows",
    "skills",
    "agents",
    "context/rules",
    "context/memory",
    "mcp",
    "db",
];

/// 初始化项目 `.archbot/` 目录骨架
///
/// 业务逻辑：
/// 1. 解析 `.ab` 文件路径 → 计算项目目录
/// 2. 创建 `.archbot/` 及其所有子目录
/// 3. 创建默认 `scenario.yml`（空场景配置）
/// 4. 幂等：已存在的目录不报错
#[tauri::command]
pub fn init_archbot_dir(project_path: String) -> Result<(), String> {
    let archbot_dir = get_archbot_dir(&project_path)?;

    for subdir in ARCHBOT_SKELETON_DIRS {
        let dir = archbot_dir.join(subdir);
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("创建目录失败 {}: {e}", dir.display()))?;
    }

    // 创建默认 scenario.yml（空场景，用户后续可在面板中设置）
    let scenario_path = archbot_dir.join("scenario.yml");
    if !scenario_path.exists() {
        let default_scenario = "type: \"\"\noverrides: {}\napplied_at: \"\"\n";
        std::fs::write(&scenario_path, default_scenario)
            .map_err(|e| format!("创建 scenario.yml 失败: {e}"))?;
    }

    Ok(())
}

/// 确保项目 `.gitignore` 中排除 `.archbot/db`
///
/// 业务逻辑：
/// 1. 查找项目目录下的 `.gitignore` 文件
/// 2. 不存在则创建并写入 `.archbot/db`
/// 3. 存在但不含 `.archbot/db` 则追加
/// 4. 已包含则不操作
/// 返回 `true` 表示进行了修改，`false` 表示无需修改
#[tauri::command]
pub fn ensure_gitignore(project_path: String) -> Result<bool, String> {
    let ab_path = std::path::Path::new(&project_path);
    let project_dir = ab_path.parent().ok_or("无法获取项目目录")?;
    let gitignore_path = project_dir.join(".gitignore");
    let pattern = ".archbot/db";

    if gitignore_path.exists() {
        let content =
            std::fs::read_to_string(&gitignore_path).map_err(|e| format!("读取 .gitignore 失败: {e}"))?;
        if content.lines().any(|line| line.trim() == pattern) {
            return Ok(false);
        }
        let mut new_content = content;
        if !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str(&format!("\n# ArchBot database files\n{pattern}\n"));
        std::fs::write(&gitignore_path, new_content)
            .map_err(|e| format!("写入 .gitignore 失败: {e}"))?;
    } else {
        std::fs::write(&gitignore_path, format!("# ArchBot database files\n{pattern}\n"))
            .map_err(|e| format!("创建 .gitignore 失败: {e}"))?;
    }

    Ok(true)
}

// ═══════════════════════════════════════════════════════════════
// Tauri Commands — 通用文件操作（通过 fs_type 切换 local/remote）
// ═══════════════════════════════════════════════════════════════

/// 配置本地文件系统后端的基础目录
///
/// 此后所有 `fs_type="local"` 的操作都在该目录下进行，
/// 通过 `canonicalize` + `starts_with` 防止路径穿越。
#[tauri::command]
pub async fn fs_configure_local(base_dir: String) -> Result<(), String> {
    let mut guard = local_cell().lock().await;
    *guard = LocalFs::new(&base_dir);
    Ok(())
}

/// 配置远程文件系统后端
///
/// 参数 `base_url` 为远程 REST API 的根地址。
/// 可选 `token` 用作 Bearer Authorization 头。
/// 配置后所有 `fs_type="remote"` 的操作通过此连接执行。
#[tauri::command]
pub async fn fs_configure_remote(base_url: String, token: Option<String>) -> Result<(), String> {
    let mut guard = remote_cell().lock().await;
    *guard = Some(RemoteFs::new(&base_url, token.as_deref()));
    Ok(())
}

/// 读取文件内容（通用）
///
/// `fs_type`: `"local"` 读取本地文件，`"remote"` 通过 REST API 读取。
#[tauri::command]
pub async fn fs_read(path: String, fs_type: String) -> Result<String, String> {
    dispatch!(fs_type, read_file, &path)
}

/// 写入文件内容（通用）
#[tauri::command]
pub async fn fs_write(path: String, content: String, fs_type: String) -> Result<(), String> {
    dispatch!(fs_type, write_file, &path, &content)
}

/// 列出目录内容（通用）
#[tauri::command]
pub async fn fs_list(path: String, fs_type: String) -> Result<Vec<FileEntry>, String> {
    dispatch!(fs_type, list_dir, &path)
}

/// 删除文件或目录（通用）
#[tauri::command]
pub async fn fs_delete(path: String, fs_type: String) -> Result<(), String> {
    dispatch!(fs_type, delete, &path)
}

/// 检查路径是否存在（通用）
#[tauri::command]
pub async fn fs_exists(path: String, fs_type: String) -> Result<bool, String> {
    dispatch!(fs_type, exists, &path)
}

/// 创建目录（通用）
#[tauri::command]
pub async fn fs_mkdir(path: String, fs_type: String) -> Result<(), String> {
    dispatch!(fs_type, create_dir, &path)
}

// ═══════════════════════════════════════════════════════════════
// Tauri Commands — 本地文件操作（直接使用 std::fs）
// ═══════════════════════════════════════════════════════════════

/// 读取本地文件内容
///
/// 业务逻辑：
/// 1. 从传入的绝对路径提取文件名
/// 2. 读取文件 UTF-8 文本内容
/// 3. 返回文件名和内容供前端 Tab 编辑器展示
///
/// 此命令读取任意路径的文件，不受 LocalFs base_dir 限制。
/// 使用 canonicalize 防止 `..` 路径穿越。
#[tauri::command]
pub async fn read_local_file(path: String) -> Result<FileContent, String> {
    let raw = std::path::Path::new(&path);
    let canonical = raw
        .canonicalize()
        .map_err(|e| format!("路径无效: {e}"))?;
    let name = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let content =
        std::fs::read_to_string(&canonical).map_err(|e| format!("读取文件失败: {e}"))?;
    Ok(FileContent { name, content })
}

/// 读取用户配置
///
/// 业务逻辑：
/// 1. 获取 ~/.ArchBot/settings.json 路径
/// 2. 文件不存在时返回空字符串（表示使用默认配置）
/// 3. 文件存在时读取 JSON 内容返回给前端
#[tauri::command]
pub async fn load_settings() -> Result<String, String> {
    let path = get_settings_path()?;
    if !path.exists() {
        return Ok(String::new());
    }
    std::fs::read_to_string(&path).map_err(|e| format!("读取配置失败: {e}"))
}

/// 保存用户配置
///
/// 业务逻辑：
/// 1. 获取 ~/.ArchBot 目录路径
/// 2. 目录不存在则递归创建
/// 3. 将前端传入的配置 JSON 字符串写入 settings.json
#[tauri::command]
pub async fn save_settings(content: String) -> Result<(), String> {
    let path = get_settings_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {e}"))?;
    }
    std::fs::write(&path, content).map_err(|e| format!("保存配置失败: {e}"))
}

/// 创建 .ab 项目文件
///
/// 业务逻辑：
/// 1. canonicalize 解析目录路径（防 .. 穿越）
/// 2. 校验项目名称非空且不含非法字符（/\:*?"<>|）
/// 3. 拼接完整路径：dir/name.ab
/// 4. 检查文件是否已存在，避免覆盖
/// 5. 生成默认项目 YAML 内容（name + version + description + created_at）
/// 6. 通过 serde_yml 序列化后写入文件
/// 7. 返回完整文件路径
#[tauri::command]
pub async fn create_project(dir: String, name: String) -> Result<String, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("项目名称不能为空".into());
    }

    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    if name.chars().any(|c| invalid_chars.contains(&c)) {
        return Err(format!(
            "项目名称不能包含以下字符: {}",
            invalid_chars.iter().collect::<String>()
        ));
    }

    let filename = if name.ends_with(".ab") {
        name.clone()
    } else {
        format!("{name}.ab")
    };

    // canonicalize 防 .. 目录穿越
    let dir_path = std::path::Path::new(&dir)
        .canonicalize()
        .map_err(|e| format!("目录路径无效: {e}"))?;
    let path = dir_path.join(&filename);

    if path.exists() {
        return Err(format!("文件已存在: {}", path.display()));
    }

    let now = now_iso();
    let project = AbProject {
        name: name.trim_end_matches(".ab").to_string(),
        version: "1.0.0".to_string(),
        description: String::new(),
        created_at: now,
    };

    let yaml = serde_yml::to_string(&project).map_err(|e| format!("生成项目文件失败: {e}"))?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }

    std::fs::write(&path, &yaml).map_err(|e| format!("写入文件失败: {e}"))?;

    Ok(path.to_string_lossy().to_string())
}

/// 打开并校验 .ab 项目文件
///
/// 业务逻辑：
/// 1. 扩展名校验 —— 仅接受 .ab 文件
/// 2. canonicalize 解析路径中的 .. / . / 符号链接，防止目录穿越
/// 3. 读取文件内容
/// 4. serde_yml 反序列化校验 YAML 格式是否合法
/// 5. 校验必填字段（name、version）非空
/// 6. 返回文件名和原始内容供前端 store 存储
#[tauri::command]
pub async fn open_project(path: String) -> Result<FileContent, String> {
    let raw = std::path::Path::new(&path);

    if raw.extension().map(|e| e != "ab").unwrap_or(true) {
        return Err("仅支持打开 .ab 项目文件".into());
    }

    let canonical = raw
        .canonicalize()
        .map_err(|e| format!("项目路径无效: {e}"))?;

    if !canonical.is_file() {
        return Err("项目路径不是一个有效的文件".into());
    }

    let content =
        std::fs::read_to_string(&canonical).map_err(|e| format!("读取项目文件失败: {e}"))?;

    let project: AbProject =
        serde_yml::from_str(&content).map_err(|e| format!("项目文件格式错误: {e}"))?;

    if project.name.is_empty() {
        return Err("项目文件缺少 name 字段".into());
    }
    if project.version.is_empty() {
        return Err("项目文件缺少 version 字段".into());
    }

    let name = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(FileContent { name, content })
}

// ═══════════════════════════════════════════════════════════════
// Tauri Commands — 远程操作
// ═══════════════════════════════════════════════════════════════

/// 请求远程地址并返回响应
///
/// 业务逻辑：
/// 1. 创建 HTTP 客户端
/// 2. 使用 Basic Auth（用户名+密码）向目标 URL 发起 GET 请求
/// 3. 提取 HTTP 状态码和响应体文本
/// 4. 任一步骤失败则返回中文错误信息供前端提示
///
/// 此命令发送请求到任意 URL，不受 RemoteFs base_url 限制。
///
/// 安全措施：禁用 HTTP 重定向防止 SSRF，仅允许 https 协议。
#[tauri::command]
pub async fn fetch_remote(
    url: String,
    username: String,
    password: String,
) -> Result<RemoteResponse, String> {
    // 校验协议，拒绝非 https URL
    let parsed =
        reqwest::Url::parse(&url).map_err(|e| format!("URL 无效: {e}"))?;
    if parsed.scheme() != "https" {
        return Err("仅支持 https 协议".into());
    }
    // 拒绝内网地址
    if let Some(host) = parsed.host_str() {
        if host == "localhost" || host.starts_with("127.") || host.starts_with("10.")
            || host.starts_with("172.16.") || host.starts_with("192.168.")
        {
            return Err("不允许访问内网地址".into());
        }
    }

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| format!("创建客户端失败: {e}"))?;
    let resp = client
        .get(parsed)
        .basic_auth(&username, Some(&password))
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?;

    let status = resp.status().as_u16();
    let body = resp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {e}"))?;

    Ok(RemoteResponse { status, body })
}
