import { call } from './transport'
import type {
  ContextEntry,
  Conventions,
  DbRow,
  DigitalEmployee,
  DomainInfo,
  EntityDef,
  EnumDef,
  FileContent,
  FileEntry,
  LicenseStatus,
  ProjectScenario,
  QueryParams,
  QueryResult,
} from './types'

// ── Project ──

export function openProject(path: string) {
  return call<FileContent>('open_project', 'POST', '/project/open', { path })
}

export function createProject(dir: string, name: string) {
  return call<string>('create_project', 'POST', '/project/create', { dir, name })
}

export function initArchbotDir(projectPath: string) {
  return call<void>('init_archbot_dir', 'POST', '/project/init-dir', { project_path: projectPath })
}

export function ensureGitignore(projectPath: string) {
  return call<boolean>('ensure_gitignore', 'POST', '/project/ensure-gitignore', { project_path: projectPath })
}

export function readLocalFile(path: string) {
  return call<FileContent>('read_local_file', 'POST', '/project/read-file', { path })
}

// ── Data Standard ──

export function listDomains(projectDir: string) {
  return call<DomainInfo[]>(
    'ds_list_domains', 'GET',
    `/ds/domains?project_dir=${encodeURIComponent(projectDir)}`,
  )
}

export function createDomain(projectDir: string, code: string, name: string, owner: string, description: string) {
  return call<string>('ds_create_domain', 'POST', '/ds/domains', {
    project_dir: projectDir, code, name, owner, description,
  })
}

export function loadDomain(projectDir: string, domainCode: string) {
  return call<DomainInfo>(
    'ds_load_domain', 'GET',
    `/ds/domains/${encodeURIComponent(domainCode)}?project_dir=${encodeURIComponent(projectDir)}`,
  )
}

export function loadConventions(projectDir: string, domainCode: string) {
  return call<Conventions>(
    'ds_load_conventions', 'GET',
    `/ds/conventions?project_dir=${encodeURIComponent(projectDir)}&domain_code=${encodeURIComponent(domainCode)}`,
  )
}

export function listEntities(projectDir: string, domainCode: string) {
  return call<EntityDef[]>(
    'ds_list_entities', 'GET',
    `/ds/entities?project_dir=${encodeURIComponent(projectDir)}&domain_code=${encodeURIComponent(domainCode)}`,
  )
}

export function saveEntity(projectDir: string, domainCode: string, entity: EntityDef) {
  return call<EntityDef>('ds_save_entity', 'POST', '/ds/entities', {
    project_dir: projectDir, domain_code: domainCode, entity,
  })
}

export function deleteEntity(projectDir: string, domainCode: string, entityCode: string) {
  return call<void>('ds_delete_entity', 'DELETE', '/ds/entities', {
    project_dir: projectDir, domain_code: domainCode, entity_code: entityCode,
  })
}

export function listEnums(projectDir: string, domainCode: string) {
  return call<EnumDef[]>(
    'ds_list_enums', 'GET',
    `/ds/enums?project_dir=${encodeURIComponent(projectDir)}&domain_code=${encodeURIComponent(domainCode)}`,
  )
}

export function saveEnum(projectDir: string, domainCode: string, enumDef: EnumDef) {
  return call<EnumDef>('ds_save_enum', 'POST', '/ds/enums', {
    project_dir: projectDir, domain_code: domainCode, enum_def: enumDef,
  })
}

export function deleteEnum(projectDir: string, domainCode: string, enumCode: string) {
  return call<void>('ds_delete_enum', 'DELETE', '/ds/enums', {
    project_dir: projectDir, domain_code: domainCode, enum_code: enumCode,
  })
}

// ── Database ──

export function dbConnect(path: string) {
  return call<void>('db_connect', 'POST', '/db/connect', { path })
}

export function dbDisconnect() {
  return call<void>('db_disconnect', 'POST', '/db/disconnect')
}

export function dbFindAll(table: string, params: QueryParams, dbType: string) {
  return call<QueryResult>('db_find_all', 'POST', '/db/find-all', { table, params, db_type: dbType })
}

export function dbFindById(table: string, id: string, dbType: string) {
  return call<DbRow | null>('db_find_by_id', 'POST', '/db/find-by-id', { table, id, db_type: dbType })
}

export function dbInsert(table: string, data: DbRow, dbType: string) {
  return call<string>('db_insert', 'POST', '/db/insert', { table, data, db_type: dbType })
}

export function dbUpdate(table: string, id: string, data: DbRow, dbType: string) {
  return call<void>('db_update', 'POST', '/db/update', { table, id, data, db_type: dbType })
}

export function dbDelete(table: string, id: string, dbType: string) {
  return call<void>('db_delete', 'POST', '/db/delete', { table, id, db_type: dbType })
}

export function dbExecuteRaw(sql: string, dbType: string) {
  return call<QueryResult>('db_execute_raw', 'POST', '/db/execute-raw', { sql, db_type: dbType })
}

// ── Context Engineering ──

export function getContextConfig(projectPath: string, section: string) {
  return call<string>(
    'get_context_config', 'GET',
    `/context/config?project_path=${encodeURIComponent(projectPath)}&section=${encodeURIComponent(section)}`,
  )
}

export function saveContextConfig(projectPath: string, section: string, content: string) {
  return call<void>('save_context_config', 'POST', '/context/config', {
    project_path: projectPath, section, content,
  })
}

export function listContextEntries(projectPath: string, section: string) {
  return call<string[]>(
    'list_context_entries', 'GET',
    `/context/entries?project_path=${encodeURIComponent(projectPath)}&section=${encodeURIComponent(section)}`,
  )
}

export function getContextEntry(projectPath: string, section: string, name: string) {
  return call<ContextEntry>(
    'get_context_entry', 'GET',
    `/context/entry?project_path=${encodeURIComponent(projectPath)}&section=${encodeURIComponent(section)}&name=${encodeURIComponent(name)}`,
  )
}

export function saveContextEntry(projectPath: string, section: string, entry: ContextEntry) {
  return call<void>('save_context_entry', 'POST', '/context/entry', {
    project_path: projectPath, section, entry,
  })
}

export function deleteContextEntry(projectPath: string, section: string, name: string) {
  return call<void>('delete_context_entry', 'DELETE', '/context/entry', {
    project_path: projectPath, section, name,
  })
}

// ── Scenario ──

export function getScenario(projectPath: string) {
  return call<ProjectScenario>(
    'get_scenario', 'GET',
    `/scenario?project_path=${encodeURIComponent(projectPath)}`,
  )
}

export function saveScenario(projectPath: string, scenario: ProjectScenario) {
  return call<void>('save_scenario', 'POST', '/scenario', { project_path: projectPath, scenario })
}

// ── Digital Employee ──

export function deInit(dbType: string, projectPath: string) {
  return call<void>('de_init', 'POST', '/de/init', { db_type: dbType, project_path: projectPath })
}

export function deList(dbType: string) {
  return call<DigitalEmployee[]>(
    'de_list', 'GET',
    `/de/list?db_type=${encodeURIComponent(dbType)}`,
  )
}

export function deGet(code: string, dbType: string) {
  return call<DigitalEmployee | null>(
    'de_get', 'GET',
    `/de/get?code=${encodeURIComponent(code)}&db_type=${encodeURIComponent(dbType)}`,
  )
}

export function deSave(employee: DigitalEmployee, dbType: string) {
  return call<void>('de_save', 'POST', '/de/save', { employee, db_type: dbType })
}

export function deDelete(id: number, dbType: string) {
  return call<void>('de_delete', 'POST', '/de/delete', { id, db_type: dbType })
}

// ── Settings ──

export function loadSettings() {
  return call<string>('load_settings', 'GET', '/settings')
}

export function saveSettings(content: string) {
  return call<void>('save_settings', 'POST', '/settings', { content })
}

// ── License ──

export function getLicenseStatus() {
  return call<LicenseStatus>('get_license_status', 'GET', '/license/status')
}

export function registerSoftware(verificationCode: string) {
  return call<boolean>('register_software', 'POST', '/license/register', { verification_code: verificationCode })
}

// ── File System ──

export function fsConfigureLocal(baseDir: string) {
  return call<void>('fs_configure_local', 'POST', '/fs/configure-local', { base_dir: baseDir })
}

export function fsRead(path: string, fsType: string) {
  return call<string>('fs_read', 'POST', '/fs/read', { path, fs_type: fsType })
}

export function fsWrite(path: string, content: string, fsType: string) {
  return call<void>('fs_write', 'POST', '/fs/write', { path, content, fs_type: fsType })
}

export function fsList(path: string, fsType: string) {
  return call<FileEntry[]>('fs_list', 'POST', '/fs/list', { path, fs_type: fsType })
}

export function fsDelete(path: string, fsType: string) {
  return call<void>('fs_delete', 'POST', '/fs/delete', { path, fs_type: fsType })
}

export function fsExists(path: string, fsType: string) {
  return call<boolean>('fs_exists', 'POST', '/fs/exists', { path, fs_type: fsType })
}

export function fsMkdir(path: string, fsType: string) {
  return call<void>('fs_mkdir', 'POST', '/fs/mkdir', { path, fs_type: fsType })
}

// ── Vector / LanceDB ──

export function vecConnect(path: string) {
  return call<void>('vec_connect', 'POST', '/vec/connect', { path })
}

export function vecCreateTable(name: string, dimension: number, vecType: string) {
  return call<void>('vec_create_table', 'POST', '/vec/create-table', { name, dimension, vec_type: vecType })
}

export function vecInsert(table: string, id: string, vector: number[], vecType: string) {
  return call<void>('vec_insert', 'POST', '/vec/insert', { table, id, vector, vec_type: vecType })
}

export function vecSearch(table: string, queryVector: number[], topK: number, vecType: string) {
  return call<{ id: string; distance: number }[]>(
    'vec_search', 'POST', '/vec/search',
    { table, query_vector: queryVector, top_k: topK, vec_type: vecType },
  )
}

export function vecDelete(table: string, id: string, vecType: string) {
  return call<void>('vec_delete', 'POST', '/vec/delete', { table, id, vec_type: vecType })
}

export function vecListTables(vecType: string) {
  return call<string[]>(
    'vec_list_tables', 'GET',
    `/vec/tables?vec_type=${encodeURIComponent(vecType)}`,
  )
}

export function vecTableInfo(table: string, vecType: string) {
  return call<{ name: string; dimension: number }>(
    'vec_table_info', 'GET',
    `/vec/table-info?table=${encodeURIComponent(table)}&vec_type=${encodeURIComponent(vecType)}`,
  )
}
