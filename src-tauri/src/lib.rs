mod context;
mod data_standard;
mod db;
mod digital_employee;
mod fs;
mod handlers;
mod lancedb_store;
mod license;
mod resource;
mod scenario;
mod server;
mod vector;

/// 获取当前 ISO 8601 格式 UTC 时间戳
///
/// 供 `fs` 模块的 `create_project` 和 `data_standard` 的文件元信息使用。
pub(crate) fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Read HTTP server configuration from `~/.ArchBot/settings.json`.
///
/// Defaults to disabled on localhost:1421 when the file is missing or
/// the `httpServer` key is absent.
fn load_http_config() -> server::HttpConfig {
    let path = dirs::home_dir()
        .unwrap_or_default()
        .join(".ArchBot")
        .join("settings.json");
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str::<serde_json::Value>(&content)
            .ok()
            .and_then(|v| v.get("httpServer").cloned())
            .and_then(|hs| serde_json::from_value::<server::HttpConfig>(hs).ok())
            .unwrap_or_default(),
        Err(_) => server::HttpConfig::default(),
    }
}

/// 应用入口：初始化 Tauri 运行时并注册所有插件和命令
///
/// 启动流程：
/// 1. 检测 `ARCHBOT_DEBUG` 环境变量，决定是否启用调试模式
/// 2. 检查 license 文件，确定注册状态
/// 3. 初始化 Tauri 插件（opener / dialog / fs）
/// 4. 注册 31 个 Tauri commands（分属于 6 个模块）
/// 5. 启动事件循环
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 调试模式：设置环境变量 ARCHBOT_DEBUG=1 可绕过注册检查
    if std::env::var("ARCHBOT_DEBUG").is_ok_and(|v| v == "1" || v == "true") {
        license::set_debug_mode(true);
    }

    // 启动时从 ~/.ArchBot/license.dat 加载注册状态
    license::check_license_on_startup();

    // HTTP server: read config from settings and conditionally start.
    // In debug builds, force-enable so browser access works without
    // requiring the user to toggle the setting first.
    let mut http_config = load_http_config();
    if cfg!(debug_assertions) {
        http_config.enabled = true;
    }

    if http_config.enabled {
        tauri::async_runtime::spawn(async move {
            server::start(http_config).await;
        });
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // fs — local
            fs::read_local_file,
            fs::load_settings,
            fs::save_settings,
            fs::create_project,
            fs::open_project,
            fs::init_archbot_dir,
            fs::ensure_gitignore,
            // fs — remote
            fs::fetch_remote,
            // fs — generic (local/remote switchable)
            fs::fs_configure_local,
            fs::fs_configure_remote,
            fs::fs_read,
            fs::fs_write,
            fs::fs_list,
            fs::fs_delete,
            fs::fs_exists,
            fs::fs_mkdir,
            // data_standard
            data_standard::ds_create_domain,
            data_standard::ds_list_domains,
            data_standard::ds_load_domain,
            data_standard::ds_load_conventions,
            data_standard::ds_list_entities,
            data_standard::ds_save_entity,
            data_standard::ds_delete_entity,
            data_standard::ds_list_enums,
            data_standard::ds_save_enum,
            data_standard::ds_delete_enum,
            // license
            license::get_machine_id_cmd,
            license::register_software,
            license::get_license_status,
            // db
            db::db_connect,
            db::db_disconnect,
            db::db_configure_remote,
            db::db_find_all,
            db::db_find_by_id,
            db::db_insert,
            db::db_update,
            db::db_delete,
            db::db_execute_raw,
            // digital_employee
            digital_employee::de_init,
            digital_employee::de_list,
            digital_employee::de_get,
            digital_employee::de_save,
            digital_employee::de_delete,
            // scenario
            scenario::get_scenario,
            scenario::save_scenario,
            // context
            context::get_context_config,
            context::save_context_config,
            context::list_context_entries,
            context::get_context_entry,
            context::save_context_entry,
            context::delete_context_entry,
            // lancedb
            // vector
            vector::vec_connect,
            vector::vec_configure_remote,
            vector::vec_create_table,
            vector::vec_insert,
            vector::vec_search,
            vector::vec_delete,
            vector::vec_list_tables,
            vector::vec_table_info,
            // lancedb (legacy)
            lancedb_store::lancedb_init,
            lancedb_store::lancedb_list_tables,
            lancedb_store::lancedb_create_table,
            lancedb_store::lancedb_insert,
            lancedb_store::lancedb_search
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
