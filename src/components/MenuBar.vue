<script setup lang="ts">
import { ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useMenuConfig, getPlatformShortcut } from '../config/menu'
import { useI18n } from '../i18n'
import type { MenuCategory } from '../config/menu'

const { menuConfig } = useMenuConfig()
const { t } = useI18n()
const activeMenu = ref<string | null>(null)
const menuBarActive = ref(false)

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
    console.debug('[Menu Action]', action)
  }
}

function handleClickOutside() {
  activeMenu.value = null
  menuBarActive.value = false
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
                @click="handleItemClick(item.action)"
              >
                <span class="menu-item-label">{{ item.name }}</span>
                <span v-if="item.shortcut" class="menu-item-shortcut">
                  {{ getPlatformShortcut(item.shortcut) }}
                </span>
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
  background: #f5f5f5;
  border-bottom: 1px solid #e0e0e0;
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
  color: #333;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.15s;
}

.menu-category-label:hover,
.menu-category-label.active {
  background: #e0e0e0;
}

.menu-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  min-width: 220px;
  background: #fff;
  border: 1px solid #e0e0e0;
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
  color: #333;
  cursor: pointer;
  transition: background-color 0.1s;
}

.menu-item:hover {
  background: #f0f0f0;
}

.menu-item.disabled {
  color: #999;
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
  color: #999;
  white-space: nowrap;
}

.menu-divider {
  height: 1px;
  margin: 4px 8px;
  background: #e8e8e8;
}

.window-controls {
  display: flex;
  align-items: center;
  gap: 12px;
}

.window-title {
  font-size: 12px;
  color: #666;
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
  color: #666;
  cursor: pointer;
  border-radius: 0;
  transition: background 0.15s;
}

.win-btn:hover {
  background: #e0e0e0;
}

.win-btn-close:hover {
  background: #e81123;
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

@media (prefers-color-scheme: dark) {
  .menu-bar {
    background: #2b2b2b;
    border-bottom-color: #3c3c3c;
  }

  .menu-category-label {
    color: #ccc;
  }

  .menu-category-label:hover,
  .menu-category-label.active {
    background: #3c3c3c;
  }

  .menu-dropdown {
    background: #2d2d2d;
    border-color: #444;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .menu-item {
    color: #ccc;
  }

  .menu-item:hover {
    background: #3c3c3c;
  }

  .menu-divider {
    background: #444;
  }

  .window-title {
    color: #999;
  }
}
</style>
