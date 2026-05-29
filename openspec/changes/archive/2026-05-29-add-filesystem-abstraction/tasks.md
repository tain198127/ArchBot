## 1. Rust 依赖

- [x] 1.1 在 `src-tauri/Cargo.toml` 中添加 `async-trait` 依赖

## 2. Rust FsBackend trait + Local 实现

- [x] 2.1 新建 `src-tauri/src/fs/mod.rs` —— `FsBackend` trait + `FileEntry` 结构体 + backend 管理
- [x] 2.2 新建 `src-tauri/src/fs/local.rs` —— `LocalFs` 实现（6 个方法）
- [x] 2.3 新建 `src-tauri/src/fs/remote.rs` —— `RemoteFs` 实现（6 个方法 + base_url 配置）
- [x] 2.4 在 `src-tauri/src/lib.rs` 中注册 module 和 7 个 Tauri commands

## 3. 前端 FsClient

- [x] 3.1 新建 `src/fs/types.ts` —— `FileEntry` 接口
- [x] 3.2 新建 `src/fs/FsClient.ts` —— 抽象类（6 个方法）
- [x] 3.3 新建 `src/fs/LocalFsClient.ts` —— 调用 `invoke('fs_*', {fsType:'local'})`
- [x] 3.4 新建 `src/fs/RemoteFsClient.ts` —— 调用 `invoke('fs_*', {fsType:'remote'})`

## 4. PRD

- [x] 4.1 更新 `prd.yml`
