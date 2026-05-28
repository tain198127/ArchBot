mod data_standard;
mod fs;
mod lancedb_store;
mod license;

/// 获取当前 ISO 8601 格式 UTC 时间戳
///
/// 供 `fs` 模块的 `create_project` 和 `data_standard` 的文件元信息使用。
pub(crate) fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
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
            // lancedb
            lancedb_store::db_list_tables,
            lancedb_store::db_create_table,
            lancedb_store::db_insert,
            lancedb_store::db_search
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
