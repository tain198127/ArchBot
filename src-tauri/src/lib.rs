use serde::{Deserialize, Serialize};

/// 本地文件读取结果
#[derive(Serialize)]
struct FileContent {
    name: String,
    content: String,
}

/// 远程请求响应结果
#[derive(Serialize)]
struct RemoteResponse {
    status: u16,
    body: String,
}

/// 读取本地文件内容
///
/// 根据前端传入的文件绝对路径，提取文件名并读取文本内容，
/// 返回文件名和内容供前端展示。
#[tauri::command]
async fn read_local_file(path: String) -> Result<FileContent, String> {
    let name = std::path::Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取文件失败: {e}"))?;
    Ok(FileContent { name, content })
}

/// 请求远程地址并返回响应
///
/// 业务逻辑：
/// 1. 创建 HTTP 客户端
/// 2. 使用 Basic Auth（用户名+密码）向目标 URL 发起 GET 请求
/// 3. 提取 HTTP 状态码和响应体文本
/// 4. 任一步骤失败则返回中文错误信息供前端提示
#[tauri::command]
async fn fetch_remote(url: String, username: String, password: String) -> Result<RemoteResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
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

/// 获取配置文件路径：~/.ArchBot/settings.json
fn get_settings_path() -> Result<std::path::PathBuf, String> {
    let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
    Ok(home.join(".ArchBot").join("settings.json"))
}

/// 读取用户配置
///
/// 从 ~/.ArchBot/settings.json 读取配置，
/// 文件不存在时返回空字符串表示使用默认配置。
#[tauri::command]
async fn load_settings() -> Result<String, String> {
    let path = get_settings_path()?;
    if !path.exists() {
        return Ok(String::new());
    }
    std::fs::read_to_string(&path)
        .map_err(|e| format!("读取配置失败: {e}"))
}

/// 保存用户配置
///
/// 业务逻辑：
/// 1. 获取 ~/.ArchBot 目录路径
/// 2. 目录不存在则递归创建
/// 3. 将配置 JSON 写入 settings.json
#[tauri::command]
async fn save_settings(content: String) -> Result<(), String> {
    let path = get_settings_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建配置目录失败: {e}"))?;
    }
    std::fs::write(&path, content)
        .map_err(|e| format!("保存配置失败: {e}"))
}

/// 应用入口：初始化 Tauri 并注册插件和命令
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            read_local_file,
            fetch_remote,
            load_settings,
            save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
