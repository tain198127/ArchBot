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
import { ref, onMounted, onUnmounted, computed, markRaw } from 'vue'
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

// ─── Drag-and-drop from palette (mouse-event based — WKWebView blocks HTML5 DnD) ──

// Context: palette mousedown stores what's being dragged; mouseup on canvas creates node
let _dragCtx: { type: string; employeeCode?: string; employeeName?: string; avatar?: string; personality?: string } | null = null
let _dragGhost: HTMLElement | null = null

function createDragGhost(text: string, x: number, y: number) {
  const el = document.createElement('div')
  el.textContent = text.slice(0, 20)
  el.style.cssText = [
    'position:fixed', 'z-index:99999', 'pointer-events:none',
    'background:#4f6ef7', 'color:white', 'padding:4px 10px', 'border-radius:6px',
    'font-size:12px', 'font-weight:600', 'white-space:nowrap',
    `left:${x + 10}px`, `top:${y - 14}px`,
    'opacity:0.9', 'transition:transform 0.05s',
  ].join(';')
  document.body.appendChild(el)
  return el
}

function onPaletteMouseDown(event: MouseEvent, type: string) {
  event.preventDefault()
  _dragCtx = { type }
  _dragGhost = createDragGhost(
    (event.currentTarget as HTMLElement)?.textContent?.trim() ?? type,
    event.clientX, event.clientY)
  window.addEventListener('mousemove', onWindowMouseMove)
  window.addEventListener('mouseup', onWindowMouseUp)
  console.log('[DnD] mousedown | type=%s | x=%d y=%d', type, event.clientX, event.clientY)
}

function onPaletteMouseDownEmployee(event: MouseEvent, code: string, name: string, avatar: string, personality: string) {
  event.preventDefault()
  _dragCtx = { type: 'employee', employeeCode: code, employeeName: name, avatar, personality }
  _dragGhost = createDragGhost(name, event.clientX, event.clientY)
  window.addEventListener('mousemove', onWindowMouseMove)
  window.addEventListener('mouseup', onWindowMouseUp)
  console.log('[DnD] mousedown EMPLOYEE | code=%s name=%s', code, name)
}

function onWindowMouseMove(event: MouseEvent) {
  if (_dragGhost) {
    _dragGhost.style.left = `${event.clientX + 10}px`
    _dragGhost.style.top = `${event.clientY - 14}px`
  }
}

function onWindowMouseUp(event: MouseEvent) {
  window.removeEventListener('mousemove', onWindowMouseMove)
  window.removeEventListener('mouseup', onWindowMouseUp)

  // Remove ghost
  if (_dragGhost) {
    _dragGhost.remove()
    _dragGhost = null
  }

  if (!_dragCtx) {
    console.log('[DnD] mouseup — no drag context (was not a palette drag)')
    return
  }

  const { type: nodeType, ...employeeFields } = _dragCtx
  _dragCtx = null

  console.log('[DnD] mouseup | type=%s | x=%d y=%d', nodeType, event.clientX, event.clientY)

  // Check if mouse is over the VueFlow pane
  const pane = document.querySelector('.vue-flow__pane')
  if (!pane) {
    console.warn('[DnD] ❌ mouseup — .vue-flow__pane not found in DOM')
    return
  }
  const paneRect = pane.getBoundingClientRect()
  const overPane = event.clientX >= paneRect.left && event.clientX <= paneRect.right
                && event.clientY >= paneRect.top && event.clientY <= paneRect.bottom
  if (!overPane) {
    console.log('[DnD] mouseup — outside pane (pane=%s)', JSON.stringify(paneRect))
    return
  }

  const position = screenToFlowCoordinate({ x: event.clientX, y: event.clientY })
  let data = getDefaultNodeData(nodeType)

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
  console.log('[DnD] ✅ node ADDED | id=%s type=%s pos=(%d,%d) total=%d',
    newNode.id, newNode.type, position.x, position.y, getNodes.value.length)
}

function getDefaultNodeData(type: string): Record<string, unknown> {
  switch (type) {
    case 'start': return { label: 'Start' }
    case 'end': return { label: 'End' }
    case 'agent': return { agentId: '', agentName: '', skillName: '', inputPaths: [], outputPath: '', forbiddenPaths: [], timeout: 300, retryPolicy: { maxRetries: 0, onFail: 'abort' } }
    case 'gateway_xor': return { label: 'XOR Gateway', conditions: {} }
    case 'gateway_and': return { label: 'AND Gateway' }
    case 'gateway_or': return { label: 'OR Gateway', conditions: {} }
    case 'material_input': return { label: 'Material', filePath: '', fileName: '', fileType: '' }
    case 'quality_gate': return { label: 'Quality Gate', metric: '', threshold: 0.8, onFail: 'abort' }
    case 'employee': return { label: 'Employee', employeeCode: '', employeeName: '', avatar: '🧑‍💼', personality: '', skillName: '' }
    case 'skill': return { label: 'Skill', skillCode: '', skillName: '', command: '' }
    case 'reference': return { label: 'Reference', refType: 'file', refId: '', refName: '', filePath: '' }
    case 'numeric_gate': return { label: 'Numeric Gate', operation: 'compare', operands: [], threshold: 0, comparisonOp: 'gte' }
    default: return { label: type }
  }
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
    console.log('[DnD] loadFlow SUCCESS | nodes=%d edges=%d', nodes.value.length, edges.value.length)

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

onMounted(() => {
  console.log('[DnD] onMounted — component mounted')
  loadFlow()
  loadEmployees()
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  console.log('[DnD] onUnmounted — cleaning up')
  window.removeEventListener('keydown', handleKeydown)
  window.removeEventListener('mousemove', onWindowMouseMove)
  window.removeEventListener('mouseup', onWindowMouseUp)
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
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" @mousedown="onPaletteMouseDown($event, 'start')">
                <span class="text-base w-5 text-center">●</span><span class="truncate">{{ tt('businessFlow.editor.nodes.start') }}</span>
              </div>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" @mousedown="onPaletteMouseDown($event, 'end')">
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
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" @mousedown="onPaletteMouseDown($event, 'reference')">
                <span class="text-base w-5 text-center">📁</span><span class="truncate">File</span>
              </div>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" @mousedown="onPaletteMouseDown($event, 'reference')">
                <span class="text-base w-5 text-center">🤖</span><span class="truncate">Agent</span>
              </div>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" @mousedown="onPaletteMouseDown($event, 'reference')">
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
                @mousedown="onPaletteMouseDownEmployee($event, emp.code, emp.name, emp.avatar, emp.personality_tags)"
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
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" @mousedown="onPaletteMouseDown($event, 'agent')">
                <span class="text-base w-5 text-center">👤</span><span class="truncate">{{ tt('businessFlow.editor.nodes.agent') }}</span>
              </div>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" @mousedown="onPaletteMouseDown($event, 'skill')">
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
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" @mousedown="onPaletteMouseDown($event, 'gateway_and')">
                <span class="text-base w-5 text-center">◈</span><span class="truncate">{{ tt('businessFlow.editor.nodes.gatewayAnd') }}</span>
              </div>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" @mousedown="onPaletteMouseDown($event, 'gateway_xor')">
                <span class="text-base w-5 text-center">◇</span><span class="truncate">{{ tt('businessFlow.editor.nodes.gatewayXor') }}</span>
              </div>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" @mousedown="onPaletteMouseDown($event, 'gateway_or')">
                <span class="text-base w-5 text-center">◆</span><span class="truncate">{{ tt('businessFlow.editor.nodes.gatewayOr') }}</span>
              </div>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" @mousedown="onPaletteMouseDown($event, 'numeric_gate')">
                <span class="text-base w-5 text-center">🔢</span><span class="truncate">Numeric Gate</span>
              </div>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" @mousedown="onPaletteMouseDown($event, 'quality_gate')">
                <span class="text-base w-5 text-center">🛡</span><span class="truncate">{{ tt('businessFlow.editor.nodes.qualityGate') }}</span>
              </div>
              <div class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors" @mousedown="onPaletteMouseDown($event, 'material_input')">
                <span class="text-base w-5 text-center">📄</span><span class="truncate">{{ tt('businessFlow.editor.nodes.materialInput') }}</span>
              </div>
            </div>
          </details>
        </div>
      </div>

      <!-- Canvas -->
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
