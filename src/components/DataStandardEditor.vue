<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useI18n } from '../i18n'
import { useProject } from '../stores/project'
import type { DomainInfo, EntityDef, EnumDef, IndexDef } from '../types/dataStandard'
import { newEntity, newEnumDef, newEntityField } from '../types/dataStandard'

const props = defineProps<{
  initialDomainCode?: string
  initialFocus?: string
}>()

const { t } = useI18n()
const { currentProject } = useProject()

const ds = t.value.dataStandard as Record<string, string>

const domains = ref<DomainInfo[]>([])
const currentDomain = ref<string>('')
const entities = ref<EntityDef[]>([])
const enums = ref<EnumDef[]>([])
const selectedEntity = ref<EntityDef | null>(null)
const selectedEnum = ref<EnumDef | null>(null)
const activeTab = ref('fields')
const viewMode = ref<'entityList' | 'enumList' | ''>('')
const showNewDomainDialog = ref(false)
const newDomainForm = ref({ name: '', code: '', owner: '', description: '' })

const fieldTypes = [
  'varchar', 'int', 'bigint', 'smallint', 'tinyint',
  'decimal', 'float', 'double', 'boolean',
  'text', 'longtext', 'date', 'datetime', 'timestamp',
  'json', 'enum'
]

const sensitivityOptions = [
  { value: 'public', labelKey: 'sensitivityPublic' },
  { value: 'internal', labelKey: 'sensitivityInternal' },
  { value: 'confidential', labelKey: 'sensitivityConfidential' },
  { value: 'secret', labelKey: 'sensitivitySecret' }
]

function projectDir(): string {
  if (!currentProject.value) return ''
  const p = currentProject.value.path
  const idx = Math.max(p.lastIndexOf('/'), p.lastIndexOf('\\'))
  return idx >= 0 ? p.substring(0, idx) : p
}

async function loadDomains() {
  const dir = projectDir()
  if (!dir) return
  try {
    domains.value = await invoke<DomainInfo[]>('ds_list_domains', { projectDir: dir })
    if (domains.value.length > 0 && !currentDomain.value) {
      currentDomain.value = props.initialDomainCode &&
        domains.value.some(d => d.code === props.initialDomainCode)
        ? props.initialDomainCode
        : domains.value[0].code
    }
  } catch (e) {
    console.error('Failed to load domains:', e)
  }
}

/**
 * 切换域后加载该域下的实体和枚举列表
 *
 * 业务逻辑：
 * 1. 清空当前选中的实体/枚举
 * 2. 并行请求实体列表和枚举列表
 * 3. 更新视图状态
 */
async function loadDomainData() {
  const dir = projectDir()
  if (!dir || !currentDomain.value) return
  selectedEntity.value = null
  selectedEnum.value = null
  try {
    const [ents, enm] = await Promise.all([
      invoke<EntityDef[]>('ds_list_entities', { projectDir: dir, domainCode: currentDomain.value }),
      invoke<EnumDef[]>('ds_list_enums', { projectDir: dir, domainCode: currentDomain.value })
    ])
    entities.value = ents
    enums.value = enm
  } catch (e) {
    console.error('Failed to load domain data:', e)
  }
}

watch(currentDomain, () => loadDomainData())
watch(() => currentProject.value, () => {
  currentDomain.value = ''
  loadDomains()
}, { immediate: true })
watch(() => props.initialDomainCode, (code) => {
  if (code && domains.value.some(d => d.code === code)) {
    currentDomain.value = code
  }
})
watch(() => props.initialFocus, (focus) => {
  if (focus === 'entity') viewMode.value = 'entityList'
  else if (focus === 'enum') viewMode.value = 'enumList'
})

function backToList() {
  selectedEntity.value = null
  selectedEnum.value = null
  viewMode.value = viewMode.value || 'entityList'
}

async function createDomain() {
  const dir = projectDir()
  if (!dir) return
  try {
    await invoke('ds_create_domain', {
      projectDir: dir,
      code: newDomainForm.value.code,
      name: newDomainForm.value.name,
      owner: newDomainForm.value.owner,
      description: newDomainForm.value.description
    })
    showNewDomainDialog.value = false
    newDomainForm.value = { name: '', code: '', owner: '', description: '' }
    await loadDomains()
    currentDomain.value = domains.value[domains.value.length - 1]?.code || ''
  } catch (e) {
    ElMessage.error(String(e))
  }
}

function selectEntity(entity: EntityDef) {
  selectedEntity.value = entity
  selectedEnum.value = null
  activeTab.value = 'fields'
  viewMode.value = ''
}

function selectEnum(enumDef: EnumDef) {
  selectedEnum.value = enumDef
  selectedEntity.value = null
  viewMode.value = ''
}

function addNewEntity() {
  const entity = newEntity()
  entity.code = `entity_${entities.value.length + 1}`
  entity.name = entity.code
  entities.value = [...entities.value, entity]
  selectEntity(entity)
}

function addNewEnum() {
  const enumDef = newEnumDef()
  enumDef.code = `enum_${enums.value.length + 1}`
  enumDef.name = enumDef.code
  enums.value = [...enums.value, enumDef]
  selectEnum(enumDef)
}

function addField() {
  if (!selectedEntity.value) return
  selectedEntity.value.fields = [...selectedEntity.value.fields, newEntityField()]
}

function removeField(index: number) {
  if (!selectedEntity.value) return
  selectedEntity.value.fields = selectedEntity.value.fields.filter((_, i) => i !== index)
}

function addEnumValue() {
  if (!selectedEnum.value) return
  selectedEnum.value.values = [...selectedEnum.value.values, { code: '', label: '' }]
}

function removeEnumValue(index: number) {
  if (!selectedEnum.value) return
  selectedEnum.value.values = selectedEnum.value.values.filter((_, i) => i !== index)
}

function addIndex() {
  if (!selectedEntity.value) return
  selectedEntity.value.indexes = [...selectedEntity.value.indexes, { fields: [], unique: false }]
}

function addRelation() {
  if (!selectedEntity.value) return
  selectedEntity.value.relations = [...selectedEntity.value.relations, { target: '', relation_type: 'one_to_many', foreign_key: '' }]
}

function fieldsToStr(fields: string[]): string {
  return (fields || []).join(', ')
}

function strToFields(val: unknown): string[] {
  return String(val ?? '')
    .split(',')
    .map(s => s.trim())
    .filter(Boolean)
}

function onFieldsChange(idx: IndexDef, val: unknown) {
  idx.fields = strToFields(val)
}

/**
 * 保存实体到后端文件
 *
 * 业务逻辑：
 * 1. 调用 Rust 后端 ds_save_entity 命令（含乐观锁校验）
 * 2. 后端返回新版本的实体（version+1），更新本地状态
 * 3. 同步更新左侧列表中对应的实体引用
 * 4. 版本冲突时后端会返回错误，前端展示提示
 */
async function saveEntity() {
  if (!selectedEntity.value) return
  const dir = projectDir()
  if (!dir) return
  try {
    const saved = await invoke<EntityDef>('ds_save_entity', {
      projectDir: dir,
      domainCode: currentDomain.value,
      entity: selectedEntity.value
    })
    selectedEntity.value = saved
    const idx = entities.value.findIndex(e => e.code === saved.code)
    if (idx >= 0) {
      entities.value = entities.value.map((e, i) => i === idx ? saved : e)
    }
    ElMessage.success(ds.saved)
  } catch (e) {
    ElMessage.error(String(e))
  }
}

/**
 * 保存枚举到后端文件
 *
 * 业务逻辑：
 * 1. 调用 Rust 后端 ds_save_enum 命令（含乐观锁校验）
 * 2. 后端返回新版本的枚举（version+1），更新本地状态
 * 3. 同步更新左侧列表中对应的枚举引用
 * 4. 版本冲突时后端会返回错误，前端展示提示
 */
async function saveEnum() {
  if (!selectedEnum.value) return
  const dir = projectDir()
  if (!dir) return
  try {
    const saved = await invoke<EnumDef>('ds_save_enum', {
      projectDir: dir,
      domainCode: currentDomain.value,
      enumDef: selectedEnum.value
    })
    selectedEnum.value = saved
    const idx = enums.value.findIndex(e => e.code === saved.code)
    if (idx >= 0) {
      enums.value = enums.value.map((e, i) => i === idx ? saved : e)
    }
    ElMessage.success(ds.saved)
  } catch (e) {
    ElMessage.error(String(e))
  }
}

async function deleteEntity(entity: EntityDef) {
  try {
    await ElMessageBox.confirm(ds.confirmDelete, '', { type: 'warning' })
  } catch { return }
  const dir = projectDir()
  if (!dir) return
  try {
    await invoke('ds_delete_entity', { projectDir: dir, domainCode: currentDomain.value, entityCode: entity.code })
    entities.value = entities.value.filter(e => e.code !== entity.code)
    if (selectedEntity.value?.code === entity.code) selectedEntity.value = null
  } catch (e) {
    ElMessage.error(String(e))
  }
}

async function deleteEnum(enumDef: EnumDef) {
  try {
    await ElMessageBox.confirm(ds.confirmDelete, '', { type: 'warning' })
  } catch { return }
  const dir = projectDir()
  if (!dir) return
  try {
    await invoke('ds_delete_enum', { projectDir: dir, domainCode: currentDomain.value, enumCode: enumDef.code })
    enums.value = enums.value.filter(e => e.code !== enumDef.code)
    if (selectedEnum.value?.code === enumDef.code) selectedEnum.value = null
  } catch (e) {
    ElMessage.error(String(e))
  }
}

/**
 * 根据当前实体定义生成 CREATE TABLE DDL
 *
 * 业务逻辑：
 * 1. 生成表头（CREATE TABLE + 主键 id）
 * 2. 遍历字段：拼接类型、长度、NOT NULL、UNIQUE、DEFAULT
 * 3. 追加审计字段（created_at/updated_at/created_by/updated_by）
 * 4. 追加软删除字段（deleted_at）
 * 5. 遍历索引定义生成 CREATE INDEX 语句
 */
function generateDDL(): string {
  if (!selectedEntity.value) return ''
  const e = selectedEntity.value
  const lines = [`CREATE TABLE ${e.code} (`]
  lines.push('  id BIGINT AUTO_INCREMENT PRIMARY KEY,')
  for (const f of e.fields) {
    let col = `  ${f.code} ${f.field_type.toUpperCase()}`
    if (f.length && f.field_type !== 'enum') col += `(${f.length})`
    if (!f.nullable) col += ' NOT NULL'
    if (f.unique) col += ' UNIQUE'
    if (f.default_value) col += ` DEFAULT '${f.default_value}'`
    col += ','
    lines.push(col)
  }
  lines.push('  created_at DATETIME,')
  lines.push('  updated_at DATETIME,')
  lines.push('  created_by VARCHAR(64),')
  lines.push('  updated_by VARCHAR(64),')
  lines.push('  deleted_at DATETIME')
  lines.push(');')

  for (const idx of e.indexes) {
    const unique = idx.unique ? 'UNIQUE ' : ''
    lines.push(`CREATE ${unique}INDEX idx_${e.code}_${idx.fields.join('_')} ON ${e.code} (${idx.fields.join(', ')});`)
  }
  return lines.join('\n')
}
</script>

<template>
  <div class="ds-editor">
    <!-- 右侧编辑区 -->
    <div class="ds-content">
      <!-- 实体编辑 -->
      <template v-if="selectedEntity">
        <div class="ds-entity-header">
          <el-form :inline="true" size="small">
            <el-form-item :label="ds.entityCode">
              <el-input v-model="selectedEntity.code" style="width:140px" />
            </el-form-item>
            <el-form-item :label="ds.entityName">
              <el-input v-model="selectedEntity.name" style="width:140px" />
            </el-form-item>
            <el-form-item :label="ds.sensitivity">
              <el-select v-model="selectedEntity.sensitivity" style="width:100px">
                <el-option
                  v-for="opt in sensitivityOptions"
                  :key="opt.value"
                  :value="opt.value"
                  :label="ds[opt.labelKey]"
                />
              </el-select>
            </el-form-item>
          </el-form>
          <div class="ds-header-actions">
            <el-button size="small" text @click="backToList">← {{ ds.entityGroup || ds.entity }}</el-button>
            <el-button type="primary" size="small" @click="saveEntity">{{ ds.save }}</el-button>
            <el-button type="danger" size="small" plain @click="deleteEntity(selectedEntity)">{{ ds.deleteEntity }}</el-button>
          </div>
        </div>

        <el-tabs v-model="activeTab" class="ds-tabs">
          <el-tab-pane :label="ds.field" name="fields">
            <div class="ds-table-wrap">
              <table class="ds-table">
                <thead>
                  <tr>
                    <th>{{ ds.fieldCode }}</th>
                    <th>{{ ds.fieldName }}</th>
                    <th>{{ ds.fieldType }}</th>
                    <th>{{ ds.fieldLength }}</th>
                    <th>{{ ds.fieldNullable }}</th>
                    <th>{{ ds.fieldUnique }}</th>
                    <th>{{ ds.fieldDefault }}</th>
                    <th>{{ ds.fieldDesc }}</th>
                    <th></th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(field, idx) in selectedEntity.fields" :key="idx">
                    <td><el-input v-model="field.code" size="small" /></td>
                    <td><el-input v-model="field.name" size="small" /></td>
                    <td>
                      <el-select v-model="field.field_type" size="small" style="width:100px">
                        <el-option v-for="ft in fieldTypes" :key="ft" :value="ft" :label="ft" />
                      </el-select>
                    </td>
                    <td><el-input v-model="field.length" size="small" style="width:70px" /></td>
                    <td><el-checkbox v-model="field.nullable" /></td>
                    <td><el-checkbox v-model="field.unique" /></td>
                    <td><el-input v-model="field.default_value" size="small" style="width:80px" /></td>
                    <td><el-input v-model="field.description" size="small" /></td>
                    <td><el-button size="small" type="danger" text @click="removeField(idx)">×</el-button></td>
                  </tr>
                </tbody>
              </table>
              <el-button size="small" class="ds-add-btn" @click="addField">+ {{ ds.addField }}</el-button>
            </div>
          </el-tab-pane>

          <el-tab-pane :label="ds.indexes" name="indexes">
            <div class="ds-table-wrap">
              <table class="ds-table">
                <thead>
                  <tr>
                    <th>{{ ds.indexFields }}</th>
                    <th>{{ ds.indexUnique }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(idx, i) in selectedEntity.indexes" :key="i">
                    <td>
                      <el-input
                        :model-value="fieldsToStr(idx.fields)"
                        @update:model-value="onFieldsChange(idx, $event)"
                        size="small"
                      />
                    </td>
                    <td><el-checkbox v-model="idx.unique" /></td>
                  </tr>
                </tbody>
              </table>
              <el-button size="small" class="ds-add-btn" @click="addIndex">+ {{ ds.addIndex }}</el-button>
            </div>
          </el-tab-pane>

          <el-tab-pane :label="ds.relations" name="relations">
            <div class="ds-table-wrap">
              <table class="ds-table">
                <thead>
                  <tr>
                    <th>{{ ds.relationTarget }}</th>
                    <th>{{ ds.relationType }}</th>
                    <th>{{ ds.relationFK }}</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(rel, i) in selectedEntity.relations" :key="i">
                    <td><el-input v-model="rel.target" size="small" /></td>
                    <td>
                      <el-select v-model="rel.relation_type" size="small" style="width:140px">
                        <el-option value="one_to_one" label="one_to_one" />
                        <el-option value="one_to_many" label="one_to_many" />
                        <el-option value="many_to_many" label="many_to_many" />
                      </el-select>
                    </td>
                    <td><el-input v-model="rel.foreign_key" size="small" /></td>
                  </tr>
                </tbody>
              </table>
              <el-button size="small" class="ds-add-btn" @click="addRelation">+ {{ ds.addRelation }}</el-button>
            </div>
          </el-tab-pane>

          <el-tab-pane :label="ds.previewDDL" name="ddl">
            <pre class="ds-ddl-preview">{{ generateDDL() }}</pre>
          </el-tab-pane>
        </el-tabs>
      </template>

      <!-- 枚举编辑 -->
      <template v-else-if="selectedEnum">
        <div class="ds-entity-header">
          <el-form :inline="true" size="small">
            <el-form-item :label="ds.enumCode">
              <el-input v-model="selectedEnum.code" style="width:140px" />
            </el-form-item>
            <el-form-item :label="ds.enumName">
              <el-input v-model="selectedEnum.name" style="width:140px" />
            </el-form-item>
          </el-form>
          <div class="ds-header-actions">
            <el-button size="small" text @click="backToList">← {{ ds.dictGroup || ds.enum }}</el-button>
            <el-button type="primary" size="small" @click="saveEnum">{{ ds.save }}</el-button>
            <el-button type="danger" size="small" plain @click="deleteEnum(selectedEnum)">{{ ds.deleteEnum }}</el-button>
          </div>
        </div>

        <div class="ds-table-wrap">
          <table class="ds-table">
            <thead>
              <tr>
                <th>{{ ds.enumValueCode }}</th>
                <th>{{ ds.enumValueLabel }}</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(val, idx) in selectedEnum.values" :key="idx">
                <td><el-input v-model="val.code" size="small" /></td>
                <td><el-input v-model="val.label" size="small" /></td>
                <td><el-button size="small" type="danger" text @click="removeEnumValue(idx)">×</el-button></td>
              </tr>
            </tbody>
          </table>
          <el-button size="small" class="ds-add-btn" @click="addEnumValue">+ {{ ds.addEnumValue }}</el-button>
        </div>
      </template>

      <!-- 实体列表视图 -->
      <div v-else-if="viewMode === 'entityList'" class="ds-list-view">
        <div class="ds-list-header">
          <h3>{{ ds.entityGroup || ds.entity }}</h3>
          <el-button size="small" type="primary" @click="addNewEntity">+ {{ ds.newEntity }}</el-button>
        </div>
        <table class="ds-table ds-list-table" v-if="entities.length > 0">
          <thead>
            <tr>
              <th>{{ ds.entityCode }}</th>
              <th>{{ ds.entityName }}</th>
              <th>{{ ds.entityDesc }}</th>
              <th>{{ ds.sensitivity }}</th>
              <th>{{ ds.field }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="entity in entities"
              :key="entity.code"
              class="ds-list-row"
              @click="selectEntity(entity)"
            >
              <td>{{ entity.code }}</td>
              <td>{{ entity.name }}</td>
              <td>{{ entity.description }}</td>
              <td>{{ ds['sensitivity' + entity.sensitivity.charAt(0).toUpperCase() + entity.sensitivity.slice(1)] || entity.sensitivity }}</td>
              <td>{{ entity.fields.length }}</td>
            </tr>
          </tbody>
        </table>
        <div v-else class="ds-empty">
          <p>{{ ds.entity }}</p>
        </div>
      </div>

      <!-- 字典列表视图 -->
      <div v-else-if="viewMode === 'enumList'" class="ds-list-view">
        <div class="ds-list-header">
          <h3>{{ ds.dictGroup || ds.enum }}</h3>
          <el-button size="small" type="primary" @click="addNewEnum">+ {{ ds.newEnum }}</el-button>
        </div>
        <table class="ds-table ds-list-table" v-if="enums.length > 0">
          <thead>
            <tr>
              <th>{{ ds.enumCode }}</th>
              <th>{{ ds.enumName }}</th>
              <th>{{ ds.enumValues }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="enumDef in enums"
              :key="enumDef.code"
              class="ds-list-row"
              @click="selectEnum(enumDef)"
            >
              <td>{{ enumDef.code }}</td>
              <td>{{ enumDef.name }}</td>
              <td>{{ enumDef.values.length }}</td>
            </tr>
          </tbody>
        </table>
        <div v-else class="ds-empty">
          <p>{{ ds.enum }}</p>
        </div>
      </div>

      <!-- 空状态 -->
      <div v-else class="ds-empty">
        <p>{{ ds.title }}</p>
      </div>
    </div>

    <!-- 新建域对话框 -->
    <el-dialog v-model="showNewDomainDialog" :title="ds.newDomain" width="420px">
      <el-form label-width="80px" size="default">
        <el-form-item :label="ds.domainCode">
          <el-input v-model="newDomainForm.code" />
        </el-form-item>
        <el-form-item :label="ds.domainName">
          <el-input v-model="newDomainForm.name" />
        </el-form-item>
        <el-form-item :label="ds.domainOwner">
          <el-input v-model="newDomainForm.owner" />
        </el-form-item>
        <el-form-item :label="ds.domainDesc">
          <el-input v-model="newDomainForm.description" type="textarea" :rows="2" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showNewDomainDialog = false">{{ t.newProject.cancel }}</el-button>
        <el-button type="primary" @click="createDomain">{{ t.newProject.create }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.ds-editor {
  display: flex;
  height: 100%;
  overflow: hidden;
}

.ds-content {
  flex: 1;
  overflow: auto;
  padding: 12px 16px;
}

.ds-entity-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 8px;
  flex-wrap: wrap;
  gap: 8px;
}

.ds-header-actions {
  display: flex;
  gap: 8px;
}

.ds-tabs {
  margin-top: 4px;
}

.ds-table-wrap {
  overflow-x: auto;
}

.ds-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.ds-table th {
  text-align: left;
  padding: 6px 8px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  font-weight: 600;
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
}

.ds-table td {
  padding: 4px 4px;
  border-bottom: 1px solid var(--border-light);
  vertical-align: middle;
}

.ds-add-btn {
  margin-top: 8px;
}

.ds-ddl-preview {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 16px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 13px;
  line-height: 1.6;
  overflow-x: auto;
  white-space: pre;
  color: var(--text-primary);
}

.ds-list-view {
  padding: 0;
}

.ds-list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.ds-list-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.ds-list-table {
  margin-top: 0;
}

.ds-list-row {
  cursor: pointer;
  transition: background 0.1s;
}

.ds-list-row:hover {
  background: var(--bg-hover);
}

.ds-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-muted);
  font-size: 14px;
}
</style>
