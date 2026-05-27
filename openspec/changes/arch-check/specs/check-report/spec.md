## ADDED Requirements

### Requirement: 分级报告输出
系统 SHALL 以颜色分级方式展示检查报告：❌ 红色阻断级（安全、漏洞、Rust clippy 错误），⚠️ 黄色警告级（复杂度、长度、命名、架构），✅ 绿色通过，⏭️ 灰色跳过。

#### Scenario: 阻断级和警告级混合
- **WHEN** 一次检查中既有阻断级失败也有警告级失败
- **THEN** 阻断级用红色 ❌ 展示在顶部，警告级用黄色 ⚠️ 展示在下方，通过项用绿色 ✅ 展示

#### Scenario: 全部通过
- **WHEN** 所有检查项通过
- **THEN** 展示全部绿色 ✅ 并通过的系统消息

### Requirement: 用户交互确认
当存在不可自动修复的问题时，系统 SHALL 展示选项供用户选择：A) 拦截本次修改，B) 放行本次修改，C) 本次 session 剩余修改全部放行。

#### Scenario: 用户选择拦截
- **WHEN** 用户选择「A) 拦截」
- **THEN** Edit/Write 工具调用被取消，代码不落地，问题记录保留

#### Scenario: 用户选择放行
- **WHEN** 用户选择「B) 放行」
- **THEN** 仅本次 Edit/Write 操作绕过检查，代码正常落地

#### Scenario: 用户选择 session 全放行
- **WHEN** 用户选择「C) 本次 session 全放行」
- **THEN** 当前 session 内后续所有 Edit/Write 操作自动跳过检查，Session 结束后恢复

### Requirement: 报告格式
报告 SHALL 包含：文件路径、检查项名称、级别标识、具体行号（如适用）、问题描述、修复建议。

#### Scenario: XSS 漏洞报告示例
- **WHEN** 检测到 XSS 漏洞
- **THEN** 报告格式为：`❌ [阻断] XSS 风险: src/Login.vue:42 - 用户输入未转义直接插入 innerHTML。建议：使用 textContent 或 v-text`
