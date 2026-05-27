## 1. 基础设施搭建

- [x] 1.1 新建 `.claude/settings.json`，配置 PreToolUse Hook 绑定 Edit/Write 工具
- [x] 1.2 新建 `eslint.config.mjs`，配置 TS/Vue 规则（naming-convention, max-lines-per-function, no-duplicate-imports）
- [x] 1.3 新增 devDependencies 到 `package.json`：eslint, @typescript-eslint/parser, @typescript-eslint/eslint-plugin, eslint-plugin-vue, jscpd

## 2. 快速通道脚本

- [x] 2.1 新建 `.claude/scripts/code-check.sh`，串联 ESLint → jscpd → check-complexity → cargo clippy → cargo audit
- [x] 2.2 脚本接收参数：`--file <path>`（检查文件路径），`--changed-lines <N>`（改动行数）+ stdin JSON 模式
- [x] 2.3 实现跳过检测逻辑：检查环境变量 `SKIP_CODE_CHECK` 和代码注释 `// skip-checks:`
- [x] 2.4 实现自动修复循环：`eslint --fix` + `cargo clippy --fix`，最多 3 轮
- [x] 2.5 脚本输出 JSON 格式结果，包含每项检查状态/级别/描述

## 3. 深度通道 Agent

- [x] 3.1 新建 `.claude/agents/code-check.md`，定义深度审查 agent（架构/安全/性能/防篡改）
- [x] 3.2 Agent 接收快速通道结果和改动上下文，输出分级审查报告 (JSON格式)
- [x] 3.3 实现自动修复能力：补充注释、重命名、提取公共函数
- [x] 3.4 实现 Tauri 防篡改专项检查：IPC 参数校验、Command 权限边界、unsafe 代码审查

## 4. 检查 Skill 定义

- [x] 4.1 新建 `.claude/skills/code-check.md`，定义检查规则、阈值、分级策略
- [x] 4.2 引用已有 `function-comments` skill 中的复杂度检查逻辑 (复用 check-complexity.cjs)

## 5. /arch-review 命令

- [x] 5.1 新建 `.claude/commands/arch-review.md`，注册手动触发命令
- [x] 5.2 命令接收文件路径参数，执行完整检查流程并展示报告

## 6. 用户确认交互

- [x] 6.1 实现分级报告输出：❌ 红色阻断 / ⚠️ 黄色警告 / ✅ 绿色通过 / ⏭️ 灰色跳过（skill 中定义）
- [x] 6.2 实现三级选择：A) 拦截 B) 放行（本次） C) 全放行（本 session）（skill 中定义）
- [x] 6.3 Session 全放行状态管理（通过环境变量 `SKIP_CODE_CHECK=1`）

## 7. 集成测试

- [x] 7.1 测试 TS 文件修改自动触发快速通道 (stdin 模式测试通过)
- [x] 7.2 测试 Vue 文件修改的 `<script>` 复杂度分析 (stdin 模式测试通过)
- [x] 7.3 测试 Rust 文件修改的 Clippy + audit (检测通过, clippy 验证成功)
- [x] 7.5 测试 `SKIP_CODE_CHECK=1` 全跳过 (验证通过)
- [x] 7.9 测试 `/arch-review` 命令行手动触发 (--deep 标志测试通过)
- [ ] 7.4 测试大改动（≥20 行）自动触发深度通道 (需实际大改动场景)
- [ ] 7.6 测试 `// skip-checks:` 注释标记跳过 (需在代码中手动插入测试)
- [ ] 7.7 测试自动修复后重新检查通过 (需有 ESLint 错误的文件测试)
- [ ] 7.8 测试用户交互三种选择的正确行为 (需实际 Claude 交互场景)
