## ADDED Requirements

### Requirement: PreToolUse Hook 拦截代码修改
系统 SHALL 在每次 Edit/Write 工具调用前触发 PreToolUse Hook，对即将修改的代码执行质量检查门禁。

#### Scenario: 编辑 TypeScript 文件
- **WHEN** Claude 尝试对 `src/**/*.ts` 调用 Edit 或 Write 工具
- **THEN** 系统在代码落地前执行快速通道检查（ESLint、jscpd、check-complexity）

#### Scenario: 编辑 Vue 文件
- **WHEN** Claude 尝试对 `src/**/*.vue` 调用 Edit 或 Write 工具
- **THEN** 系统在代码落地前执行快速通道检查，包含 `<script>` 部分的复杂度分析

#### Scenario: 编辑 Rust 文件
- **WHEN** Claude 尝试对 `src-tauri/src/**/*.rs` 调用 Edit 或 Write 工具
- **THEN** 系统在代码落地前执行 `cargo clippy -- -D warnings` 和 `cargo audit`

### Requirement: 双通道检查架构
系统 SHALL 根据改动规模自动选择检查深度：小改动（<20 行且单文件）只跑快速通道，大改动（≥20 行或跨文件）快速通道 + 深度通道全跑。

#### Scenario: 小改动只跑快速通道
- **WHEN** 改动 < 20 行且仅涉及单个文件
- **THEN** 仅执行快速通道检查，跳过 Claude Agent 深度审查

#### Scenario: 大改动触发深度通道
- **WHEN** 改动 ≥ 20 行或跨多个文件
- **THEN** 快速通道通过后，自动进入 Claude Agent 深度审查（架构/安全/性能/防篡改）

### Requirement: 快速通道检查规则
快速通道 SHALL 覆盖以下检查项，并按指定级别分类：

| 检查项 | 工具 | 阈值 | 级别 |
|--------|------|------|------|
| 复杂度 (TS/Rust) | check-complexity.cjs | 10 | ⚠️ 警告 |
| 函数长度 (TS) | ESLint max-lines-per-function | 200 行 | ⚠️ 警告 |
| 命名规范 (TS) | ESLint naming-convention | camelCase/PascalCase | ⚠️ 警告 |
| 重复代码 (TS) | jscpd | ≥20 行 | ⚠️ 警告 |
| Rust 基础 | cargo clippy -- -D warnings | - | ❌ 阻断 |
| Rust 漏洞 | cargo audit | - | ❌ 阻断 |

#### Scenario: Rust Clippy 检查不通过
- **WHEN** `cargo clippy` 返回错误
- **THEN** 系统标记为 ❌ 阻断级，阻止修改，展示具体错误信息

#### Scenario: 函数超过 200 行
- **WHEN** 被修改的函数体超过 200 行有效代码
- **THEN** 系统标记为 ⚠️ 警告级，提示用户拆分，但不强制拦截

### Requirement: 深度通道检查规则
深度通道（Claude Agent）SHALL 覆盖以下检查项：

| 检查项 | 内容 | 级别 |
|--------|------|------|
| 安全检查 | XSS、SQL 注入、CSRF、硬编码密钥、输入验证 | ❌ 阻断 |
| 架构检查 | 模块职责、循环依赖、层级穿透 | ⚠️ 警告 |
| 性能检查 | N+1 查询、不必要分配、大对象拷贝 | ⚠️ 警告 |
| 防篡改 | IPC 参数校验、Tauri command 权限、unsafe 代码边界 | ❌ 阻断 |

#### Scenario: 发现 XSS 漏洞
- **WHEN** Agent 检测到用户输入未经转义直接插入 DOM
- **THEN** 系统标记为 ❌ 阻断级，阻止修改，给出修复建议

#### Scenario: 发现架构问题
- **WHEN** Agent 检测到跨层级调用或循环依赖
- **THEN** 系统标记为 ⚠️ 警告级，提示但不强制拦截

### Requirement: 跳过机制
系统 SHALL 支持两种跳过方式：注释标记 `// skip-checks: reason`（精确控制单个位置），环境变量 `SKIP_CODE_CHECK=1`（跳过本次 session 所有检查）。

#### Scenario: 注释标记跳过
- **WHEN** 修改行的上下文中包含 `// skip-checks: <原因>`
- **THEN** 系统跳过该次修改的代码检查，直接放行

#### Scenario: 环境变量跳过
- **WHEN** 环境变量 `SKIP_CODE_CHECK=1` 被设置
- **THEN** 当前 ClCode session 中所有 Edit/Write 操作跳过代码检查
