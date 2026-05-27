export interface FileMeta {
  version: number
  locked_by: string | null
  locked_at: string | null
  updated_by: string
  updated_at: string
  checksum: string
}

export interface DomainInfo {
  _meta: FileMeta
  name: string
  code: string
  owner: string
  description: string
}

export interface Conventions {
  _meta: FileMeta
  table_prefix: string
  field_naming: string
  primary_key: string
  audit_fields: string[]
  soft_delete: string
}

export interface EnumValue {
  code: string
  label: string
}

export interface EnumDef {
  _meta: FileMeta
  code: string
  name: string
  values: EnumValue[]
}

export interface FieldRule {
  rule_type: string
  value: string
}

export interface EntityField {
  code: string
  name: string
  field_type: string
  length: string | null
  nullable: boolean
  unique: boolean
  default_value: string | null
  enum_ref: string | null
  description: string | null
  rules: FieldRule[]
}

export interface IndexDef {
  fields: string[]
  unique: boolean
}

export interface Relation {
  target: string
  relation_type: string
  foreign_key: string
}

export interface EntityDef {
  _meta: FileMeta
  code: string
  name: string
  description: string
  sensitivity: string
  fields: EntityField[]
  indexes: IndexDef[]
  relations: Relation[]
}

export function newMeta(): FileMeta {
  return {
    version: 0,
    locked_by: null,
    locked_at: null,
    updated_by: 'local',
    updated_at: new Date().toISOString(),
    checksum: ''
  }
}

export function newEntityField(): EntityField {
  return {
    code: '',
    name: '',
    field_type: 'varchar',
    length: '255',
    nullable: true,
    unique: false,
    default_value: null,
    enum_ref: null,
    description: null,
    rules: []
  }
}

export function newEntity(): EntityDef {
  return {
    _meta: newMeta(),
    code: '',
    name: '',
    description: '',
    sensitivity: 'internal',
    fields: [],
    indexes: [],
    relations: []
  }
}

export function newEnumDef(): EnumDef {
  return {
    _meta: newMeta(),
    code: '',
    name: '',
    values: []
  }
}
