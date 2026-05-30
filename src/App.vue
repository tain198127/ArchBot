<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { openProject as apiOpenProject, initArchbotDir, ensureGitignore, dbDisconnect } from './api'
import { openFileDialog, browserOpenProjectFile } from './api/filePicker'
import { pushLog } from './stores/log'
import { isTauri } from './api/env'
import { useToast } from './composables/useToast'
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
import { useMenuAction } from './composables/useMenuAction'
import { useProject } from './stores/project'
const { t } = useI18n()
const toast = useToast()
const { initSettings, saveSettings } = useSettings()
const { on } = useMenuAction()
const { setProject, closeProject } = useProject()

const newProjectDialogRef = ref<InstanceType<typeof NewProjectDialog> | null>(null)
const licenseDialogRef = ref<InstanceType<typeof LicenseDialog> | null>(null)

onMounted(() => {
  initSettings()
})

async function handleOpenProject() {
  const filter = [{ name: t.value.openProject.filterName, extensions: ['ab'] }]

  if (isTauri) {
    // Desktop: get absolute path from native dialog, have backend read the file
    let selected: string | null = null
    try {
      selected = await openFileDialog(filter)
    } catch (e) {
      toast.error(`${t.value.openProject.failed}: ${e}`)
      return
    }
    if (!selected) return

    pushLog('info', 'app', `Opening project: ${selected} (mode: tauri)`)
    try {
      const result = await apiOpenProject(selected)
      pushLog('info', 'app', `Project opened: ${result.name}`)
      setProject({ name: result.name, path: selected, content: result.content })
      await initProjectDir(selected)
      toast.success(t.value.openProject.success)
    } catch (e) {
      toast.error(`${t.value.openProject.failed}: ${e}`)
    }
  } else {
    // Browser: no absolute paths available — read file content directly via FileReader
    let result: { name: string; path: string; content: string } | null = null
    try {
      result = await browserOpenProjectFile(filter)
    } catch (e) {
      toast.error(`${t.value.openProject.failed}: ${e}`)
      return
    }
    if (!result) return

    pushLog('info', 'app', `Opening project: ${result.path} (mode: browser, content-read locally)`)
    setProject({ name: result.name, path: result.path, content: result.content })
    toast.success(t.value.openProject.success)
    // initProjectDir is skipped — no filesystem access in browser mode
  }
}

async function handleProjectCreated(filePath: string, name: string) {
  try {
    const result = await apiOpenProject(filePath)
    setProject({ name: result.name, path: filePath, content: result.content })
  } catch {
    setProject({ name, path: filePath, content: '' })
  }
  await initProjectDir(filePath)
}

async function initProjectDir(projectPath: string) {
  try {
    await initArchbotDir(projectPath)
    await ensureGitignore(projectPath)
  } catch {
    // Non-critical: project still usable without .archbot directory
  }
}

async function handleCloseProject() {
  try {
    await dbDisconnect()
  } catch { /* ignore */ }
  closeProject()
  toast.success(t.value.menuFile.closeProject)
}

function handleClearCache() {
  saveSettings()
  window.location.reload()
}

let unsubscribe: (() => void) | null = null
onMounted(() => {
  unsubscribe = on((action) => {
    if (action === 'file.newProject') {
      newProjectDialogRef.value?.show()
    } else if (action === 'file.openProject') {
      handleOpenProject()
    } else if (action === 'file.register') {
      licenseDialogRef.value?.show()
    } else if (action === 'file.closeProject') {
      handleCloseProject()
    } else if (action === 'file.clearCache') {
      handleClearCache()
    }
  })
})
onUnmounted(() => {
  unsubscribe?.()
})

const bottomCollapseLabels = computed(() => ['', t.value.panel.bottomPanel])
const rightCollapseLabels = computed(() => ['', '', t.value.panel.model])
</script>

<template>
  <div class="flex flex-col w-full h-full overflow-hidden">
    <ToastProvider />
    <MenuBar />
    <SplitPanel
      direction="vertical"
      :initial-sizes="[-1, 200]"
      :min-sizes="[300, 120]"
      :collapsible="[false, true]"
      :collapse-icons="['', '⌨']"
      :collapse-labels="bottomCollapseLabels"
    >
      <template #panel-0>
        <SplitPanel
          direction="horizontal"
          :initial-sizes="[240, -1, 320]"
          :min-sizes="[180, 300, 240]"
          :collapsible="[false, false, true]"
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
