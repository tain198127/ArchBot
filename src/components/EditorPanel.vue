<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { useI18n } from '../i18n'
import { useMenuAction, getAction, getPayload } from '../composables/useMenuAction'
import SettingsPanel from './SettingsPanel.vue'
import DataStandardEditor from './DataStandardEditor.vue'
import DigitalEmployeePanel from './DigitalEmployeePanel.vue'

const { t } = useI18n()

const { on } = useMenuAction()

interface EditorTab {
  id: string
  label: string
  type: 'welcome' | 'settings' | 'dataStandard' | 'digital-employee'
  closable: boolean
  pinned: boolean
  domainCode: string
  focus: string
}

const tabs = ref<EditorTab[]>([
  { id: 'welcome', label: t.value.editor.welcome, type: 'welcome', closable: false, pinned: true, domainCode: '', focus: '' }
])
const activeTab = ref('welcome')

const tabCtxVisible = ref(false)
const tabCtxPos = ref({ x: 0, y: 0 })
const tabCtxId = ref('')

const tm = t.value.tabMenu as Record<string, string>

// pinned tabs first, then unpinned; welcome always pinned at 0
const sortedTabs = computed(() => {
  const pinned = tabs.value.filter(t => t.pinned)
  const unpinned = tabs.value.filter(t => !t.pinned)
  return [...pinned, ...unpinned]
})

function openTab(id: string, label: string, type: EditorTab['type'], domainCode = '', focus = '') {
  const existing = tabs.value.find(tab => tab.id === id)
  if (existing) {
    activeTab.value = id
    return
  }
  tabs.value = [...tabs.value, { id, label, type, closable: true, pinned: false, domainCode, focus }]
  activeTab.value = id
}

function closeTab(id: string) {
  const index = tabs.value.findIndex(tab => tab.id === id)
  if (index === -1) return
  tabs.value = tabs.value.filter(tab => tab.id !== id)
  if (activeTab.value === id) {
    activeTab.value = tabs.value[Math.min(index, tabs.value.length - 1)]?.id || ''
  }
}

function activeTabData(): EditorTab | undefined {
  return tabs.value.find(tab => tab.id === activeTab.value)
}

// ── tab context menu ──

function handleTabContextMenu(event: MouseEvent, tabId: string) {
  event.preventDefault()
  const tab = tabs.value.find(t => t.id === tabId)
  if (!tab || !tab.closable) return
  tabCtxId.value = tabId
  tabCtxPos.value = { x: event.clientX, y: event.clientY }
  tabCtxVisible.value = true
  nextTick(() => {
    document.addEventListener('click', closeTabCtx, { once: true })
  })
}

function closeTabCtx() {
  tabCtxVisible.value = false
}

function tabIndex(id: string): number {
  return sortedTabs.value.findIndex(t => t.id === id)
}

function closeAll() {
  closeTabCtx()
  tabs.value = tabs.value.filter(t => !t.closable || t.pinned)
  if (!tabs.value.find(t => t.id === activeTab.value)) {
    activeTab.value = tabs.value[tabs.value.length - 1]?.id || 'welcome'
  }
}

function closeRight() {
  closeTabCtx()
  const idx = tabIndex(tabCtxId.value)
  if (idx < 0) return
  const rightIds = new Set(sortedTabs.value.slice(idx + 1).map(t => t.id))
  tabs.value = tabs.value.filter(t => t.pinned || !t.closable || !rightIds.has(t.id))
  if (!tabs.value.find(t => t.id === activeTab.value)) {
    activeTab.value = tabCtxId.value
  }
}

function closeLeft() {
  closeTabCtx()
  const idx = tabIndex(tabCtxId.value)
  if (idx < 0) return
  const leftIds = new Set(sortedTabs.value.slice(0, idx).map(t => t.id))
  tabs.value = tabs.value.filter(t => t.pinned || !t.closable || !leftIds.has(t.id))
}

function closeOthers() {
  closeTabCtx()
  tabs.value = tabs.value.filter(t => !t.closable || t.pinned || t.id === tabCtxId.value)
  activeTab.value = tabCtxId.value
}

function togglePin() {
  closeTabCtx()
  const tab = tabs.value.find(t => t.id === tabCtxId.value)
  if (!tab) return
  tab.pinned = !tab.pinned
}

function isPinned(id: string): boolean {
  return tabs.value.find(t => t.id === id)?.pinned || false
}

let unsubscribe: (() => void) | null = null

onMounted(() => {
  unsubscribe = on((rawAction) => {
    const action = getAction(rawAction)
    const payload = getPayload(rawAction)

    if (action === 'config.system') {
      openTab('settings', t.value.settings.title, 'settings')
    } else if (action === 'config.digitalEmployee') {
      openTab('digital-employee', (t.value.digitalEmployee as Record<string, string>).title || '数字员工', 'digital-employee')
    } else if (action === 'open.dataStandard') {
      const dc = payload?.domainCode || ''
      const dn = payload?.domainName || ''
      const focus = (payload?.focus as string) || ''
      const dsLabel = (t.value.dataStandard as Record<string, string>).title || 'Data Standard'
      const entityLabel = (t.value.dataStandard as Record<string, string>).entityGroup || 'Entity'
      const dictLabel = (t.value.dataStandard as Record<string, string>).dictGroup || 'Dict'

      let id: string
      let label: string

      if (focus === 'entity') {
        id = `ds-entity-${dc}`
        label = `${dsLabel} / ${dn || dc} / ${entityLabel}`
      } else if (focus === 'enum') {
        id = `ds-enum-${dc}`
        label = `${dsLabel} / ${dn || dc} / ${dictLabel}`
      } else if (dc) {
        id = `ds-domain-${dc}`
        label = `${dsLabel} / ${dn || dc}`
      } else {
        id = 'dataStandard'
        label = dsLabel
      }
      openTab(id, label, 'dataStandard', dc, focus)
    }
  })
})

onUnmounted(() => {
  unsubscribe?.()
})
</script>

<template>
  <div class="editor-panel">
    <div class="editor-tabs">
      <div
        v-for="tab in sortedTabs"
        :key="tab.id"
        class="editor-tab"
        :class="{ active: activeTab === tab.id, pinned: tab.pinned }"
        @click="activeTab = tab.id"
        @contextmenu="handleTabContextMenu($event, tab.id)"
      >
        <span v-if="tab.pinned && tab.id !== 'welcome'" class="tab-pin" title="pinned">&#128204;</span>
        <span class="tab-label">{{ tab.label }}</span>
        <span
          v-if="tab.closable && !tab.pinned"
          class="tab-close"
          @click.stop="closeTab(tab.id)"
        >×</span>
      </div>
    </div>
    <div class="editor-content">
      <div v-if="activeTab === 'welcome'" class="welcome-page">
        <h2>{{ t.editor.appName }}</h2>
        <p>{{ t.editor.appDesc }}</p>
        <p class="hint">{{ t.editor.startHint }}</p>
      </div>
      <SettingsPanel v-else-if="activeTabData()?.type === 'settings'" />
      <DigitalEmployeePanel v-else-if="activeTabData()?.type === 'digital-employee'" />
      <DataStandardEditor
        v-else-if="activeTabData()?.type === 'dataStandard'"
        :key="activeTab"
        :initialDomainCode="activeTabData()?.domainCode || ''"
        :initialFocus="activeTabData()?.focus || ''"
      />
    </div>

    <!-- tab context menu -->
    <Teleport to="body">
      <div
        v-if="tabCtxVisible"
        class="tab-ctx-menu"
        :style="{ left: tabCtxPos.x + 'px', top: tabCtxPos.y + 'px' }"
      >
        <div class="tab-ctx-item" @click="closeAll">{{ tm.closeAll }}</div>
        <div class="tab-ctx-item" @click="closeRight">{{ tm.closeRight }}</div>
        <div class="tab-ctx-item" @click="closeLeft">{{ tm.closeLeft }}</div>
        <div class="tab-ctx-item" @click="closeOthers">{{ tm.closeOthers }}</div>
        <div class="tab-ctx-separator" />
        <div class="tab-ctx-item" @click="togglePin">
          {{ isPinned(tabCtxId) ? tm.unpin : tm.pin }}
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.editor-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary, #fff);
}

.editor-tabs {
  display: flex;
  align-items: center;
  height: 34px;
  background: var(--bg-secondary, #f0f0f0);
  border-bottom: 1px solid var(--border-color, #e0e0e0);
  overflow-x: auto;
  flex-shrink: 0;
}

.editor-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 14px;
  height: 100%;
  font-size: 13px;
  color: var(--text-secondary, #666);
  cursor: pointer;
  border-right: 1px solid var(--border-color, #e0e0e0);
  white-space: nowrap;
  transition: background 0.15s;
  user-select: none;
}

.editor-tab:hover {
  background: var(--bg-hover, #e8e8e8);
}

.editor-tab.active {
  background: var(--bg-primary, #fff);
  color: var(--text-primary, #333);
  border-bottom: 2px solid #409eff;
}

.editor-tab.pinned {
  border-left: 2px solid #e6a23c;
}

.tab-pin {
  font-size: 11px;
  line-height: 1;
}

.tab-close {
  font-size: 14px;
  line-height: 1;
  color: var(--text-muted, #999);
  border-radius: 3px;
  padding: 0 2px;
}

.tab-close:hover {
  background: var(--bg-hover, #ddd);
  color: var(--text-primary, #333);
}

.editor-content {
  flex: 1;
  overflow: hidden;
}

.welcome-page {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-secondary, #666);
}

.welcome-page h2 {
  font-size: 28px;
  font-weight: 300;
  margin-bottom: 8px;
  color: var(--text-primary, #333);
}

.welcome-page p {
  font-size: 14px;
  margin: 4px 0;
}

.welcome-page .hint {
  margin-top: 16px;
  font-size: 13px;
  color: var(--text-muted, #999);
}

/* tab context menu */
.tab-ctx-menu {
  position: fixed;
  z-index: 9999;
  min-width: 150px;
  background: var(--bg-primary, #fff);
  border: 1px solid var(--border-color, #e0e0e0);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
  padding: 4px 0;
}

.tab-ctx-item {
  padding: 6px 16px;
  font-size: 13px;
  color: var(--text-primary, #333);
  cursor: pointer;
  transition: background 0.1s;
}

.tab-ctx-item:hover {
  background: var(--bg-hover, #f0f0f0);
}

.tab-ctx-separator {
  height: 1px;
  margin: 4px 8px;
  background: var(--border-color, #e0e0e0);
}
</style>
