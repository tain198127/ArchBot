export interface ApiResponse<T> {
  success: boolean
  data: T
  error: string | null
}

export interface FileContent {
  name: string
  content: string
}

export interface DomainInfo {
  code: string
  name: string
  owner: string
  description: string
}

export interface EntityDef {
  _meta: EntityMeta
  code: string
  name: string
  description: string
  sensitivity: string
  fields: unknown[]
  indexes: unknown[]
  relations: unknown[]
}

export interface EntityMeta {
  version: number
  locked_by: string | null
  locked_at: string | null
  updated_by: string
  updated_at: string
  checksum: string
}

export interface EnumDef {
  _meta: EntityMeta
  code: string
  name: string
  values: EnumValue[]
}

export interface EnumValue {
  code: string
  name: string
  description: string
}

export interface Conventions {
  naming: Record<string, unknown>
  types: Record<string, unknown>
}

export interface FileEntry {
  name: string
  path: string
  is_dir: boolean
  size: number
  modified: string
}

export interface QueryParams {
  filters: unknown[]
  orderBy: unknown[]
  limit: number | null
  offset: number | null
}

export interface QueryResult {
  rows: Record<string, unknown>[]
  total: number
}

export interface DbRow {
  [key: string]: unknown
}

export interface ProjectScenario {
  profile: string
  dimensions: Record<string, unknown>
}

export interface ContextEntry {
  name: string
  description: string
  content: string
}

export interface DigitalEmployee {
  id: number
  code: string
  name: string
  role: string
  description: string
  avatar: string
  sort_order: number
  is_active: boolean
  created_at?: string
  updated_at?: string
}

export interface LicenseStatus {
  registered: boolean
  machine_id: string
  restricted_actions: string[]
}
