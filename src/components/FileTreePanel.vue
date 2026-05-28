<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import { useI18n } from '../i18n'
import { useProject } from '../stores/project'
import { useMenuAction } from '../composables/useMenuAction'
import { projectCategories } from '../config/projectDirs'
import type { ProjectDirChild } from '../config/projectDirs'
import type { DomainInfo, EntityDef, EnumDef } from '../types/dataStandard'

const { t } = useI18n()
const { currentProject } = useProject()
const { emit: emitMenuAction } = useMenuAction()

const ds = t.value.dataStandard as Record<string, string>
const ctx = t.value.contextMenu as Record<string, string>

const DEFAULT_DOMAIN_CODE = 'default'

const domains = ref<DomainInfo[]>([])
const domainEntities = ref<Record<string, EntityDef[]>>({})
const domainEnums = ref<Record<string, EnumDef[]>>({})
const expandedCategories = ref<Set<string>>(new Set())
const expandedDirs = ref<Set<string>>(new Set())
const expandedDomains = ref<Set<string>>(new Set())
const selectedNode = ref('')

const contextMenuVisible = ref(false)
const contextMenuPos = ref({ x: 0, y: 0 })
const contextMenuType = ref<'dataStandard' | 'domain'>('dataStandard')
const contextMenuDomain = ref('')

const showNewDomainDialog = ref(false)
const showNewEntityDialog = ref(false)
const showNewDictDialog = ref(false)
const activeDomainCode = ref('')
const newDomainForm = ref({ name: '', code: '', owner: '', description: '' })
const newEntityForm = ref({ code: '', name: '' })
const newDictForm = ref({ code: '', name: '' })

function getDirLabel(labelKey: string): string {
  return (t.value.projectDirs as Record<string, string>)[labelKey] || labelKey
}

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
    for (const d of domains.value) {
      await loadDomainData(d.code)
    }
  } catch {
    // backend may not be ready
  }
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
  } catch {
    // ignore load errors
  }
}

watch(() => currentProject.value, () => {
  domains.value = []
  domainEntities.value = {}
  domainEnums.value = {}
  expandedCategories.value = new Set()
  expandedDirs.value = new Set()
  expandedDomains.value = new Set()
  if (currentProject.value) loadDomains()
}, { immediate: true })

function toggleCategory(key: string) {
  const next = new Set(expandedCategories.value)
  if (next.has(key)) next.delete(key)
  else next.add(key)
  expandedCategories.value = next
}

function isCategoryExpanded(key: string): boolean {
  return expandedCategories.value.has(key)
}

function toggleDir(dirKey: string) {
  const next = new Set(expandedDirs.value)
  if (next.has(dirKey)) next.delete(dirKey)
  else next.add(dirKey)
  expandedDirs.value = next
}

function toggleDomain(code: string) {
  const next = new Set(expandedDomains.value)
  if (next.has(code)) next.delete(code)
  else next.add(code)
  expandedDomains.value = next
}

function isExpanded(dirKey: string): boolean {
  return expandedDirs.value.has(dirKey)
}

function isDomainExpanded(code: string): boolean {
  return expandedDomains.value.has(code)
}

function handleDirClick(child: ProjectDirChild) {
  if (child.key === 'dataStandard') {
    toggleDir('dataStandard')
  }
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

// ── context menu ──

function handleContextMenu(event: MouseEvent, child: ProjectDirChild) {
  event.preventDefault()
  if (child.key === 'dataStandard') {
    contextMenuType.value = 'dataStandard'
    contextMenuDomain.value = ''
    contextMenuPos.value = { x: event.clientX, y: event.clientY }
    contextMenuVisible.value = true
    nextTick(() => {
      document.addEventListener('click', closeContextMenu, { once: true })
    })
  }
}

function handleDomainContextMenu(event: MouseEvent, domainCode: string) {
  event.preventDefault()
  event.stopPropagation()
  contextMenuType.value = 'domain'
  contextMenuDomain.value = domainCode
  contextMenuPos.value = { x: event.clientX, y: event.clientY }
  contextMenuVisible.value = true
  nextTick(() => {
    document.addEventListener('click', closeContextMenu, { once: true })
  })
}

function closeContextMenu() {
  contextMenuVisible.value = false
}

// ── context menu actions ──

function handleNewDomain() {
  closeContextMenu()
  showNewDomainDialog.value = true
}

function handleNewEntity() {
  closeContextMenu()
  if (contextMenuDomain.value) {
    activeDomainCode.value = contextMenuDomain.value
    showNewEntityDialog.value = true
  } else {
    ensureDefaultDomain(() => {
      activeDomainCode.value = DEFAULT_DOMAIN_CODE
      showNewEntityDialog.value = true
    })
  }
}

function handleNewDict() {
  closeContextMenu()
  if (contextMenuDomain.value) {
    activeDomainCode.value = contextMenuDomain.value
    showNewDictDialog.value = true
  } else {
    ensureDefaultDomain(() => {
      activeDomainCode.value = DEFAULT_DOMAIN_CODE
      showNewDictDialog.value = true
    })
  }
}

async function ensureDefaultDomain(onDone: () => void) {
  if (domains.value.length > 0) {
    activeDomainCode.value = domains.value[0].code
    onDone()
    return
  }
  const dir = projectDir()
  if (!dir) return
  try {
    await invoke('ds_create_domain', {
      projectDir: dir,
      code: DEFAULT_DOMAIN_CODE,
      name: ds.defaultDomain,
      owner: '',
      description: ''
    })
    await loadDomains()
    onDone()
  } catch (e) {
    ElMessage.error(String(e))
  }
}

function handleImportFile() {
  closeContextMenu()
  ElMessage.info(ctx.importFileHint)
}

function handleReverseDb() {
  closeContextMenu()
  ElMessage.info(ctx.reverseDbHint)
}

function handleReverseDdl() {
  closeContextMenu()
  ElMessage.info(ctx.reverseDdlHint)
}

// ── dialogs ──

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
    expandedDirs.value = new Set([...expandedDirs.value, 'dataStandard'])
  } catch (e) {
    ElMessage.error(String(e))
  }
}

async function createEntity() {
  const dir = projectDir()
  if (!dir || !activeDomainCode.value) return
  try {
    await invoke('ds_save_entity', {
      projectDir: dir,
      domainCode: activeDomainCode.value,
      entity: {
        _meta: { version: 0, locked_by: null, locked_at: null, updated_by: 'local', updated_at: new Date().toISOString(), checksum: '' },
        code: newEntityForm.value.code,
        name: newEntityForm.value.name,
        description: '',
        sensitivity: 'internal',
        fields: [],
        indexes: [],
        relations: []
      }
    })
    showNewEntityDialog.value = false
    newEntityForm.value = { code: '', name: '' }
    await loadDomainData(activeDomainCode.value)
    emitMenuAction({ action: 'open.dataStandard', payload: { domainCode: activeDomainCode.value } })
  } catch (e) {
    ElMessage.error(String(e))
  }
}

async function createDict() {
  const dir = projectDir()
  if (!dir || !activeDomainCode.value) return
  try {
    await invoke('ds_save_enum', {
      projectDir: dir,
      domainCode: activeDomainCode.value,
      enumDef: {
        _meta: { version: 0, locked_by: null, locked_at: null, updated_by: 'local', updated_at: new Date().toISOString(), checksum: '' },
        code: newDictForm.value.code,
        name: newDictForm.value.name,
        values: []
      }
    })
    showNewDictDialog.value = false
    newDictForm.value = { code: '', name: '' }
    await loadDomainData(activeDomainCode.value)
    emitMenuAction({ action: 'open.dataStandard', payload: { domainCode: activeDomainCode.value } })
  } catch (e) {
    ElMessage.error(String(e))
  }
}

function entityCount(domainCode: string): number {
  return domainEntities.value[domainCode]?.length || 0
}

function enumCount(domainCode: string): number {
  return domainEnums.value[domainCode]?.length || 0
}
</script>

<template>
  <div class="file-tree-panel">
    <div class="panel-header">
      <span class="panel-title">{{ t.panel.project }}</span>
    </div>
    <div class="panel-body">
      <template v-if="currentProject">
        <div class="project-name">{{ currentProject.name }}</div>
        <div class="dir-list">
          <template v-for="cat in projectCategories" :key="cat.key">
            <!-- category header -->
            <div
              class="dir-item"
              :class="{ active: selectedNode === `cat.${cat.key}` }"
              @click="handleCategoryClick(cat.key)"
            >
              <svg
                class="expand-chevron"
                :class="{ expanded: isCategoryExpanded(cat.key) }"
                viewBox="0 0 24 24"
              >
                <path fill="currentColor" d="M8 5l8 7-8 7z"/>
              </svg>
              <svg class="dir-icon" viewBox="0 0 24 24" :style="{ color: cat.color }">
                <path fill="currentColor" d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
              </svg>
              <span class="dir-label">{{ getDirLabel(cat.labelKey) }}</span>
            </div>

            <!-- category children -->
            <div v-if="isCategoryExpanded(cat.key)" class="dir-children">
              <template v-for="child in cat.children" :key="child.key">
                <!-- dataStandard: expandable domain tree -->
                <template v-if="child.key === 'dataStandard'">
                  <div
                    class="dir-item"
                    :class="{ active: selectedNode === 'dir.dataStandard' }"
                    @click="handleDirClick(child)"
                    @contextmenu="handleContextMenu($event, child)"
                  >
                    <svg
                      class="expand-chevron"
                      :class="{ expanded: isExpanded('dataStandard') }"
                      viewBox="0 0 24 24"
                      @click.stop="toggleDir('dataStandard')"
                    >
                      <path fill="currentColor" d="M8 5l8 7-8 7z"/>
                    </svg>
                    <svg class="dir-icon" viewBox="0 0 24 24" :style="{ color: child.color }">
                      <path fill="currentColor" d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
                    </svg>
                    <span class="dir-label">{{ getDirLabel(child.labelKey) }}</span>
                  </div>

                  <!-- domain children -->
                  <div v-if="isExpanded('dataStandard')" class="dir-children">
                    <div
                      v-for="domain in domains"
                      :key="domain.code"
                      class="domain-tree"
                    >
                      <div
                        class="dir-item domain-item"
                        :class="{ active: selectedNode === `domain.${domain.code}` }"
                        @click="handleDomainClick(domain.code)"
                        @contextmenu="handleDomainContextMenu($event, domain.code)"
                      >
                        <svg
                          class="expand-chevron"
                          :class="{ expanded: isDomainExpanded(domain.code) }"
                          viewBox="0 0 24 24"
                          @click.stop="toggleDomain(domain.code)"
                        >
                          <path fill="currentColor" d="M8 5l8 7-8 7z"/>
                        </svg>
                        <svg class="dir-icon" viewBox="0 0 24 24" style="color: #a0cfff">
                          <path fill="currentColor" d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
                        </svg>
                        <span class="dir-label">{{ domain.name || domain.code }}</span>
                      </div>

                      <!-- entity & dict groups under domain -->
                      <div v-if="isDomainExpanded(domain.code)" class="domain-children">
                        <div
                          class="dir-item group-item"
                          :class="{ active: selectedNode === `entity.${domain.code}` }"
                          @click="handleEntityGroupClick(domain.code)"
                        >
                          <svg class="dir-icon" viewBox="0 0 24 24" style="color: #67c23a">
                            <path fill="currentColor" d="M3 13h8V3H3v10zm0 8h8v-6H3v6zm10 0h8V11h-8v10zm0-18v6h8V3h-8z"/>
                          </svg>
                          <span class="dir-label">{{ ds.entityGroup }}</span>
                          <span class="group-count">{{ entityCount(domain.code) }}</span>
                        </div>
                        <div
                          class="dir-item group-item"
                          :class="{ active: selectedNode === `enum.${domain.code}` }"
                          @click="handleDictGroupClick(domain.code)"
                        >
                          <svg class="dir-icon" viewBox="0 0 24 24" style="color: #e6a23c">
                            <path fill="currentColor" d="M4 6H2v14c0 1.1.9 2 2 2h14v-2H4V6zm16-4H8c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm0 14H8V4h12v12z"/>
                          </svg>
                          <span class="dir-label">{{ ds.dictGroup }}</span>
                          <span class="group-count">{{ enumCount(domain.code) }}</span>
                        </div>
                      </div>
                    </div>

                    <div v-if="domains.length === 0" class="tree-hint">
                      {{ ds.defaultDomain }}
                    </div>
                  </div>
                </template>

                <!-- regular child -->
                <div
                  v-else
                  class="dir-item"
                  :class="{ active: selectedNode === `dir.${child.key}` }"
                  @click="handleDirClick(child)"
                  @contextmenu="handleContextMenu($event, child)"
                >
                  <svg class="dir-icon-placeholder" viewBox="0 0 24 24" />
                  <svg class="dir-icon" viewBox="0 0 24 24" :style="{ color: child.color }">
                    <path fill="currentColor" d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/>
                  </svg>
                  <span class="dir-label">{{ getDirLabel(child.labelKey) }}</span>
                </div>
              </template>
            </div>
          </template>
        </div>
      </template>
      <div v-else class="tree-placeholder">
        <p class="placeholder-text">{{ t.panel.openProjectHint }}</p>
      </div>
    </div>

    <!-- context menu -->
    <Teleport to="body">
      <div
        v-if="contextMenuVisible"
        class="context-menu"
        :style="{ left: contextMenuPos.x + 'px', top: contextMenuPos.y + 'px' }"
      >
        <template v-if="contextMenuType === 'dataStandard'">
          <div class="context-menu-item" @click="handleNewDomain">{{ ctx.newDomain }}</div>
          <div class="context-menu-item" @click="handleNewEntity">{{ ctx.newEntity }}</div>
          <div class="context-menu-item" @click="handleNewDict">{{ ctx.newDictionary }}</div>
          <div class="context-menu-separator" />
          <div class="context-menu-item" @click="handleImportFile">{{ ctx.importFile }}</div>
          <div class="context-menu-item" @click="handleReverseDb">{{ ctx.reverseDb }}</div>
          <div class="context-menu-item" @click="handleReverseDdl">{{ ctx.reverseDdl }}</div>
        </template>
        <template v-else>
          <div class="context-menu-item" @click="handleNewEntity">{{ ctx.newEntity }}</div>
          <div class="context-menu-item" @click="handleNewDict">{{ ctx.newDictionary }}</div>
        </template>
      </div>
    </Teleport>

    <!-- new domain dialog -->
    <Teleport to="body">
      <div v-if="showNewDomainDialog" class="dialog-overlay" @click.self="showNewDomainDialog = false">
        <div class="dialog-box">
          <div class="dialog-title">{{ ds.newDomain }}</div>
          <div class="dialog-body">
            <div class="dialog-field">
              <label>{{ ds.domainCode }}</label>
              <input v-model="newDomainForm.code" class="dialog-input" placeholder="code" />
            </div>
            <div class="dialog-field">
              <label>{{ ds.domainName }}</label>
              <input v-model="newDomainForm.name" class="dialog-input" placeholder="name" />
            </div>
            <div class="dialog-field">
              <label>{{ ds.domainOwner }}</label>
              <input v-model="newDomainForm.owner" class="dialog-input" placeholder="owner" />
            </div>
            <div class="dialog-field">
              <label>{{ ds.domainDesc }}</label>
              <input v-model="newDomainForm.description" class="dialog-input" placeholder="description" />
            </div>
          </div>
          <div class="dialog-footer">
            <button class="dialog-btn dialog-btn-cancel" @click="showNewDomainDialog = false">{{ t.newProject.cancel }}</button>
            <button class="dialog-btn dialog-btn-ok" @click="createDomain">{{ t.newProject.create }}</button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- new entity dialog -->
    <Teleport to="body">
      <div v-if="showNewEntityDialog" class="dialog-overlay" @click.self="showNewEntityDialog = false">
        <div class="dialog-box">
          <div class="dialog-title">{{ ctx.newEntity }}</div>
          <div class="dialog-body">
            <div class="dialog-field">
              <label>{{ ds.entityCode }}</label>
              <input v-model="newEntityForm.code" class="dialog-input" placeholder="code" />
            </div>
            <div class="dialog-field">
              <label>{{ ds.entityName }}</label>
              <input v-model="newEntityForm.name" class="dialog-input" placeholder="name" />
            </div>
          </div>
          <div class="dialog-footer">
            <button class="dialog-btn dialog-btn-cancel" @click="showNewEntityDialog = false">{{ t.newProject.cancel }}</button>
            <button class="dialog-btn dialog-btn-ok" @click="createEntity">{{ t.newProject.create }}</button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- new dict dialog -->
    <Teleport to="body">
      <div v-if="showNewDictDialog" class="dialog-overlay" @click.self="showNewDictDialog = false">
        <div class="dialog-box">
          <div class="dialog-title">{{ ctx.newDictionary }}</div>
          <div class="dialog-body">
            <div class="dialog-field">
              <label>{{ ds.enumCode }}</label>
              <input v-model="newDictForm.code" class="dialog-input" placeholder="code" />
            </div>
            <div class="dialog-field">
              <label>{{ ds.enumName }}</label>
              <input v-model="newDictForm.name" class="dialog-input" placeholder="name" />
            </div>
          </div>
          <div class="dialog-footer">
            <button class="dialog-btn dialog-btn-cancel" @click="showNewDictDialog = false">{{ t.newProject.cancel }}</button>
            <button class="dialog-btn dialog-btn-ok" @click="createDict">{{ t.newProject.create }}</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.file-tree-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-panel);
}

.panel-header {
  display: flex;
  align-items: center;
  height: 32px;
  padding: 0 12px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.panel-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
}

.panel-body {
  flex: 1;
  overflow-y: auto;
  padding: 4px 0;
}

.project-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  padding: 8px 12px 4px;
}

.dir-list {
  display: flex;
  flex-direction: column;
}

.dir-item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 5px 12px 5px 8px;
  cursor: pointer;
  transition: background 0.12s;
  user-select: none;
}

.dir-item:hover {
  background: var(--bg-hover);
}

.dir-item.active {
  background: var(--bg-active, #dbeafe);
}

.expand-chevron {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  color: var(--text-muted);
  transition: transform 0.15s;
  border-radius: 3px;
}

.expand-chevron:hover {
  background: var(--bg-active);
}

.expand-chevron.expanded {
  transform: rotate(90deg);
}

.dir-icon-placeholder {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  visibility: hidden;
}

.dir-icon {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.dir-label {
  font-size: 13px;
  color: var(--text-primary);
}

/* children indentation — each nesting level adds visual depth */
.dir-children {
  display: flex;
  flex-direction: column;
  padding-left: 16px;
}

.domain-children {
  display: flex;
  flex-direction: column;
  padding-left: 12px;
}

.group-count {
  font-size: 11px;
  color: var(--text-muted);
  margin-left: auto;
}

.tree-hint {
  padding: 4px 12px 4px 48px;
  font-size: 12px;
  color: var(--text-muted);
  font-style: italic;
}

.tree-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.placeholder-text {
  font-size: 13px;
  color: var(--text-muted);
}

/* context menu */
.context-menu {
  position: fixed;
  z-index: 9999;
  min-width: 140px;
  background: var(--bg-primary, #fff);
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
  padding: 4px 0;
}

.context-menu-item {
  padding: 6px 16px;
  font-size: 13px;
  color: var(--text-primary, #333);
  cursor: pointer;
  transition: background 0.1s;
}

.context-menu-item:hover {
  background: var(--bg-hover, #f0f0f0);
}

.context-menu-separator {
  height: 1px;
  margin: 4px 8px;
  background: var(--border-color, #e0e0e0);
}

/* dialog */
.dialog-overlay {
  position: fixed;
  inset: 0;
  z-index: 10000;
  background: rgba(0, 0, 0, 0.35);
  display: flex;
  align-items: center;
  justify-content: center;
}

.dialog-box {
  background: var(--bg-primary, #fff);
  border-radius: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.18);
  width: 400px;
  max-width: 90vw;
}

.dialog-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  padding: 16px 20px 8px;
}

.dialog-body {
  padding: 8px 20px 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.dialog-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.dialog-field label {
  font-size: 12px;
  color: var(--text-secondary);
}

.dialog-input {
  height: 32px;
  padding: 0 10px;
  border: 1px solid var(--border-color, #d0d0d0);
  border-radius: 4px;
  font-size: 13px;
  color: var(--text-primary);
  background: var(--bg-primary, #fff);
  outline: none;
}

.dialog-input:focus {
  border-color: #409eff;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 0 20px 16px;
}

.dialog-btn {
  padding: 6px 18px;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  border: 1px solid var(--border-color, #d0d0d0);
  background: var(--bg-primary, #fff);
  color: var(--text-primary);
}

.dialog-btn-ok {
  background: #409eff;
  color: #fff;
  border-color: #409eff;
}

.dialog-btn-ok:hover {
  background: #3a8ee6;
}

.dialog-btn-cancel:hover {
  background: var(--bg-hover, #f0f0f0);
}
</style>
