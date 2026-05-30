import { computed, type Ref } from 'vue'
import { useI18n } from '../i18n'
import type { ProjectInfo } from '../stores/project'

export interface MenuItem {
  name: string
  shortcut?: string | null
  action?: string
  submenu?: boolean
  disabled?: boolean
}

export interface MenuGroup {
  items: MenuItem[]
}

export interface MenuCategory {
  name: string
  groups: MenuGroup[]
  note?: string
  disabled?: boolean
}

function getPlatformShortcut(shortcut: string | null | undefined): string {
  if (!shortcut) return ''
  const isMac = navigator.platform.toUpperCase().includes('MAC')
  const parts = shortcut.split(' / ')
  if (parts.length === 2) {
    return isMac ? parts[1] : parts[0]
  }
  return shortcut
}

export { getPlatformShortcut }

const NO_PROJECT_ACTIONS = new Set([
  'file.saveProject',
  'file.saveAsProject',
  'file.closeProject',
])

/** @deprecated Use ConfigLoader (src/orchestration/ConfigLoader.ts) + menus.yml instead. */
export function useMenuConfig(currentProject: Ref<ProjectInfo | null>) {
  const { t } = useI18n()

  const hasProject = computed(() => currentProject.value !== null)

  function fileItem(name: string, shortcut: string | null, action: string, submenu = false): MenuItem {
    return {
      name,
      shortcut,
      action,
      submenu,
      disabled: NO_PROJECT_ACTIONS.has(action) ? !hasProject.value : false,
    }
  }

  const menuConfig = computed<MenuCategory[]>(() => [
    {
      name: t.value.menu.file,
      groups: [
        {
          items: [
            fileItem(t.value.menuFile.newProject, 'Ctrl+Shift+N / ⌘+Shift+N', 'file.newProject'),
            fileItem(t.value.menuFile.openProject, 'Ctrl+O / ⌘+O', 'file.openProject'),
            fileItem(t.value.menuFile.openRecentProject, null, 'file.openRecentProject', true),
            fileItem(t.value.menuFile.openRemoteProject, null, 'file.openRemoteProject'),
          ],
        },
        {
          items: [
            fileItem(t.value.menuFile.saveProject, 'Ctrl+S / ⌘+S', 'file.saveProject'),
            fileItem(t.value.menuFile.saveAsProject, 'Ctrl+Shift+S / ⌘+Shift+S', 'file.saveAsProject'),
          ],
        },
        {
          items: [
            fileItem(t.value.menuFile.closeProject, 'Ctrl+W / ⌘+W', 'file.closeProject'),
          ],
        },
        {
          items: [
            { name: t.value.menuFile.clearCache, shortcut: null, action: 'file.clearCache' },
          ],
        },
      ],
    },
    {
      name: t.value.menu.edit,
      disabled: !hasProject.value,
      groups: [
        {
          items: [
            { name: t.value.menuEdit.find, shortcut: 'Ctrl+F / ⌘+F', action: 'edit.find' },
            { name: t.value.menuEdit.replace, shortcut: 'Ctrl+R / ⌘+R', action: 'edit.replace' },
            { name: t.value.menuEdit.goTo, shortcut: 'Ctrl+G / ⌘+G', action: 'edit.goTo' },
          ],
        },
        {
          items: [
            { name: t.value.menuEdit.cut, shortcut: 'Ctrl+X / ⌘+X', action: 'edit.cut' },
            { name: t.value.menuEdit.copy, shortcut: 'Ctrl+C / ⌘+C', action: 'edit.copy' },
          ],
        },
        {
          items: [
            { name: t.value.menuEdit.format, shortcut: 'Ctrl+Alt+L / ⌘+Option+L', action: 'edit.format' },
          ],
        },
      ],
    },
    {
      name: t.value.menu.view,
      groups: [],
      note: t.value.menu.comingSoon,
      disabled: !hasProject.value,
    },
    {
      name: t.value.menu.config,
      disabled: !hasProject.value,
      groups: [
        {
          items: [
            { name: t.value.menuConfig.digitalEmployee, shortcut: null, action: 'config.digitalEmployee' },
            { name: t.value.menuConfig.businessFlow, shortcut: null, action: 'config.businessFlow' },
            { name: t.value.menuConfig.sharedDocs, shortcut: null, action: 'config.sharedDocs' },
            { name: t.value.menuConfig.imIntegration, shortcut: null, action: 'config.imIntegration' },
            { name: t.value.menuConfig.agentConfig, shortcut: null, action: 'config.agentConfig' },
          ],
        },
        {
          items: [
            { name: t.value.menuConfig.ai, shortcut: null, action: 'config.ai' },
            { name: t.value.menuConfig.skill, shortcut: null, action: 'config.skill' },
            { name: t.value.menuConfig.mcp, shortcut: null, action: 'config.mcp' },
          ],
        },
        {
          items: [
            { name: t.value.menuConfig.contextEngineering, shortcut: null, action: 'config.contextEngineering' },
          ],
        },
        {
          items: [
            { name: t.value.menuConfig.scenario, shortcut: null, action: 'config.scenario' },
          ],
        },
        {
          items: [
            { name: t.value.menuConfig.system, shortcut: null, action: 'config.system' },
          ],
        },
      ],
    },
    {
      name: t.value.menu.run,
      disabled: !hasProject.value,
      groups: [
        {
          items: [
            { name: t.value.menuRun.terminal, shortcut: 'Alt+F12 / Option+F12', action: 'run.terminal' },
          ],
        },
        {
          items: [
            { name: t.value.menuRun.genRequirement, shortcut: null, action: 'run.genRequirement' },
            { name: t.value.menuRun.genDesign, shortcut: null, action: 'run.genDesign' },
            { name: t.value.menuRun.genCode, shortcut: null, action: 'run.genCode' },
            { name: t.value.menuRun.genSkeleton, shortcut: null, action: 'run.genSkeleton' },
          ],
        },
        {
          items: [
            { name: t.value.menuRun.genDbTable, shortcut: null, action: 'run.genDbTable' },
            { name: t.value.menuRun.genDataStandard, shortcut: null, action: 'run.genDataStandard' },
            { name: t.value.menuRun.genCallChain, shortcut: null, action: 'run.genCallChain' },
          ],
        },
        {
          items: [
            { name: t.value.menuRun.genTestCase, shortcut: null, action: 'run.genTestCase' },
            { name: t.value.menuRun.genE2eTest, shortcut: null, action: 'run.genE2eTest' },
          ],
        },
      ],
    },
    {
      name: t.value.menu.help,
      groups: [
        {
          items: [
            { name: t.value.license.register, shortcut: null, action: 'file.register' },
          ],
        },
      ],
    },
  ])

  return { menuConfig }
}
