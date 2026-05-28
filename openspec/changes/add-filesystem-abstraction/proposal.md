## Why

ArchBot 需要统一的文件操作接口，既能操作本地文件系统（当前已有 `read_local_file`），也能对接远程 REST API 后端。通过 trait 抽象实现「一处定义，两种实现」，为后续远程协作功能打基础。

## What Changes

- 新增 Rust trait `FsBackend`：定义统一的文件操作接口（读/写/列表/删除/存在/创建目录）
- 实现 `LocalFs`：封装 `std::fs`
- 实现 `RemoteFs`：封装 `reqwest` HTTP 调用，通过 REST API 操作远程文件
- 注册 6 个 Tauri commands，通过 `fs_type` 参数区分 local/remote
- 前端提供 `FsClient` 抽象类 + 两个实现

## Capabilities

### New Capabilities
- `filesystem-abstraction`: 统一文件系统接口——LocalFs + RemoteFs 两种 backend，通过 `fs_type` 参数切换

## Impact

- `src-tauri/src/fs/mod.rs`：`FsBackend` trait + `FileEntry` + 工厂函数
- `src-tauri/src/fs/local.rs`：`LocalFs` 实现
- `src-tauri/src/fs/remote.rs`：`RemoteFs` 实现
- `src-tauri/src/lib.rs`：注册 module 和 6 个 commands
- `src/fs/`：TypeScript 抽象类 + local/remote 实现
- `prd.yml`：功能记录
