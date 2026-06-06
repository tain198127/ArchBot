/**
 * Business Flow Store
 *
 * Manages state for the Business Flow Designer: flow list, current flow,
 * open tabs, and dirty-state tracking. Wraps all Tauri IPC calls.
 *
 * Follows the project's ref()-based store pattern (see stores/project.ts).
 */

import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type {
  FlowSummary,
  FlowRow,
  FlowDefinition,
  ValidationResult,
  ConductorEvent,
  ScenarioBinding,
} from '../types/businessFlow'
import { registerFlowMenuProvider } from '../orchestration/ContextMenuResolver'

// ─── State ────────────────────────────────────────────────────

const flows = ref<FlowSummary[]>([])
const currentFlow = ref<FlowRow | null>(null)
const openTabs = ref<Array<{ id: string; name: string }>>([])
const loading = ref(false)
const error = ref<string | null>(null)

/** Track dirty state per flow id */
const dirtyFlows = ref<Set<string>>(new Set())

/** Streaming conductor events for the current run */
const runEvents = ref<ConductorEvent[]>([])

// ─── Computed ─────────────────────────────────────────────────

const flowSummaries = computed(() => flows.value)
const tabCount = computed(() => openTabs.value.length)

function isDirty(flowId: string): boolean {
  return dirtyFlows.value.has(flowId)
}

// ─── Actions ──────────────────────────────────────────────────

// ─── Flow menu provider ─────────────────────────────────────

function buildFlowMenuItems(filePath: string) {
  const fileName = filePath.split('/').pop() ?? ''
  const ext = fileName.includes('.') ? '.' + fileName.split('.').pop() : ''

  return flows.value
    .filter(f => f.published || f.type === 'builtin')
    .flatMap(flow => {
      let bindings: ScenarioBinding[] = []
      try {
        bindings = JSON.parse(flow.scenarioBindings)
      } catch { /* ignore malformed JSON */ }

      const matches = bindings.filter(b => {
        if (b.pattern.startsWith('.') && b.pattern === ext) return true
        if (b.pattern.startsWith('*.')) {
          return b.pattern.slice(1) === ext
        }
        if (b.pattern === fileName) return true
        return false
      })

      return matches.map(b => ({
        id: `bf.run.${flow.id}`,
        label: `Run: ${flow.name}`,
        action: 'bf.runFlow',
        params: {
          flowId: flow.id,
          flowName: flow.name,
          filePath,
        },
      }))
    })
}

async function init() {
  loading.value = true
  error.value = null
  try {
    await invoke('bf_init')
  } catch (e: unknown) {
    // Tables may already exist — that's fine
    console.warn('bf_init (tables may already exist):', e)
  }
  try {
    await loadFlows()
  } finally {
    loading.value = false
  }
  // Register the dynamic menu provider for file context menus
  registerFlowMenuProvider(buildFlowMenuItems)
}

async function loadFlows() {
  try {
    flows.value = await invoke<FlowSummary[]>('bf_list_flows')
    // Re-register menu provider with updated flow list
    registerFlowMenuProvider(buildFlowMenuItems)
  } catch (e: unknown) {
    error.value = String(e)
  }
}

async function loadFlow(id: string): Promise<FlowRow> {
  loading.value = true
  error.value = null
  try {
    const row = await invoke<FlowRow>('bf_get_flow', { id })
    currentFlow.value = row
    // Add tab if not already open
    if (!openTabs.value.find(t => t.id === id)) {
      openTabs.value = [...openTabs.value, { id: row.id, name: row.name }]
    }
    return row
  } catch (e: unknown) {
    error.value = String(e)
    throw e
  } finally {
    loading.value = false
  }
}

async function saveFlow(flow: FlowRow, input: Record<string, unknown>): Promise<FlowRow> {
  loading.value = true
  error.value = null
  try {
    const updated = await invoke<FlowRow>('bf_update_flow', {
      id: flow.id,
      input,
      expectedVersion: flow.version,
    })
    currentFlow.value = updated
    dirtyFlows.value = new Set([...dirtyFlows.value].filter(id => id !== flow.id))
    // Update tab name in case it changed
    openTabs.value = openTabs.value.map(t =>
      t.id === flow.id ? { ...t, name: updated.name } : t
    )
    await loadFlows()
    return updated
  } catch (e: unknown) {
    error.value = String(e)
    throw e
  } finally {
    loading.value = false
  }
}

async function createFlow(name: string, description = ''): Promise<FlowRow> {
  loading.value = true
  error.value = null
  try {
    const defaultGraph: FlowDefinition = {
      id: '',
      name,
      description,
      type: 'custom',
      nodes: [
        { id: 'start-1', type: 'start', position: { x: 250, y: 50 }, data: { label: 'Start' } },
        { id: 'end-1', type: 'end', position: { x: 250, y: 400 }, data: { label: 'End' } },
      ],
      edges: [],
      materials: [],
      scenarioBindings: [],
      outputConfig: {
        outputDir: './output',
        filenamePattern: '{flow}_{date}',
        extension: '.md',
      },
      version: 1,
    }

    const row = await invoke<FlowRow>('bf_create_flow', {
      input: {
        name,
        description,
        type: 'custom',
        flowJson: JSON.stringify({
          nodes: defaultGraph.nodes,
          edges: defaultGraph.edges,
        }),
        outputDir: './output',
        outputFilenamePattern: '{flow}_{date}',
        outputExtension: '.md',
        scenarioBindings: '[]',
      },
    })

    currentFlow.value = row
    openTabs.value = [...openTabs.value, { id: row.id, name: row.name }]
    await loadFlows()
    return row
  } catch (e: unknown) {
    error.value = String(e)
    throw e
  } finally {
    loading.value = false
  }
}

async function deleteFlow(id: string) {
  loading.value = true
  error.value = null
  try {
    await invoke('bf_delete_flow', { id })
    closeTab(id)
    if (currentFlow.value?.id === id) {
      currentFlow.value = null
    }
    await loadFlows()
  } catch (e: unknown) {
    error.value = String(e)
    throw e
  } finally {
    loading.value = false
  }
}

async function publishFlow(id: string) {
  loading.value = true
  error.value = null
  try {
    const updated = await invoke<FlowRow>('bf_publish_flow', { id })
    if (currentFlow.value?.id === id) {
      currentFlow.value = updated
    }
    await loadFlows()
    return updated
  } catch (e: unknown) {
    error.value = String(e)
    throw e
  } finally {
    loading.value = false
  }
}

async function copyFlow(id: string) {
  loading.value = true
  error.value = null
  try {
    const copied = await invoke<FlowRow>('bf_copy_flow', { id })
    openTabs.value = [...openTabs.value, { id: copied.id, name: copied.name }]
    currentFlow.value = copied
    await loadFlows()
    return copied
  } catch (e: unknown) {
    error.value = String(e)
    throw e
  } finally {
    loading.value = false
  }
}

async function validateFlow(id: string): Promise<ValidationResult> {
  return invoke<ValidationResult>('bf_validate_flow', { id })
}

async function validateGraph(flowJson: string): Promise<ValidationResult> {
  return invoke<ValidationResult>('bf_validate_graph', { flowJson })
}

// ─── Tab Management ───────────────────────────────────────────

function closeTab(id: string) {
  openTabs.value = openTabs.value.filter(t => t.id !== id)
  if (currentFlow.value?.id === id) {
    currentFlow.value = null
  }
}

function markDirty(flowId: string) {
  dirtyFlows.value = new Set(dirtyFlows.value).add(flowId)
}

// ─── Export ───────────────────────────────────────────────────

export function useFlowStore() {
  return {
    // state
    flows: flowSummaries,
    currentFlow,
    openTabs,
    loading,
    error,
    runEvents,
    // computed
    tabCount,
    isDirty,
    // actions
    init,
    loadFlows,
    loadFlow,
    saveFlow,
    createFlow,
    deleteFlow,
    publishFlow,
    copyFlow,
    validateFlow,
    validateGraph,
    // tab management
    closeTab,
    markDirty,
  }
}
