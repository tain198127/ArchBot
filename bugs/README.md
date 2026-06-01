# Bugs 缺陷追踪目录

## 目录用途

本目录用于记录和管理 ArchBot 项目中发现的**所有缺陷**。

## 文件结构

```
bugs/
├── README.md                          # 本文件 — 目录约定说明
├── bug-tracker.xlsx                   # 缺陷状态追踪表
├── BUG-001-<简短描述>.md              # 单条缺陷详细报告
├── BUG-002-<简短描述>.md
└── ...
```

## 缺陷报告模板

每条缺陷以 Markdown 文件记录，文件名格式：`BUG-<NNN>-<kebab-case-title>.md`

报告须包含以下章节：

1. **现象 (Symptom)** — 用户/系统观察到的异常行为
2. **环境 (Environment)** — OS，版本，架构，依赖版本等上下文信息
3. **错误日志 (Error Log)** — 完整的错误输出、堆栈跟踪、crash report
4. **根因分析 (Root Cause Analysis)** — 为什么发生、触发条件、影响范围
5. **修复方案 (Fix Plan)** — 具体修改方案、涉及文件、代码示例
6. **修复历史 (Fix History)** — 日期、操作、操作人

状态枚举：`open` | `in_progress` | `fixed` | `verified` | `wont_fix`

严重程度枚举：`critical` | `high` | `medium` | `low`

## XLSX 追踪表

`bug-tracker.xlsx` 包含以下列：

| Column      | Description                            |
|-------------|----------------------------------------|
| Bug ID      | BUG-001, BUG-002, ...                  |
| Title       | 缺陷简要标题                              |
| Severity    | critical / high / medium / low          |
| Status      | open / in_progress / fixed / verified / wont_fix |
| Module      | 涉及的代码模块                             |
| Found Date  | 发现日期                                  |
| Fixed Date  | 修复日期（修复后填写）                        |
| Fix Commit  | 修复提交 hash（修复后填写）                   |
| Report File | 对应的 md 报告文件路径                       |

## 工作流

1. **发现缺陷** → 创建 `BUG-NNN-title.md`（从模板复制），更新 `bug-tracker.xlsx` 添加新行
2. **开始修复** → 更新 xlsx Status 为 `in_progress`
3. **修复完成** → 更新 xlsx Status 为 `fixed`，填写 Fixed Date 和 Fix Commit；在 md 报告的"修复历史"章节追加记录
4. **验证通过** → 更新 xlsx Status 为 `verified`
5. **决定不修** → 更新 xlsx Status 为 `wont_fix`，在 md 报告中说明原因
