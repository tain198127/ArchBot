## Context

`projectDirs` 是一个静态配置数组（`src/config/projectDirs.ts`），定义项目打开后文件树中展示的子目录列表。每个目录有 3 个属性：`key`（用作 action 标识）、`labelKey`（i18n key）、`color`（图标颜色）。当前列表有 13 个条目，从"数据标准"到"部署"。需求是在"混沌测试"和"部署"之间插入"缺陷"。

## Goals / Non-Goals

**Goals:**
- 新增 `defect` 目录项，插入到 `chaosTest` 和 `deployment` 之间
- 更新中英文 i18n 文件
- 更新 PRD 文档

**Non-Goals:**
- 不创建缺陷管理的实际功能编辑器（属于后续迭代）
- 不修改目录项的渲染逻辑或右键菜单行为

## Decisions

### 新增配置项而非重构

直接在 `projectDirs` 数组中插入新条目。`projectDirs` 是纯静态数据，数组长度 14 不会影响性能。无需引入动态配置机制。

### 颜色选择

选择 `#d35400`（深橙色）作为缺陷目录颜色 —— 区别于测试类的绿色系和部署的紫色，视觉上表达"需要关注"的语义。

### 插入位置

插入在 `chaosTest`（混沌测试）之后、`deployment`（部署）之前。逻辑依据：
- 各种测试（testCases → businessTest → stressTest → chaosTest → defect）按验证类型排列
- 部署（deployment）是生命周期的最后阶段，defect 属于验证阶段的最后一环

## Risks / Trade-offs

- **数组顺序即展示顺序**: 目录项的视觉顺序完全由数组索引决定，修改顺序只需调整数组位置
