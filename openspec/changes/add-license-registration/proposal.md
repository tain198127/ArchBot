## Why

ArchBot 需支持商业化分发。在未注册状态下限制部分高级功能，用户通过线下购买获取验证码完成注册后解锁全部功能。这是商业化软件的基础能力。

## What Changes

- **新增**：Rust 后端硬件指纹采集模块（MAC 地址 + 主机名 → SHA-256 截断 → 稳定 16 进制注册码）
- **新增**：注册码安全存储（加密写入 `~/.ArchBot/license.dat`）
- **新增**：注册 UI 对话框（展示注册码 + 验证码输入框）
- **新增**：线下验证码校验逻辑（RSA 公钥解密验证码 → 比对注册码）
- **新增**：受限功能列表配置（Rust 常量数组，列出未注册时禁用的菜单 action）
- **新增**：启动时注册状态检查（读取 license 文件 → 校验签名 → 置灰受限菜单）
- **新增**：`--debug` 启动参数绕过注册检查

## Capabilities

### New Capabilities
- `machine-fingerprint`: 采集硬件信息生成稳定的 16 进制机器码，同一台机器始终生成相同结果
- `license-storage`: 注册码和验证状态的安全持久化存储，防止用户篡改
- `registration-ui`: 注册对话框 UI——展示本机注册码、验证码输入、注册状态显示
- `license-validation`: 验证码校验——使用嵌入式公钥解密验证码并与本机注册码比对
- `feature-restriction`: 受限功能管理——Rust 端维护受限 action 列表，前端菜单根据注册状态置灰
- `debug-bypass`: `--debug` 启动参数——调试模式下忽略注册状态，所有功能可用

### Modified Capabilities
<!-- None -->

## Impact

- `src-tauri/Cargo.toml`：新增 `sha2`, `rsa`, `base64` 等加密相关依赖
- `src-tauri/src/`：新增 `license.rs`（指纹采集 + 存储 + 验证 + 受限列表 + 状态检查）
- `src-tauri/src/lib.rs`：注册 Tauri commands、启动时检查注册状态、`--debug` 参数处理
- `src/components/`：新增 `LicenseDialog.vue`（注册对话框）
- `src/App.vue`：启动时获取注册状态、菜单置灰逻辑
- `src/config/menu.ts`：菜单项新增 `restricted?: boolean` 标记
- `src/i18n/`：注册相关标签（中/英）
- `prd.yml`：注册功能描述
