## Context

ArchBot 项目当前有分散的代码质量工具（`check-complexity.cjs`、`function-comments` skill），但没有统一的代码修改前的质量门禁。每次 Edit/Write 操作直接落地，缺乏前置审查。项目技术栈为 Vue 3 + TypeScript + Element Plus（前端）+ Tauri 2 / Rust（后端），需要跨语言一致的检查体验。

目前项目无 ESLint 配置、无 settings.json Hook 配置、无 jscpd 配置。约束条件：安全/漏洞检查必须硬阻断，其他检查级为警告。

## Goals / Non-Goals

**Goals:**
- PreToolUse Hook 拦截所有 Edit/Write 操作，强制执行代码检查
- 双通道架构：快速通道（Shell 脚本，1-3s）+ 深度通道（Claude Agent，5-15s）
- 自动修复能力：ESLint --fix、cargo clippy --fix、agent 自动修复可修复问题
- 分级报告：阻断级（❌红色）强制拦截，警告级（⚠️黄色）提示
- 用户交互确认：拦截/放行/本 session 全放行三种选择
- `/arch-review` 命令手动触发
- 绕过机制：`// skip-checks: reason` 注释标记 + `SKIP_CODE_CHECK=1` 环境变量
- 阈值：函数长度 200 行、复杂度 10、重复代码 20 行

**Non-Goals:**
- 不替换项目已有的 `check-complexity.cjs`，而是复用
- 不在 CI/CD 中运行（当前只做本地开发时拦截）
- 不做跨项目通用化（仅针对 ArchBot 定制）

## Decisions

### 1. 双通道混合架构

**选择**：快速通道（Shell）+ 深度通道（Claude Agent）

**理由**：Shell 脚本毫秒级执行（ESLint/Clippy/cargo audit），覆盖 80% 的常规检查，不消耗 token。深度通道用 Claude agent 做架构/安全/性能审查，覆盖 Shell 做不到的语义分析。小改动（<20 行/单文件）只跑快速通道，大改动才触发深度通道。

**替代方案**：全 Shell（无法做架构审查）、全 Agent（太慢、耗 token）。

### 2. 自动修复 + 重新检查循环

**选择**：检查失败 → `--fix` 自动修复 → 重新检查 → 仍有问题才交互

**理由**：减少用户打断次数。ESLint/Clippy 的 `--fix` 能自动处理命名、格式、简单重构。Agent 能自动修复加注释、重命名、提取公共函数。不可修复的（架构设计问题、安全漏洞业务逻辑）留给用户决策。

**替代方案**：每次都问用户 → 打断过多，用户体验差。

### 3. PreToolUse Hook 而非 PostToolUse

**选择**：在 Edit/Write 执行前拦截

**理由**：代码检查不通过时不落地到文件系统，避免坏代码进入编辑历史。PostToolUse 需要回滚机制，更复杂。

**风险**：每次编辑增加 1-15 秒延迟。缓解：快速通道只需 1-3s，且有 `SKIP_CODE_CHECK=1` 可跳过。

### 4. ESLint 平地配置而非复用外部预设

**选择**：新建 `eslint.config.mjs`，从零配置最小规则集

**理由**：项目当前无 ESLint 配置，引入外部 preset（如 `eslint:recommended`）会引入大量无关规则，增加误报。从最小规则集开始，按需扩展。

## Risks / Trade-offs

- [编辑延迟增加 1-15s] → 提供 `SKIP_CODE_CHECK=1` 快速跳过，小改动只跑快速通道
- [ESLint 规则误报] → 使用注释 `// skip-checks: reason` 精确跳过单次检查
- [Rust cargo audit 网络依赖] → 失败时不阻断，降级为 ⚠️ 警告
- [深度通道 token 消耗] → 仅大改动触发，且用户可选择 session 全放行
- [自动修复可能改坏代码] → 只对确定性高的规则启用自动修复（ESLint --fix、Clippy --fix），agent 修复在可控范围内
