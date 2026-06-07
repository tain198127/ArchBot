/**
 * Business Flow Designer — TypeScript type definitions
 *
 * Maps to the 3-table SQLite schema: business_flows, flow_runs, flow_run_artifacts.
 * All types mirror the Rust structs in business_flow/model.rs.
 */

// ─── Node Types ───────────────────────────────────────────────

/** Supported flow node types, organized by priority tier */
export type NodeType =
  // P0 — Core
  | 'start'
  | 'end'
  | 'agent'
  | 'gateway_xor'
  | 'gateway_and'
  | 'gateway_or'
  // P1 — Extended
  | 'material_input'
  | 'quality_gate'
  | 'sub_flow'
  // P2 — Advanced
  | 'timer'
  | 'signal'
  | 'error_handler'
  | 'human_approval'
  // P3 — AgentFlow IR standard
  | 'employee'
  | 'skill'
  | 'reference'
  | 'numeric_gate'

/** 2D position on the Vue Flow canvas */
export interface Position {
  x: number
  y: number
}

// ─── Node Data (per type) ─────────────────────────────────────

export interface StartNodeData {
  label: string
}

export interface EndNodeData {
  label: string
}

export interface AgentNodeData {
  agentId: string
  agentName: string
  skillName: string
  inputPaths: string[]
  outputPath: string
  forbiddenPaths: string[]
  timeout: number
  retryPolicy: RetryPolicy
  personality?: string
}

export interface RetryPolicy {
  maxRetries: number
  onFail: 'retry' | 'skip' | 'abort'
}

export interface GatewayNodeData {
  label: string
  /** For XOR/OR: condition expressions keyed by outgoing edge id */
  conditions?: Record<string, string>
}

export interface MaterialInputNodeData {
  label: string
  filePath: string
  fileName: string
  fileType: string
}

export interface QualityGateNodeData {
  label: string
  metric: string
  threshold: number
  onFail: 'retry' | 'skip' | 'abort'
}

export interface SubFlowNodeData {
  label: string
  flowId: string
  flowName: string
}

export interface TimerNodeData {
  label: string
  durationMs: number
}

export interface SignalNodeData {
  label: string
  signalName: string
  direction: 'send' | 'receive'
}

export interface ErrorHandlerNodeData {
  label: string
  errorType: string
  fallbackAction: 'skip' | 'abort' | 'notify'
}

export interface HumanApprovalNodeData {
  label: string
  approverRole: string
  timeoutMs: number
  autoApproveOnTimeout: boolean
}

/** 硅基军团角色节点 */
export interface EmployeeNodeData {
  label: string
  employeeCode: string
  employeeName: string
  avatar: string
  personality: string
  skillName?: string
}

/** 独立 Skill 节点 */
export interface SkillNodeData {
  label: string
  skillCode: string
  skillName: string
  command: string
}

/** 引用节点（引用文件、Agent、Skill、角色） */
export interface ReferenceNodeData {
  label: string
  refType: 'file' | 'agent' | 'skill' | 'employee'
  refId: string
  refName: string
  filePath?: string
}

/** 数字控制门 */
export interface NumericGateNodeData {
  label: string
  operation: 'add' | 'subtract' | 'multiply' | 'divide' | 'compare' | 'aggregate'
  operands: string[]
  threshold?: number
  comparisonOp?: 'eq' | 'neq' | 'gt' | 'gte' | 'lt' | 'lte'
  onPass?: string
  onFail?: string
}

/** Union of all node data types — discriminated by the parent node's `type` field */
export type NodeData =
  | StartNodeData
  | EndNodeData
  | AgentNodeData
  | GatewayNodeData
  | MaterialInputNodeData
  | QualityGateNodeData
  | SubFlowNodeData
  | TimerNodeData
  | SignalNodeData
  | ErrorHandlerNodeData
  | HumanApprovalNodeData
  | EmployeeNodeData
  | SkillNodeData
  | ReferenceNodeData
  | NumericGateNodeData

// ─── Flow Graph ───────────────────────────────────────────────

export interface FlowNode {
  id: string
  type: NodeType
  position: Position
  data: NodeData
}

export interface QualityGateConfig {
  metric: string
  threshold: number
  onFail: 'retry' | 'skip' | 'abort'
}

export interface FlowEdge {
  id: string
  source: string
  target: string
  action?: string
  condition?: string
  qualityGate?: QualityGateConfig
  label?: string
}

// ─── Material & Binding ───────────────────────────────────────

export interface MaterialRef {
  nodeId: string
  filePath: string
  fileName: string
  fileType: string
}

export interface ScenarioBinding {
  /** File extension or glob pattern, e.g. ".java", "*.py" */
  pattern: string
  /** Human-readable label */
  label: string
}

export interface OutputConfig {
  outputDir: string
  filenamePattern: string
  extension: string
}

// ─── Flow Definition (top-level) ──────────────────────────────

export interface FlowDefinition {
  id: string
  name: string
  description: string
  type: 'builtin' | 'custom'
  nodes: FlowNode[]
  edges: FlowEdge[]
  materials: MaterialRef[]
  scenarioBindings: ScenarioBinding[]
  outputConfig: OutputConfig
  version: number
}

/** Summary row returned by list_flows (no graph payload) */
export interface FlowSummary {
  id: string
  name: string
  description: string
  type: 'builtin' | 'custom'
  published: boolean
  scenarioBindings: string
  version: number
  updatedAt: string
}

/** Full flow row from DB (includes flowJson, outputConfig, timestamps) */
export interface FlowRow {
  id: string
  name: string
  description: string
  type: 'builtin' | 'custom'
  published: boolean
  flowJson: string
  outputDir: string
  outputFilenamePattern: string
  outputExtension: string
  scenarioBindings: string
  yamlExport: string | null
  createdAt: string
  updatedAt: string
  publishedAt: string | null
  version: number
}

// ─── Run Tracking ─────────────────────────────────────────────

export type RunStatus = 'pending' | 'running' | 'completed' | 'failed' | 'aborted'
export type TriggerSource = 'menu' | 'manual' | 'api'

export interface FlowRun {
  id: string
  flowId: string
  status: RunStatus
  triggeredBy: TriggerSource
  materialPaths: string[]
  startedAt: string
  completedAt: string | null
  outputLog: string
  errorMessage: string | null
}

export interface FlowRunArtifact {
  id: string
  runId: string
  nodeId: string
  agentId: string
  artifactPath: string
  artifactType: string
  createdAt: string
  checksum: string
}

// ─── Run Configuration ────────────────────────────────────────

export interface RunConfig {
  flowId: string
  materialPaths: string[]
  outputDirOverride?: string
  triggeredBy: TriggerSource
}

// ─── Validation ───────────────────────────────────────────────

export interface ValidationIssue {
  severity: 'error' | 'warning'
  message: string
  nodeId?: string
  edgeId?: string
}

export interface ValidationResult {
  valid: boolean
  issues: ValidationIssue[]
}

// ─── Conductor Events ─────────────────────────────────────────

export type ConductorEventType =
  | 'node_started'
  | 'node_completed'
  | 'node_failed'
  | 'quality_gate_result'
  | 'flow_completed'
  | 'flow_failed'

export interface ConductorEvent {
  runId: string
  eventType: ConductorEventType
  nodeId?: string
  agentId?: string
  message: string
  timestamp: string
  payload?: Record<string, unknown>
}
