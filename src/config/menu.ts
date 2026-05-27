import { computed } from 'vue'
import { useI18n } from '../i18n'

export interface MenuItem {
  name: string
  shortcut?: string | null
  action?: string
  submenu?: boolean
}

export interface MenuGroup {
  items: MenuItem[]
}

export interface MenuCategory {
  name: string
  groups: MenuGroup[]
  note?: string
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

export function useMenuConfig() {
  const { t } = useI18n()

  const menuConfig = computed<MenuCategory[]>(() => [
    {
      name: t.value.menu.file,
      groups: [
        {
          items: [
            { name: t.value.menuFile.newProject, shortcut: 'Ctrl+Shift+N / ⌘+Shift+N', action: 'file.newProject' },
            { name: t.value.menuFile.openProject, shortcut: 'Ctrl+O / ⌘+O', action: 'file.openProject' },
            { name: t.value.menuFile.openRecentProject, shortcut: null, action: 'file.openRecentProject', submenu: true },
            { name: t.value.menuFile.openRemoteProject, shortcut: null, action: 'file.openRemoteProject' }
          ]
        },
        {
          items: [
            { name: t.value.menuFile.saveProject, shortcut: 'Ctrl+S / ⌘+S', action: 'file.saveProject' },
            { name: t.value.menuFile.saveAsProject, shortcut: 'Ctrl+Shift+S / ⌘+Shift+S', action: 'file.saveAsProject' }
          ]
        },
        {
          items: [
            { name: t.value.menuFile.closeProject, shortcut: 'Ctrl+W / ⌘+W', action: 'file.closeProject' }
          ]
        }
      ]
    },
    {
      name: t.value.menu.edit,
      groups: [
        {
          items: [
            { name: t.value.menuEdit.find, shortcut: 'Ctrl+F / ⌘+F', action: 'edit.find' },
            { name: t.value.menuEdit.replace, shortcut: 'Ctrl+R / ⌘+R', action: 'edit.replace' },
            { name: t.value.menuEdit.goTo, shortcut: 'Ctrl+G / ⌘+G', action: 'edit.goTo' }
          ]
        },
        {
          items: [
            { name: t.value.menuEdit.cut, shortcut: 'Ctrl+X / ⌘+X', action: 'edit.cut' },
            { name: t.value.menuEdit.copy, shortcut: 'Ctrl+C / ⌘+C', action: 'edit.copy' }
          ]
        },
        {
          items: [
            { name: t.value.menuEdit.format, shortcut: 'Ctrl+Alt+L / ⌘+Option+L', action: 'edit.format' }
          ]
        }
      ]
    },
    {
      name: t.value.menu.view,
      groups: [],
      note: t.value.menu.comingSoon
    },
    {
      name: t.value.menu.config,
      groups: [
        {
          items: [
            { name: t.value.menuConfig.ai, shortcut: null, action: 'config.ai' },
            { name: t.value.menuConfig.skill, shortcut: null, action: 'config.skill' },
            { name: t.value.menuConfig.mcp, shortcut: null, action: 'config.mcp' }
          ]
        },
        {
          items: [
            { name: t.value.menuConfig.session, shortcut: null, action: 'config.session' },
            { name: t.value.menuConfig.rules, shortcut: null, action: 'config.rules' },
            { name: t.value.menuConfig.memory, shortcut: null, action: 'config.memory' }
          ]
        },
        {
          items: [
            { name: t.value.menuConfig.codebase, shortcut: null, action: 'config.codebase' },
            { name: t.value.menuConfig.git, shortcut: null, action: 'config.git' }
          ]
        },
        {
          items: [
            { name: t.value.menuConfig.system, shortcut: null, action: 'config.system' }
          ]
        }
      ]
    },
    {
      name: t.value.menu.run,
      groups: [
        {
          items: [
            { name: t.value.menuRun.terminal, shortcut: 'Alt+F12 / Option+F12', action: 'run.terminal' }
          ]
        },
        {
          items: [
            { name: t.value.menuRun.genRequirement, shortcut: null, action: 'run.genRequirement' },
            { name: t.value.menuRun.genDesign, shortcut: null, action: 'run.genDesign' },
            { name: t.value.menuRun.genCode, shortcut: null, action: 'run.genCode' },
            { name: t.value.menuRun.genSkeleton, shortcut: null, action: 'run.genSkeleton' }
          ]
        },
        {
          items: [
            { name: t.value.menuRun.genDbTable, shortcut: null, action: 'run.genDbTable' },
            { name: t.value.menuRun.genDataStandard, shortcut: null, action: 'run.genDataStandard' },
            { name: t.value.menuRun.genCallChain, shortcut: null, action: 'run.genCallChain' }
          ]
        },
        {
          items: [
            { name: t.value.menuRun.genTestCase, shortcut: null, action: 'run.genTestCase' },
            { name: t.value.menuRun.genE2eTest, shortcut: null, action: 'run.genE2eTest' }
          ]
        }
      ]
    },
    {
      name: t.value.menu.help,
      groups: [],
      note: t.value.menu.comingSoon
    }
  ])

  return { menuConfig }
}
