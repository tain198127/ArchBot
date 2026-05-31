<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { openProject as apiOpenProject, initArchbotDir, ensureGitignore } from './api'
import { pushLog } from './stores/log'
import { useToast } from './composables/useToast'
import { registerAllActions } from './actions'
import { getActionRegistry, type ActionRuntime } from './orchestration/ActionRegistry'
import { getPredicateRegistry } from './orchestration/PredicateRegistry'
import ToastProvider from './components/base/ToastProvider.vue'
import MenuBar from './components/layout/MenuBar.vue'
import SplitPanel from './components/layout/SplitPanel.vue'
import FileTreePanel from './components/domain/FileTreePanel.vue'
import EditorPanel from './components/domain/EditorPanel.vue'
import ModelPanel from './components/domain/ModelPanel.vue'
import BottomPanel from './components/domain/BottomPanel.vue'
import NewProjectDialog from './components/domain/NewProjectDialog.vue'
import LicenseDialog from './components/domain/LicenseDialog.vue'
import { useI18n } from './i18n'
import { useSettings } from './stores/settings'
import { useProject } from './stores/project'
import { usePanelLayout } from './composables/usePanelLayout'

const { t } = useI18n()
const toast = useToast()
const { initSettings } = useSettings()
const { setProject } = useProject()

const newProjectDialogRef = ref<InstanceType<typeof NewProjectDialog> | null>(null)
const licenseDialogRef = ref<InstanceType<typeof LicenseDialog> | null>(null)
const vertSplitRef = ref<InstanceType<typeof SplitPanel> | null>(null)
const horizSplitRef = ref<InstanceType<typeof SplitPanel> | null>(null)

const { leftCollapsed, rightCollapsed, bottomCollapsed } = usePanelLayout()

watch(leftCollapsed, v => {
  if (v) horizSplitRef.value?.collapsePanel(0)
  else horizSplitRef.value?.expandPanel(0)
})
watch(rightCollapsed, v => {
  if (v) horizSplitRef.value?.collapsePanel(2)
  else horizSplitRef.value?.expandPanel(2)
})
watch(bottomCollapsed, v => {
  if (v) vertSplitRef.value?.collapsePanel(1)
  else vertSplitRef.value?.expandPanel(1)
})

// Two-way sync: when splitter's internal button collapses a panel, update toolbar state
let _syncDone = false
function startSync() {
  if (_syncDone) return; _syncDone = true
  const h = horizSplitRef.value; const v = vertSplitRef.value
  if (!h || !v) return
  watch(() => h.collapsed[0], val => { leftCollapsed.value = val })
  watch(() => h.collapsed[2], val => { rightCollapsed.value = val })
  watch(() => v.collapsed[1], val => { bottomCollapsed.value = val })
}

// Create ActionRuntime — bridges actions to app-level capabilities
const runtime: ActionRuntime = {
  invoke: async (cmd: string, args?: Record<string, unknown>) => {
    // Tauri invoke or HTTP fallback
    const { isTauri } = await import('./api/env')
    if (isTauri) {
      const { invoke } = await import('@tauri-apps/api/core')
      return invoke(cmd, args)
    }
    const { call } = await import('./api/transport')
    return call(cmd, 'POST', `/${cmd.replace(/_/g, '-')}`, args)
  },
  openFile: () => {},
  toast: {
    success: toast.success,
    error: toast.error,
    warning: toast.warning,
  },
  pushLog: (level, source, msg) => pushLog(level, source, msg),
  confirm: async (msg: string) => {
    return window.confirm(msg)
  },
}

onMounted(() => {
  initSettings()

  // Register all actions from YML config (guard against double-mount in dev HMR)
  try { registerAllActions(runtime) } catch (e: any) {
    if (!String(e).includes('Duplicate')) throw e
  }

  // Two-way sync panel toolbar ↔ splitter buttons
  setTimeout(() => { try { startSync() } catch { /* refs not ready yet */ } }, 200)

  // Register Predicates for complex conditions
  const predicates = getPredicateRegistry()
  predicates.register('groupSpecificAction', (state, context) => {
    const groupKey = (context?.resource as Record<string, unknown>)?.groupKey as string | undefined
    return state.project.loaded && !!groupKey
  })

  // Override dialog-triggering actions that need component refs
  const registry = getActionRegistry()
  registry.register('project.create', async () => {
    newProjectDialogRef.value?.show()
  })
  registry.register('license.openDialog', async () => {
    licenseDialogRef.value?.show()
  })
})

function handleProjectCreated(filePath: string, name: string) {
  apiOpenProject(filePath).then(result => {
    setProject({ name: result.name, path: filePath, content: result.content })
  }).catch(() => {
    setProject({ name, path: filePath, content: '' })
  })
  initArchbotDir(filePath).catch(() => {})
  ensureGitignore(filePath).catch(() => {})
}

const bottomCollapseLabels = computed(() => ['', t.value.panel.bottomPanel])
const rightCollapseLabels = computed(() => ['', '', t.value.panel.model])
</script>

<template>
  <div class="flex flex-col w-full h-full overflow-hidden">
    <ToastProvider />
    <MenuBar />
    <SplitPanel
      ref="vertSplitRef"
      direction="vertical"
      :initial-sizes="[-1, 200]"
      :min-sizes="[300, 120]"
      :collapsible="[false, true]"
      :collapse-icons="['', '⌨']"
      :collapse-labels="bottomCollapseLabels"
    >
      <template #panel-0>
        <SplitPanel
          ref="horizSplitRef"
          direction="horizontal"
          :initial-sizes="[240, -1, 320]"
          :min-sizes="[100, 300, 200]"
          :collapsible="[true, false, true]"
          :collapse-icons="['', '', '🤖']"
          :collapse-labels="rightCollapseLabels"
        >
          <template #panel-0>
            <FileTreePanel />
          </template>
          <template #panel-1>
            <EditorPanel />
          </template>
          <template #panel-2>
            <ModelPanel />
          </template>
        </SplitPanel>
      </template>
      <template #panel-1>
        <BottomPanel />
      </template>
    </SplitPanel>
    <NewProjectDialog ref="newProjectDialogRef" @created="handleProjectCreated" />
    <LicenseDialog ref="licenseDialogRef" />
  </div>
</template>
