## ADDED Requirements

### Requirement: 注册状态持久化存储

系统 SHALL 将注册状态（注册码 + 验证码 + HMAC 签名）持久化存储到 `~/.ArchBot/license.dat` 文件中，并在启动时进行完整性校验。

#### Scenario: 注册成功后写入 license 文件

- **WHEN** 用户输入有效验证码并通过校验
- **THEN** 系统将 `machine_id + verification_code + HMAC` 写入 `~/.ArchBot/license.dat`
- **AND** 系统标记当前状态为"已注册"

#### Scenario: 启动时读取并校验 license 文件

- **WHEN** 系统启动且 `~/.ArchBot/license.dat` 文件存在
- **THEN** 系统读取文件内容并重新计算 HMAC
- **AND** HMAC 一致则恢复注册状态为"已注册"
- **AND** HMAC 不一致则视为"未注册"（文件被篡改）

#### Scenario: license 文件不存在时视为未注册

- **WHEN** 系统启动且 `~/.ArchBot/license.dat` 文件不存在
- **THEN** 系统标记为"未注册"

#### Scenario: 注册码变更后原 license 失效

- **WHEN** 硬件变更导致注册码与 license 文件中存储的不一致
- **THEN** 系统视为"未注册"
