## 1. Rust 依赖与模块骨架

- [x] 1.1 在 `src-tauri/Cargo.toml` 中添加 `sha2`, `rsa`, `pbkdf2`, `hmac`, `hex`, `rand`, `hostname` 依赖
- [x] 1.2 新建 `src-tauri/src/license.rs` 模块骨架（空函数 + 结构体定义）
- [x] 1.3 在 `src-tauri/src/lib.rs` 中声明 `mod license` 并注册 Tauri commands

## 2. 硬件指纹采集

- [x] 2.1 实现 `get_mac_address()` —— 获取首个非 loopback 网卡的 MAC 地址（支持 macOS/Linux/Windows）
- [x] 2.2 实现 `get_machine_id()` —— MAC + hostname → SHA-256 → 前 16 位 hex
- [x] 2.3 实现 `get_machine_id` Tauri command，前端可调用获取注册码

## 3. 密码学与验证

- [x] 3.1 在 Rust 中嵌入 RSA-2048 公钥常量（PEM 格式，作为 `&str` 常量）
- [x] 3.2 实现 `validate_verification_code(machine_id: &str, code: &str) -> bool`（RSA PKCS1v15 签名验证）
- [x] 3.3 实现 `derive_storage_key(machine_id: &str) -> [u8; 32]`（PBKDF2 派生密钥）
- [x] 3.4 实现 HMAC 签名和校验函数

## 4. License 文件持久化

- [x] 4.1 实现 `save_license(machine_id, verification_code)` —— 生成 HMAC → 写 JSON → `~/.ArchBot/license.dat`
- [x] 4.2 实现 `load_license() -> Option<LicenseData>` —— 读取文件 → 校验 HMAC → 校验 machine_id 匹配
- [x] 4.3 实现 `get_license_status()` Tauri command —— 返回 `{ registered, restricted_actions, machine_id }`

## 5. 受限功能配置

- [x] 5.1 在 `license.rs` 中定义 `RESTRICTED_ACTIONS` 常量数组（列出未注册时禁用的菜单 action）
- [x] 5.2 实现 `is_registered() -> bool` 函数——读取全局 `AtomicBool`（启动时从 license 文件初始化）
- [x] 5.3 实现 `license_gate!()` macro —— 未注册时直接 `return Err(...)`
- [x] 5.4 在所有受限 Tauri command 函数体顶部添加 `license_gate!()` 调用
- [x] 5.5 实现启动时注册状态检查：`check_license_on_startup()` 在 `run()` 中调用

## 6. Debug 模式

- [x] 6.1 实现 `is_debug_mode()` —— 检查 `ARCHBOT_DEBUG` 环境变量
- [x] 6.2 debug 模式下 `is_registered()` 始终返回 `true`，`license_gate!()` 永不拦截

## 7. 注册 UI

- [x] 7.1 新建 `src/components/LicenseDialog.vue` —— 注册对话框组件（展示注册码 + 验证码输入 + 注册按钮）
- [x] 7.2 在 `src/App.vue` 中引入 LicenseDialog 并处理注册逻辑
- [x] 7.3 在 `src/i18n/zh-CN.ts` 和 `en-US.ts` 中添加注册相关标签
- [x] 7.4 在 `src/config/menu.ts` 的 File 菜单中添加"注册"菜单项（action: `file.register`）

## 8. 菜单置灰逻辑

- [x] 8.1 在 `src/config/menu.ts` 的 `MenuItem` 类型中添加 `restricted?: boolean` 标记
- [x] 8.2 在 `src/stores/` 中新建 `license.ts` store —— 调用 `get_license_status` 并暴露 `isRegistered` + `restrictedActions`
- [x] 8.3 在 `src/App.vue` 启动时初始化 license store
- [x] 8.4 在 `src/components/MenuBar.vue` 中根据 license store 对受限菜单项置灰

## 9. PRD 同步

- [x] 9.1 在 `prd.yml` 中添加注册功能描述和受限功能列表
