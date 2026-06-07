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
import { useProject } from './project'
import type {
  FlowSummary,
  FlowRow,
  FlowDefinition,
  ValidationResult,
  ConductorEvent,
  ScenarioBinding,
} from '../types/businessFlow'
import { registerFlowMenuProvider } from '../orchestration/ContextMenuResolver'
import { getDagEngine } from '../orchestration/DagEngine'
import type { ActionRuntime } from '../orchestration/ActionRegistry'
import { pushLog } from './log'

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

/** Currently executing run (null when idle) */
const currentRunId = ref<string | null>(null)
const isRunning = ref(false)
const runAbortController = ref<AbortController | null>(null)

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

  // Ensure the local DB is connected before any flow operation.
  // If the user opens Business Flow before any other panel that
  // triggers db_connect (e.g., Digital Employee), the DB static
  // is still None and every command fails with "本地数据库未连接".
  const { currentProject } = useProject()
  const projectPath = currentProject.value?.path || ''
  try {
    await invoke('de_init', { dbType: 'local', projectPath })
  } catch (e: unknown) {
    console.warn('de_init (DB may already be connected):', e)
  }

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

// ─── Flow Execution (DagEngine) ───────────────────────────────

/** Feature flag: set to false to fall back to direct Rust invoke */
const USE_TS_DAG = true

/**
 * Run a business flow using the TypeScript DagEngine.
 *
 * 1. Calls Rust bf_run_flow to create the DB run record
 * 2. Parses the returned flowJson into FlowDefinition
 * 3. Executes via DagEngine, consuming events into runEvents[]
 */
async function runFlow(flowId: string, materialPaths: string[] = []) {
  if (isRunning.value) {
    pushLog('warn', 'flow', 'A flow is already running')
    return
  }

  error.value = null
  runEvents.value = []

  try {
    // 1. Create run record via Rust
    const result = await invoke<{ runId: string; flowJson: string; outputDir: string }>(
      'bf_run_flow',
      { flowId, materialPaths, outputDirOverride: null },
    )

    currentRunId.value = result.runId

    // 2. Parse flow graph
    const graph: FlowDefinition = JSON.parse(result.flowJson)
    graph.id = flowId

    // 3. Build AbortController
    const controller = new AbortController()
    runAbortController.value = controller

    // 4. Build ActionRuntime (IPC bridge to Rust)
    const runtime: ActionRuntime = {
      invoke: (cmd, args) => invoke(cmd, args ?? {}),
      openFile: (_path: string) => { /* no-op during flow execution */ },
      toast: {
        success: (msg: string) => pushLog('info', 'flow', msg),
        error: (msg: string) => pushLog('error', 'flow', msg),
        warning: (msg: string) => pushLog('warn', 'flow', msg),
      },
      pushLog: (level: 'info' | 'warn' | 'error', source: string, msg: string) =>
        pushLog(level, source, msg),
      confirm: async (_msg: string) => {
        // During batch execution, auto-approve; real approval dialogs
        // would be wired through the human_approval node's UI integration
        return true
      },
    }

    // 5. Execute
    isRunning.value = true

    const dagEngine = USE_TS_DAG ? getDagEngine() : null
    if (!dagEngine) {
      throw new Error('DagEngine not available')
    }

    const runResult = await dagEngine.execute(graph, runtime, {
      runId: result.runId,
      outputDir: result.outputDir,
      materialPaths,
      signal: controller.signal,
      onEvent: (event) => {
        runEvents.value = [...runEvents.value, event]
      },
    })

    if (!runResult.completed) {
      error.value = runResult.error ?? 'Flow execution failed'
    }
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e)
    pushLog('error', 'flow', error.value ?? 'Unknown error')
  } finally {
    isRunning.value = false
    runAbortController.value = null
    currentRunId.value = null
  }
}

/** Abort the currently running flow */
function abortRun() {
  runAbortController.value?.abort()
  if (currentRunId.value) {
    invoke('bf_abort_run', { runId: currentRunId.value }).catch(() => {})
  }
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
    currentRunId,
    isRunning,
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
    // execution
    runFlow,
    abortRun,
    // tab management
    closeTab,
    markDirty,
  }
}
