/**
 * useBusinessFlow composable
 *
 * Convenience wrapper around flowStore that provides reactive computed
 * properties for the current flow, tab management, and dirty-state tracking.
 */

import { computed } from 'vue'
import { useFlowStore } from '../stores/flowStore'

export function useBusinessFlow() {
  const store = useFlowStore()

  /** Currently loaded flow, parsed from flowJson */
  const currentGraph = computed(() => {
    if (!store.currentFlow.value) return null
    try {
      return JSON.parse(store.currentFlow.value.flowJson)
    } catch {
      return { nodes: [], edges: [] }
    }
  })

  /** Is the current flow tab dirty? */
  const isCurrentDirty = computed(() => {
    const id = store.currentFlow.value?.id
    return id ? store.isDirty(id) : false
  })

  /** Is the current flow published (read-only)? */
  const isPublished = computed(() => {
    return store.currentFlow.value?.published ?? false
  })

  /** Is the current flow a built-in flow? */
  const isBuiltin = computed(() => {
    return store.currentFlow.value?.type === 'builtin'
  })

  return {
    // Delegate all store members
    ...store,

    // Additional computed properties
    currentGraph,
    isCurrentDirty,
    isPublished,
    isBuiltin,
  }
}
