<script setup lang="ts">
import { computed, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { PanelLeft, PanelRight, PanelBottom } from '@lucide/vue'
import { useToast } from '../../composables/useToast'
import { openProject as apiOpenProject } from '../../api'
import { useMenuAction } from '../../composables/useMenuAction'
import { getConfig } from '../../orchestration/ConfigLoader'
import { evaluateExpression } from '../../orchestration/ExpressionEvaluator'
import { createRuntimeState } from '../../orchestration/RuntimeContext'
import { getActionRegistry } from '../../orchestration/ActionRegistry'
import { getPlatformShortcut } from '../../config/menu'
import type { MenuCategory } from '../../config/menu'
import { useI18n } from '../../i18n'
import { useProject } from '../../stores/project'
import { usePanelLayout } from '../../composables/usePanelLayout'
import type { RecentProject } from '../../stores/project'

const { leftCollapsed, rightCollapsed, bottomCollapsed } = usePanelLayout()

const { t, tt } = useI18n()
const toast = useToast()
const { recentProjects, setProject, clearRecentProjects, currentProject } = useProject()
const { emit: emitMenuAction } = useMenuAction()
const activeMenu = ref<string | null>(null)
const menuBarActive = ref(false)
const activeSubmenu = ref<string | null>(null)
let submenuTimer: ReturnType<typeof setTimeout> | null = null
let hideSubmenuTimer: ReturnType<typeof setTimeout> | null = null

// YML-driven menu config — replaces useMenuConfig()
const menuConfig = computed<MenuCategory[]>(() => {
  const config = getConfig()
  const state = createRuntimeState(currentProject.value)

  return config.menus.menus.map(cat => {
    const catDisabled = cat.disabledWhen ? evaluateExpression(cat.disabledWhen, state) === true : false
    return {
      name: tt(cat.label),
      disabled: catDisabled,
      groups: cat.groups.map(grp => ({
        items: grp.items.map(it => ({
          name: tt(it.label),
          shortcut: it.shortcut ?? null,
          action: it.action,
          submenu: it.type === 'submenu',
          disabled: it.enabledWhen ? !evaluateExpression(it.enabledWhen, state) : false,
        })),
      })),
      note: cat.note ? tt(cat.note) : undefined,
    }
  })
})

function handleItemClick(action?: string) {
  activeMenu.value = null; menuBarActive.value = false
  if (!action) return

  // Bridge: emit to legacy useMenuAction bus for EditorPanel / FileTreePanel
  emitMenuAction(action)

  // Also execute via ActionRegistry if registered
  const registry = getActionRegistry()
  if (registry.has(action)) {
    registry.execute(action, {}, {
      invoke: async () => {},
      openFile: () => {},
      toast: {
        success: toast.success,
        error: toast.error,
        warning: toast.warning,
      },
      pushLog: () => {},
      confirm: async () => false,
    }).catch(e => toast.error(String(e)))
  }
}

async function startDrag(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (target.closest('.menu-items') || target.closest('.window-controls')) return
  await getCurrentWindow().startDragging()
}

async function toggleMaximize() {
  const win = getCurrentWindow()
  const maximized = await win.isMaximized()
  if (maximized) await win.unmaximize()
  else await win.maximize()
}

async function minimizeWindow() { await getCurrentWindow().minimize() }
async function closeWindow() { await getCurrentWindow().close() }

function handleMenuClick(category: MenuCategory) {
  if (category.disabled) return
  if (activeMenu.value === category.name) { activeMenu.value = null; menuBarActive.value = false }
  else { activeMenu.value = category.name; menuBarActive.value = true }
}

function handleMenuEnter(category: MenuCategory) {
  if (category.disabled) return
  if (menuBarActive.value) activeMenu.value = category.name
}

function handleClickOutside() { activeMenu.value = null; menuBarActive.value = false; activeSubmenu.value = null }

function onSubmenuEnter(itemName: string) {
  clearHideTimer()
  submenuTimer = setTimeout(() => { activeSubmenu.value = itemName }, 200)
}

function onSubmenuLeave() { clearSubmenuTimer(); startHideTimer() }
function onFlyoutEnter() { clearHideTimer() }
function onFlyoutLeave() { startHideTimer() }

function clearSubmenuTimer() { if (submenuTimer) { clearTimeout(submenuTimer); submenuTimer = null } }
function startHideTimer() { hideSubmenuTimer = setTimeout(() => { activeSubmenu.value = null }, 100) }
function clearHideTimer() { if (hideSubmenuTimer) { clearTimeout(hideSubmenuTimer); hideSubmenuTimer = null } }

async function handleRecentProjectClick(project: RecentProject) {
  try {
    const result = await apiOpenProject(project.path)
    setProject({ name: result.name, path: project.path, content: result.content })
    toast.success(t.value.openProject.success)
  } catch (e) { toast.error(`${t.value.openProject.failed}: ${e}`) }
  activeMenu.value = null; menuBarActive.value = false; activeSubmenu.value = null
}

function handleClearRecent() { clearRecentProjects(); activeSubmenu.value = null }
</script>

<template>
  <div
    class="flex items-center justify-between h-8 px-2 bg-surface-50/80 dark:bg-surface-50/80 backdrop-blur-md border-b border-border-default select-none relative z-[1000]"
    @mousedown="startDrag" @dblclick="toggleMaximize"
  >
    <div class="flex items-center menu-items">
      <div v-for="category in menuConfig" :key="category.name" class="relative" @click.stop="handleMenuClick(category)" @mouseenter="handleMenuEnter(category)">
        <span
          class="inline-block px-2.5 py-1 text-[12.5px] rounded-md select-none transition-colors duration-100"
          :class="{
            'text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200': !category.disabled,
            'bg-surface-100 dark:bg-surface-200': activeMenu === category.name && !category.disabled,
            'text-text-muted/40 cursor-default': category.disabled
          }"
        >
          {{ category.name }}
        </span>

        <Transition name="menu-dropdown">
          <div v-if="activeMenu === category.name && category.groups.length > 0" class="absolute top-full left-0 min-w-[220px] bg-surface-0 dark:bg-surface-50 border border-border-default rounded-lg shadow-lg py-1 z-[2000] ring-1 ring-black/5" @click.stop>
            <template v-for="(group, gIdx) in category.groups" :key="gIdx">
              <div
                v-for="item in group.items" :key="item.name"
                class="flex items-center justify-between px-3 py-1.5 text-[13px] select-none"
                :class="{
                  'text-text-primary cursor-pointer hover:bg-surface-50 dark:hover:bg-surface-100': !item.disabled,
                  'text-text-muted/40 cursor-default': item.disabled,
                  'relative': item.submenu
                }"
                @click="!item.submenu && !item.disabled && handleItemClick(item.action)"
                @mouseenter="item.submenu && !item.disabled && onSubmenuEnter(item.name)"
                @mouseleave="item.submenu && !item.disabled && onSubmenuLeave()"
              >
                <span class="flex-1">{{ item.name }}</span>
                <span v-if="item.submenu" class="ml-auto text-[10px] text-text-muted">▶</span>
                <span v-else-if="item.shortcut" class="ml-8 text-[11px] text-text-muted whitespace-nowrap font-mono">{{ getPlatformShortcut(item.shortcut) }}</span>

                <div
                  v-if="item.submenu && activeSubmenu === item.name"
                  class="absolute left-full top-0 min-w-[320px] bg-surface-0 dark:bg-surface-50 border border-border-default rounded-lg shadow-lg py-1 z-[2001] ring-1 ring-black/5"
                  @mouseenter="onFlyoutEnter()"
                  @mouseleave="onFlyoutLeave()"
                  @click.stop
                >
                  <template v-if="recentProjects.length > 0">
                    <div v-for="rp in recentProjects" :key="rp.path" class="flex flex-col px-3 py-1.5 cursor-pointer hover:bg-surface-50 dark:hover:bg-surface-100" @click="handleRecentProjectClick(rp)">
                      <span class="text-[13px] text-text-primary">{{ rp.name }}</span>
                      <span class="text-[11px] text-text-muted mt-0.5 truncate font-mono">{{ rp.path }}</span>
                    </div>
                    <div class="h-px mx-2 my-1 bg-border-default" />
                    <div class="px-3 py-1.5 text-[12px] text-text-secondary cursor-pointer hover:bg-surface-50 dark:hover:bg-surface-100" @click="handleClearRecent()">{{ t.menuFile.clearRecent }}</div>
                  </template>
                  <div v-else class="px-3 py-2 text-[12px] text-text-muted cursor-default">{{ t.menuFile.noRecentProjects }}</div>
                </div>
              </div>
              <div v-if="gIdx < category.groups.length - 1" class="h-px mx-2 my-1 bg-border-default" />
            </template>
          </div>
        </Transition>

        <div v-if="activeMenu === category.name && category.groups.length === 0" class="absolute top-full left-0 min-w-[140px] bg-surface-0 dark:bg-surface-50 border border-border-default rounded-lg shadow-lg py-1 z-[2000] ring-1 ring-black/5">
          <div class="px-3 py-2 text-[12px] text-text-muted cursor-default">{{ category.note || t.menu.noContent }}</div>
        </div>
      </div>
    </div>

    <!-- Panel toggle tools (top-right, Cursor-style) -->
    <div class="flex items-center gap-0.5 px-2">
      <button
        class="w-7 h-7 flex items-center justify-center rounded-md transition-colors"
        :class="leftCollapsed ? 'text-primary-500 bg-primary-50 dark:bg-primary-950' : 'text-text-muted hover:text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200'"
        title="Toggle Left Panel"
        @click="leftCollapsed = !leftCollapsed"
      ><PanelLeft :size="15" /></button>
      <button
        class="w-7 h-7 flex items-center justify-center rounded-md transition-colors"
        :class="rightCollapsed ? 'text-primary-500 bg-primary-50 dark:bg-primary-950' : 'text-text-muted hover:text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200'"
        title="Toggle Right Panel"
        @click="rightCollapsed = !rightCollapsed"
      ><PanelRight :size="15" /></button>
      <button
        class="w-7 h-7 flex items-center justify-center rounded-md transition-colors"
        :class="bottomCollapsed ? 'text-primary-500 bg-primary-50 dark:bg-primary-950' : 'text-text-muted hover:text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200'"
        title="Toggle Bottom Panel"
        @click="bottomCollapsed = !bottomCollapsed"
      ><PanelBottom :size="15" /></button>
    </div>

    <div class="flex items-center window-controls">
      <span class="text-[11px] text-text-muted font-medium tracking-wide mr-1 opacity-50">ArchBot</span>
      <div class="flex items-center -mr-2">
        <button class="w-8 h-8 flex items-center justify-center bg-transparent border-0 text-xs text-text-secondary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200 transition-colors" @click.stop="minimizeWindow">─</button>
        <button class="w-8 h-8 flex items-center justify-center bg-transparent border-0 text-xs text-text-secondary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200 transition-colors" @click.stop="toggleMaximize">□</button>
        <button class="w-8 h-8 flex items-center justify-center bg-transparent border-0 text-xs text-text-secondary cursor-pointer hover:bg-danger-500 hover:text-white transition-colors" @click.stop="closeWindow">×</button>
      </div>
    </div>

    <div v-if="menuBarActive" class="fixed top-8 left-0 right-0 bottom-0 z-[999]" @click="handleClickOutside" />
  </div>
</template>
