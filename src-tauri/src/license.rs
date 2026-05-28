use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};

static REGISTERED: AtomicBool = AtomicBool::new(false);
static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

/// 未注册时禁用的菜单 action 列表
const RESTRICTED_ACTIONS: &[&str] = &[
    "run.genRequirement",
    "run.genDesign",
    // "run.genCode",
    // "run.genSkeleton",
    // "run.genDbTable",
    // "run.genDataStandard",
    // "run.genCallChain",
    // "run.genTestCase",
    // "run.genE2eTest",
];

/// License 文件数据结构
#[derive(Serialize, Deserialize)]
struct LicenseData {
    machine_id: String,
    verification_code: String,
}

/// get_license_status 返回结构
#[derive(Serialize)]
pub struct LicenseStatus {
    pub registered: bool,
    pub machine_id: String,
    pub restricted_actions: Vec<String>,
}

// ─── Debug Mode ───────────────────────────────────────────────

pub fn is_debug_mode() -> bool {
    DEBUG_MODE.load(Ordering::Relaxed)
}

pub fn set_debug_mode(enabled: bool) {
    DEBUG_MODE.store(enabled, Ordering::Relaxed);
    if enabled {
        REGISTERED.store(true, Ordering::Relaxed);
    }
}

// ─── Registration State ───────────────────────────────────────

pub fn is_registered() -> bool {
    if DEBUG_MODE.load(Ordering::Relaxed) {
        return true;
    }
    REGISTERED.load(Ordering::Relaxed)
}

pub fn set_registered(val: bool) {
    REGISTERED.store(val, Ordering::Relaxed);
}

#[macro_export]
macro_rules! license_gate {
    () => {
        if !$crate::license::is_registered() {
            return Err("此功能需要注册后才能使用".into());
        }
    };
}

// ─── Machine Fingerprint ──────────────────────────────────────

fn get_mac_address() -> Option<String> {
    use std::process::Command;

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("ifconfig").args(["en0"]).output().ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("ether") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Some(parts[1].to_string().replace(':', "").to_uppercase());
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("ip").args(["link", "show"]).output().ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut found_lo = false;
        for line in stdout.lines() {
            if line.contains("link/ether") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Some(parts[1].to_string().replace(':', "").to_uppercase());
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("getmac").output().ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.contains("Media disconnected") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if !parts.is_empty() {
                    return Some(parts[0].to_string().replace('-', "").to_uppercase());
                }
            }
        }
    }

    None
}

fn get_machine_id() -> String {
    let mac = get_mac_address().unwrap_or_else(|| String::from("000000000000"));
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| String::from("unknown"));

    let mut hasher = Sha256::new();
    hasher.update(mac.as_bytes());
    hasher.update(b":");
    hasher.update(hostname.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..8])
}

// ─── License Validation ───────────────────────────────────────

const EMBEDDED_PUBLIC_KEY: &str = "\
-----BEGIN PUBLIC KEY-----
MIIBCgKCAQEA0Z3ENB+KqPl2TZHflg2PU2qAGKzRCj7G7L5bBYyrOqj2NPkZTp5E
BcBA0XpJGBlXJ7o5KNGr4LiYrIHCgY8DGn3lWJfQMA7KYAlV7kQpN0QZRxKFlE6l
R2NFkZGOF0qDjCOe8zLyhKErZmjDHBjHj4HiBZMOaqGVeBQqFJFnFTkDIwIDAQAB
-----END PUBLIC KEY-----";

fn validate_verification_code(machine_id: &str, code: &str) -> bool {
    use rsa::pkcs1v15::VerifyingKey;
    use rsa::pkcs8::DecodePublicKey;
    use rsa::signature::Verifier;

    let code_bytes = match hex::decode(code) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let public_key = match rsa::RsaPublicKey::from_public_key_pem(EMBEDDED_PUBLIC_KEY) {
        Ok(k) => k,
        Err(_) => return false,
    };

    let signature = match rsa::pkcs1v15::Signature::try_from(code_bytes.as_slice()) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let verifying_key = VerifyingKey::<sha2::Sha256>::new_unprefixed(public_key);
    verifying_key
        .verify(machine_id.as_bytes(), &signature)
        .is_ok()
}

// ─── License File I/O ─────────────────────────────────────────

fn license_path() -> std::path::PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    home.join(".ArchBot").join("license.dat")
}

fn save_license(machine_id: &str, verification_code: &str) -> Result<(), String> {
    let dir = license_path()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {e}"))?;

    let data = LicenseData {
        machine_id: machine_id.to_string(),
        verification_code: verification_code.to_string(),
    };

    let json = serde_json::to_string(&data).map_err(|e| format!("序列化失败: {e}"))?;

    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(license_path())
            .map_err(|e| format!("写入失败: {e}"))?;
        f.write_all(json.as_bytes())
            .map_err(|e| format!("写入失败: {e}"))?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(license_path(), &json).map_err(|e| format!("写入失败: {e}"))?;
    }

    Ok(())
}

fn load_license() -> Option<LicenseData> {
    let path = license_path();
    if !path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&path).ok()?;
    let data: LicenseData = serde_json::from_str(&content).ok()?;

    // Re-verify RSA signature — this proves the developer authorized this machine
    if !validate_verification_code(&data.machine_id, &data.verification_code) {
        return None;
    }

    // Verify the license belongs to this hardware
    let current_id = get_machine_id();
    if data.machine_id != current_id {
        return None;
    }

    Some(data)
}

// ─── Tauri Commands ───────────────────────────────────────────

#[tauri::command]
pub async fn get_machine_id_cmd() -> Result<String, String> {
    Ok(get_machine_id())
}

#[tauri::command]
pub async fn register_software(verification_code: String) -> Result<bool, String> {
    let machine_id = get_machine_id();

    if !validate_verification_code(&machine_id, &verification_code) {
        return Err("验证码无效".into());
    }

    save_license(&machine_id, &verification_code)?;
    set_registered(true);
    Ok(true)
}

#[tauri::command]
pub async fn get_license_status() -> Result<LicenseStatus, String> {
    let registered = is_registered();
    let machine_id = get_machine_id();

    let restricted_actions = if registered {
        vec![]
    } else {
        RESTRICTED_ACTIONS.iter().map(|s| s.to_string()).collect()
    };

    Ok(LicenseStatus {
        registered,
        machine_id,
        restricted_actions,
    })
}

// ─── Startup ──────────────────────────────────────────────────

pub fn check_license_on_startup() {
    if is_debug_mode() {
        set_registered(true);
        return;
    }

    if let Some(_license) = load_license() {
        set_registered(true);
    }
}
