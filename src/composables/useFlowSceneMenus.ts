/**
 * useFlowSceneMenus composable
 *
 * Computes context menu items from flows' scenario_bindings.
 * Dynamically registers/unregisters menu items when flows change.
 *
 * Used by the FileTree context menu system to add "Run Business Flow" entries.
 */

import { computed } from 'vue'
import { useFlowStore } from '../stores/flowStore'
import type { ScenarioBinding } from '../types/businessFlow'

interface FlowMenuItem {
  flowId: string
  flowName: string
  pattern: string
  label: string
}

/**
 * Get menu items that match a given file extension.
 * E.g. fileExt=".java" returns all flows with a binding matching "*.java" or ".java".
 */
export function useFlowSceneMenus() {
  const store = useFlowStore()

  /** All bindings flattened into a simple list */
  const allMenuItems = computed<FlowMenuItem[]>(() => {
    const items: FlowMenuItem[] = []

    for (const flow of store.flows.value) {
      if (!flow.published && flow.type !== 'builtin') continue // only published or built-in flows appear in menus

      let bindings: ScenarioBinding[] = []
      try {
        bindings = JSON.parse(flow.scenarioBindings)
      } catch {
        continue
      }

      for (const binding of bindings) {
        items.push({
          flowId: flow.id,
          flowName: flow.name,
          pattern: binding.pattern,
          label: binding.label || flow.name,
        })
      }
    }

    return items
  })

  /**
   * Find matching menu items for a given file path.
   * Supports exact extension matching (".java") and glob patterns ("*.py").
   */
  function getMatchingFlows(filePath: string): FlowMenuItem[] {
    const fileName = filePath.split('/').pop() ?? ''
    const ext = fileName.includes('.') ? '.' + fileName.split('.').pop() : ''

    return allMenuItems.value.filter(item => {
      const pattern = item.pattern
      // Exact extension match: ".java"
      if (pattern.startsWith('.') && pattern === ext) return true
      // Glob pattern: "*.java" → match extension
      if (pattern.startsWith('*.')) {
        const patternExt = pattern.slice(1) // ".java"
        return patternExt === ext
      }
      // Exact file name match
      if (pattern === fileName) return true
      return false
    })
  }

  return {
    allMenuItems,
    getMatchingFlows,
  }
}
