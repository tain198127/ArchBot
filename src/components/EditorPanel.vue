<script setup lang="ts">
import { ref } from 'vue'

interface EditorTab {
  id: string
  label: string
  closable: boolean
}

const tabs = ref<EditorTab[]>([
  { id: 'welcome', label: '欢迎', closable: false }
])
const activeTab = ref('welcome')

function closeTab(id: string) {
  const index = tabs.value.findIndex(t => t.id === id)
  if (index === -1) return
  tabs.value = tabs.value.filter(t => t.id !== id)
  if (activeTab.value === id) {
    activeTab.value = tabs.value[Math.min(index, tabs.value.length - 1)]?.id || ''
  }
}
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
        <span class="tab-label">{{ tab.label }}</span>
        <span
          v-if="tab.closable"
          class="tab-close"
          @click.stop="closeTab(tab.id)"
        >×</span>
      </div>
    </div>
    <div class="editor-content">
      <div v-if="activeTab === 'welcome'" class="welcome-page">
        <h2>ArchBot</h2>
        <p>全生命周期开发管理工具</p>
        <p class="hint">从「文件」菜单打开或新建项目开始</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.editor-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #fff;
}

.editor-tabs {
  display: flex;
  align-items: center;
  height: 34px;
  background: #f0f0f0;
  border-bottom: 1px solid #e0e0e0;
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
  color: #666;
  cursor: pointer;
  border-right: 1px solid #e0e0e0;
  white-space: nowrap;
  transition: background 0.15s;
}

.editor-tab:hover {
  background: #e8e8e8;
}

.editor-tab.active {
  background: #fff;
  color: #333;
  border-bottom: 2px solid #409eff;
}

.tab-close {
  font-size: 14px;
  line-height: 1;
  color: #999;
  border-radius: 3px;
  padding: 0 2px;
}

.tab-close:hover {
  background: #ddd;
  color: #333;
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
  color: #666;
}

.welcome-page h2 {
  font-size: 28px;
  font-weight: 300;
  margin-bottom: 8px;
  color: #333;
}

.welcome-page p {
  font-size: 14px;
  margin: 4px 0;
}

.welcome-page .hint {
  margin-top: 16px;
  font-size: 13px;
  color: #999;
}

@media (prefers-color-scheme: dark) {
  .editor-panel {
    background: #1e1e1e;
  }

  .editor-tabs {
    background: #252525;
    border-bottom-color: #3c3c3c;
  }

  .editor-tab {
    color: #999;
    border-right-color: #3c3c3c;
  }

  .editor-tab:hover {
    background: #2d2d2d;
  }

  .editor-tab.active {
    background: #1e1e1e;
    color: #ddd;
  }

  .welcome-page {
    color: #999;
  }

  .welcome-page h2 {
    color: #ccc;
  }
}
</style>
