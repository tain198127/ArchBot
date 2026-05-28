## ADDED Requirements

### Requirement: 未注册时限制指定功能

未注册状态下，Rust 后端定义的受限功能列表中的菜单项 SHALL 置灰且不可点击。

#### Scenario: 未注册状态——受限菜单置灰

- **WHEN** 系统为未注册状态且用户打开包含受限 action 的菜单
- **THEN** 受限菜单项显示为灰色
- **AND** 点击受限菜单项无响应

#### Scenario: 注册状态——所有菜单可用

- **WHEN** 系统为已注册状态
- **THEN** 所有菜单项（包括原先受限的）正常显示且可点击

### Requirement: 前端查询注册状态

前端 SHALL 通过 Tauri command 查询当前注册状态和受限 action 列表。

#### Scenario: 启动时获取注册状态

- **WHEN** 前端完成初始化
- **THEN** 调用 `get_license_status` Tauri command
- **AND** 获取 `{ registered: boolean, restricted_actions: string[] }`
- **AND** 根据结果更新菜单项的禁用状态

### Requirement: Rust 后端 command 级注册检查

每一个受限的 Tauri command SHALL 在执行核心逻辑之前检查注册状态，未注册时直接返回错误，不执行任何业务逻辑。

#### Scenario: 未注册时调用受限 command

- **WHEN** 系统为未注册状态
- **AND** 前端（或被篡改后的代码）直接通过 `invoke()` 调用受限的 Tauri command
- **THEN** Rust 后端在执行任何业务逻辑之前检查注册状态
- **AND** 返回包含"需要注册"提示的 Error
- **AND** 不执行该 command 的任何核心逻辑

#### Scenario: 已注册时调用受限 command

- **WHEN** 系统为已注册状态
- **AND** 调用受限的 Tauri command
- **THEN** Rust 后端的 license gate 通过
- **AND** command 正常执行业务逻辑并返回结果
