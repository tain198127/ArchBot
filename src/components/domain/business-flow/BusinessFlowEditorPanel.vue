<script setup lang="ts">
/**
 * BusinessFlowEditorPanel
 *
 * Visual BPMN flow editor using Vue Flow. Layout:
 * - Left: Toolbar (node palette + agent list)
 * - Center: Canvas with pan/zoom
 * - Bottom: Action bar (Save/Delete/Validate/Run)
 *
 * Lazy-loaded by EditorPanel via defineAsyncComponent.
 */
import { ref, onMounted, onUnmounted, computed, markRaw, watch, nextTick } from 'vue'
import { VueFlow, useVueFlow, type GraphNode, type GraphEdge, type Node, type Edge } from '@vue-flow/core'
import { Background } from '@vue-flow/background'
import { Controls } from '@vue-flow/controls'
import { MiniMap } from '@vue-flow/minimap'
import '@vue-flow/core/dist/style.css'
import '@vue-flow/core/dist/theme-default.css'
import '@vue-flow/controls/dist/style.css'
import '@vue-flow/minimap/dist/style.css'

import { useI18n } from '../../../i18n'
import { invoke } from '@tauri-apps/api/core'
import type { FlowRow, ValidationResult } from '../../../types/businessFlow'

import StartNode from './nodes/StartNode.vue'
import EndNode from './nodes/EndNode.vue'
import AgentNode from './nodes/AgentNode.vue'
import GatewayNode from './nodes/GatewayNode.vue'
import MaterialInputNode from './nodes/MaterialInputNode.vue'
import QualityGateNode from './nodes/QualityGateNode.vue'
import EmployeeNode from './nodes/EmployeeNode.vue'
import SkillNode from './nodes/SkillNode.vue'
import ReferenceNode from './nodes/ReferenceNode.vue'
import NumericGateNode from './nodes/NumericGateNode.vue'

const props = defineProps<{ flowId: string; materialFile?: string }>()

const { tt } = useI18n()

const {
  onConnect,
  addEdges,
  addNodes,
  onNodesChange,
  onEdgesChange,
  screenToFlowCoordinate,
  removeNodes,
  removeEdges,
  getNodes,
  getEdges,
} = useVueFlow()

// ─── State ────────────────────────────────────────────────────

const flowData = ref<FlowRow | null>(null)
const loading = ref(true)
const saving = ref(false)
const error = ref<string | null>(null)
const validationResults = ref<ValidationResult | null>(null)
const showValidation = ref(false)
const dirty = ref(false)

// Vue Flow nodes/edges
const nodes = ref<Node[]>([])
const edges = ref<Edge[]>([])

// Node type map for Vue Flow — markRaw prevents Vue from making components reactive
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const nodeTypes: Record<string, any> = {
  start: markRaw(StartNode),
  end: markRaw(EndNode),
  agent: markRaw(AgentNode),
  gateway_xor: markRaw(GatewayNode),
  gateway_and: markRaw(GatewayNode),
  gateway_or: markRaw(GatewayNode),
  material_input: markRaw(MaterialInputNode),
  quality_gate: markRaw(QualityGateNode),
  employee: markRaw(EmployeeNode),
  skill: markRaw(SkillNode),
  reference: markRaw(ReferenceNode),
  numeric_gate: markRaw(NumericGateNode),
}

// ─── Palette: grouped drag-and-drop items ──────────────────────

/** 硅基军团 18 个角色 — 从 de_list 动态加载 */
const employees = ref<Array<{ code: string; name: string; avatar: string; personality_tags: string; skills: string[] }>>([])

async function loadEmployees() {
  try {
    const list = await invoke<Array<Record<string, unknown>>>('de_list', { dbType: 'local' })
    employees.value = list.map((e: Record<string, unknown>) => ({
      code: e.code as string,
      name: e.name as string,
      avatar: e.avatar as string,
      personality_tags: (e.personality_tags as string) ?? '',
      skills: (e.skills as string[]) ?? [],
    }))
  } catch {
    // de_list may fail if DB not initialized — palette still works without roles
  }
}

// ─── Drag-and-drop from palette ───────────────────────────────

// Tauri WKWebView intercepts all dataTransfer MIME types (even text/plain).
// Pass drag context via a closure variable instead — proven to work everywhere.
let _dragCtx: { type: string; employeeCode?: string; employeeName?: string; avatar?: string; personality?: string } | null = null

function onDragStart(event: DragEvent, type: string) {
  _dragCtx = { type }
  // setData is only needed so the browser treats this as a real drag; value is irrelevant
  event.dataTransfer!.setData('text/plain', '')
  event.dataTransfer!.effectAllowed = 'move'
}

function onDragOver(event: DragEvent) {
  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move'
  }
}

/** Drag-and-drop entry point — uses VueFlow's addNodes to go through the correct
  * internal change pipeline (applyChanges → store update → v-model sync → render). */

function getDefaultNodeData(type: string): Record<string, unknown> {
  switch (type) {
    case 'start':
      return { label: 'Start' }
    case 'end':
      return { label: 'End' }
    case 'agent':
      return { agentId: '', agentName: '', skillName: '', inputPaths: [], outputPath: '', forbiddenPaths: [], timeout: 300, retryPolicy: { maxRetries: 0, onFail: 'abort' } }
    case 'gateway_xor':
      return { label: 'XOR Gateway', conditions: {} }
    case 'gateway_and':
      return { label: 'AND Gateway' }
    case 'gateway_or':
      return { label: 'OR Gateway', conditions: {} }
    case 'material_input':
      return { label: 'Material', filePath: '', fileName: '', fileType: '' }
    case 'quality_gate':
      return { label: 'Quality Gate', metric: '', threshold: 0.8, onFail: 'abort' }
    case 'employee':
      return { label: 'Employee', employeeCode: '', employeeName: '', avatar: '🧑‍💼', personality: '', skillName: '' }
    case 'skill':
      return { label: 'Skill', skillCode: '', skillName: '', command: '' }
    case 'reference':
      return { label: 'Reference', refType: 'file', refId: '', refName: '', filePath: '' }
    case 'numeric_gate':
      return { label: 'Numeric Gate', operation: 'compare', operands: [], threshold: 0, comparisonOp: 'gte' }
    default:
      return { label: type }
  }
}

/** Called when an employee palette item is dragged — context stored in closure */
function onDragStartEmployee(event: DragEvent, code: string, name: string, avatar: string, personality: string) {
  _dragCtx = { type: 'employee', employeeCode: code, employeeName: name, avatar, personality }
  event.dataTransfer!.setData('text/plain', '')
  event.dataTransfer!.effectAllowed = 'move'
}

/** Drop handler — reads drag context from closure variable (avoids WKWebView dataTransfer bugs) */
function onDropExtended(event: DragEvent) {
  if (!_dragCtx) {
    console.warn('[BusinessFlowEditor] Drop without drag context — was dragstart outside the palette?')
    return
  }

  const { type: nodeType, ...employeeFields } = _dragCtx
  _dragCtx = null

  if (!nodeType) {
    console.warn('[BusinessFlowEditor] Drag context missing type field')
    return
  }

  const position = screenToFlowCoordinate({ x: event.clientX, y: event.clientY })
  let data = getDefaultNodeData(nodeType)

  // Enrich employee nodes with the dragged role's metadata
  if (nodeType === 'employee') {
    data = {
      ...data,
      employeeCode: employeeFields.employeeCode ?? '',
      employeeName: employeeFields.employeeName ?? '',
      avatar: employeeFields.avatar ?? '🧑‍💼',
      personality: employeeFields.personality ?? '',
    }
  }

  const newNode: Node = {
    id: `${nodeType}-${Date.now()}`,
    type: nodeType,
    position,
    data,
  }

  addNodes([newNode])
  dirty.value = true
}

// ─── Connection handling ──────────────────────────────────────

onConnect((params) => {
  addEdges([{
    id: `edge-${Date.now()}`,
    source: params.source,
    target: params.target,
    type: 'default',
  }])
  dirty.value = true
})

onNodesChange((_changes) => {
  // With v-model:nodes, VueFlow applies all changes internally.
  // The callback only needs to track dirty state for unsaved-changes indicator.
  dirty.value = true
})

onEdgesChange((_changes) => {
  dirty.value = true
})

// ─── Load flow ────────────────────────────────────────────────

async function loadFlow() {
  loading.value = true
  error.value = null
  try {
    const row = await invoke<FlowRow>('bf_get_flow', { id: props.flowId })
    flowData.value = row
    try {
      const graph = JSON.parse(row.flowJson)
      nodes.value = graph.nodes || []
      edges.value = graph.edges || []
    } catch {
      nodes.value = []
      edges.value = []
    }
    dirty.value = false

    // If opened from context menu with a material file, add a material input node
    if (props.materialFile) {
      const fileName = props.materialFile.split('/').pop() ?? props.materialFile
      const ext = fileName.includes('.') ? fileName.split('.').pop() ?? '' : ''
      const materialNode = {
        id: `material-${Date.now()}`,
        type: 'material_input',
        position: { x: 400, y: 200 },
        data: {
          label: fileName,
          fileName,
          filePath: props.materialFile,
          fileType: ext,
        },
      }
      nodes.value = [...nodes.value, materialNode as Node]
    }
  } catch (e: unknown) {
    error.value = String(e)
  } finally {
    loading.value = false
  }
}

// ─── Save flow ────────────────────────────────────────────────

async function handleSave() {
  if (!flowData.value) return
  saving.value = true
  try {
    const flowJson = JSON.stringify({ nodes: getNodes.value, edges: getEdges.value })
    await invoke('bf_update_flow', {
      id: flowData.value.id,
      input: {
        name: flowData.value.name,
        description: flowData.value.description,
        type: flowData.value.type,
        flowJson,
        outputDir: flowData.value.outputDir,
        outputFilenamePattern: flowData.value.outputFilenamePattern,
        outputExtension: flowData.value.outputExtension,
        scenarioBindings: flowData.value.scenarioBindings,
      },
      expectedVersion: flowData.value.version,
    })
    dirty.value = false
    await loadFlow()
  } catch (e: unknown) {
    error.value = String(e)
  } finally {
    saving.value = false
  }
}

// ─── Validate ─────────────────────────────────────────────────

async function handleValidate() {
  if (!flowData.value) return
  try {
    const flowJson = JSON.stringify({ nodes: getNodes.value, edges: getEdges.value })
    validationResults.value = await invoke<ValidationResult>('bf_validate_graph', { flowJson })
    showValidation.value = true
  } catch (e: unknown) {
    error.value = String(e)
  }
}

// ─── Delete selected ──────────────────────────────────────────

function handleDeleteSelected() {
  const selectedNodeIds = getNodes.value
    .filter((n: GraphNode) => n.selected)
    .map((n: GraphNode) => n.id)
  const selectedEdgeIds = getEdges.value
    .filter((e: GraphEdge) => e.selected)
    .map((e: GraphEdge) => e.id)
  if (selectedNodeIds.length) removeNodes(selectedNodeIds)
  if (selectedEdgeIds.length) removeEdges(selectedEdgeIds)
  if (selectedNodeIds.length || selectedEdgeIds.length) dirty.value = true
}

// ─── Keyboard shortcuts ───────────────────────────────────────

function handleKeydown(event: KeyboardEvent) {
  if ((event.key === 'Delete' || event.key === 'Backspace') && !readOnly.value) {
    handleDeleteSelected()
  } else if ((event.metaKey || event.ctrlKey) && event.key === 's') {
    event.preventDefault()
    if (!readOnly.value) handleSave()
  }
}

// ─── Native DnD bindings (VueFlow has inheritAttrs:false — event passthrough fails) ──

let _paneDndBound = false

function bindPaneDnd(retries = 5) {
  if (_paneDndBound) return
  const pane = document.querySelector('.vue-flow__pane') as HTMLElement | null
  if (pane) {
    pane.addEventListener('dragover', onDragOver)
    pane.addEventListener('drop', onDropExtended)
    _paneDndBound = true
  } else if (retries > 0) {
    // VueFlow pane may not be in the DOM yet — retry on next frame
    requestAnimationFrame(() => bindPaneDnd(retries - 1))
  }
}

function unbindPaneDnd() {
  _paneDndBound = false
  const pane = document.querySelector('.vue-flow__pane') as HTMLElement | null
  if (pane) {
    pane.removeEventListener('dragover', onDragOver)
    pane.removeEventListener('drop', onDropExtended)
  }
}

// Watch loading → when VueFlow renders, bind native DnD
watch(loading, (v) => {
  if (!v) {
    // Flow done loading — wait for VueFlow to render its internal DOM
    nextTick(() => bindPaneDnd())
  }
})

onMounted(() => {
  loadFlow()
  loadEmployees()
  window.addEventListener('keydown', handleKeydown)
  // VueFlow renders its internal pane asynchronously — retry with rAF until found
  bindPaneDnd()
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  unbindPaneDnd()
})

const isPublished = computed(() => flowData.value?.published ?? false)
const isBuiltin = computed(() => flowData.value?.type === 'builtin')
const readOnly = computed(() => isPublished.value || isBuiltin.value)
</script>

<template>
  <div class="flex flex-col h-full bg-surface-0 dark:bg-surface-0">
    <!-- Top bar: flow name + action buttons -->
    <div class="flex items-center justify-between px-3 py-2 border-b border-border-default shrink-0">
      <div class="flex items-center gap-2">
        <input
          v-if="flowData"
          v-model="flowData.name"
          :disabled="readOnly"
          class="text-sm font-semibold text-text-primary bg-transparent border border-transparent hover:border-border-default focus:border-primary-500 rounded px-1.5 py-0.5 outline-none max-w-[220px] disabled:cursor-default disabled:opacity-100"
          @input="dirty = true"
        />
        <h3 v-else class="text-sm font-semibold text-text-muted animate-pulse">Loading…</h3>
        <span
          v-if="isPublished"
          class="text-[10px] px-1.5 py-0.5 rounded-full bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400"
        >
          {{ tt('businessFlow.list.published') }}
        </span>
        <span
          v-if="dirty"
          class="text-[10px] px-1.5 py-0.5 rounded-full bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400"
        >
          unsaved
        </span>
      </div>
      <div class="flex items-center gap-1.5">
        <button
          class="px-2.5 py-1 text-xs font-medium rounded-md
                 bg-surface-100 text-text-secondary hover:bg-surface-200
                 dark:bg-surface-100 dark:hover:bg-surface-200 transition-colors cursor-pointer"
          @click="handleValidate"
        >
          {{ tt('businessFlow.editor.validate') }}
        </button>
        <button
          v-if="!readOnly"
          class="px-2.5 py-1 text-xs font-medium rounded-md
                 bg-primary-500 text-white hover:bg-primary-600
                 dark:bg-primary-500 dark:hover:bg-primary-600
                 transition-colors cursor-pointer disabled:opacity-50"
          :disabled="saving"
          @click="handleSave"
        >
          {{ saving ? '…' : tt('businessFlow.editor.save') }}
        </button>
      </div>
    </div>

    <!-- Main area: toolbar + canvas -->
    <div class="flex flex-1 overflow-hidden">
      <!-- Left toolbar: grouped palette -->
      <div
        v-if="!readOnly"
        class="w-[180px] shrink-0 border-r border-border-default bg-surface-50 dark:bg-surface-50 overflow-y-auto"
      >
        <div class="p-2 flex flex-col gap-2">
          <!-- Group 1: Flow Control -->
          <details open class="group">
            <summary class="text-[10px] font-semibold uppercase text-text-muted tracking-wider cursor-pointer py-0.5 select-none">
              {{ tt('businessFlow.editor.palette.flowControl') }}
            </summary>
            <div class="flex flex-col gap-0.5 mt-1">
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" draggable="true" @dragstart="onDragStart($event, 'start')">
                <span class="text-base w-5 text-center">●</span><span class="truncate">{{ tt('businessFlow.editor.nodes.start') }}</span>
              </div>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" draggable="true" @dragstart="onDragStart($event, 'end')">
                <span class="text-base w-5 text-center">◉</span><span class="truncate">{{ tt('businessFlow.editor.nodes.end') }}</span>
              </div>
            </div>
          </details>

          <!-- Group 2: References -->
          <details open class="group">
            <summary class="text-[10px] font-semibold uppercase text-text-muted tracking-wider cursor-pointer py-0.5 select-none">
              {{ tt('businessFlow.editor.palette.references') }}
            </summary>
            <div class="flex flex-col gap-0.5 mt-1">
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" draggable="true" @dragstart="onDragStart($event, 'reference')">
                <span class="text-base w-5 text-center">📁</span><span class="truncate">File</span>
              </div>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" draggable="true" @dragstart="onDragStart($event, 'reference')">
                <span class="text-base w-5 text-center">🤖</span><span class="truncate">Agent</span>
              </div>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" draggable="true" @dragstart="onDragStart($event, 'reference')">
                <span class="text-base w-5 text-center">⚡</span><span class="truncate">Skill</span>
              </div>
            </div>
          </details>

          <!-- Group 3: Silicon Corps (18 roles, dynamic) -->
          <details open class="group">
            <summary class="text-[10px] font-semibold uppercase text-text-muted tracking-wider cursor-pointer py-0.5 select-none">
              {{ tt('businessFlow.editor.palette.siliconCorps') }} ({{ employees.length }})
            </summary>
            <div class="flex flex-col gap-0.5 mt-1">
              <div
                v-for="emp in employees" :key="emp.code"
                class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors"
                draggable="true"
                @dragstart="onDragStartEmployee($event, emp.code, emp.name, emp.avatar, emp.personality_tags)"
              >
                <span class="text-base w-5 text-center">{{ emp.avatar || '🧑‍💼' }}</span>
                <span class="truncate">{{ emp.name }}</span>
              </div>
              <div v-if="employees.length === 0" class="text-[10px] text-text-muted italic px-2 py-1">
                Loading roles…
              </div>
            </div>
          </details>

          <!-- Group 4: Execution Nodes -->
          <details open class="group">
            <summary class="text-[10px] font-semibold uppercase text-text-muted tracking-wider cursor-pointer py-0.5 select-none">
              {{ tt('businessFlow.editor.palette.execution') }}
            </summary>
            <div class="flex flex-col gap-0.5 mt-1">
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" draggable="true" @dragstart="onDragStart($event, 'agent')">
                <span class="text-base w-5 text-center">👤</span><span class="truncate">{{ tt('businessFlow.editor.nodes.agent') }}</span>
              </div>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" draggable="true" @dragstart="onDragStart($event, 'skill')">
                <span class="text-base w-5 text-center">⚡</span><span class="truncate">Skill</span>
              </div>
            </div>
          </details>

          <!-- Group 5: Gates -->
          <details open class="group">
            <summary class="text-[10px] font-semibold uppercase text-text-muted tracking-wider cursor-pointer py-0.5 select-none">
              {{ tt('businessFlow.editor.palette.gates') }}
            </summary>
            <div class="flex flex-col gap-0.5 mt-1">
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" draggable="true" @dragstart="onDragStart($event, 'gateway_and')">
                <span class="text-base w-5 text-center">◈</span><span class="truncate">{{ tt('businessFlow.editor.nodes.gatewayAnd') }}</span>
              </div>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" draggable="true" @dragstart="onDragStart($event, 'gateway_xor')">
                <span class="text-base w-5 text-center">◇</span><span class="truncate">{{ tt('businessFlow.editor.nodes.gatewayXor') }}</span>
              </div>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" draggable="true" @dragstart="onDragStart($event, 'gateway_or')">
                <span class="text-base w-5 text-center">◆</span><span class="truncate">{{ tt('businessFlow.editor.nodes.gatewayOr') }}</span>
              </div>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" draggable="true" @dragstart="onDragStart($event, 'numeric_gate')">
                <span class="text-base w-5 text-center">🔢</span><span class="truncate">Numeric Gate</span>
              </div>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" draggable="true" @dragstart="onDragStart($event, 'quality_gate')">
                <span class="text-base w-5 text-center">🛡</span><span class="truncate">{{ tt('businessFlow.editor.nodes.qualityGate') }}</span>
              </div>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" draggable="true" @dragstart="onDragStart($event, 'material_input')">
                <span class="text-base w-5 text-center">📄</span><span class="truncate">{{ tt('businessFlow.editor.nodes.materialInput') }}</span>
              </div>
            </div>
          </details>
        </div>
      </div>

      <!-- Canvas — DnD handlers MUST be on VueFlow (its internal SVG pane consumes dragover otherwise) -->
      <div class="flex-1 relative">
        <div v-if="loading" class="absolute inset-0 flex items-center justify-center bg-surface-0/80 z-50">
          <span class="text-sm text-text-secondary animate-pulse">Loading flow…</span>
        </div>

        <VueFlow
          v-model:nodes="nodes"
          v-model:edges="edges"
          :node-types="nodeTypes"
          :default-edge-options="{ type: 'default', animated: false }"
          :snap-to-grid="true"
          :snap-grid="[15, 15]"
          fit-view-on-init
          class="w-full h-full"
          :class="{ 'pointer-events-none opacity-70': readOnly }"
          @dragover="onDragOver"
          @drop="onDropExtended"
        >
          <Background />
          <Controls />
          <MiniMap />
        </VueFlow>
      </div>
    </div>

    <!-- Validation panel (toggle) -->
    <div
      v-if="showValidation && validationResults"
      class="border-t border-border-default bg-surface-50 dark:bg-surface-50 max-h-[200px] overflow-y-auto"
    >
      <div class="flex items-center justify-between px-3 py-1.5">
        <span class="text-xs font-semibold text-text-primary">
          {{ tt('businessFlow.editor.validation.title') }}
        </span>
        <button class="text-xs text-text-muted hover:text-text-primary cursor-pointer" @click="showValidation = false">&times;</button>
      </div>
      <div v-if="validationResults.issues.length === 0" class="px-3 pb-2 text-xs text-green-600 dark:text-green-400">
        {{ tt('businessFlow.editor.validation.noIssues') }}
      </div>
      <div v-else class="px-3 pb-2">
        <div
          v-for="(issue, i) in validationResults.issues"
          :key="i"
          class="flex items-start gap-1.5 py-1 text-xs"
        >
          <span
            class="shrink-0 font-semibold"
            :class="issue.severity === 'error' ? 'text-red-500' : 'text-amber-500'"
          >
            {{ issue.severity === 'error' ? '✗' : '⚠' }}
          </span>
          <span class="text-text-primary">{{ issue.message }}</span>
        </div>
      </div>
    </div>

    <!-- Error bar -->
    <div v-if="error" class="px-3 py-2 text-xs text-red-500 bg-red-50 dark:bg-red-900/10 border-t border-border-default">
      {{ error }}
    </div>
  </div>
</template>
