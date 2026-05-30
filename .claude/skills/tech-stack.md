---
name: tech-stack
description: |
  Auto-sync tech stack changes to tech.yml, detect conflicts with existing stack
  (3-layer check: hard/overlap/arch-constraint), and provide architecture review
  with 5-dimension scoring (cohesion/coupling/maintainability/security/consistency).
  Triggers automatically on dependency file changes. Manual /tech-review for
  incremental/full/focused architecture audits.
globs:
  - "package.json"
  - "Cargo.toml"
  - "vite.config.ts"
  - "tsconfig.json"
  - "tauri.conf.json"
  - "src/tailwind.css"
triggers:
  - "/tech-review"
  - "/tech-review --full"
  - "/tech-review --focus="
  - "技术栈检查"
  - "架构评审"
  - "tech stack check"
---

# Tech Stack Skill — 技术栈同步、冲突检测与架构评审

## 设计思路

### 为什么需要这个 Skill

ArchBot 有严格的技术栈约束（Tailwind-only、PrimeVue 排他、Tokio 唯一异步运行时等）。
每次添加依赖都可能在不知不觉中引入冲突。这个 skill 在技术栈变更的**第一时间**自动检测冲突；
在需要架构评审时提供**五维评分卡**，量化评估技术决策质量。

### 核心概念

**三阶冲突检测**：冲突不是二元的是/否，而是分层级的判断：

```
Layer 1: 硬冲突 (BLOCK)
  运行时不可能共存，必须先解决
Layer 2: 功能重叠 (WARN)
  新栈与现有栈解决同一问题，建议合并
Layer 3: 架构约束 (ERROR)
  违反项目既定技术决策，除非显式批准否则不可引入
```

**五维评分卡**：架构评审不是主观感觉，而是五维度量：
内聚性 / 耦合度 / 可维护性 / 安全性 / 一致性。

**tech.yml 作为单一事实来源**：所有技术栈信息聚合在 `tech.yml`，skill 保持它与实际依赖配置同步。

### 与 ui-ux-tech-stack 的分工

| 维度 | ui-ux-tech-stack | tech-stack |
|------|-----------------|------------|
| 触发时机 | 前端文件变更 (.vue/.ts/.css) | 依赖配置文件变更 |
| 检查范围 | 前端 UI 层 | 全栈 |
| 检查方式 | 实时拦截违规 import/样式 | 冲突矩阵 + 同步 tech.yml |
| 引用关系 | 定义前端 UI 层规则 | 引用 ui-ux-tech-stack 规则做 Layer 3 检查 |
| 独立性 | 可单独运行 | 可单独运行 |

---

## 前置条件

1. `tech.yml` 存在（不存在则提示用户先执行扫描生成）
2. Git 仓库状态可查询（用于增量评审确定变更范围）

---

## 内部路由

启动后根据触发来源选择模式：

```
触发来自 globs 文件变更 → sync 模式
触发来自 slash 命令    → review 模式
```

---

# Part 1: Sync 模式 — 自动同步与冲突检测

## 触发条件

当以下文件发生变更时自动激活：
- `package.json` — npm 依赖
- `Cargo.toml` — Rust crate 依赖
- `vite.config.ts` — 构建工具/插件
- `tsconfig.json` — TypeScript 编译目标
- `tauri.conf.json` — Tauri 插件/配置
- `src/tailwind.css` — design tokens

## Sync 工作流

### Step S1 — 解析变更

**目标**: 从变更文件中提取新增、升级、移除的依赖。

**AI 动作**:
1. `git diff` 获取变更内容
2. 对 `package.json`:
   - 解析 `dependencies` / `devDependencies` 的新增/版本变更/删除
   - 识别主版本跳跃（`^1.x` → `^2.x` 或 `~1.x` → `~2.x`）
3. 对 `Cargo.toml`:
   - 解析 `[dependencies]` 的新增/版本变更/删除
   - 识别 feature flag 变更
4. 对 `tsconfig.json`:
   - 检查 `target` / `module` / `lib` 变更
5. 对 `tauri.conf.json`:
   - 检查 plugins 变更
6. 对 `tailwind.css`:
   - 检查 `@theme` 块中的 design token 变更

**输出**: 变更清单（结构化列表）

---

### Step S2 — 三阶冲突检测

**目标**: 将每个新增/升级的依赖与现有 tech.yml 对照，按三层规则判断冲突等级。

#### Layer 1: 硬冲突 (BLOCK)

必须解决才能继续。检测以下模式：

| 规则 | 检测方式 |
|------|---------|
| 同库多版本并存 | 检查 `package.json` 中同一包是否有多个版本约束，检查 `Cargo.toml` 是否同一 crate 有两个 semver 不兼容的版本 |
| Rust crate 语义不兼容 | semver 主版本冲突（如 tokio 0.x 与 tokio 1.x 同现） |
| 运行时冲突 | 新包依赖 Node.js API，但项目目标含纯浏览器环境 |
| TypeScript 编译目标不兼容 | 新包要求 ES2022+，但 `tsconfig.json` target 为更低版本 |
| 许可证冲突 | 新增 GPL/AGPL 依赖，项目为商业闭源 |

**输出格式**:
```
🔴 硬冲突 — 必须解决
   - [冲突描述]
     文件: package.json
     新增: xxx@2.0.0
     冲突原因: 与现有 xxx@1.5.0 不兼容（主版本不兼容）
     建议: 统一升级到 2.x 或保留 1.x 并移除非兼容版本
```

#### Layer 2: 功能重叠 (WARN)

检测功能性重复：

| 规则 | 检测方式 |
|------|---------|
| 状态管理重复 | 新增 pinia/vuex，但现有自建 store (`src/stores/`) 已存在 |
| 路由方案重复 | 新增 vue-router，但项目是 SPA 面板式无路由架构 |
| 同类库竞争 | 新增同类工具库（dayjs vs 已有 chrono、lodash vs 已有 ramda 等） |
| ORM/DB 引擎重叠 | 新增 diesel/sqlx 但已有 sea-orm |
| 构建工具链冲突 | 新增 webpack/babel 但已有 vite |
| 图标库重复 | 新增其他图标库，但已有 @lucide/vue |
| CSS 方案重复 | 新增 CSS-in-JS/Sass，但已有 Tailwind v4 |

**输出格式**:
```
🟡 功能重叠 — 建议处理
   - [重叠描述]
     新增: pinia@3.0
     现有: src/stores/ 自建 store (project.ts, settings.ts, license.ts, log.ts)
     建议: 统一选择一种状态管理方案。保留自建 store 则不需要 pinia；
           引入 pinia 则需迁移现有 4 个 store
```

#### Layer 3: 架构约束 (ERROR)

引用 `ui-ux-tech-stack` 的 banned list，加上后端约束：

| 约束域 | 规则 | 例外审批条件 |
|--------|------|------------|
| 前端 UI 层 | 见 ui-ux-tech-stack banned list | 需明确说明不可替代性 |
| 前端组件库 | 仅 PrimeVue Unstyled | 需证明 PrimeVue 无法满足且无可替代方案 |
| CSS 方案 | 仅 Tailwind CSS v4 | 无例外 |
| Rust 异步运行时 | 仅 Tokio（禁止 async-std、smol） | 需证明 Tokio 无法满足 |
| HTTP 框架 | 仅 Axum（禁止 actix-web、warp） | 需证明 Axum 无法满足 |
| 序列化 | JSON: serde_json, YAML: serde_yml | 需证明现有方案不满足 |
| 数据库 | 主存储 SQLite/SeaORM，向量存储 LanceDB | 需提供数据迁移方案 |
| 桌面框架 | 仅 Tauri 2（禁止 Electron） | 需证明 Tauri 2 无法满足 |

**输出格式**:
```
🟠 架构约束冲突 — 违反项目技术决策
   - [冲突描述]
     新增: actix-web@4
     约束: 项目 HTTP 框架仅允许 Axum
     建议: 使用 Axum 0.8 替代，或说明 actix-web 的不可替代原因
```

---

### Step S3 — 冲突裁决

**AI 动作**:
1. 汇总所有冲突，按严重程度排序（Layer1 > Layer3 > Layer2）
2. 对每个冲突给出决策建议
3. 无冲突 → 直接进入 Step S4
4. 有冲突 → 输出完整冲突报告，等待用户决定

**用户选项**:
- `保留` — 接受新栈，更新 tech.yml，记录例外原因
- `替换` — 按建议切换到兼容方案
- `撤销` — 回退变更

**原则**: 任何冲突未解决前，不修改 tech.yml。

---

### Step S4 — 同步 tech.yml

**目标**: 根据变更清单（经冲突裁决后）更新 tech.yml。

**同步规则**:

| 变更类型 | 同步行为 |
|---------|---------|
| 新增 npm 依赖 | 追加到 `frontend.core.dependencies` 或 `frontend.testing/linting` 等对应段 |
| 新增 Rust crate | 追加到 `backend.dependencies` 或对应模块 |
| 升级主版本 | 更新版本号 + 在 changelog 段标记 `⚠️ breaking: xxx` |
| 移除依赖 | 标记 `# deprecated` 注释，不删除（人工确认后清理） |
| 新增 Tauri 插件 | 追加到 `backend.tauri_plugins` |
| 新增 Rust 模块 | 追加到 `backend.modules` |
| 构建工具变更 | 追加到 `dev_toolchain` 段 |
| design tokens 变更 | 更新 `frontend.design_system` 对应字段 |

**写入原则**:
- 自动追加的数据标注 `# auto-generated YYYY-MM-DD`，人工内容不动
- 保留所有现有注释和手动添加的说明
- 不覆盖任何已存在的非 auto-generated 条目
- 同步完成后 git diff 显示 tech.yml 变更

**完成后通知**:
```
✅ tech.yml 已同步
   + frontend.dependencies: 新增 xxx@1.2.3
   ~ backend.tauri_plugins: 升级 yyy@2→3
   
   查看: git diff tech.yml
```

---

# Part 2: Review 模式 — 架构评审

## 触发方式

| 命令 | 行为 |
|------|------|
| `/tech-review` | 增量评审：仅评审当前工作区变更涉及的技术决策 |
| `/tech-review --full` | 全量评审：遍历 tech.yml 所有条目做完整审计 |
| `/tech-review --focus=<模块>` | 聚焦评审：如 `--focus=db`、`--focus=frontend`、`--focus=vector` |

## Review 工作流

### Step R1 — 确定评审范围

**增量评审** (默认):
1. `git diff` 获取所有变更文件
2. 识别涉及技术栈变更的文件（依赖配置、新增模块、构建配置）
3. 仅评估变更部分

**全量评审** (`--full`):
1. 读取 `tech.yml` 所有条目
2. 逐模块评审

**聚焦评审** (`--focus=xxx`):
1. 仅读取 `tech.yml` 中指定模块的条目
2. 模块名: `frontend`, `backend`, `data`, `runtime`, `project_system`, `dev_toolchain`

---

### Step R2 — 五维评分卡

对每个评审对象打分（0-10 分）：

```
┌──────────────┬──────────────────────────────────────────┐
│ D1. 内聚性   │ 模块职责是否单一，边界是否清晰            │
│              │ 0=混杂污染边界  10=职责单一、接口清晰     │
├──────────────┼──────────────────────────────────────────┤
│ D2. 耦合度   │ 外部依赖链的深度和广度                    │
│              │ 0=深度耦合、长依赖链                      │
│              │ 10=零外部依赖、松耦合                     │
├──────────────┼──────────────────────────────────────────┤
│ D3. 可维护性 │ 社区活跃度、文档质量、版本稳定性           │
│              │ 0=弃坑/无文档/alpha版                     │
│              │ 10=活跃维护、文档完善、LTS                 │
├──────────────┼──────────────────────────────────────────┤
│ D4. 安全性   │ 已知CVE、依赖审计、权限最小化              │
│              │ 0=有高危CVE未修                           │
│              │ 10=无已知漏洞、最小权限                    │
├──────────────┼──────────────────────────────────────────┤
│ D5. 一致性   │ 是否符合项目既有技术选型和设计模式         │
│              │ 0=完全违背项目约定                         │
│              │ 10=与现有栈一致、符合编码规范               │
└──────────────┴──────────────────────────────────────────┘

总分判定:
  40-50 → ✅ 通过
  30-39 → ⚠️  有条件通过（列出风险点）
   0-29 → ❌ 阻塞（需重新评估）
```

**评分指南**:
- 不要给中庸分（5-6 分只是逃避决策）
- 低分必须附带具体证据（如 CVE 编号、文档链接）
- 对 npm 包，验证 npm 下载量、最后发布时间、issues 活跃度
- 对 Rust crate，验证 crates.io 下载量、docs.rs 文档覆盖率
- 安全评分需运行 `cargo audit` 或检查 npm advisory

---

### Step R3 — 生成评审报告

**报告结构**:

```markdown
# 技术架构评审报告
> 时间: YYYY-MM-DD HH:mm
> 类型: [增量 | 全量 | 聚焦: xxx]
> 评审人: AI Agent

## 1. 评审范围
（列出被评审的模块/依赖）

## 2. 五维评分卡

| 评审对象 | D1 内聚 | D2 耦合 | D3 可维护 | D4 安全 | D5 一致 | 总分 | 判定 |
|---------|--------|--------|---------|--------|--------|-----|-----|
| xxx     | 8      | 7      | 9       | 8      | 9      | 41  | ✅  |

## 3. 风险清单

| 风险 | 严重程度 | 评分维度 | 说明 | 建议 |
|------|---------|---------|------|------|
| ...  | 高/中/低 | Dx      | ...  | ...  |

## 4. 改进建议

（优先排序的建议列表）

## 5. 技术债务标记

（已识别的技术债务，建议在未来 N 个迭代内偿还）
```

---

### Step R4 — 保存报告

保存到 `.archbot/tech-reviews/YYYY-MM-DD-HHmmss-{incremental|full|focus-xxx}.md`

---

## 交互协议

### Sync 模式交互

```
🔍 Tech Stack Sync 检测到变更:

  新增: xxx@1.0.0 (package.json)
  升级: yyy@1.5.0 → 2.0.0 (Cargo.toml)

  冲突检测中...
  
  [无冲突]
  ✅ 通过，tech.yml 已自动更新
  
  [有冲突]
  🔴 发现 2 个冲突:
    1. [Layer 1] 同库多版本: xxx 同时有 1.x 和 2.x
    2. [Layer 3] 架构约束: zzz 违反 HTTP 框架约束
  
  如何处理? 
    A) 保留并记录例外  B) 按建议替换  C) 撤销变更  D) 逐个决定
```

### Review 模式交互

```
📊 技术架构评审: [增量 | 全量 | 聚焦: xxx]

  评审 N 个对象中...
  
  评分摘要:
  ┌──────────┬────┬────┬────┬────┬────┬────┬──────┐
  │ 对象     │ D1 │ D2 │ D3 │ D4 │ D5 │总分│ 判定 │
  ├──────────┼────┼────┼────┼────┼────┼────┼──────┤
  │ frontend │ 8  │ 7  │ 9  │ 7  │ 9  │ 40 │ ✅   │
  │ backend  │ 8  │ 7  │ 8  │ 8  │ 9  │ 40 │ ✅   │
  │ data     │ 7  │ 6  │ 8  │ 8  │ 9  │ 38 │ ⚠️   │
  └──────────┴────┴────┴────┴────┴────┴────┴──────┘
  
  ⚠️ data 层: D2 耦合度偏低 — LanceDB 依赖 Arrow 53，
    版本锁定较紧，升级路径窄。建议关注 Arrow 版本兼容策略。

  报告已保存: .archbot/tech-reviews/2026-05-30-140000-full.md
```

---

## 文件结构

```
.archbot/
└── tech-reviews/                    # 架构评审报告
    ├── 2026-05-30-140000-full.md
    ├── 2026-05-30-150000-incremental.md
    └── 2026-05-30-160000-focus-db.md

tech.yml                             # 技术栈声明文件（项目根目录）
```

---

## 边界与限制

- **不自动修改代码** — sync 模式只更新 tech.yml，不修改 package.json/Cargo.toml
- **不替代 code review** — 架构评审关注技术选型质量，不检查代码实现质量
- **不处理间接依赖** — 只检查直接声明的依赖，不递归分析依赖树（可扩展）
- **冲突裁决权归用户** — 任何冲突都需用户决策，不会自动覆盖
