---
name: git-commit
description: |
  Commits all local changes. Auto-generates a conventional-commits message
  (Chinese by default, English on request) from git status and diff,
  presents it for user confirmation, then stages and commits locally.
  Never pushes.
globs:
  - ".git/**"
triggers:
  - "提交"
  - "commit"
  - "git commit"
  - "/git-commit"
---

# Git Commit Skill

## Rule: Every commit message must be reviewed by the user before execution

This skill automates staging, message generation, and committing — but the
commit message is always presented for user confirmation first.

## Workflow

1. Run `git status` to list all changed files
2. Run `git diff` and `git diff --staged` to read the actual changes
3. Run `git log --oneline -5` to capture the project's commit style
4. Generate a commit message (see format below)
5. **Present the message to the user and wait for confirmation**
6. If confirmed: `git add .` followed by `git commit -m "<message>"`
7. If rejected: ask the user what to change, regenerate, or accept a handwritten message
8. Show the result of `git status` after commit
9. **Never push** — this skill ends after local commit

## Commit Message Format

All commit messages follow the conventional commits format:

```
<type>: <简短中文描述>

<详细说明 — 可选，说明为什么要做这个变更>

Authored-By: danebrown <tain198127@163.com>
```

### Type 类型

| Type | 使用场景 |
|------|---------|
| `feat` | 新功能、新特性 |
| `fix` | Bug 修复 |
| `refactor` | 重构，无功能变更 |
| `docs` | 文档、注释变更 |
| `test` | 测试用例新增或修改 |
| `chore` | 构建、依赖、配置等杂项 |
| `perf` | 性能优化 |
| `ci` | CI/CD 配置变更 |

### 生成规则

- **默认中文**：描述部分使用中文
- **英文请求**：如果用户在触发时说"用英文"或"in English"，生成英文 message
- **1-2 行摘要**：第一行 type + 描述不超过 72 字符
- **WHY 不是 WHAT**：描述变更的原因和目的，而不是罗列修改的文件
- **参考历史**：读取 `git log --oneline -5` 了解项目现有的 commit 风格

### 示例

```
feat: 新增数字员工管理面板

18 个内置数字员工覆盖需求工程全部角色，
支持列表查阅、信息编辑和权限控制。

Authored-By: danebrown <tain198127@163.com>
```

```
fix: 修复资源路径遍历漏洞

在 parse_uri 中拒绝包含 .. 的相对路径，
并通过 canonicalize 二次校验结果不脱离基目录。

Authored-By: danebrown <tain198127@163.com>
```

## Implementation

```bash
# Step 1: Gather context
git status
git diff
git diff --staged
git log --oneline -5

# Step 2: Generate and present message
# (AI generates message from diff context → show to user)

# Step 3: Commit (after user confirms)
git add .
git commit -m "$(cat <<'EOF'
<generated-message>

Authored-By: danebrown <tain198127@163.com>
EOF
)"

# Step 4: Verify
git status
```

## Edge Cases

| 场景 | 处理方式 |
|------|---------|
| 无变更 (`nothing to commit`) | 提示用户，结束 |
| 有未追踪文件 | 一并 `git add .` 包含 |
| .env / credentials 等敏感文件 | 警告用户，询问是否要提交 |
| 合并冲突未解决 | 提示用户先解决冲突 |
| 用户说"用英文" | 生成英文 message |
| 用户说"简短" | 仅生成 type + 一行描述，无详细说明 |
| 用户直接给了 message | 跳过生成，直接使用用户提供的 message |
