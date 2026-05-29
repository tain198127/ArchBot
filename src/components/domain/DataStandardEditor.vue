<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import VInput from '../base/VInput.vue'
import VSelect from '../base/VSelect.vue'
import VCheckbox from '../base/VCheckbox.vue'
import VButton from '../base/VButton.vue'
import VDialog from '../base/VDialog.vue'
import VFormItem from '../base/VFormItem.vue'
import VTextarea from '../base/VTextarea.vue'
import VTabs from '../base/VTabs.vue'
import { useToast } from '../../composables/useToast'
import { useI18n } from '../../i18n'
import { useProject } from '../../stores/project'
import { useProjectDir } from '../../composables/useProjectDir'
import type { DomainInfo, EntityDef, EnumDef, IndexDef } from '../../types/dataStandard'
import { newEntity, newEnumDef, newEntityField } from '../../types/dataStandard'

const props = defineProps<{
  initialDomainCode?: string
  initialFocus?: string
}>()

const { t } = useI18n()
const toast = useToast()
const { currentProject } = useProject()
const projectDir = useProjectDir()

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

const sensitivitySelectOptions = computed(() =>
  sensitivityOptions.map((o) => ({ value: o.value, label: ds[o.labelKey] || o.value }))
)

const fieldTypeOptions = computed(() =>
  fieldTypes.map((ft) => ({ value: ft, label: ft }))
)

const relationTypeOptions = [
  { value: 'one_to_one', label: 'one_to_one' },
  { value: 'one_to_many', label: 'one_to_many' },
  { value: 'many_to_many', label: 'many_to_many' },
]

const entityTabs = computed(() => [
  { value: 'fields', label: ds.field || 'Fields' },
  { value: 'indexes', label: ds.indexes || 'Indexes' },
  { value: 'relations', label: ds.relations || 'Relations' },
  { value: 'ddl', label: ds.previewDDL || 'DDL Preview' },
])

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
  } catch { /* backend may not be ready */ }
}

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
  } catch { /* ignore */ }
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
    toast.error(String(e))
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
    toast.success(ds.saved)
  } catch (e) {
    toast.error(String(e))
  }
}

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
    toast.success(ds.saved)
  } catch (e) {
    toast.error(String(e))
  }
}

async function deleteEntity(entity: EntityDef) {
  const ok = await toast.confirm('', ds.confirmDelete)
  if (!ok) return
  const dir = projectDir()
  if (!dir) return
  try {
    await invoke('ds_delete_entity', { projectDir: dir, domainCode: currentDomain.value, entityCode: entity.code })
    entities.value = entities.value.filter(e => e.code !== entity.code)
    if (selectedEntity.value?.code === entity.code) selectedEntity.value = null
  } catch (e) {
    toast.error(String(e))
  }
}

async function deleteEnum(enumDef: EnumDef) {
  const ok = await toast.confirm('', ds.confirmDelete)
  if (!ok) return
  const dir = projectDir()
  if (!dir) return
  try {
    await invoke('ds_delete_enum', { projectDir: dir, domainCode: currentDomain.value, enumCode: enumDef.code })
    enums.value = enums.value.filter(e => e.code !== enumDef.code)
    if (selectedEnum.value?.code === enumDef.code) selectedEnum.value = null
  } catch (e) {
    toast.error(String(e))
  }
}

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
  <div class="flex h-full overflow-hidden">
    <div class="flex-1 overflow-auto px-4 py-3">

      <!-- Entity Edit -->
      <template v-if="selectedEntity">
        <div class="flex justify-between items-start gap-2 mb-2 flex-wrap">
          <div class="flex flex-wrap items-end gap-3">
            <VFormItem :label="ds.entityCode" class="!mb-0">
              <VInput v-model="selectedEntity.code" class="!w-[140px]" />
            </VFormItem>
            <VFormItem :label="ds.entityName" class="!mb-0">
              <VInput v-model="selectedEntity.name" class="!w-[140px]" />
            </VFormItem>
            <VFormItem :label="ds.sensitivity" class="!mb-0">
              <VSelect v-model="selectedEntity.sensitivity" :options="sensitivitySelectOptions" class="!w-[100px]" />
            </VFormItem>
          </div>
          <div class="flex gap-2">
            <VButton size="sm" variant="ghost" @click="backToList">
              &larr; {{ ds.entityGroup || ds.entity }}
            </VButton>
            <VButton size="sm" @click="saveEntity">{{ ds.save }}</VButton>
            <VButton size="sm" variant="danger" @click="deleteEntity(selectedEntity)">
              {{ ds.deleteEntity }}
            </VButton>
          </div>
        </div>

        <VTabs v-model="activeTab" :tabs="entityTabs">
          <!-- Fields Tab -->
          <div v-if="activeTab === 'fields'" class="overflow-x-auto">
            <table class="ds-table w-full text-sm border-collapse">
              <thead>
                <tr>
                  <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary whitespace-nowrap">
                    {{ ds.fieldCode }}
                  </th>
                  <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary whitespace-nowrap">
                    {{ ds.fieldName }}
                  </th>
                  <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary whitespace-nowrap">
                    {{ ds.fieldType }}
                  </th>
                  <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary whitespace-nowrap">
                    {{ ds.fieldLength }}
                  </th>
                  <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary whitespace-nowrap">
                    {{ ds.fieldNullable }}
                  </th>
                  <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary whitespace-nowrap">
                    {{ ds.fieldUnique }}
                  </th>
                  <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary whitespace-nowrap">
                    {{ ds.fieldDefault }}
                  </th>
                  <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary whitespace-nowrap">
                    {{ ds.fieldDesc }}
                  </th>
                  <th class="w-8" />
                </tr>
              </thead>
              <tbody>
                <tr v-for="(field, idx) in selectedEntity.fields" :key="idx">
                  <td class="p-1 border-b border-border-light"><VInput v-model="field.code" /></td>
                  <td class="p-1 border-b border-border-light"><VInput v-model="field.name" /></td>
                  <td class="p-1 border-b border-border-light">
                    <VSelect v-model="field.field_type" :options="fieldTypeOptions" class="!w-[100px]" />
                  </td>
                  <td class="p-1 border-b border-border-light"><VInput v-model="field.length" class="!w-[70px]" /></td>
                  <td class="p-1 border-b border-border-light text-center">
                    <VCheckbox v-model="field.nullable" />
                  </td>
                  <td class="p-1 border-b border-border-light text-center">
                    <VCheckbox v-model="field.unique" />
                  </td>
                  <td class="p-1 border-b border-border-light"><VInput v-model="field.default_value" class="!w-[80px]" /></td>
                  <td class="p-1 border-b border-border-light"><VInput v-model="field.description" /></td>
                  <td class="p-1 border-b border-border-light">
                    <VButton size="sm" variant="danger" @click="removeField(idx)">&times;</VButton>
                  </td>
                </tr>
              </tbody>
            </table>
            <VButton size="sm" class="mt-2" @click="addField">+ {{ ds.addField }}</VButton>
          </div>

          <!-- Indexes Tab -->
          <div v-if="activeTab === 'indexes'" class="overflow-x-auto">
            <table class="ds-table w-full text-sm border-collapse">
              <thead>
                <tr>
                  <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary">{{ ds.indexFields }}</th>
                  <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary">{{ ds.indexUnique }}</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(idx, i) in selectedEntity.indexes" :key="i">
                  <td class="p-1 border-b border-border-light">
                    <VInput :model-value="fieldsToStr(idx.fields)" @update:model-value="onFieldsChange(idx, $event)" />
                  </td>
                  <td class="p-1 border-b border-border-light text-center">
                    <VCheckbox v-model="idx.unique" />
                  </td>
                </tr>
              </tbody>
            </table>
            <VButton size="sm" class="mt-2" @click="addIndex">+ {{ ds.addIndex }}</VButton>
          </div>

          <!-- Relations Tab -->
          <div v-if="activeTab === 'relations'" class="overflow-x-auto">
            <table class="ds-table w-full text-sm border-collapse">
              <thead>
                <tr>
                  <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary">{{ ds.relationTarget }}</th>
                  <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary">{{ ds.relationType }}</th>
                  <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary">{{ ds.relationFK }}</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(rel, i) in selectedEntity.relations" :key="i">
                  <td class="p-1 border-b border-border-light"><VInput v-model="rel.target" /></td>
                  <td class="p-1 border-b border-border-light">
                    <VSelect v-model="rel.relation_type" :options="relationTypeOptions" class="!w-[140px]" />
                  </td>
                  <td class="p-1 border-b border-border-light"><VInput v-model="rel.foreign_key" /></td>
                </tr>
              </tbody>
            </table>
            <VButton size="sm" class="mt-2" @click="addRelation">+ {{ ds.addRelation }}</VButton>
          </div>

          <!-- DDL Tab -->
          <div v-if="activeTab === 'ddl'">
            <pre class="bg-surface-100 dark:bg-surface-100 border border-border-default rounded-lg p-4 font-mono text-sm leading-relaxed overflow-x-auto whitespace-pre text-text-primary">{{ generateDDL() }}</pre>
          </div>
        </VTabs>
      </template>

      <!-- Enum Edit -->
      <template v-else-if="selectedEnum">
        <div class="flex justify-between items-start gap-2 mb-2 flex-wrap">
          <div class="flex flex-wrap items-end gap-3">
            <VFormItem :label="ds.enumCode" class="!mb-0">
              <VInput v-model="selectedEnum.code" class="!w-[140px]" />
            </VFormItem>
            <VFormItem :label="ds.enumName" class="!mb-0">
              <VInput v-model="selectedEnum.name" class="!w-[140px]" />
            </VFormItem>
          </div>
          <div class="flex gap-2">
            <VButton size="sm" variant="ghost" @click="backToList">
              &larr; {{ ds.dictGroup || ds.enum }}
            </VButton>
            <VButton size="sm" @click="saveEnum">{{ ds.save }}</VButton>
            <VButton size="sm" variant="danger" @click="deleteEnum(selectedEnum)">
              {{ ds.deleteEnum }}
            </VButton>
          </div>
        </div>

        <div class="overflow-x-auto">
          <table class="ds-table w-full text-sm border-collapse">
            <thead>
              <tr>
                <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary">{{ ds.enumValueCode }}</th>
                <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary">{{ ds.enumValueLabel }}</th>
                <th class="w-8" />
              </tr>
            </thead>
            <tbody>
              <tr v-for="(val, idx) in selectedEnum.values" :key="idx">
                <td class="p-1 border-b border-border-light"><VInput v-model="val.code" /></td>
                <td class="p-1 border-b border-border-light"><VInput v-model="val.label" /></td>
                <td class="p-1 border-b border-border-light">
                  <VButton size="sm" variant="danger" @click="removeEnumValue(idx)">&times;</VButton>
                </td>
              </tr>
            </tbody>
          </table>
          <VButton size="sm" class="mt-2" @click="addEnumValue">+ {{ ds.addEnumValue }}</VButton>
        </div>
      </template>

      <!-- Entity List View -->
      <div v-else-if="viewMode === 'entityList'" class="p-0">
        <div class="flex justify-between items-center mb-3">
          <h3 class="text-base font-semibold text-text-primary">{{ ds.entityGroup || ds.entity }}</h3>
          <VButton size="sm" @click="addNewEntity">+ {{ ds.newEntity }}</VButton>
        </div>
        <table v-if="entities.length > 0" class="ds-table w-full text-sm border-collapse">
          <thead>
            <tr>
              <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary">{{ ds.entityCode }}</th>
              <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary">{{ ds.entityName }}</th>
              <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary">{{ ds.entityDesc }}</th>
              <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary">{{ ds.sensitivity }}</th>
              <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary">{{ ds.field }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="entity in entities"
              :key="entity.code"
              class="cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-100 transition-colors"
              @click="selectEntity(entity)"
            >
              <td class="px-2 py-1 border-b border-border-light">{{ entity.code }}</td>
              <td class="px-2 py-1 border-b border-border-light">{{ entity.name }}</td>
              <td class="px-2 py-1 border-b border-border-light">{{ entity.description }}</td>
              <td class="px-2 py-1 border-b border-border-light">{{ ds['sensitivity' + entity.sensitivity.charAt(0).toUpperCase() + entity.sensitivity.slice(1)] || entity.sensitivity }}</td>
              <td class="px-2 py-1 border-b border-border-light">{{ entity.fields.length }}</td>
            </tr>
          </tbody>
        </table>
        <div v-else class="flex items-center justify-center h-48 text-sm text-text-muted">
          <p>{{ ds.entity }}</p>
        </div>
      </div>

      <!-- Enum List View -->
      <div v-else-if="viewMode === 'enumList'" class="p-0">
        <div class="flex justify-between items-center mb-3">
          <h3 class="text-base font-semibold text-text-primary">{{ ds.dictGroup || ds.enum }}</h3>
          <VButton size="sm" @click="addNewEnum">+ {{ ds.newEnum }}</VButton>
        </div>
        <table v-if="enums.length > 0" class="ds-table w-full text-sm border-collapse">
          <thead>
            <tr>
              <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary">{{ ds.enumCode }}</th>
              <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary">{{ ds.enumName }}</th>
              <th class="text-left px-2 py-1.5 bg-surface-100 dark:bg-surface-100 border-b border-border-default text-xs font-semibold text-text-secondary">{{ ds.enumValues }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="enumDef in enums"
              :key="enumDef.code"
              class="cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-100 transition-colors"
              @click="selectEnum(enumDef)"
            >
              <td class="px-2 py-1 border-b border-border-light">{{ enumDef.code }}</td>
              <td class="px-2 py-1 border-b border-border-light">{{ enumDef.name }}</td>
              <td class="px-2 py-1 border-b border-border-light">{{ enumDef.values.length }}</td>
            </tr>
          </tbody>
        </table>
        <div v-else class="flex items-center justify-center h-48 text-sm text-text-muted">
          <p>{{ ds.enum }}</p>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else class="flex items-center justify-center h-full text-sm text-text-muted">
        <p>{{ ds.title }}</p>
      </div>
    </div>

    <!-- New Domain Dialog -->
    <VDialog :visible="showNewDomainDialog" :title="ds.newDomain" @update:visible="showNewDomainDialog = $event">
      <VFormItem :label="ds.domainCode">
        <VInput v-model="newDomainForm.code" />
      </VFormItem>
      <VFormItem :label="ds.domainName">
        <VInput v-model="newDomainForm.name" />
      </VFormItem>
      <VFormItem :label="ds.domainOwner">
        <VInput v-model="newDomainForm.owner" />
      </VFormItem>
      <VFormItem :label="ds.domainDesc">
        <VTextarea v-model="newDomainForm.description" :rows="2" />
      </VFormItem>
      <template #footer>
        <VButton variant="secondary" @click="showNewDomainDialog = false">{{ t.newProject.cancel }}</VButton>
        <VButton @click="createDomain">{{ t.newProject.create }}</VButton>
      </template>
    </VDialog>
  </div>
</template>
