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
}

// ─── Palette items for drag-and-drop ──────────────────────────

interface PaletteItem {
  type: string
  label: string
  icon: string
}

const paletteItems: PaletteItem[] = [
  { type: 'start', label: tt('businessFlow.editor.nodes.start'), icon: '●' },
  { type: 'end', label: tt('businessFlow.editor.nodes.end'), icon: '◉' },
  { type: 'agent', label: tt('businessFlow.editor.nodes.agent'), icon: '👤' },
  { type: 'gateway_xor', label: tt('businessFlow.editor.nodes.gatewayXor'), icon: '◇' },
  { type: 'gateway_and', label: tt('businessFlow.editor.nodes.gatewayAnd'), icon: '◈' },
  { type: 'gateway_or', label: tt('businessFlow.editor.nodes.gatewayOr'), icon: '◆' },
  { type: 'material_input', label: tt('businessFlow.editor.nodes.materialInput'), icon: '📄' },
  { type: 'quality_gate', label: tt('businessFlow.editor.nodes.qualityGate'), icon: '🛡' },
]

// ─── Drag-and-drop from palette ───────────────────────────────

const DRAG_MIME = 'application/vueflow'

function onDragStart(event: DragEvent, type: string) {
  if (!event.dataTransfer) return
  event.dataTransfer.setData(DRAG_MIME, type)
  event.dataTransfer.effectAllowed = 'move'
}

function onDragOver(event: DragEvent) {
  // preventDefault() is REQUIRED on every dragover for the browser
  // to permit the subsequent drop event. Without this, dropping
  // shows a "no-drop" cursor and onDrop never fires.
  event.preventDefault()
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move'
  }
}

/** Drag-and-drop entry point — uses VueFlow's addNodes to go through the correct
  * internal change pipeline (applyChanges → store update → v-model sync → render). */
function onDrop(event: DragEvent) {
  const nodeType = event.dataTransfer?.getData(DRAG_MIME)
  if (!nodeType) return

  const position = screenToFlowCoordinate({
    x: event.clientX,
    y: event.clientY,
  })

  const newNode: Node = {
    id: `${nodeType}-${Date.now()}`,
    type: nodeType,
    position,
    data: getDefaultNodeData(nodeType),
  }

  // addNodes triggers the store's change pipeline, which fires onNodesChange
  // → applyNodeChanges → nodes.value updated via v-model sync.
  // This ensures both the canvas renders AND getNodes.value returns the node.
  addNodes([newNode])
  dirty.value = true
}

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
    default:
      return { label: type }
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
  loadFlow()
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
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
        <h3 class="text-sm font-semibold text-text-primary truncate max-w-[200px]">
          {{ flowData?.name || 'Loading…' }}
        </h3>
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
      <!-- Left toolbar: palette -->
      <div
        v-if="!readOnly"
        class="w-[160px] shrink-0 border-r border-border-default bg-surface-50 dark:bg-surface-50 overflow-y-auto"
      >
        <div class="p-2">
          <p class="text-[10px] font-semibold uppercase text-text-muted mb-2 tracking-wider">
            {{ tt('businessFlow.editor.toolbar.flowControls') }}
          </p>
          <div class="flex flex-col gap-1">
            <div
              v-for="item in paletteItems"
              :key="item.type"
              class="flex items-center gap-2 px-2 py-1.5 rounded-md text-xs text-text-primary
                     hover:bg-surface-100 dark:hover:bg-surface-200 cursor-grab select-none transition-colors"
              draggable="true"
              @dragstart="onDragStart($event, item.type)"
            >
              <span class="text-base w-5 text-center">{{ item.icon }}</span>
              <span class="truncate">{{ item.label }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Canvas — all DnD handlers on VueFlow (official pattern) -->
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
          @drop="onDrop"
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
