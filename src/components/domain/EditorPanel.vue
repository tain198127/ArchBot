<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { useI18n } from '../../i18n'
import { useMenuAction, getAction, getPayload } from '../../composables/useMenuAction'
import SettingsPanel from './SettingsPanel.vue'
import DataStandardEditor from './DataStandardEditor.vue'
import DigitalEmployeePanel from './DigitalEmployeePanel.vue'
import ScenarioPanel from './ScenarioPanel.vue'
import AgentConfigPanel from './AgentConfigPanel.vue'
import AIConfigPanel from './AIConfigPanel.vue'
import ContextEngineeringPanel from './ContextEngineeringPanel.vue'
import SkillConfigPanel from './SkillConfigPanel.vue'
import BusinessFlowListPanel from './BusinessFlowListPanel.vue'
import { defineAsyncComponent } from 'vue'

const BusinessFlowEditorPanel = defineAsyncComponent(
  () => import('./business-flow/BusinessFlowEditorPanel.vue')
)
const BusinessFlowRunPanel = defineAsyncComponent(
  () => import('./business-flow/BusinessFlowRunPanel.vue')
)

const { t, tt } = useI18n()

const { on } = useMenuAction()

interface EditorTab {
  id: string
  labelKey: string
  type: string
  domainCode?: string
  focus?: string
  closable: boolean
  pinned: boolean
}

const tabs = ref<EditorTab[]>([
  { id: 'welcome', labelKey: 'editor.appName', type: 'welcome', closable: false, pinned: true }
])

const activeTab = ref('welcome')
const tm = computed(() => t.value.tabMenu as Record<string, string>)

const tabCtxVisible = ref(false)
const tabCtxPos = ref({ x: 0, y: 0 })
const tabCtxId = ref('')

const sortedTabs = computed(() => {
  const arr = [...tabs.value]
  arr.sort((a, b) => {
    if (a.pinned && !b.pinned) return -1
    if (!a.pinned && b.pinned) return 1
    return 0
  })
  return arr
})

function getTabLabel(tab: EditorTab): string {
  if (tab.labelKey.includes(' / ')) {
    return tab.labelKey.split(' / ').map(k => tt(k.trim()) || k.trim()).join(' / ')
  }
  return tt(tab.labelKey) || tab.labelKey
}

function activeTabData() {
  return tabs.value.find(t => t.id === activeTab.value)
}

function openTab(id: string, labelKey: string, type: string, domainCode?: string, focus?: string) {
  const existing = tabs.value.find(t => t.id === id)
  if (existing) {
    activeTab.value = id
    return
  }
  tabs.value.push({ id, labelKey, type, domainCode, focus, closable: true, pinned: false })
  activeTab.value = id
}

function closeTab(id: string) {
  const idx = tabs.value.findIndex(t => t.id === id)
  if (idx < 0) return
  tabs.value.splice(idx, 1)
  if (activeTab.value === id) {
    activeTab.value = tabs.value.length > 0 ? tabs.value[Math.min(idx, tabs.value.length - 1)].id : 'welcome'
    if (tabs.value.length === 0) {
      tabs.value.push({ id: 'welcome', labelKey: 'editor.appName', type: 'welcome', closable: false, pinned: true })
      activeTab.value = 'welcome'
    }
  }
}

function closeAll() { closeTabCtx(); tabs.value = tabs.value.filter(t => !t.closable); activeTab.value = 'welcome' }
function closeRight() { closeTabCtx(); const idx = tabs.value.findIndex(t => t.id === tabCtxId.value); if (idx >= 0) tabs.value = tabs.value.filter((t, i) => i <= idx || !t.closable) }
function closeLeft() { closeTabCtx(); const idx = tabs.value.findIndex(t => t.id === tabCtxId.value); if (idx >= 0) tabs.value = tabs.value.filter((t, i) => i >= idx || !t.closable) }
function closeOthers() { closeTabCtx(); tabs.value = tabs.value.filter(t => t.id === tabCtxId.value || !t.closable) }

function handleTabContextMenu(event: MouseEvent, id: string) {
  event.preventDefault()
  tabCtxId.value = id
  tabCtxPos.value = { x: event.clientX, y: event.clientY }
  tabCtxVisible.value = true
  nextTick(() => document.addEventListener('click', closeTabCtx, { once: true }))
}

function closeTabCtx() { tabCtxVisible.value = false }

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

    if (action === 'config.ai') {
      openTab('ai-config', 'aiConfig.title', 'ai-config')
    } else if (action === 'config.system') {
      openTab('settings', 'settings.title', 'settings')
    } else if (action === 'config.contextEngineering') {
      openTab('context-engineering', 'context.title', 'context-engineering')
    } else if (action === 'config.scenario') {
      openTab('scenario', 'scenario.title', 'scenario')
    } else if (action === 'config.digitalEmployee') {
      openTab('digital-employee', 'digitalEmployee.title', 'digital-employee')
    } else if (action === 'config.agentConfig') {
      openTab('agent-config', 'menuConfig.agentConfig', 'agent-config')
    } else if (action === 'config.skill') {
      openTab('skill-config', 'menuConfig.skill', 'skill-config')
    } else if (action === 'open.dataStandard') {
      const dc = payload?.domainCode || ''
      const dn = payload?.domainName || ''
      const focus = (payload?.focus as string) || ''

      let id: string; let labelKey: string

      if (focus === 'entity') { id = `ds-entity-${dc}`; labelKey = `dataStandard.title / ${dn || dc} / dataStandard.entityGroup` }
      else if (focus === 'enum') { id = `ds-enum-${dc}`; labelKey = `dataStandard.title / ${dn || dc} / dataStandard.dictGroup` }
      else if (dc) { id = `ds-domain-${dc}`; labelKey = `dataStandard.title / ${dn || dc}` }
      else { id = 'dataStandard'; labelKey = 'dataStandard.title' }
      openTab(id, labelKey, 'dataStandard', dc, focus)
    } else if (action === 'config.businessFlow') {
      openTab('business-flow-list', 'businessFlow.list.title', 'business-flow-list')
    } else if (action === 'open.businessFlowEditor') {
      const flowId = payload?.flowId || ''
      const flowName = payload?.flowName || 'Flow'
      openTab(`bf-editor-${flowId}`, `businessFlow.editor.title / ${flowName}`, 'business-flow-editor', flowId)
    } else if (action === 'bf.runFlow') {
      // Context menu → Run flow with clicked file as material
      const flowId = payload?.flowId || ''
      const flowName = payload?.flowName || 'Flow'
      const filePath = payload?.filePath || ''
      openTab(`bf-editor-${flowId}`, `businessFlow.editor.title / ${flowName}`, 'business-flow-editor', flowId, filePath)
    }
  })
})

onUnmounted(() => { unsubscribe?.() })
</script>

<template>
  <div class="flex flex-col h-full bg-surface-0 dark:bg-surface-0">
    <div class="flex items-center h-[34px] bg-surface-100 dark:bg-surface-100 border-b border-border-default overflow-x-auto shrink-0">
      <div
        v-for="tab in sortedTabs" :key="tab.id"
        class="flex items-center gap-1.5 px-3.5 h-full text-sm cursor-pointer border-r border-border-default whitespace-nowrap select-none"
        :class="activeTab === tab.id
          ? 'bg-surface-0 dark:bg-surface-0 text-text-primary border-b-2 border-primary-500'
          : 'text-text-secondary hover:bg-surface-100 dark:hover:bg-surface-100'"
        :style="tab.pinned ? 'border-left: 2px solid var(--color-warning-500)' : ''"
        @click="activeTab = tab.id"
        @contextmenu="handleTabContextMenu($event, tab.id)"
      >
        <span v-if="tab.pinned && tab.id !== 'welcome'" class="text-xs" title="pinned">&#128204;</span>
        <span>{{ getTabLabel(tab) }}</span>
        <span
          v-if="tab.closable && !tab.pinned"
          class="text-sm text-text-muted rounded px-0.5 hover:bg-surface-200 dark:hover:bg-surface-200 hover:text-text-primary"
          @click.stop="closeTab(tab.id)"
        >&times;</span>
      </div>
    </div>
    <div class="flex-1 overflow-hidden">
      <div v-if="activeTab === 'welcome'" class="flex flex-col items-center justify-center h-full select-none">
        <div class="text-center animate-fade-in">
          <div class="mb-6 inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-primary-500/10 ring-1 ring-primary-500/20">
            <span class="text-2xl font-bold text-primary-500 tracking-tight">AB</span>
          </div>
          <h2 class="text-2xl font-semibold text-text-primary tracking-tight mb-2">{{ t.editor.appName }}</h2>
          <p class="text-sm text-text-secondary max-w-md leading-relaxed">{{ t.editor.appDesc }}</p>
          <div class="mt-6 flex items-center gap-2 justify-center">
            <span class="inline-flex items-center gap-1.5 px-3 py-1.5 bg-surface-50 dark:bg-surface-100 border border-border-default rounded-lg text-xs text-text-secondary">
              <kbd class="font-mono text-[11px] text-text-muted">⌘O</kbd> {{ t.menuFile.openProject }}
            </span>
            <span class="text-xs text-text-muted">or</span>
            <span class="inline-flex items-center gap-1.5 px-3 py-1.5 bg-surface-50 dark:bg-surface-100 border border-border-default rounded-lg text-xs text-text-secondary">
              <kbd class="font-mono text-[11px] text-text-muted">⌘⇧N</kbd> {{ t.menuFile.newProject }}
            </span>
          </div>
        </div>
      </div>
      <SettingsPanel v-else-if="activeTabData()?.type === 'settings'" />
      <ScenarioPanel v-else-if="activeTabData()?.type === 'scenario'" />
      <ContextEngineeringPanel v-else-if="activeTabData()?.type === 'context-engineering'" />
      <AIConfigPanel v-else-if="activeTabData()?.type === 'ai-config'" />
      <AgentConfigPanel v-else-if="activeTabData()?.type === 'agent-config'" />
      <SkillConfigPanel v-else-if="activeTabData()?.type === 'skill-config'" />
      <DigitalEmployeePanel v-else-if="activeTabData()?.type === 'digital-employee'" />
      <BusinessFlowListPanel v-else-if="activeTabData()?.type === 'business-flow-list'" />
      <BusinessFlowEditorPanel
        v-else-if="activeTabData()?.type === 'business-flow-editor'"
        :flowId="(activeTabData()?.domainCode as string) || ''"
        :materialFile="(activeTabData()?.focus as string) || ''"
        :key="activeTab"
      />
      <DataStandardEditor
        v-else-if="activeTabData()?.type === 'dataStandard'"
        :key="activeTab"
        :initialDomainCode="activeTabData()?.domainCode || ''"
        :initialFocus="activeTabData()?.focus || ''"
      />
    </div>

    <Teleport to="body">
      <div
        v-if="tabCtxVisible"
        class="fixed z-[9999] min-w-[150px] bg-white dark:bg-surface-0 border border-border-default rounded-lg shadow-lg py-1"
        :style="{ left: tabCtxPos.x + 'px', top: tabCtxPos.y + 'px' }"
      >
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-pointer" @click="closeAll">{{ tm.closeAll }}</button>
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-pointer" @click="closeRight">{{ tm.closeRight }}</button>
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-pointer" @click="closeLeft">{{ tm.closeLeft }}</button>
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-pointer" @click="closeOthers">{{ tm.closeOthers }}</button>
        <div class="h-px mx-2 my-1 bg-border-default" />
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-pointer" @click="togglePin">
          {{ isPinned(tabCtxId) ? tm.unpin : tm.pin }}
        </button>
      </div>
    </Teleport>
  </div>
</template>
