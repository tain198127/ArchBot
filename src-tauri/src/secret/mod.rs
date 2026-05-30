use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

/// Secret Manager：
/// - 加密存储 API key / token 到 `~/.archbot/secrets/`
/// - 密钥由机器 ID 派生（与 license 模块的 `get_machine_id_cmd` 一致）
/// - 支持 `secret://runtime/key` 引用格式解析
pub struct SecretManager {
    secrets_dir: PathBuf,
    cipher: Aes256Gcm,
}

impl SecretManager {
    /// 创建 SecretManager。
    ///
    /// `machine_id` 用作加密密钥的派生材料。
    /// 同一台机器上的 ArchBot 重启后可以用相同的 machine_id 解密。
    pub fn new(machine_id: &str) -> Result<Self, String> {
        let secrets_dir = dirs::home_dir()
            .unwrap_or_default()
            .join(".archbot")
            .join("secrets");

        fs::create_dir_all(&secrets_dir)
            .map_err(|e| format!("Failed to create secrets dir {:?}: {}", secrets_dir, e))?;

        // 从 machine_id 派生 AES-256 密钥
        let mut hasher = Sha256::new();
        hasher.update(b"archbot-secret-key-v1:");
        hasher.update(machine_id.as_bytes());
        let key_bytes = hasher.finalize();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        Ok(Self {
            secrets_dir,
            cipher,
        })
    }

    /// 加密并存储一个 secret
    pub fn store(&self, runtime: &str, key: &str, value: &str) -> Result<(), String> {
        // 生成 12 字节随机 nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, value.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        // 格式: nonce (12 bytes) || ciphertext
        let mut output = nonce_bytes.to_vec();
        output.extend_from_slice(&ciphertext);

        let file_path = self.secret_path(runtime, key);
        fs::write(&file_path, &output)
            .map_err(|e| format!("Failed to write secret {:?}: {}", file_path, e))?;

        // 设置权限 600
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&file_path, fs::Permissions::from_mode(0o600));
        }

        Ok(())
    }

    /// 解密并返回一个 secret
    pub fn get(&self, runtime: &str, key: &str) -> Result<String, String> {
        let file_path = self.secret_path(runtime, key);
        let data =
            fs::read(&file_path).map_err(|e| format!("Secret not found {:?}: {}", file_path, e))?;

        if data.len() < 12 {
            return Err("Secret data corrupt (too short)".to_string());
        }

        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| "Decryption failed — wrong machine or corrupt data".to_string())?;

        String::from_utf8(plaintext)
            .map_err(|e| format!("Secret is not valid UTF-8: {}", e))
    }

    /// 解析 `secret://runtime/key` 引用并返回实际值
    pub fn resolve(&self, reference: &str) -> Result<String, String> {
        let path = reference
            .strip_prefix("secret://")
            .ok_or_else(|| format!("Invalid secret reference: {}", reference))?;

        let parts: Vec<&str> = path.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid secret reference: {}", reference));
        }

        self.get(parts[0], parts[1])
    }

    /// 列出所有 runtime 的 secret key 名
    pub fn list_runtimes(&self) -> Vec<String> {
        let mut runtimes = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.secrets_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    runtimes.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        runtimes
    }

    fn secret_path(&self, runtime: &str, key: &str) -> PathBuf {
        let runtime_dir = self.secrets_dir.join(runtime);
        let _ = fs::create_dir_all(&runtime_dir);
        runtime_dir.join(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_get_roundtrip() {
        let sm = SecretManager::new("test-machine-001").unwrap();
        sm.store("test_runtime", "api_token", "sk-test-secret-value")
            .unwrap();

        let result = sm.get("test_runtime", "api_token").unwrap();
        assert_eq!(result, "sk-test-secret-value");
    }

    #[test]
    fn resolve_reference() {
        let sm = SecretManager::new("test-machine-002").unwrap();
        sm.store("claude_code", "api_token", "sk-claude-token")
            .unwrap();

        let result = sm.resolve("secret://claude_code/api_token").unwrap();
        assert_eq!(result, "sk-claude-token");
    }

    #[test]
    fn different_machine_cannot_decrypt() {
        let sm1 = SecretManager::new("machine-a").unwrap();
        sm1.store("rt", "key", "secret-value").unwrap();

        let sm2 = SecretManager::new("machine-b").unwrap();
        let result = sm2.get("rt", "key");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_reference_format() {
        let sm = SecretManager::new("test-machine-003").unwrap();
        assert!(sm.resolve("bad-format").is_err());
        assert!(sm.resolve("secret://missing-slash").is_err());
    }
}
