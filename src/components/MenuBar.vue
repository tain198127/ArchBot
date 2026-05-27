<script setup lang="ts">
import { ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import { useMenuConfig, getPlatformShortcut } from '../config/menu'
import { useI18n } from '../i18n'
import { useMenuAction } from '../composables/useMenuAction'
import { useProject } from '../stores/project'
import type { MenuCategory } from '../config/menu'
import type { RecentProject } from '../stores/project'

const { menuConfig } = useMenuConfig()
const { t } = useI18n()
const { emit: emitMenuAction } = useMenuAction()
const { recentProjects, setProject, clearRecentProjects } = useProject()
const activeMenu = ref<string | null>(null)
const menuBarActive = ref(false)
const activeSubmenu = ref<string | null>(null)
let submenuTimer: ReturnType<typeof setTimeout> | null = null
let hideSubmenuTimer: ReturnType<typeof setTimeout> | null = null

async function startDrag(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (target.closest('.menu-items') || target.closest('.window-controls')) return
  await getCurrentWindow().startDragging()
}

async function toggleMaximize() {
  const win = getCurrentWindow()
  const maximized = await win.isMaximized()
  if (maximized) {
    await win.unmaximize()
  } else {
    await win.maximize()
  }
}

async function minimizeWindow() {
  await getCurrentWindow().minimize()
}

async function closeWindow() {
  await getCurrentWindow().close()
}

function handleMenuClick(category: MenuCategory) {
  if (activeMenu.value === category.name) {
    activeMenu.value = null
    menuBarActive.value = false
  } else {
    activeMenu.value = category.name
    menuBarActive.value = true
  }
}

function handleMenuEnter(category: MenuCategory) {
  if (menuBarActive.value) {
    activeMenu.value = category.name
  }
}

function handleItemClick(action?: string) {
  activeMenu.value = null
  menuBarActive.value = false
  if (action) {
    emitMenuAction(action)
  }
}

function handleClickOutside() {
  activeMenu.value = null
  menuBarActive.value = false
  activeSubmenu.value = null
}

function onSubmenuEnter(itemName: string) {
  clearHideTimer()
  submenuTimer = setTimeout(() => {
    activeSubmenu.value = itemName
  }, 500)
}

function onSubmenuLeave() {
  clearSubmenuTimer()
  startHideTimer()
}

function onFlyoutEnter() {
  clearHideTimer()
}

function onFlyoutLeave() {
  startHideTimer()
}

function clearSubmenuTimer() {
  if (submenuTimer) {
    clearTimeout(submenuTimer)
    submenuTimer = null
  }
}

function startHideTimer() {
  hideSubmenuTimer = setTimeout(() => {
    activeSubmenu.value = null
  }, 150)
}

function clearHideTimer() {
  if (hideSubmenuTimer) {
    clearTimeout(hideSubmenuTimer)
    hideSubmenuTimer = null
  }
}

async function handleRecentProjectClick(project: RecentProject) {
  try {
    const result = await invoke<{ name: string; content: string }>('open_project', { path: project.path })
    setProject({ name: result.name, path: project.path, content: result.content })
    ElMessage.success(t.value.openProject.success)
  } catch (e) {
    ElMessage.error(`${t.value.openProject.failed}: ${e}`)
  }
  activeMenu.value = null
  menuBarActive.value = false
  activeSubmenu.value = null
}

function handleClearRecent() {
  clearRecentProjects()
  activeSubmenu.value = null
}
</script>

<template>
  <div class="menu-bar" @mousedown="startDrag" @dblclick="toggleMaximize">
    <div class="menu-items">
      <div
        v-for="category in menuConfig"
        :key="category.name"
        class="menu-category"
        @click.stop="handleMenuClick(category)"
        @mouseenter="handleMenuEnter(category)"
      >
        <span class="menu-category-label" :class="{ active: activeMenu === category.name }">
          {{ category.name }}
        </span>

        <Transition name="dropdown">
          <div
            v-if="activeMenu === category.name && category.groups.length > 0"
            class="menu-dropdown"
            @click.stop
          >
            <template v-for="(group, gIdx) in category.groups" :key="gIdx">
              <div
                v-for="item in group.items"
                :key="item.name"
                class="menu-item"
                :class="{ 'has-submenu': item.submenu }"
                @click="!item.submenu && handleItemClick(item.action)"
                @mouseenter="item.submenu && onSubmenuEnter(item.name)"
                @mouseleave="item.submenu && onSubmenuLeave()"
              >
                <span class="menu-item-label">{{ item.name }}</span>
                <span v-if="item.submenu" class="menu-submenu-arrow">▶</span>
                <span v-else-if="item.shortcut" class="menu-item-shortcut">
                  {{ getPlatformShortcut(item.shortcut) }}
                </span>

                <!-- Submenu flyout for recent projects -->
                <div
                  v-if="item.submenu && activeSubmenu === item.name"
                  class="submenu-flyout"
                  @mouseenter="onFlyoutEnter()"
                  @mouseleave="onFlyoutLeave()"
                  @click.stop
                >
                  <template v-if="recentProjects.length > 0">
                    <div
                      v-for="rp in recentProjects"
                      :key="rp.path"
                      class="submenu-item"
                      @click="handleRecentProjectClick(rp)"
                    >
                      <span class="submenu-item-name">{{ rp.name }}</span>
                      <span class="submenu-item-path">{{ rp.path }}</span>
                    </div>
                    <div class="menu-divider" />
                    <div class="submenu-item submenu-clear" @click="handleClearRecent()">
                      {{ t.menuFile.clearRecent }}
                    </div>
                  </template>
                  <div v-else class="submenu-item submenu-empty">
                    {{ t.menuFile.noRecentProjects }}
                  </div>
                </div>
              </div>
              <div v-if="gIdx < category.groups.length - 1" class="menu-divider" />
            </template>
          </div>
        </Transition>

        <div
          v-if="activeMenu === category.name && category.groups.length === 0"
          class="menu-dropdown menu-dropdown-empty"
        >
          <div class="menu-item disabled">{{ category.note || t.menu.noContent }}</div>
        </div>
      </div>
    </div>

    <div class="window-controls">
      <div class="window-title">ArchBot</div>
      <div class="window-buttons">
        <button class="win-btn" @click.stop="minimizeWindow">─</button>
        <button class="win-btn" @click.stop="toggleMaximize">□</button>
        <button class="win-btn win-btn-close" @click.stop="closeWindow">×</button>
      </div>
    </div>

    <div v-if="menuBarActive" class="menu-overlay" @click="handleClickOutside" />
  </div>
</template>

<style scoped>
.menu-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 32px;
  padding: 0 8px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-color);
  user-select: none;
  position: relative;
  z-index: 1000;
}

.menu-items {
  display: flex;
  align-items: center;
  gap: 0;
}

.menu-category {
  position: relative;
}

.menu-category-label {
  display: inline-block;
  padding: 4px 10px;
  font-size: 13px;
  color: var(--text-primary);
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.15s;
}

.menu-category-label:hover,
.menu-category-label.active {
  background: var(--bg-active);
}

.menu-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  min-width: 220px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
  padding: 4px 0;
  z-index: 2000;
}

.menu-dropdown-empty {
  min-width: 140px;
}

.menu-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 16px;
  font-size: 13px;
  color: var(--text-primary);
  cursor: pointer;
  transition: background-color 0.1s;
}

.menu-item:hover {
  background: var(--bg-hover);
}

.menu-item.disabled {
  color: var(--text-muted);
  cursor: default;
}

.menu-item.disabled:hover {
  background: none;
}

.menu-item-label {
  flex: 1;
}

.menu-item-shortcut {
  margin-left: 24px;
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
}

.menu-divider {
  height: 1px;
  margin: 4px 8px;
  background: var(--border-light);
}

.window-controls {
  display: flex;
  align-items: center;
  gap: 12px;
}

.window-title {
  font-size: 12px;
  color: var(--text-secondary);
}

.window-buttons {
  display: flex;
  align-items: center;
}

.win-btn {
  width: 36px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  box-shadow: none;
  font-size: 13px;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 0;
  transition: background 0.15s;
}

.win-btn:hover {
  background: var(--bg-active);
}

.win-btn-close:hover {
  background: var(--danger-color);
  color: #fff;
}

.menu-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 999;
}

.dropdown-enter-active,
.dropdown-leave-active {
  transition: opacity 0.15s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
}

/* Submenu */
.menu-item.has-submenu {
  position: relative;
}

.menu-submenu-arrow {
  margin-left: auto;
  font-size: 10px;
  color: var(--text-muted);
}

.submenu-flyout {
  position: absolute;
  left: 100%;
  top: 0;
  min-width: 320px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  padding: 4px 0;
  z-index: 2001;
}

.submenu-item {
  display: flex;
  flex-direction: column;
  padding: 7px 16px;
  cursor: pointer;
  transition: background-color 0.1s;
}

.submenu-item:hover {
  background: var(--bg-hover);
}

.submenu-item-name {
  font-size: 13px;
  color: var(--text-primary);
  line-height: 1.4;
}

.submenu-item-path {
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1.3;
  margin-top: 1px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.submenu-clear {
  font-size: 12px;
  color: var(--text-secondary);
  flex-direction: row;
}

.submenu-empty {
  font-size: 12px;
  color: var(--text-muted);
  cursor: default;
}

.submenu-empty:hover {
  background: none;
}
</style>
