## ADDED Requirements

### Requirement: Debug 模式绕过注册检查

当以 debug 模式启动时，系统 SHALL 忽略注册状态，所有功能不受限制。

#### Scenario: Debug 模式启动

- **WHEN** 系统以 `ARCHBOT_DEBUG=1` 环境变量或 `--debug` CLI 参数启动
- **THEN** `get_license_status` 始终返回 `{ registered: true, restricted_actions: [] }`
- **AND** 所有菜单项正常可用，无置灰

#### Scenario: 正常模式启动（无 debug 标志）

- **WHEN** 系统以正常模式启动（无 `ARCHBOT_DEBUG` 环境变量或 `--debug` 参数）
- **THEN** 系统按实际 license 文件状态返回注册信息
