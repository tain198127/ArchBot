<script setup lang="ts">
import { ref, nextTick } from 'vue'
import { useI18n } from '../i18n'
import { useProject } from '../stores/project'
import { useMenuAction } from '../composables/useMenuAction'
import { projectDirs } from '../config/projectDirs'
import type { ProjectDir } from '../config/projectDirs'

const { t } = useI18n()
const { currentProject } = useProject()
const { emit: emitMenuAction } = useMenuAction()

const contextMenuVisible = ref(false)
const contextMenuPos = ref({ x: 0, y: 0 })
const contextMenuDir = ref<ProjectDir | null>(null)

function getDirLabel(labelKey: string): string {
  return (t.value.projectDirs as Record<string, string>)[labelKey] || labelKey
}

/**
 * 处理子目录右键菜单
 *
 * 业务逻辑：
 * 1. 阻止浏览器默认右键菜单
 * 2. 记录点击位置和对应的目录信息
 * 3. 显示自定义右键菜单
 * 4. 等待 DOM 更新后注册全局点击监听，用于点击外部关闭菜单
 */
function handleContextMenu(event: MouseEvent, dir: ProjectDir) {
  event.preventDefault()
  contextMenuDir.value = dir
  contextMenuPos.value = { x: event.clientX, y: event.clientY }
  contextMenuVisible.value = true

  nextTick(() => {
    document.addEventListener('click', closeContextMenu, { once: true })
  })
}

function closeContextMenu() {
  contextMenuVisible.value = false
  contextMenuDir.value = null
}

function handleNew() {
  closeContextMenu()
}

function handleImport() {
  closeContextMenu()
}

function handleDirClick(dir: ProjectDir) {
  emitMenuAction(`open.${dir.key}`)
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
          <div
            v-for="dir in projectDirs"
            :key="dir.key"
            class="dir-item"
            @click="handleDirClick(dir)"
            @contextmenu="handleContextMenu($event, dir)"
          >
            <svg class="dir-icon" viewBox="0 0 24 24" :style="{ color: dir.color }">
              <path
                fill="currentColor"
                d="M10 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"
              />
            </svg>
            <span class="dir-label">{{ getDirLabel(dir.labelKey) }}</span>
          </div>
        </div>
      </template>
      <div v-else class="tree-placeholder">
        <p class="placeholder-text">{{ t.panel.openProjectHint }}</p>
      </div>
    </div>

    <Teleport to="body">
      <div
        v-if="contextMenuVisible"
        class="context-menu"
        :style="{ left: contextMenuPos.x + 'px', top: contextMenuPos.y + 'px' }"
      >
        <div class="context-menu-item" @click="handleNew">
          {{ t.contextMenu.new }}
        </div>
        <div class="context-menu-item" @click="handleImport">
          {{ t.contextMenu.import }}
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
  gap: 8px;
  padding: 5px 12px 5px 16px;
  cursor: pointer;
  transition: background 0.12s;
  user-select: none;
}

.dir-item:hover {
  background: var(--bg-hover);
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

.context-menu {
  position: fixed;
  z-index: 9999;
  min-width: 120px;
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
</style>
