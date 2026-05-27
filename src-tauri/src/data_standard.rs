use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::now_iso;

/// 文件元信息，用于乐观锁和协作追踪
#[derive(Serialize, Deserialize, Clone)]
pub struct FileMeta {
    pub version: u64,
    pub locked_by: Option<String>,
    pub locked_at: Option<String>,
    pub updated_by: String,
    pub updated_at: String,
    pub checksum: String,
}

/// 域定义
#[derive(Serialize, Deserialize, Clone)]
pub struct DomainInfo {
    pub _meta: FileMeta,
    pub name: String,
    pub code: String,
    pub owner: String,
    pub description: String,
}

/// 编码规范
#[derive(Serialize, Deserialize, Clone)]
pub struct Conventions {
    pub _meta: FileMeta,
    pub table_prefix: String,
    pub field_naming: String,
    pub primary_key: String,
    pub audit_fields: Vec<String>,
    pub soft_delete: String,
}

/// 枚举值
#[derive(Serialize, Deserialize, Clone)]
pub struct EnumValue {
    pub code: String,
    pub label: String,
}

/// 枚举定义
#[derive(Serialize, Deserialize, Clone)]
pub struct EnumDef {
    pub _meta: FileMeta,
    pub code: String,
    pub name: String,
    pub values: Vec<EnumValue>,
}

/// 字段校验规则
#[derive(Serialize, Deserialize, Clone)]
pub struct FieldRule {
    pub rule_type: String,
    pub value: String,
}

/// 实体字段
#[derive(Serialize, Deserialize, Clone)]
pub struct EntityField {
    pub code: String,
    pub name: String,
    pub field_type: String,
    pub length: Option<String>,
    pub nullable: bool,
    pub unique: bool,
    pub default_value: Option<String>,
    pub enum_ref: Option<String>,
    pub description: Option<String>,
    pub rules: Vec<FieldRule>,
}

/// 索引定义
#[derive(Serialize, Deserialize, Clone)]
pub struct IndexDef {
    pub fields: Vec<String>,
    pub unique: bool,
}

/// 关联关系
#[derive(Serialize, Deserialize, Clone)]
pub struct Relation {
    pub target: String,
    pub relation_type: String,
    pub foreign_key: String,
}

/// 实体定义
#[derive(Serialize, Deserialize, Clone)]
pub struct EntityDef {
    pub _meta: FileMeta,
    pub code: String,
    pub name: String,
    pub description: String,
    pub sensitivity: String,
    pub fields: Vec<EntityField>,
    pub indexes: Vec<IndexDef>,
    pub relations: Vec<Relation>,
}

fn calc_checksum(content: &str) -> String {
    let mut hash: u64 = 5381;
    for b in content.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(b as u64);
    }
    format!("{hash:016x}")
}

fn new_meta(author: &str) -> FileMeta {
    FileMeta {
        version: 1,
        locked_by: None,
        locked_at: None,
        updated_by: author.to_string(),
        updated_at: now_iso(),
        checksum: String::new(),
    }
}

fn domain_dir(project_dir: &str, domain_code: &str) -> PathBuf {
    Path::new(project_dir)
        .join("data-standard")
        .join(domain_code)
}

/// 初始化数据标准域目录
///
/// 业务逻辑：
/// 1. 根据项目目录和域 code 创建目录结构：domain/entities/、domain/enums/
/// 2. 生成 _domain.yml（域基本信息）和 _conventions.yml（编码规范默认值）
/// 3. 序列化为 YAML 写入文件
/// 4. 返回域目录路径
#[tauri::command]
pub async fn ds_create_domain(
    project_dir: String,
    code: String,
    name: String,
    owner: String,
    description: String,
) -> Result<String, String> {
    let dir = domain_dir(&project_dir, &code);
    std::fs::create_dir_all(dir.join("entities")).map_err(|e| format!("创建目录失败: {e}"))?;
    std::fs::create_dir_all(dir.join("enums")).map_err(|e| format!("创建目录失败: {e}"))?;

    let domain = DomainInfo {
        _meta: new_meta("system"),
        name,
        code: code.clone(),
        owner,
        description,
    };
    let yaml = serde_yml::to_string(&domain).map_err(|e| format!("序列化失败: {e}"))?;
    std::fs::write(dir.join("_domain.yml"), &yaml).map_err(|e| format!("写入失败: {e}"))?;

    let conventions = Conventions {
        _meta: new_meta("system"),
        table_prefix: "t_".to_string(),
        field_naming: "snake_case".to_string(),
        primary_key: "id (bigint, auto_increment)".to_string(),
        audit_fields: vec![
            "created_at (datetime)".into(),
            "updated_at (datetime)".into(),
            "created_by (varchar(64))".into(),
            "updated_by (varchar(64))".into(),
        ],
        soft_delete: "deleted_at (datetime, nullable)".to_string(),
    };
    let yaml = serde_yml::to_string(&conventions).map_err(|e| format!("序列化失败: {e}"))?;
    std::fs::write(dir.join("_conventions.yml"), &yaml).map_err(|e| format!("写入失败: {e}"))?;

    Ok(dir.to_string_lossy().to_string())
}

/// 列出域下所有实体
#[tauri::command]
pub async fn ds_list_entities(
    project_dir: String,
    domain_code: String,
) -> Result<Vec<EntityDef>, String> {
    let dir = domain_dir(&project_dir, &domain_code).join("entities");
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut entities = Vec::new();
    let entries = std::fs::read_dir(&dir).map_err(|e| format!("读取目录失败: {e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取条目失败: {e}"))?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "yml") {
            let content =
                std::fs::read_to_string(&path).map_err(|e| format!("读取文件失败: {e}"))?;
            let entity: EntityDef =
                serde_yml::from_str(&content).map_err(|e| format!("解析失败: {e}"))?;
            entities.push(entity);
        }
    }
    Ok(entities)
}

/// 保存实体定义
///
/// 业务逻辑：
/// 1. 读取已有文件的 _meta.version（如果存在）
/// 2. 比对前端传入的 version，不匹配则拒绝保存（乐观锁冲突）
/// 3. version + 1，更新 updated_at 和 checksum
/// 4. 序列化为 YAML 写入文件
#[tauri::command]
pub async fn ds_save_entity(
    project_dir: String,
    domain_code: String,
    entity: EntityDef,
) -> Result<EntityDef, String> {
    let dir = domain_dir(&project_dir, &domain_code).join("entities");
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {e}"))?;

    let file_path = dir.join(format!("{}.yml", entity.code));

    if file_path.exists() {
        let existing = std::fs::read_to_string(&file_path).map_err(|e| format!("读取失败: {e}"))?;
        let existing_entity: EntityDef =
            serde_yml::from_str(&existing).map_err(|e| format!("解析失败: {e}"))?;
        if existing_entity._meta.version != entity._meta.version {
            return Err(format!(
                "版本冲突：当前版本 {}，你的版本 {}",
                existing_entity._meta.version, entity._meta.version
            ));
        }
    }

    let mut saved = entity;
    saved._meta.version += 1;
    saved._meta.updated_at = now_iso();
    saved._meta.checksum = String::new();

    let body_yaml = serde_yml::to_string(&saved).map_err(|e| format!("序列化失败: {e}"))?;
    saved._meta.checksum = calc_checksum(&body_yaml);
    let yaml = serde_yml::to_string(&saved).map_err(|e| format!("序列化失败: {e}"))?;

    std::fs::write(&file_path, &yaml).map_err(|e| format!("写入失败: {e}"))?;
    Ok(saved)
}

/// 删除实体定义
#[tauri::command]
pub async fn ds_delete_entity(
    project_dir: String,
    domain_code: String,
    entity_code: String,
) -> Result<(), String> {
    let path = domain_dir(&project_dir, &domain_code)
        .join("entities")
        .join(format!("{entity_code}.yml"));
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("删除失败: {e}"))?;
    }
    Ok(())
}

/// 列出域下所有枚举
#[tauri::command]
pub async fn ds_list_enums(
    project_dir: String,
    domain_code: String,
) -> Result<Vec<EnumDef>, String> {
    let dir = domain_dir(&project_dir, &domain_code).join("enums");
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut enums = Vec::new();
    let entries = std::fs::read_dir(&dir).map_err(|e| format!("读取目录失败: {e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取条目失败: {e}"))?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "yml") {
            let content =
                std::fs::read_to_string(&path).map_err(|e| format!("读取文件失败: {e}"))?;
            let enum_def: EnumDef =
                serde_yml::from_str(&content).map_err(|e| format!("解析失败: {e}"))?;
            enums.push(enum_def);
        }
    }
    Ok(enums)
}

/// 保存枚举定义（含乐观锁校验）
#[tauri::command]
pub async fn ds_save_enum(
    project_dir: String,
    domain_code: String,
    enum_def: EnumDef,
) -> Result<EnumDef, String> {
    let dir = domain_dir(&project_dir, &domain_code).join("enums");
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {e}"))?;

    let file_path = dir.join(format!("{}.yml", enum_def.code));

    if file_path.exists() {
        let existing = std::fs::read_to_string(&file_path).map_err(|e| format!("读取失败: {e}"))?;
        let existing_enum: EnumDef =
            serde_yml::from_str(&existing).map_err(|e| format!("解析失败: {e}"))?;
        if existing_enum._meta.version != enum_def._meta.version {
            return Err(format!(
                "版本冲突：当前版本 {}，你的版本 {}",
                existing_enum._meta.version, enum_def._meta.version
            ));
        }
    }

    let mut saved = enum_def;
    saved._meta.version += 1;
    saved._meta.updated_at = now_iso();
    saved._meta.checksum = String::new();

    let body_yaml = serde_yml::to_string(&saved).map_err(|e| format!("序列化失败: {e}"))?;
    saved._meta.checksum = calc_checksum(&body_yaml);
    let yaml = serde_yml::to_string(&saved).map_err(|e| format!("序列化失败: {e}"))?;

    std::fs::write(&file_path, &yaml).map_err(|e| format!("写入失败: {e}"))?;
    Ok(saved)
}

/// 删除枚举定义
#[tauri::command]
pub async fn ds_delete_enum(
    project_dir: String,
    domain_code: String,
    enum_code: String,
) -> Result<(), String> {
    let path = domain_dir(&project_dir, &domain_code)
        .join("enums")
        .join(format!("{enum_code}.yml"));
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("删除失败: {e}"))?;
    }
    Ok(())
}

/// 读取域信息
#[tauri::command]
pub async fn ds_load_domain(
    project_dir: String,
    domain_code: String,
) -> Result<DomainInfo, String> {
    let path = domain_dir(&project_dir, &domain_code).join("_domain.yml");
    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取失败: {e}"))?;
    serde_yml::from_str(&content).map_err(|e| format!("解析失败: {e}"))
}

/// 读取编码规范
#[tauri::command]
pub async fn ds_load_conventions(
    project_dir: String,
    domain_code: String,
) -> Result<Conventions, String> {
    let path = domain_dir(&project_dir, &domain_code).join("_conventions.yml");
    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取失败: {e}"))?;
    serde_yml::from_str(&content).map_err(|e| format!("解析失败: {e}"))
}

/// 列出项目下所有域
#[tauri::command]
pub async fn ds_list_domains(project_dir: String) -> Result<Vec<DomainInfo>, String> {
    let dir = Path::new(&project_dir).join("data-standard");
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut domains = Vec::new();
    let entries = std::fs::read_dir(&dir).map_err(|e| format!("读取目录失败: {e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("读取条目失败: {e}"))?;
        let path = entry.path();
        if path.is_dir() {
            let domain_file = path.join("_domain.yml");
            if domain_file.exists() {
                let content =
                    std::fs::read_to_string(&domain_file).map_err(|e| format!("读取失败: {e}"))?;
                let domain: DomainInfo =
                    serde_yml::from_str(&content).map_err(|e| format!("解析失败: {e}"))?;
                domains.push(domain);
            }
        }
    }
    Ok(domains)
}
