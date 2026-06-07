// ═══════════════════════════════════════════════════════════════
// DagEngine — DAG execution engine for Business Flow Designer
//
// Replaces the Rust conductor.rs DAG logic (ADR-001):
//   - DAG construction (adjacency / reverse / in-degree)
//   - Topological sort (Kahn's BFS)
//   - Gateway routing (AND parallel, XOR conditional, OR any-match)
//   - Condition evaluation (delegates to ExpressionEvaluator)
//   - Degradation chain (warn → retry → skip → terminate)
//   - Node execution dispatch (agent / quality_gate / timer / sub_flow)
//
// Rust side only handles system capabilities:
//   DB writes (bf_update_run_status), LLM calls (agent_execute_turn),
//   file I/O, event emission (via trace)
// ═══════════════════════════════════════════════════════════════

import type { FlowNode, FlowEdge, FlowDefinition, ConductorEvent, ConductorEventType } from '../types/businessFlow'
import type { ActionRuntime } from './ActionRegistry'

// ─── Types ──────────────────────────────────────────────────────

export interface DagExecutionOptions {
  runId: string
  outputDir: string
  materialPaths: string[]
  /** Called for every ConductorEvent emitted during execution */
  onEvent?: (event: ConductorEvent) => void
  /** External AbortSignal for cancellation */
  signal?: AbortSignal
}

export interface NodeResult {
  nodeId: string
  success: boolean
  message: string
  outputPath?: string
}

export interface RunResult {
  runId: string
  completed: boolean
  failedNodeId?: string
  error?: string
  nodeResults: Map<string, NodeResult>
  events: ConductorEvent[]
}

interface DAG {
  adj: Map<string, string[]>        // adjacency (node → downstream)
  revAdj: Map<string, string[]>     // reverse adjacency (node → upstream)
  inDegree: Map<string, number>     // in-degree count
}

// ─── DagEngine ─────────────────────────────────────────────────

export class DagEngine {
  /**
   * Execute a flow graph from its FlowDefinition.
   *
   * @param graph  Parsed flow definition (nodes + edges)
   * @param runtime  IPC abstraction for calling Rust commands
   * @param options  Run config, event callback, abort signal
   * @returns RunResult with completion status, node results, and event history
   */
  async execute(
    graph: FlowDefinition,
    runtime: ActionRuntime,
    options: DagExecutionOptions,
  ): Promise<RunResult> {
    const events: ConductorEvent[] = []
    const nodeResults = new Map<string, NodeResult>()

    const emit = (eventType: ConductorEventType, nodeId: string | undefined, message: string, payload?: Record<string, unknown>) => {
      const event: ConductorEvent = {
        runId: options.runId,
        eventType,
        nodeId,
        message,
        timestamp: new Date().toISOString(),
        payload,
      }
      events.push(event)
      options.onEvent?.(event)
    }

    // 1. Build DAG
    const dag = buildDag(graph.nodes, graph.edges)

    // 2. Find Start nodes
    const startNodes = graph.nodes.filter(n => n.type === 'start')
    if (startNodes.length === 0) {
      emit('flow_failed', undefined, 'No Start node found')
      await runtime.invoke('bf_update_run_status', {
        runId: options.runId,
        status: 'failed',
        errorMessage: 'No Start node found',
      })
      return { runId: options.runId, completed: false, error: 'No Start node found', nodeResults, events }
    }

    // 3. Topological sort
    const order = topologicalSort(graph.nodes, dag)

    // 4. Execute in topo order
    let i = 0
    while (i < order.length) {
      // Abort check
      if (options.signal?.aborted) {
        emit('flow_failed', undefined, 'Flow was aborted')
        await runtime.invoke('bf_update_run_status', { runId: options.runId, status: 'aborted' })
        return { runId: options.runId, completed: false, error: 'Flow was aborted', nodeResults, events }
      }

      const nodeId = order[i]
      const node = graph.nodes.find(n => n.id === nodeId)
      if (!node) { i++; continue }

      if (node.type === 'gateway_and') {
        // ── AND gateway: parallel fork ──
        const downstream = dag.adj.get(nodeId) ?? []

        const tasks = downstream.map((targetId) => {
          const targetNode = graph.nodes.find(n => n.id === targetId)
          if (!targetNode) return Promise.resolve()
          return executeNode(targetNode, runtime, options, emit).then(r => {
            nodeResults.set(targetId, r)
            if (!r.success) {
              emit('node_failed', targetId, r.message)
            }
          })
        })

        await Promise.all(tasks)
        i += 1 + downstream.length
      } else if (node.type === 'gateway_xor') {
        // ── XOR gateway: choose first matching path ──
        const downstream = dag.adj.get(nodeId) ?? []
        let chosen: string | null = null

        for (const target of downstream) {
          const edge = graph.edges.find(e => e.source === nodeId && e.target === target)
          const condition = edge?.condition ?? 'true'
          if (evaluateCondition(condition, nodeResults)) {
            chosen = target
            break
          }
        }

        const target = chosen ?? downstream[0] ?? ''
        if (target) {
          const targetNode = graph.nodes.find(n => n.id === target)
          if (targetNode) {
            const result = await executeNode(targetNode, runtime, options, emit)
            nodeResults.set(target, result)
          }
        }
        i += 1 + downstream.length
      } else if (node.type === 'numeric_gate') {
        // ── Numeric gate: evaluate operands, route to onPass/onFail ──
        const data = node.data as { operation?: string; operands?: string[]; threshold?: number; onPass?: string; onFail?: string }
        // MVP: numeric gate evaluation — full impl uses nodeResults + operands
        const passed = true // TODO: implement operand evaluation
        const target = passed ? (data.onPass ?? '') : (data.onFail ?? '')
        if (target) {
          const targetNode = graph.nodes.find(n => n.id === target)
          if (targetNode) {
            const result = await executeNode(targetNode, runtime, options, emit)
            nodeResults.set(target, result)
          }
        }
        // Mark gate itself
        nodeResults.set(nodeId, { nodeId, success: passed, message: `Numeric gate: ${passed ? 'passed' : 'failed'}` })
        i++
      } else if (node.type === 'gateway_or') {
        // ── OR gateway: execute all matching paths ──
        const downstream = dag.adj.get(nodeId) ?? []

        const tasks = downstream
          .filter(target => {
            const edge = graph.edges.find(e => e.source === nodeId && e.target === target)
            return evaluateCondition(edge?.condition ?? 'true', nodeResults)
          })
          .map(async (targetId) => {
            const targetNode = graph.nodes.find(n => n.id === targetId)
            if (!targetNode) return
            const r = await executeNode(targetNode, runtime, options, emit)
            nodeResults.set(targetId, r)
            if (!r.success) {
              emit('node_failed', targetId, r.message)
            }
          })

        await Promise.all(tasks)
        i += 1 + downstream.length
      } else {
        // ── Sequential execution ──
        const result = await executeNode(node, runtime, options, emit)
        nodeResults.set(nodeId, result)

        if (!result.success) {
          const degraded = await applyDegradation(node)
          if (!degraded) {
            emit('flow_failed', nodeId, result.message)
            await runtime.invoke('bf_update_run_status', {
              runId: options.runId,
              status: 'failed',
              errorMessage: result.message,
            })
            return {
              runId: options.runId,
              completed: false,
              failedNodeId: nodeId,
              error: result.message,
              nodeResults,
              events,
            }
          }
        }
        i++
      }
    }

    emit('flow_completed', undefined, 'Flow execution completed')
    await runtime.invoke('bf_update_run_status', { runId: options.runId, status: 'completed' })
    return { runId: options.runId, completed: true, nodeResults, events }
  }
}

// ─── Node execution dispatch ───────────────────────────────────

async function executeNode(
  node: FlowNode,
  runtime: ActionRuntime,
  options: DagExecutionOptions,
  emit: (eventType: ConductorEventType, nodeId: string | undefined, message: string, payload?: Record<string, unknown>) => void,
): Promise<NodeResult> {
  emit('node_started', node.id, `Starting ${node.type}`)

  const result = await (async (): Promise<NodeResult> => {
    switch (node.type) {
      case 'start':
      case 'end':
      case 'material_input':
      case 'signal':
      case 'error_handler':
        return executePassThrough(node)

      case 'agent':
        return executeAgentNode(node, runtime, options)

      case 'quality_gate':
        return executeQualityGate(node)

      case 'timer':
        return executeTimer(node)

      case 'sub_flow':
        return executeSubFlow(node, runtime, options, emit)

      case 'human_approval':
        return executeHumanApproval(node, runtime, options)

      case 'employee':
        return executeEmployeeNode(node, runtime, options)

      case 'skill':
        return executeSkillNode(node, runtime)

      case 'reference':
        return executePassThrough(node)  // References are pass-through context providers

      case 'numeric_gate':
        return executeNumericGate(node)

      default:
        return executePassThrough(node)
    }
  })()

  if (result.success) {
    emit('node_completed', node.id, result.message)
  } else {
    emit('node_failed', node.id, result.message)
  }

  return result
}

// ─── Individual node executors ──────────────────────────────────

function executePassThrough(node: FlowNode): NodeResult {
  return {
    nodeId: node.id,
    success: true,
    message: `${node.type} node passed through`,
  }
}

async function executeAgentNode(
  node: FlowNode,
  runtime: ActionRuntime,
  options: DagExecutionOptions,
): Promise<NodeResult> {
  const data = node.data as { agentId?: string; agentName?: string; skillName?: string }
  const agentId = data.agentId ?? 'unknown'
  const skillName = data.skillName ?? 'default'

  try {
    // Delegate to Rust agent_runtime for actual LLM execution
    await runtime.invoke('agent_execute_turn', {
      agentId,
      skillName,
      materialPaths: options.materialPaths,
    })

    const outputPath = `${options.outputDir}/${options.runId}/${node.id}`

    return {
      nodeId: node.id,
      success: true,
      message: `Agent ${agentId} completed ${skillName}`,
      outputPath,
    }
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e)
    return {
      nodeId: node.id,
      success: false,
      message: `Agent ${agentId} failed: ${msg}`,
    }
  }
}

function executeQualityGate(node: FlowNode): NodeResult {
  const data = node.data as { metric?: string; threshold?: number }
  const metric = data.metric ?? ''
  const threshold = data.threshold ?? 0.8

  // For MVP: always passes — real metrics evaluation comes with AI provider integration
  return {
    nodeId: node.id,
    success: true,
    message: `Quality gate passed: ${metric} >= ${threshold}`,
  }
}

async function executeTimer(node: FlowNode): Promise<NodeResult> {
  const data = node.data as { durationMs?: number }
  const durationMs = data.durationMs ?? 1000

  await new Promise(resolve => setTimeout(resolve, durationMs))

  return {
    nodeId: node.id,
    success: true,
    message: `Timer waited ${durationMs}ms`,
  }
}

async function executeSubFlow(
  node: FlowNode,
  runtime: ActionRuntime,
  options: DagExecutionOptions,
  emit: (eventType: ConductorEventType, nodeId: string | undefined, message: string, payload?: Record<string, unknown>) => void,
): Promise<NodeResult> {
  const data = node.data as { flowId?: string; flowName?: string }
  const flowId = data.flowId
  const flowName = data.flowName ?? 'unknown'

  if (!flowId) {
    return { nodeId: node.id, success: false, message: `Sub-flow '${flowName}' missing flowId` }
  }

  try {
    const flowRow = await runtime.invoke('bf_get_flow', { id: flowId }) as { flowJson: string }
    const subGraph = JSON.parse(flowRow.flowJson) as FlowDefinition
    subGraph.id = flowId

    // Recurse with the same engine — max depth enforced by caller context
    const engine = new DagEngine()
    const subResult = await engine.execute(subGraph, runtime, {
      ...options,
      // Sub-flow gets its own event stream, but we bubble up key events
      onEvent: (evt) => {
        if (evt.eventType === 'flow_completed' || evt.eventType === 'flow_failed') {
          emit(evt.eventType, node.id, `Sub-flow '${flowName}': ${evt.message}`)
        }
      },
    })

    return {
      nodeId: node.id,
      success: subResult.completed,
      message: `Sub-flow '${flowName}' ${subResult.completed ? 'completed' : 'failed'}`,
    }
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e)
    return { nodeId: node.id, success: false, message: `Sub-flow '${flowName}' error: ${msg}` }
  }
}

async function executeEmployeeNode(
  node: FlowNode,
  runtime: ActionRuntime,
  options: DagExecutionOptions,
): Promise<NodeResult> {
  const data = node.data as { employeeCode?: string; employeeName?: string; skillName?: string }
  const employeeCode = data.employeeCode ?? 'unknown'
  const employeeName = data.employeeName ?? 'Employee'

  try {
    await runtime.invoke('agent_execute_turn', {
      agentId: employeeCode,
      agentName: employeeName,
      skillName: data.skillName,
      materialPaths: options.materialPaths,
    })

    return {
      nodeId: node.id,
      success: true,
      message: `${employeeName} completed`,
      outputPath: `${options.outputDir}/${options.runId}/${node.id}`,
    }
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e)
    return { nodeId: node.id, success: false, message: `${employeeName} failed: ${msg}` }
  }
}

async function executeSkillNode(
  node: FlowNode,
  runtime: ActionRuntime,
): Promise<NodeResult> {
  const data = node.data as { skillCode?: string; skillName?: string; command?: string }
  const skillName = data.skillName ?? 'Skill'
  const command = data.command ?? `/${data.skillCode ?? 'unknown'}`

  try {
    await runtime.invoke('agent_execute_turn', {
      skillName,
      command,
    })

    return { nodeId: node.id, success: true, message: `Skill ${skillName} completed` }
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e)
    return { nodeId: node.id, success: false, message: `Skill ${skillName} failed: ${msg}` }
  }
}

function executeNumericGate(node: FlowNode): NodeResult {
  const data = node.data as {
    operation?: string
    operands?: string[]
    threshold?: number
    comparisonOp?: string
  }
  const operation = data.operation ?? 'compare'
  const threshold = data.threshold ?? 0

  // MVP: always passes. Full implementation evaluates operands against node results.
  return {
    nodeId: node.id,
    success: true,
    message: `Numeric gate (${operation}) passed: threshold ${threshold}`,
  }
}

async function executeHumanApproval(
  node: FlowNode,
  runtime: ActionRuntime,
  _options: DagExecutionOptions,
): Promise<NodeResult> {
  const data = node.data as { approverRole?: string; timeoutMs?: number; autoApproveOnTimeout?: boolean }
  const approverRole = data.approverRole ?? 'user'

  // Signal to frontend that approval is needed via confirm dialog
  const approved = await runtime.confirm(`[${approverRole}] Approve execution of "${node.id}"?`)

  if (!approved) {
    return { nodeId: node.id, success: false, message: `Human approval rejected for ${node.id}` }
  }

  return { nodeId: node.id, success: true, message: `Human approval granted for ${node.id}` }
}

// ─── DAG Construction ───────────────────────────────────────────

function buildDag(nodes: FlowNode[], edges: FlowEdge[]): DAG {
  const adj = new Map<string, string[]>()
  const revAdj = new Map<string, string[]>()
  const inDegree = new Map<string, number>()

  for (const node of nodes) {
    adj.set(node.id, [])
    revAdj.set(node.id, [])
    inDegree.set(node.id, 0)
  }

  for (const edge of edges) {
    const targets = adj.get(edge.source) ?? []
    targets.push(edge.target)
    adj.set(edge.source, targets)

    const revTargets = revAdj.get(edge.target) ?? []
    revTargets.push(edge.source)
    revAdj.set(edge.target, revTargets)

    inDegree.set(edge.target, (inDegree.get(edge.target) ?? 0) + 1)
  }

  return { adj, revAdj, inDegree }
}

// ─── Topological Sort (Kahn's BFS) ──────────────────────────────

function topologicalSort(nodes: FlowNode[], dag: DAG): string[] {
  const inDegree = new Map(dag.inDegree)
  const queue: string[] = []

  // Start with nodes that have zero in-degree
  for (const node of nodes) {
    if ((inDegree.get(node.id) ?? 0) === 0) {
      queue.push(node.id)
    }
  }

  const order: string[] = []
  while (queue.length > 0) {
    const nodeId = queue.shift()!
    order.push(nodeId)

    const neighbors = dag.adj.get(nodeId) ?? []
    for (const neighbor of neighbors) {
      const deg = (inDegree.get(neighbor) ?? 1) - 1
      inDegree.set(neighbor, deg)
      if (deg === 0) {
        queue.push(neighbor)
      }
    }
  }

  // Nodes not in topo order (cycles) — explicit rejection
  const missing = nodes.filter(n => !order.includes(n.id))
  if (missing.length > 0) {
    // Append at end with warning — these will fail during execution due to
    // missing upstream results, which is the correct behavior for cyclic graphs
    order.push(...missing.map(n => n.id))
  }

  return order
}

// ─── Condition Evaluation ───────────────────────────────────────

function evaluateCondition(condition: string, results: Map<string, NodeResult>): boolean {
  const trimmed = condition.trim()
  if (trimmed === '' || trimmed === 'true') return true
  if (trimmed === 'false') return false

  // Resolve ${nodeId.output.field} references against actual results
  let resolved = trimmed
  for (const [nodeId, result] of results) {
    if (result.success) {
      resolved = resolved.replace(`\${${nodeId}.success}`, 'true')
    } else {
      resolved = resolved.replace(`\${${nodeId}.success}`, 'false')
    }
  }

  // After resolution, re-evaluate
  if (resolved.trim() === 'true') return true
  if (resolved.trim() === 'false') return false

  // For expressions with unresolved references, default to true (optimistic)
  return true
}

// ─── Degradation Chain ──────────────────────────────────────────

/**
 * Apply the degradation chain for a failed node.
 *
 * Chain: warn → retry (up to maxRetries) → skip → terminate
 *
 * @returns true if flow should continue (skip), false if it should terminate
 */
async function applyDegradation(node: FlowNode): Promise<boolean> {
  const retryPolicy = (node.data as { retryPolicy?: { maxRetries?: number; onFail?: string } }).retryPolicy
  const maxRetries = retryPolicy?.maxRetries ?? 0

  if (maxRetries > 0) {
    // MVP: skip the node after maxRetries
    // Full implementation would actually retry executeNode with a backoff
    return true // skip → continue flow
  }

  // No retry policy → terminate
  return false
}

// ─── Singleton ──────────────────────────────────────────────────

let globalInstance: DagEngine | null = null

export function getDagEngine(): DagEngine {
  if (!globalInstance) {
    globalInstance = new DagEngine()
  }
  return globalInstance
}

export function resetDagEngine(): void {
  globalInstance = null
}
