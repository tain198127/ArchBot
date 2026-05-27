## ADDED Requirements

### Requirement: 确定性工具自动修复
系统 SHALL 对可通过工具确定性修复的问题自动应用修复，无需用户干预。支持：`eslint --fix`（命名、格式）、`cargo clippy --fix`（Rust 优化建议）。

#### Scenario: ESLint 自动修复命名
- **WHEN** 快速通道检测到函数命名不符合 camelCase 规范
- **THEN** 系统自动执行 `eslint --fix` 重命名，修复后重新检查

#### Scenario: Clippy 自动修复
- **WHEN** `cargo clippy` 报告可修复的 warning
- **THEN** 系统自动执行 `cargo clippy --fix`，修复后重新检查

### Requirement: Agent 辅助自动修复
深度通道（Claude Agent）SHALL 对以下可修复问题自动修复：补充缺失注释、简单函数重命名、重复代码提取、简单格式修正。

#### Scenario: 补充缺失注释
- **WHEN** 深度通道发现函数 > 20 行且缺少注释
- **THEN** Agent 自动生成注释并插入代码，修复后重新检查

#### Scenario: 提取重复代码
- **WHEN** jscpd 检测到 ≥20 行重复代码
- **THEN** Agent 自动提取公共函数/方法，修复后重新检查

### Requirement: 自动修复循环
系统 SHALL 对自动修复后的代码重新执行检查，形成「检查 → 修复 → 重新检查」循环，最多迭代 3 次。3 次后仍有问题则转入用户确认。

#### Scenario: 自动修复后通过
- **WHEN** 第一轮检查不通过 → 自动修复 → 第二轮检查通过
- **THEN** 系统直接放行，无需用户交互

#### Scenario: 三轮后仍有问题
- **WHEN** 经过 3 轮「检查 → 修复」循环后仍有未解决的问题
- **THEN** 系统停止自动修复，展示不可修复的问题给用户确认
