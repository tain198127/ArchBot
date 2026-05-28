## Context

ArchBot 是一个 Tauri 2 桌面应用（Rust 后端 + Vue 3 前端），需要加入软件许可注册系统。注册流程为离线模式：用户获取本机注册码 → 线下发送给开发者 → 开发者返回验证码 → 用户输入验证码完成注册。不使用在线授权服务器。

## Goals / Non-Goals

**Goals:**
- 基于硬件信息生成稳定的机器注册码（同一台机器每次生成相同结果）
- 验证码采用非对称加密（嵌入式公钥解密），防止用户伪造
- 注册状态持久化且防篡改
- 未注册时限制指定菜单功能（置灰不可点击）
- debug 模式绕过所有注册检查

**Non-Goals:**
- 不实现在线授权服务器
- 不实现试用期/倒计时
- 不实现浮动许可/并发用户管理
- 不实现远程吊销

## Decisions

### 指纹算法：MAC + 主机名 → SHA-256 前 16 字符

采集首个非 loopback 网卡的 MAC 地址 + 系统主机名，拼接后 SHA-256 哈希，取前 16 个 hex 字符作为注册码。

**替代方案考虑**：`machine-uid` crate（跨平台但依赖 systemd/DBus）、CPU 序列号（macOS 上不可用）、硬盘序列号（需要 admin 权限）。MAC + hostname 在主流平台上均可无特权获取。

### 验证码机制：RSA-2048 公钥解密比对

- **嵌入式公钥**：Rust 代码中硬编码 RSA-2048 公钥（PEM 格式）
- **验证码生成**（开发者侧）：`sign(machine_id)` → hex 字符串，使用对应的私钥
- **验证码校验**（应用侧）：公钥解密验证码 → 比对是否等于本机注册码
- **注册状态**：校验通过后，将 `machine_id + verification_code + HMAC` 写入 `~/.ArchBot/license.dat`

### 存储防篡改：RSA 签名重校验

`license.dat` 内容：
```json
{
  "machine_id": "a1b2c3d4e5f6a7b8",
  "verification_code": "..."
}
```

启动时读取文件后执行双重校验：
1. `validate_verification_code(data.machine_id, data.verification_code)` — RSA 公钥重新验证签名，证明开发者授权了该 machine_id
2. `data.machine_id == get_machine_id()` — 确保 license 属于当前硬件

**不使用 HMAC**：HMAC 密钥需从 machine_id 派生，而 machine_id 存储在文件中，形成循环依赖——攻击者篡改文件后可同时重新计算 HMAC。RSA 重校验的密钥（公钥）独立于文件内容，无法被绕过。

### 文件权限

Unix 系统上 license.dat 使用 `0o600` 权限写入，仅文件所有者可读写。

### 受限功能列表

在 Rust 中维护一个常量数组，存储未注册时禁用的菜单 action 字符串：
```rust
const RESTRICTED_ACTIONS: &[&str] = &[
    "run.genRequirement",
    "run.genDesign",
    "run.genCode",
    // ...
];
```

前端通过 Tauri command `get_license_status` 获取 `{ registered: bool, restricted_actions: string[] }`，然后对菜单项做置灰处理。

### 纵深防御：Rust 端 command 级 license gate

**仅前端置灰不够**。攻击者可以通过浏览器 devtools 直接调用 `__TAURI_INTERNALS__.invoke()` 绕过前端限制。因此每个受限 Tauri command 自身必须在执行前检查注册状态。

使用 Rust macro 实现零成本 guard：

```rust
// license.rs
macro_rules! license_gate {
    () => {
        if !crate::license::is_registered() {
            return Err("此功能需要注册后才能使用".into());
        }
    };
}

// 使用示例 (每个受限 command 顶部)
#[tauri::command]
async fn gen_requirement() -> Result<String, String> {
    license_gate!();  // 未注册直接返回 Err
    // ... 实际业务逻辑
}
```

`is_registered()` 读取内存中的全局状态 `AtomicBool`（启动时初始化），避免每次调用都读文件。

### 受限功能的双层防护矩阵

| 层 | 机制 | 作用 |
|----|------|------|
| 前端 UI | `get_license_status` → 置灰菜单 | 阻止普通用户点击 |
| Rust backend | `license_gate!()` macro → 每个 command 入口检查 | 阻止绕过前端直接调用 invoke

### Debug 模式

通过 Tauri 的 `std::env::var("ARCHBOT_DEBUG")` 或 CLI 参数 `--debug` 检测。在 `tauri.conf.json` 中配置 args。debug 模式下 `get_license_status` 始终返回 `registered: true` 且 `restricted_actions: []`。

## Risks / Trade-offs

- [R] MAC 地址可被用户手动修改 → 注册码不固定 → **Mitigation**：提示用户修改 MAC 后需重新注册
- [R] 公钥嵌入客户端，私钥泄露将导致可任意生成验证码 → **Mitigation**：私钥由开发者离线保管，不接触网络
- [R] license.dat 被删除则注册状态丢失 → **Mitigation**：提示用户备份该文件
- [R] SHA-256 前 16 字符碰撞概率 → 2^64 空间，实际无风险
