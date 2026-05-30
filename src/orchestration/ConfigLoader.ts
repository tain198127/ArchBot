// ═══════════════════════════════════════════════════════════════
// ConfigLoader — 加载 4 份 YML → 类型化配置对象 → 缓存
// 加载失败时降级到内嵌默认菜单
// ═══════════════════════════════════════════════════════════════

import yaml from 'js-yaml'
import {
  menusConfigSchema,
  contextMenusConfigSchema,
  actionsConfigSchema,
  flowsConfigSchema,
  type MenusConfig,
  type ContextMenusConfig,
  type ActionsConfig,
  type FlowsConfig,
} from './ConfigValidator'

export interface LoadedConfig {
  menus: MenusConfig
  contextMenus: ContextMenusConfig
  actions: ActionsConfig
  flows: FlowsConfig
}

const FALLBACK_MENUS: MenusConfig = {
  version: '1.0',
  menus: [
    {
      id: 'file', label: 'menu.file', groups: [{
        items: [
          { id: 'file.newProject', label: 'menuFile.newProject', action: 'project.create' },
          { id: 'file.openProject', label: 'menuFile.openProject', action: 'project.open' },
        ],
      }],
    },
    {
      id: 'help', label: 'menu.help', groups: [{
        items: [
          { id: 'file.register', label: 'license.register', action: 'license.openDialog' },
        ],
      }],
    },
  ],
}

const FALLBACK_CONTEXT_MENUS: ContextMenusConfig = { version: '1.0', contextMenus: [] }
const FALLBACK_ACTIONS: ActionsConfig = { version: '1.0', actions: [] }
const FALLBACK_FLOWS: FlowsConfig = { version: '1.0', flows: [] }

let cached: LoadedConfig | null = null

export function getConfig(): LoadedConfig {
  if (cached) return cached
  cached = loadAll()
  return cached
}

export function reloadConfig(): LoadedConfig {
  cached = null
  return getConfig()
}

function loadAll(): LoadedConfig {
  return {
    menus: loadYml('menus.yml', menusConfigSchema, FALLBACK_MENUS),
    contextMenus: loadYml('context-menus.yml', contextMenusConfigSchema, FALLBACK_CONTEXT_MENUS),
    actions: loadYml('actions.yml', actionsConfigSchema, FALLBACK_ACTIONS),
    flows: loadYml('flows.yml', flowsConfigSchema, FALLBACK_FLOWS),
  }
}

function loadYml<T>(filename: string, schema: { parse: (v: unknown) => T }, fallback: T): T {
  try {
    // In Vite, raw YML imports from src/config/ are available as default exports
    // We use dynamic import to load them at runtime
    const raw = loadYmlSync(filename)
    const parsed = yaml.load(raw)
    return schema.parse(parsed)
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    console.error(`[ConfigLoader] Failed to load ${filename}: ${msg}. Using fallback.`)
    return fallback
  }
}

/** Synchronously load YML content. Uses Vite glob import in dev, network in production. */
function loadYmlSync(filename: string): string {
  // Vite statically imports all .yml files in src/config/ at build time
  const modules = import.meta.glob('/src/config/*.yml', {
    query: '?raw',
    import: 'default',
    eager: true,
  }) as Record<string, string>

  const key = `/src/config/${filename}`
  if (modules[key]) return modules[key]

  throw new Error(`Config file not found: ${filename}`)
}
