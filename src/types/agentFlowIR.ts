/**
 * AgentFlow IR — Standard intermediate representation for agent workflows
 *
 * Based on Workflow_IR_中心架构选型清稿.md (Section 5: AgentFlow IR).
 * This is the canonical serialization format that the Vue Flow editor
 * produces and the DagEngine consumes. It replaces the ad-hoc graph JSON.
 *
 * All node kinds from the reference:
 *   start, end, agent, tool, human_review, router, coordinator, subworkflow
 *
 * Extended with ArchBot-specific kinds:
 *   employee (硅基军团角色), skill (独立 skill 节点),
 *   reference (引用节点), numeric_gate (数字控制门),
 *   quality_gate, timer
 */

// ─── Node Kinds ─────────────────────────────────────────────────

export type AgentFlowNodeKind =
  // Flow control
  | 'start'
  | 'end'
  // Execution
  | 'agent'
  | 'skill'
  | 'employee'
  | 'sub_flow'
  // References
  | 'reference'
  // Routing
  | 'gateway_and'
  | 'gateway_xor'
  | 'gateway_or'
  | 'numeric_gate'
  // Quality & Control
  | 'quality_gate'
  | 'timer'
  | 'human_review'
  | 'signal'

// ─── Node Config (per kind) ─────────────────────────────────────

export interface AgentRef {
  /** Digital employee code, e.g. "ba-analyst" */
  employeeCode: string
  employeeName: string
  /** Skill code to use (default: employee's default_capability) */
  skillCode?: string
  inputMapping?: Record<string, string>
  outputMapping?: Record<string, string>
  timeout?: number
  retry?: { maxRetries: number; onFail: 'retry' | 'skip' | 'abort' }
}

export interface SkillRef {
  skillCode: string
  skillName: string
  command: string
}

export interface ReferenceConfig {
  refType: 'file' | 'agent' | 'skill' | 'employee'
  refId: string
  refName: string
  /** File path (only when refType === 'file') */
  filePath?: string
}

export interface NumericGateConfig {
  operation: 'add' | 'subtract' | 'multiply' | 'divide' | 'compare' | 'aggregate'
  operands: string[]  // ${nodeId.output.field} references
  threshold?: number
  comparisonOp?: 'eq' | 'neq' | 'gt' | 'gte' | 'lt' | 'lte'
  onPass?: string  // target node id (if branching)
  onFail?: string  // target node id (if branching)
}

export interface GatewayConfig {
  label: string
  conditions?: Record<string, string>  // edge id → condition expression
}

export interface QualityGateConfig {
  metric: string
  threshold: number
  onFail: 'retry' | 'skip' | 'abort'
}

export interface TimerConfig {
  durationMs: number
}

export interface HumanReviewConfig {
  approverRole: string
  timeoutMs: number
  autoApproveOnTimeout: boolean
}

export interface SubFlowConfig {
  flowId: string
  flowName: string
}

// ─── Edge Condition ─────────────────────────────────────────────

export interface EdgeCondition {
  field: string
  operator: 'eq' | 'neq' | 'gt' | 'gte' | 'lt' | 'lte' | 'contains' | 'matches'
  value: unknown
}

// ─── Node & Edge ─────────────────────────────────────────────────

export interface AgentFlowNode {
  id: string
  kind: AgentFlowNodeKind
  label: string
  position: { x: number; y: number }
  config:
    | AgentRef
    | SkillRef
    | ReferenceConfig
    | NumericGateConfig
    | GatewayConfig
    | QualityGateConfig
    | TimerConfig
    | HumanReviewConfig
    | SubFlowConfig
    | Record<string, never>  // start, end, signal
}

export interface AgentFlowEdge {
  id: string
  from: string
  to: string
  /** Human-readable action label */
  action?: string
  /** Condition for conditional edges (from router/gateway nodes) */
  condition?: EdgeCondition
  /** Quality gate embedded on edge */
  qualityGate?: QualityGateConfig
  label?: string
}

// ─── Policies ───────────────────────────────────────────────────

export interface WorkflowPolicy {
  maxTotalDuration?: string   // e.g. "3600s"
  maxRetriesPerNode?: number
  contextCompaction?: {
    strategy: 'summarize_keep_last' | 'summarize_all' | 'keep_all'
    maxTokens?: number
  }
  isolation?: {
    homeDir?: string
    allowedPaths?: string[]
    forbiddenPaths?: string[]
  }
}

// ─── Top-level IR ───────────────────────────────────────────────

export interface AgentFlowIR {
  id: string
  version: string
  name: string
  description: string
  nodes: AgentFlowNode[]
  edges: AgentFlowEdge[]
  policies: WorkflowPolicy[]
  /** Whether this flow is a built-in example (read-only) */
  readonly builtin?: boolean
  /** Timestamps */
  createdAt?: string
  updatedAt?: string
}

// ─── Serialization Helpers ──────────────────────────────────────

/**
 * Convert AgentFlowIR to the legacy Vue Flow graph JSON format.
 * This bridges the new IR with the existing persistence layer.
 */
export function irToLegacyGraph(ir: AgentFlowIR): { nodes: unknown[]; edges: unknown[] } {
  return {
    nodes: ir.nodes.map(n => ({
      id: n.id,
      type: n.kind,            // legacy: maps kind → NodeType
      position: n.position,
      data: { label: n.label, ...n.config },
    })),
    edges: ir.edges.map(e => ({
      id: e.id,
      source: e.from,
      target: e.to,
      action: e.action,
      condition: e.condition ? `${e.condition.field} ${e.condition.operator} ${e.condition.value}` : undefined,
      qualityGate: e.qualityGate,
      label: e.label,
    })),
  }
}

/**
 * Parse legacy Vue Flow graph JSON into AgentFlowIR.
 * Used when loading existing flows from the database.
 */
export function legacyGraphToIR(
  graph: { nodes: Array<Record<string, unknown>>; edges: Array<Record<string, unknown>> },
  overrides?: { id?: string; name?: string; description?: string },
): AgentFlowIR {
  return {
    id: overrides?.id ?? '',
    version: '1.0',
    name: overrides?.name ?? '',
    description: overrides?.description ?? '',
    nodes: (graph.nodes ?? []).map((n: Record<string, unknown>) => ({
      id: n.id as string,
      kind: (n.type as AgentFlowNodeKind) ?? 'agent',
      label: ((n.data as Record<string, unknown>)?.label as string) ?? (n.id as string),
      position: (n.position as { x: number; y: number }) ?? { x: 0, y: 0 },
      config: ((n.data as Record<string, unknown>) ?? {}) as AgentFlowNode['config'],
    })),
    edges: (graph.edges ?? []).map((e: Record<string, unknown>) => ({
      id: e.id as string,
      from: e.source as string,
      to: e.target as string,
      action: e.action as string | undefined,
      condition: e.condition ? parseCondition(e.condition as string) : undefined,
      qualityGate: e.qualityGate as QualityGateConfig | undefined,
      label: e.label as string | undefined,
    })),
    policies: [],
  }
}

function parseCondition(expr: string): EdgeCondition | undefined {
  const parts = expr.trim().split(/\s+/)
  if (parts.length < 3) {
    // Simple 'true'/'false' condition → no structured condition
    return undefined
  }
  return {
    field: parts[0],
    operator: parts[1] as EdgeCondition['operator'],
    value: parts.slice(2).join(' '),
  }
}
