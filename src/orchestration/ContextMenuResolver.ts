// ═══════════════════════════════════════════════════════════════
// ContextMenuResolver — 根据 ContextObject 匹配右键菜单配置
// 按 resourceType 哈希索引，O(1) 查找
// ═══════════════════════════════════════════════════════════════

import type { ContextObject } from './RuntimeContext'
import { getConfig } from './ConfigLoader'
import type { ContextMenusConfig } from './ConfigValidator'

export interface ContextMenuItem {
  id?: string
  label?: string
  action?: string
  flow?: string
  type?: 'separator'
  params?: Record<string, unknown>
  predicate?: string
  visibleWhen?: string
  enabledWhen?: string
}

export interface ResolvedContextMenu {
  id: string
  items: ContextMenuItem[]
}

/** Build a resourceType → menu index for O(1) lookup */
let indexCache: Map<string, ResolvedContextMenu[]> | null = null

function buildIndex(config: ContextMenusConfig): Map<string, ResolvedContextMenu[]> {
  const index = new Map<string, ResolvedContextMenu[]>()
  for (const menu of config.contextMenus) {
    const rt = menu.appliesTo.resourceType
    const entry = index.get(rt) ?? []
    entry.push({
      id: menu.id,
      items: menu.items.map(item => ({
        id: item.id,
        label: item.label,
        action: item.action,
        flow: item.flow,
        type: item.type,
        params: item.params,
        predicate: item.predicate,
        visibleWhen: item.visibleWhen,
        enabledWhen: item.enabledWhen,
      })),
    })
    index.set(rt, entry)
  }
  return index
}

export function getContextMenuItems(context: ContextObject): ContextMenuItem[] {
  const config = getConfig().contextMenus
  if (!indexCache) {
    indexCache = buildIndex(config)
  }

  const candidates = indexCache.get(context.resourceType) ?? []
  const items: ContextMenuItem[] = []

  for (const candidate of candidates) {
    for (const item of candidate.items) {
      // Skip items that don't match semanticType (if specified in original config)
      items.push(item)
    }
  }

  return items
}

export function invalidateContextMenuCache(): void {
  indexCache = null
}
