<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from '../i18n'
import { useMenuAction } from '../composables/useMenuAction'
import SettingsPanel from './SettingsPanel.vue'

const { t } = useI18n()
const { on } = useMenuAction()

interface EditorTab {
  id: string
  labelKey: string
  type: 'welcome' | 'settings' | 'editor'
  closable: boolean
}

const tabs = ref<EditorTab[]>([
  { id: 'welcome', labelKey: 'welcome', type: 'welcome', closable: false }
])
const activeTab = ref('welcome')

function getTabLabel(key: string): string {
  const editorLabels = t.value.editor as Record<string, string>
  const settingsLabels = t.value.settings as Record<string, string>
  return editorLabels[key] || settingsLabels[key] || key
}

function openTab(id: string, labelKey: string, type: EditorTab['type']) {
  const existing = tabs.value.find(tab => tab.id === id)
  if (existing) {
    activeTab.value = id
    return
  }
  tabs.value = [...tabs.value, { id, labelKey, type, closable: true }]
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

let unsubscribe: (() => void) | null = null

onMounted(() => {
  unsubscribe = on((action) => {
    if (action === 'config.system') {
      openTab('settings', 'title', 'settings')
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
        v-for="tab in tabs"
        :key="tab.id"
        class="editor-tab"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id"
      >
        <span class="tab-label">{{ getTabLabel(tab.labelKey) }}</span>
        <span
          v-if="tab.closable"
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
      <SettingsPanel v-else-if="activeTab === 'settings'" />
    </div>
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
}

.editor-tab:hover {
  background: var(--bg-hover, #e8e8e8);
}

.editor-tab.active {
  background: var(--bg-primary, #fff);
  color: var(--text-primary, #333);
  border-bottom: 2px solid #409eff;
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
  overflow: auto;
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
</style>
