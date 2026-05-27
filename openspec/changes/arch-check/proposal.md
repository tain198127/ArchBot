## Why

每次 Claude Code 修改代码时，缺乏自动化的代码质量门禁。在代码落地之前需要确保代码通过复杂度、命名规范、安全漏洞、架构合规等多维度检查，防止低质量或存在安全风险的代码进入仓库。现有的 `check-complexity.cjs` 和 `function-comments` skill 只覆盖了部分场景，需要一套完整的 PreToolUse 拦截 + 自动修复 + 深度审查的组合门禁。

## What Changes

- **新增** PreToolUse Hook：拦截 Edit/Write 工具调用，在代码落地前强制执行检查
- **新增** 快速检查通道（Shell 脚本）：ESLint、jscpd、check-complexity、cargo clippy、cargo audit
- **新增** 深度检查通道（Claude Agent）：架构审查、安全审计、性能分析、防篡改审查
- **新增** `/arch-review` 命令行触发：支持手动对一个文件发起完整审查
- **新增** 自动修复能力：ESLint `--fix`、cargo clippy `--fix`、Claude agent 自动修复可修复的问题
- **新增** 分级报告 + 用户交互确认：阻断级/警告级分色展示，支持拦截/放行/本 session 全放行
- **新增** 跳过机制：注释标记 `// skip-checks: reason` 和环境变量 `SKIP_CODE_CHECK=1`
- **依赖新增** `eslint`、`@typescript-eslint/parser`、`@typescript-eslint/eslint-plugin`、`eslint-plugin-vue`、`jscpd`

## Capabilities

### New Capabilities

- `code-check-gate`: 代码修改前的自动化检查门禁，包含快速通道（ESLint/Clippy/cargo audit）和深度通道（Claude agent 架构/安全/性能/防篡改审查）
- `arch-review-command`: `/arch-review <文件路径>` 命令，手动触发完整审查流程
- `auto-fix`: 自动修复引擎，对可修复问题（命名、格式、Clippy 建议、简单重构）自动修复后重新检查
- `check-report`: 分级报告系统，阻断级（❌）和警告级（⚠️）分色展示，用户可选择拦截/放行/本 session 全放行

### Modified Capabilities

- `function-comments`: 现有复杂度检查能力被整合到快速通道中，由 `code-check-gate` 统一调度执行

## Impact

- `.claude/settings.json` — 新建 Hook 配置
- `.claude/skills/code-check.md` — 新建检查规则 skill
- `.claude/agents/code-check.md` — 新建深度审查 agent
- `.claude/scripts/code-check.sh` — 新建快速通道脚本
- `.claude/commands/arch-review.md` — 新建命令注册
- `.claude/scripts/check-complexity.cjs` — 现有脚本被复用
- `eslint.config.mjs` — 新建 ESLint 配置
- `package.json` — 新增 devDependencies
