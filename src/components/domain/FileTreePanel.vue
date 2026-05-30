<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import FileTreeContextMenu from './FileTreeContextMenu.vue'
import VDialog from '../base/VDialog.vue'
import VInput from '../base/VInput.vue'
import VFormItem from '../base/VFormItem.vue'
import VButton from '../base/VButton.vue'
import { useToast } from '../../composables/useToast'
import { useI18n } from '../../i18n'
import { useProject } from '../../stores/project'
import { useMenuAction } from '../../composables/useMenuAction'
import { useProjectDir } from '../../composables/useProjectDir'
import { projectCategories } from '../../config/projectDirs'
import type { ProjectDirChild } from '../../config/projectDirs'
import type { DomainInfo, EntityDef, EnumDef } from '../../types/dataStandard'

const { t } = useI18n()
const toast = useToast()
const { currentProject } = useProject()
const { emit: emitMenuAction } = useMenuAction()
const projectDir = useProjectDir()

const ds = t.value.dataStandard as Record<string, string>
const ctx = t.value.contextMenu as Record<string, string>

const DEFAULT_DOMAIN_CODE = 'default'

const domains = ref<DomainInfo[]>([])
const domainEntities = ref<Record<string, EntityDef[]>>({})
const domainEnums = ref<Record<string, EnumDef[]>>({})
const expandedCategories = ref<Set<string>>(new Set())
const expandedGroups = ref<Set<string>>(new Set())
const expandedDomains = ref<Set<string>>(new Set())
const selectedNode = ref('')

const ctxVisible = ref(false)
const ctxPos = ref({ x: 0, y: 0 })
const ctxType = ref<'dataStandard' | 'domain' | 'category' | 'group' | 'item'>('dataStandard')
const ctxDomain = ref('')
const ctxGroupKey = ref('')
const ctxItemKey = ref('')

const showNewDomain = ref(false)
const showNewEntity = ref(false)
const showNewDict = ref(false)
const activeDomainCode = ref('')
const newDomainForm = ref({ name: '', code: '', owner: '', description: '' })
const newEntityForm = ref({ code: '', name: '' })
const newDictForm = ref({ code: '', name: '' })

function getDirLabel(labelKey: string): string {
  return (t.value.projectDirs as Record<string, string>)[labelKey] || labelKey
}

async function loadDomains() {
  const dir = projectDir()
  if (!dir) return
  try {
    domains.value = await invoke<DomainInfo[]>('ds_list_domains', { projectDir: dir })
    for (const d of domains.value) {
      await loadDomainData(d.code)
    }
  } catch { /* backend may not be ready */ }
}

async function loadDomainData(domainCode: string) {
  const dir = projectDir()
  if (!dir || !domainCode) return
  try {
    const [ents, enm] = await Promise.all([
      invoke<EntityDef[]>('ds_list_entities', { projectDir: dir, domainCode }),
      invoke<EnumDef[]>('ds_list_enums', { projectDir: dir, domainCode })
    ])
    domainEntities.value = { ...domainEntities.value, [domainCode]: ents }
    domainEnums.value = { ...domainEnums.value, [domainCode]: enm }
  } catch { /* ignore */ }
}

watch(() => currentProject.value, () => {
  domains.value = []
  domainEntities.value = {}
  domainEnums.value = {}
  expandedCategories.value = new Set()
  expandedGroups.value = new Set()
  expandedDomains.value = new Set()
  if (currentProject.value) loadDomains()
}, { immediate: true })

function toggleCategory(key: string) {
  const next = new Set(expandedCategories.value)
  if (next.has(key)) next.delete(key); else next.add(key)
  expandedCategories.value = next
}

function toggleGroup(groupKey: string) {
  const next = new Set(expandedGroups.value)
  if (next.has(groupKey)) next.delete(groupKey); else next.add(groupKey)
  expandedGroups.value = next
}

function toggleDomain(code: string) {
  const next = new Set(expandedDomains.value)
  if (next.has(code)) next.delete(code); else next.add(code)
  expandedDomains.value = next
}

function isCategoryExpanded(key: string) { return expandedCategories.value.has(key) }
function isGroupExpanded(key: string) { return expandedGroups.value.has(key) }
function isDomainExpanded(code: string) { return expandedDomains.value.has(code) }

function handleGroupClick(group: { key: string; labelKey: string }) {
  toggleGroup(group.key)
  selectedNode.value = `group.${group.key}`
}

function handleGroupChildClick(child: { key: string; labelKey: string }) {
  selectedNode.value = `item.${child.key}`
  emitMenuAction(`open.${child.key}`)
}

function handleDirClick(child: ProjectDirChild) {
  selectedNode.value = `dir.${child.key}`
  emitMenuAction(`open.${child.key}`)
}

function handleCategoryClick(catKey: string) {
  toggleCategory(catKey)
  selectedNode.value = `cat.${catKey}`
}

function domainName(code: string): string {
  return domains.value.find(d => d.code === code)?.name || code
}

function handleDomainClick(domainCode: string) {
  selectedNode.value = `domain.${domainCode}`
  emitMenuAction({ action: 'open.dataStandard', payload: { domainCode, domainName: domainName(domainCode) } })
}

function handleEntityGroupClick(domainCode: string) {
  selectedNode.value = `entity.${domainCode}`
  emitMenuAction({ action: 'open.dataStandard', payload: { domainCode, domainName: domainName(domainCode), focus: 'entity' } })
}

function handleDictGroupClick(domainCode: string) {
  selectedNode.value = `enum.${domainCode}`
  emitMenuAction({ action: 'open.dataStandard', payload: { domainCode, domainName: domainName(domainCode), focus: 'enum' } })
}

// ── context menu helpers ──
function showContextMenu(event: MouseEvent, type: typeof ctxType.value, opts?: { domain?: string; groupKey?: string; itemKey?: string }) {
  event.preventDefault()
  event.stopPropagation()
  ctxType.value = type
  ctxDomain.value = opts?.domain || ''
  ctxGroupKey.value = opts?.groupKey || ''
  ctxItemKey.value = opts?.itemKey || ''
  ctxPos.value = { x: event.clientX, y: event.clientY }
  ctxVisible.value = true
  nextTick(() => document.addEventListener('click', () => { ctxVisible.value = false }, { once: true }))
}

// ── context menu actions ──
function onCtxNewDomain() { ctxVisible.value = false; showNewDomain.value = true }
function onCtxNewEntity() {
  ctxVisible.value = false
  if (ctxDomain.value) { activeDomainCode.value = ctxDomain.value; showNewEntity.value = true }
  else ensureDefaultDomain(() => { activeDomainCode.value = DEFAULT_DOMAIN_CODE; showNewEntity.value = true })
}
function onCtxNewDict() {
  ctxVisible.value = false
  if (ctxDomain.value) { activeDomainCode.value = ctxDomain.value; showNewDict.value = true }
  else ensureDefaultDomain(() => { activeDomainCode.value = DEFAULT_DOMAIN_CODE; showNewDict.value = true })
}
function onCtxImportFile() { ctxVisible.value = false; toast.info(ctx.importFileHint) }
function onCtxReverseDb() { ctxVisible.value = false; toast.info(ctx.reverseDbHint) }
function onCtxReverseDdl() { ctxVisible.value = false; toast.info(ctx.reverseDdlHint) }
function onCtxReverseCode() { ctxVisible.value = false; emitMenuAction({ action: 'open.dataModel', payload: { mode: 'reverse', source: 'code' } }) }

function onCtxAction(actionType: string) {
  ctxVisible.value = false
  if (ctxType.value === 'group') {
    emitMenuAction({ action: `ctxmenu.${actionType}`, payload: { groupKey: ctxGroupKey.value } })
    toast.info(`${ctx[actionType as keyof typeof ctx] || actionType}: ${getDirLabel(ctxGroupKey.value)}`)
  } else if (ctxType.value === 'category') {
    emitMenuAction({ action: `ctxmenu.${actionType}`, payload: { categoryKey: ctxGroupKey.value } })
    toast.info(`${ctx[actionType as keyof typeof ctx] || actionType}`)
  } else if (ctxType.value === 'item') {
    emitMenuAction({ action: `ctxmenu.${actionType}`, payload: { groupKey: ctxGroupKey.value, itemKey: ctxItemKey.value } })
    toast.info(`${ctx[actionType as keyof typeof ctx] || actionType}: ${getDirLabel(ctxItemKey.value)}`)
  }
}

async function ensureDefaultDomain(onDone: () => void) {
  if (domains.value.length > 0) { activeDomainCode.value = domains.value[0].code; onDone(); return }
  const dir = projectDir()
  if (!dir) return
  try {
    await invoke('ds_create_domain', { projectDir: dir, code: DEFAULT_DOMAIN_CODE, name: ds.defaultDomain, owner: '', description: '' })
    await loadDomains()
    onDone()
  } catch (e) { toast.error(String(e)) }
}

// ── dialogs ──
async function createDomain() {
  const dir = projectDir()
  if (!dir) return
  try {
    await invoke('ds_create_domain', { projectDir: dir, code: newDomainForm.value.code, name: newDomainForm.value.name, owner: newDomainForm.value.owner, description: newDomainForm.value.description })
    showNewDomain.value = false
    newDomainForm.value = { name: '', code: '', owner: '', description: '' }
    await loadDomains()
    expandedGroups.value = new Set([...expandedGroups.value, 'dataStandard'])
  } catch (e) { toast.error(String(e)) }
}

async function createEntity() {
  const dir = projectDir()
  if (!dir || !activeDomainCode.value) return
  try {
    await invoke('ds_save_entity', { projectDir: dir, domainCode: activeDomainCode.value, entity: { _meta: { version: 0, locked_by: null, locked_at: null, updated_by: 'local', updated_at: new Date().toISOString(), checksum: '' }, code: newEntityForm.value.code, name: newEntityForm.value.name, description: '', sensitivity: 'internal', fields: [], indexes: [], relations: [] } })
    showNewEntity.value = false
    newEntityForm.value = { code: '', name: '' }
    await loadDomainData(activeDomainCode.value)
    emitMenuAction({ action: 'open.dataStandard', payload: { domainCode: activeDomainCode.value } })
  } catch (e) { toast.error(String(e)) }
}

async function createDict() {
  const dir = projectDir()
  if (!dir || !activeDomainCode.value) return
  try {
    await invoke('ds_save_enum', { projectDir: dir, domainCode: activeDomainCode.value, enumDef: { _meta: { version: 0, locked_by: null, locked_at: null, updated_by: 'local', updated_at: new Date().toISOString(), checksum: '' }, code: newDictForm.value.code, name: newDictForm.value.name, values: [] } })
    showNewDict.value = false
    newDictForm.value = { code: '', name: '' }
    await loadDomainData(activeDomainCode.value)
    emitMenuAction({ action: 'open.dataStandard', payload: { domainCode: activeDomainCode.value } })
  } catch (e) { toast.error(String(e)) }
}

function entityCount(domainCode: string): number { return domainEntities.value[domainCode]?.length || 0 }
function enumCount(domainCode: string): number { return domainEnums.value[domainCode]?.length || 0 }
</script>

<template>
  <div class="flex flex-col h-full bg-surface-50 dark:bg-surface-50">
    <div class="flex items-center h-8 px-3 border-b border-border-default shrink-0">
      <span class="text-xs font-semibold text-text-secondary uppercase">{{ t.panel.project }}</span>
    </div>
    <div class="flex-1 overflow-y-auto py-1">
      <template v-if="currentProject">
        <div class="text-sm font-semibold text-text-primary px-3 py-2">{{ currentProject.name }}</div>
        <div class="flex flex-col">
          <template v-for="cat in projectCategories" :key="cat.key">
            <!-- category header -->
            <div
              class="tree-item"
              :class="{ 'bg-surface-200 dark:bg-surface-200': selectedNode === `cat.${cat.key}` }"
              @click="handleCategoryClick(cat.key)"
              @contextmenu="showContextMenu($event, 'category', { groupKey: cat.key })"
            >
              <svg class="tree-chevron" :class="{ 'rotate-90': isCategoryExpanded(cat.key) }" viewBox="0 0 24 24">
                <path fill="currentColor" d="M8 5l8 7-8 7z"/>
              </svg>
              <svg class="tree-icon" viewBox="0 0 24 24" :style="{ color: cat.color }">
                <path fill="currentColor" d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
              </svg>
              <span class="text-sm text-text-primary">{{ getDirLabel(cat.labelKey) }}</span>
            </div>

            <!-- expanded children: each nesting level adds pl-5 (20px) -->
            <div v-if="isCategoryExpanded(cat.key)" class="pl-5 flex flex-col">
              <!-- GROUPS -->
              <template v-if="cat.groups">
                <template v-for="group in cat.groups" :key="group.key">
                  <div
                    class="tree-item"
                    :class="{ 'bg-surface-200 dark:bg-surface-200': selectedNode === `group.${group.key}` }"
                    @click="handleGroupClick(group)"
                    @contextmenu="showContextMenu($event, 'group', { groupKey: group.key })"
                  >
                    <svg class="tree-chevron" :class="{ 'rotate-90': isGroupExpanded(group.key) }" viewBox="0 0 24 24" @click.stop="toggleGroup(group.key)">
                      <path fill="currentColor" d="M8 5l8 7-8 7z"/>
                    </svg>
                    <svg class="tree-icon" viewBox="0 0 24 24" :style="{ color: group.color }">
                      <path fill="currentColor" d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
                    </svg>
                    <span class="text-sm text-text-primary">{{ getDirLabel(group.labelKey) }}</span>
                  </div>

                  <div v-if="isGroupExpanded(group.key)" class="pl-5 flex flex-col">
                    <!-- dataStandard: children + domain tree -->
                    <template v-if="group.key === 'dataStandard'">
                      <div
                        v-for="child in group.children" :key="child.key"
                        class="tree-item"
                        :class="{ 'bg-surface-200 dark:bg-surface-200': selectedNode === `item.${child.key}` }"
                        @click="handleGroupChildClick(child)"
                        @contextmenu="showContextMenu($event, 'item', { groupKey: group.key, itemKey: child.key })"
                      >
                        <svg class="tree-icon" viewBox="0 0 24 24" :style="{ color: child.color }">
                          <path fill="currentColor" d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
                        </svg>
                        <span class="text-sm text-text-primary">{{ getDirLabel(child.labelKey) }}</span>
                      </div>
                      <div class="h-px mx-3 my-1 bg-border-default" />
                      <!-- domain tree -->
                      <template v-for="domain in domains" :key="domain.code">
                        <div class="flex flex-col">
                          <div
                            class="tree-item"
                            :class="{ 'bg-surface-200 dark:bg-surface-200': selectedNode === `domain.${domain.code}` }"
                            @click="handleDomainClick(domain.code)"
                            @contextmenu="showContextMenu($event, 'domain', { domain: domain.code })"
                          >
                            <svg class="tree-chevron" :class="{ 'rotate-90': isDomainExpanded(domain.code) }" viewBox="0 0 24 24" @click.stop="toggleDomain(domain.code)">
                              <path fill="currentColor" d="M8 5l8 7-8 7z"/>
                            </svg>
                            <svg class="tree-icon" viewBox="0 0 24 24" style="color: #a0cfff">
                              <path fill="currentColor" d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
                            </svg>
                            <span class="text-sm text-text-primary">{{ domain.name || domain.code }}</span>
                          </div>
                          <div v-if="isDomainExpanded(domain.code)" class="pl-5 flex flex-col">
                            <div
                              class="tree-item"
                              :class="{ 'bg-surface-200 dark:bg-surface-200': selectedNode === `entity.${domain.code}` }"
                              @click="handleEntityGroupClick(domain.code)"
                            >
                              <svg class="tree-icon" viewBox="0 0 24 24" style="color: #67c23a">
                                <path fill="currentColor" d="M3 13h8V3H3v10zm0 8h8v-6H3v6zm10 0h8V11h-8v10zm0-18v6h8V3h-8z"/>
                              </svg>
                              <span class="text-sm text-text-primary">{{ ds.entityGroup }}</span>
                              <span class="ml-auto text-xs text-text-muted">{{ entityCount(domain.code) }}</span>
                            </div>
                            <div
                              class="tree-item"
                              :class="{ 'bg-surface-200 dark:bg-surface-200': selectedNode === `enum.${domain.code}` }"
                              @click="handleDictGroupClick(domain.code)"
                            >
                              <svg class="tree-icon" viewBox="0 0 24 24" style="color: #e6a23c">
                                <path fill="currentColor" d="M4 6H2v14c0 1.1.9 2 2 2h14v-2H4V6zm16-4H8c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm0 14H8V4h12v12z"/>
                              </svg>
                              <span class="text-sm text-text-primary">{{ ds.dictGroup }}</span>
                              <span class="ml-auto text-xs text-text-muted">{{ enumCount(domain.code) }}</span>
                            </div>
                          </div>
                        </div>
                      </template>
                      <div v-if="domains.length === 0" class="py-1 text-xs text-text-muted italic">{{ ds.defaultDomain }}</div>
                    </template>

                    <!-- regular group children -->
                    <template v-else>
                      <div
                        v-for="child in group.children" :key="child.key"
                        class="tree-item"
                        :class="{ 'bg-surface-200 dark:bg-surface-200': selectedNode === `item.${child.key}` }"
                        @click="handleGroupChildClick(child)"
                        @contextmenu="showContextMenu($event, 'item', { groupKey: group.key, itemKey: child.key })"
                      >
                        <svg class="tree-icon" viewBox="0 0 24 24" :style="{ color: child.color }">
                          <path fill="currentColor" d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
                        </svg>
                        <span class="text-sm text-text-primary">{{ getDirLabel(child.labelKey) }}</span>
                      </div>
                    </template>
                  </div>
                </template>
              </template>

              <!-- FLAT: categories without groups -->
              <template v-else-if="cat.children">
                <div
                  v-for="child in cat.children" :key="child.key"
                  class="tree-item"
                  :class="{ 'bg-surface-200 dark:bg-surface-200': selectedNode === `dir.${child.key}` }"
                  @click="handleDirClick(child)"
                  @contextmenu="showContextMenu($event, 'dataStandard')"
                >
                  <svg class="tree-icon" viewBox="0 0 24 24" :style="{ color: child.color }">
                    <path fill="currentColor" d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
                  </svg>
                  <span class="text-sm text-text-primary">{{ getDirLabel(child.labelKey) }}</span>
                </div>
              </template>
            </div>
          </template>
        </div>
      </template>
      <div v-else class="flex items-center justify-center h-full">
        <p class="text-sm text-text-muted">{{ t.panel.openProjectHint }}</p>
      </div>
    </div>

    <!-- context menu -->
    <FileTreeContextMenu
      :visible="ctxVisible"
      :x="ctxPos.x"
      :y="ctxPos.y"
      :type="ctxType"
      :domain="ctxDomain"
      :group-key="ctxGroupKey"
      :item-key="ctxItemKey"
      @new-domain="onCtxNewDomain"
      @new-entity="onCtxNewEntity"
      @new-dict="onCtxNewDict"
      @import-file="onCtxImportFile"
      @reverse-db="onCtxReverseDb"
      @reverse-ddl="onCtxReverseDdl"
      @reverse-code="onCtxReverseCode"
      @action="onCtxAction"
    />

    <!-- new domain dialog -->
    <VDialog :visible="showNewDomain" :title="ds.newDomain" @update:visible="showNewDomain = $event">
      <VFormItem :label="ds.domainCode"><VInput v-model="newDomainForm.code" placeholder="code" /></VFormItem>
      <VFormItem :label="ds.domainName"><VInput v-model="newDomainForm.name" placeholder="name" /></VFormItem>
      <VFormItem :label="ds.domainOwner"><VInput v-model="newDomainForm.owner" placeholder="owner" /></VFormItem>
      <VFormItem :label="ds.domainDesc"><VInput v-model="newDomainForm.description" placeholder="description" /></VFormItem>
      <template #footer>
        <VButton variant="secondary" @click="showNewDomain = false">{{ t.newProject.cancel }}</VButton>
        <VButton @click="createDomain">{{ t.newProject.create }}</VButton>
      </template>
    </VDialog>

    <!-- new entity dialog -->
    <VDialog :visible="showNewEntity" :title="ctx.newEntity" @update:visible="showNewEntity = $event">
      <VFormItem :label="ds.entityCode"><VInput v-model="newEntityForm.code" placeholder="code" /></VFormItem>
      <VFormItem :label="ds.entityName"><VInput v-model="newEntityForm.name" placeholder="name" /></VFormItem>
      <template #footer>
        <VButton variant="secondary" @click="showNewEntity = false">{{ t.newProject.cancel }}</VButton>
        <VButton @click="createEntity">{{ t.newProject.create }}</VButton>
      </template>
    </VDialog>

    <!-- new dict dialog -->
    <VDialog :visible="showNewDict" :title="ctx.newDictionary" @update:visible="showNewDict = $event">
      <VFormItem :label="ds.enumCode"><VInput v-model="newDictForm.code" placeholder="code" /></VFormItem>
      <VFormItem :label="ds.enumName"><VInput v-model="newDictForm.name" placeholder="name" /></VFormItem>
      <template #footer>
        <VButton variant="secondary" @click="showNewDict = false">{{ t.newProject.cancel }}</VButton>
        <VButton @click="createDict">{{ t.newProject.create }}</VButton>
      </template>
    </VDialog>
  </div>
</template>
