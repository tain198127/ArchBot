export interface MenuItem {
  name: string
  shortcut?: string | null
  action?: string
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

export const menuConfig: MenuCategory[] = [
  {
    name: '文件',
    groups: [
      {
        items: [
          { name: '新建项目', shortcut: 'Ctrl+Shift+N / ⌘+Shift+N', action: 'file.newProject' },
          { name: '打开项目', shortcut: 'Ctrl+O / ⌘+O', action: 'file.openProject' },
          { name: '打开远程项目', shortcut: null, action: 'file.openRemoteProject' }
        ]
      },
      {
        items: [
          { name: '保存项目', shortcut: 'Ctrl+S / ⌘+S', action: 'file.saveProject' },
          { name: '另存项目', shortcut: 'Ctrl+Shift+S / ⌘+Shift+S', action: 'file.saveAsProject' }
        ]
      },
      {
        items: [
          { name: '关闭项目', shortcut: 'Ctrl+W / ⌘+W', action: 'file.closeProject' }
        ]
      }
    ]
  },
  {
    name: '编辑',
    groups: [
      {
        items: [
          { name: '查找', shortcut: 'Ctrl+F / ⌘+F', action: 'edit.find' },
          { name: '替换', shortcut: 'Ctrl+R / ⌘+R', action: 'edit.replace' },
          { name: '定位', shortcut: 'Ctrl+G / ⌘+G', action: 'edit.goTo' }
        ]
      },
      {
        items: [
          { name: '剪切', shortcut: 'Ctrl+X / ⌘+X', action: 'edit.cut' },
          { name: '复制', shortcut: 'Ctrl+C / ⌘+C', action: 'edit.copy' }
        ]
      },
      {
        items: [
          { name: '格式化', shortcut: 'Ctrl+Alt+L / ⌘+Option+L', action: 'edit.format' }
        ]
      }
    ]
  },
  {
    name: '视图',
    groups: [],
    note: '后续补充'
  },
  {
    name: '配置',
    groups: [
      {
        items: [
          { name: 'AI配置', shortcut: null, action: 'config.ai' },
          { name: 'skill配置', shortcut: null, action: 'config.skill' },
          { name: 'MCP', shortcut: null, action: 'config.mcp' }
        ]
      },
      {
        items: [
          { name: '会话', shortcut: null, action: 'config.session' },
          { name: '规则', shortcut: null, action: 'config.rules' },
          { name: '记忆', shortcut: null, action: 'config.memory' }
        ]
      },
      {
        items: [
          { name: '代码库', shortcut: null, action: 'config.codebase' },
          { name: 'git集成', shortcut: null, action: 'config.git' }
        ]
      }
    ]
  },
  {
    name: '运行',
    groups: [
      {
        items: [
          { name: '命令行', shortcut: 'Alt+F12 / Option+F12', action: 'run.terminal' }
        ]
      },
      {
        items: [
          { name: '生成需规', shortcut: null, action: 'run.genRequirement' },
          { name: '生成设计', shortcut: null, action: 'run.genDesign' },
          { name: '生成代码', shortcut: null, action: 'run.genCode' },
          { name: '生成代码骨架', shortcut: null, action: 'run.genSkeleton' }
        ]
      },
      {
        items: [
          { name: '生成数据库表', shortcut: null, action: 'run.genDbTable' },
          { name: '生成数据标准', shortcut: null, action: 'run.genDataStandard' },
          { name: '生成调用链', shortcut: null, action: 'run.genCallChain' }
        ]
      },
      {
        items: [
          { name: '生成测试用例', shortcut: null, action: 'run.genTestCase' },
          { name: '生成e2e测试脚本', shortcut: null, action: 'run.genE2eTest' }
        ]
      }
    ]
  },
  {
    name: '帮助',
    groups: [],
    note: '后续补充'
  }
]
