//! 本地文件系统后端
//!
//! 封装 `std::fs`，所有操作限制在配置的 `base_dir` 目录内，
//! 通过 `resolve()` 方法进行路径安全校验。

use async_trait::async_trait;
use std::path::{Path, PathBuf};

use super::{FileEntry, FsBackend};

/// 本地文件系统后端
///
/// 内部维护一个 `base_dir`，所有用户传入的路径都经过 `resolve()` 解析，
/// 确保最终访问的文件在 `base_dir` 子树内，防止路径穿越攻击。
pub struct LocalFs {
    base_dir: PathBuf,
}

impl LocalFs {
    /// 创建新的本地后端实例
    ///
    /// `base_dir` 指定文件操作的根目录，所有相对路径均以此为基础。
    pub fn new(base_dir: &str) -> Self {
        Self {
            base_dir: PathBuf::from(base_dir),
        }
    }

    /// 路径安全解析
    ///
    /// 业务逻辑：
    /// 1. 拒绝空路径
    /// 2. 将用户路径拼接到 base_dir（自动处理前导 /）
    /// 3. canonicalize 解析符号链接和 .. 组件
    /// 4. 校验结果路径在 base_dir 子树内
    ///
    /// 第 4 步是关键安全措施——即使 canonicalize 成功，
    /// 如果结果不在 base_dir 下（极少见，但仍需防御），则拒绝访问。
    fn resolve(&self, path: &str) -> Result<PathBuf, String> {
        if path.is_empty() {
            return Err("路径不能为空".into());
        }

        let user_path = Path::new(path);

        // 处理用户传入绝对路径的情况：去掉前导 / 后拼接到 base_dir
        let relative = user_path.strip_prefix("/").unwrap_or(user_path);
        let resolved = self.base_dir.join(relative);

        // canonicalize 解析 .. / . / 符号链接，返回真实绝对路径
        let canonical = resolved
            .canonicalize()
            .map_err(|e| format!("路径无效: {e}"))?;

        let base_canonical = self
            .base_dir
            .canonicalize()
            .unwrap_or_else(|_| self.base_dir.clone());

        // 防御层：确保解析后的路径仍在 base_dir 子树内
        if !canonical.starts_with(&base_canonical) {
            return Err("路径越界".into());
        }

        Ok(canonical)
    }

    /// 提取文件的最后修改时间
    ///
    /// 返回 Unix 时间戳字符串（秒级），不可用时返回空字符串。
    fn modified_time(meta: &std::fs::Metadata) -> String {
        meta.modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default()
    }
}

#[async_trait]
impl FsBackend for LocalFs {
    async fn read_file(&self, path: &str) -> Result<String, String> {
        let p = self.resolve(path)?;
        std::fs::read_to_string(&p).map_err(|e| format!("读取文件失败: {e}"))
    }

    async fn write_file(&self, path: &str, content: &str) -> Result<(), String> {
        let p = self.resolve(path)?;
        // 确保父目录存在
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
        }
        std::fs::write(&p, content).map_err(|e| format!("写入文件失败: {e}"))
    }

    async fn list_dir(&self, path: &str) -> Result<Vec<FileEntry>, String> {
        let dir = self.resolve(path)?;
        if !dir.is_dir() {
            return Err(format!("不是目录: {path}"));
        }
        let mut entries = Vec::new();
        let read = std::fs::read_dir(&dir).map_err(|e| format!("读取目录失败: {e}"))?;
        for entry in read {
            let entry = entry.map_err(|e| format!("读取条目失败: {e}"))?;
            let meta = entry
                .metadata()
                .map_err(|e| format!("读取元数据失败: {e}"))?;
            entries.push(FileEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_string_lossy().to_string(),
                is_dir: meta.is_dir(),
                size: meta.len(),
                modified: Self::modified_time(&meta),
            });
        }
        Ok(entries)
    }

    async fn delete(&self, path: &str) -> Result<(), String> {
        let p = self.resolve(path)?;
        // 根据类型选择删除方式
        if p.is_dir() {
            std::fs::remove_dir_all(&p).map_err(|e| format!("删除目录失败: {e}"))?;
        } else {
            std::fs::remove_file(&p).map_err(|e| format!("删除文件失败: {e}"))?;
        }
        Ok(())
    }

    async fn exists(&self, path: &str) -> Result<bool, String> {
        let p = self.resolve(path)?;
        Ok(p.exists())
    }

    async fn create_dir(&self, path: &str) -> Result<(), String> {
        let p = self.resolve(path)?;
        std::fs::create_dir_all(&p).map_err(|e| format!("创建目录失败: {e}"))
    }
}
