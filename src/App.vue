<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import MenuBar from './components/MenuBar.vue'
import SplitPanel from './components/SplitPanel.vue'
import FileTreePanel from './components/FileTreePanel.vue'
import EditorPanel from './components/EditorPanel.vue'
import ModelPanel from './components/ModelPanel.vue'
import BottomPanel from './components/BottomPanel.vue'
import NewProjectDialog from './components/NewProjectDialog.vue'
import LicenseDialog from './components/LicenseDialog.vue'
import { useI18n } from './i18n'
import { useSettings } from './stores/settings'
import { useMenuAction } from './composables/useMenuAction'
import { useProject } from './stores/project'
const { t } = useI18n()
const { initSettings, saveSettings } = useSettings()
const { on } = useMenuAction()
const { setProject } = useProject()

const newProjectDialogRef = ref<InstanceType<typeof NewProjectDialog> | null>(null)
const licenseDialogRef = ref<InstanceType<typeof LicenseDialog> | null>(null)

onMounted(() => {
  initSettings()
})

async function handleOpenProject() {
  const selected = await open({
    multiple: false,
    filters: [{ name: t.value.openProject.filterName, extensions: ['ab'] }]
  })
  if (!selected) return

  try {
    const result = await invoke<{ name: string; content: string }>('open_project', { path: selected })
    setProject({ name: result.name, path: selected as string, content: result.content })
    ElMessage.success(t.value.openProject.success)
  } catch (e) {
    ElMessage.error(`${t.value.openProject.failed}: ${e}`)
  }
}

async function handleProjectCreated(filePath: string, name: string) {
  try {
    const result = await invoke<{ name: string; content: string }>('open_project', { path: filePath })
    setProject({ name: result.name, path: filePath, content: result.content })
  } catch {
    setProject({ name, path: filePath, content: '' })
  }
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
  <div class="app-root">
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

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  width: 100%;
  height: 100%;
  overflow: hidden;
}

:root,
[data-theme="light"] {
  --bg-primary: #ffffff;
  --bg-secondary: #f0f0f0;
  --bg-tertiary: #f5f5f5;
  --bg-panel: #f9f9f9;
  --bg-hover: #e8e8e8;
  --bg-active: #e0e0e0;

  --text-primary: #333333;
  --text-secondary: #666666;
  --text-muted: #999999;

  --border-color: #e0e0e0;
  --border-light: #e8e8e8;

  --accent-color: #409eff;
  --danger-color: #e81123;

  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-size: 14px;
  line-height: 1.5;
  color: var(--text-primary);
  background: var(--bg-primary);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

[data-theme="dark"] {
  --bg-primary: #1e1e1e;
  --bg-secondary: #252525;
  --bg-tertiary: #2b2b2b;
  --bg-panel: #252525;
  --bg-hover: #2d2d2d;
  --bg-active: #3c3c3c;

  --text-primary: #cccccc;
  --text-secondary: #999999;
  --text-muted: #666666;

  --border-color: #3c3c3c;
  --border-light: #444444;

  --accent-color: #409eff;
  --danger-color: #e81123;

  color: var(--text-primary);
  background: var(--bg-primary);
}
</style>

<style scoped>
.app-root {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  overflow: hidden;
}
</style>
