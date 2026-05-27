## Why

当前项目子目录列表中缺少"缺陷"（缺陷管理）这个项目资源类型，而缺陷管理是软件工程生命周期中的重要环节，应作为一个独立的项目子目录存在，和测试类目录（业务测试/压力测试/混沌测试）以及部署目录并列。

## What Changes

- 在 `projectDirs` 配置中新增 `defect` 项目子目录类型
- 插入位置：混沌测试（chaosTest）之后、部署（deployment）之前，保持逻辑顺序
- 同时更新中英文 i18n 标签
- 更新 PRD 文档与 prd.yml 保持同步

## Capabilities

### New Capabilities
- `project-defect-type`: 新增项目的缺陷目录类型，拥有独立颜色和图标，通过 i18n 显示标签

### Modified Capabilities
<!-- None -->

## Impact

- `src/config/projectDirs.ts`: 新增 1 行配置项
- `src/i18n/zh-CN.ts`: 新增 `defect: '缺陷'`
- `src/i18n/en-US.ts`: 新增 `defect: 'Defect'`
- `prd.yml`: 项目子目录列表中新增"缺陷"
