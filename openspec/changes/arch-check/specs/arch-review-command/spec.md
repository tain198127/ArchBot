## ADDED Requirements

### Requirement: /arch-review 命令
系统 SHALL 提供 `/arch-review <文件路径>` 命令，支持用户手动触发完整代码审查流程。

#### Scenario: 手动审查指定文件
- **WHEN** 用户输入 `/arch-review src/components/MenuBar.vue`
- **THEN** 系统对该文件执行快速通道 + 深度通道全部检查，展示分级报告

#### Scenario: 手动审查 Rust 文件
- **WHEN** 用户输入 `/arch-review src-tauri/src/main.rs`
- **THEN** 系统执行 cargo clippy + cargo audit + Claude Agent 深度审查（含防篡改检查）

#### Scenario: 不带参数调用
- **WHEN** 用户输入 `/arch-review` 不提供文件路径
- **THEN** 系统提示用户提供文件路径，或检查当前打开的/最近修改的文件

### Requirement: 命令注册
`/arch-review` 命令 SHALL 注册在 `.claude/commands/arch-review.md`，遵循 Claude Code 命令规范。

#### Scenario: 命令自动补全
- **WHEN** 用户输入 `/arch-re` 按 Tab
- **THEN** 系统自动补全为 `/arch-review`

#### Scenario: 命令帮助
- **WHEN** 用户输入 `/arch-review --help`
- **THEN** 系统展示使用说明和选项
