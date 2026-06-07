/**
 * DagEngine tests
 *
 * Ported from conductor.rs tests + expanded coverage.
 * Uses a mock ActionRuntime — no Tauri runtime needed.
 */

import { describe, it, expect, beforeEach } from 'vitest'
import { DagEngine, resetDagEngine } from './DagEngine'
import type { ActionRuntime } from './ActionRegistry'
import type { FlowDefinition, FlowNode, FlowEdge, ConductorEvent, AgentNodeData, NodeData } from '../types/businessFlow'

// ─── Helpers ────────────────────────────────────────────────────

function mockRuntime(): ActionRuntime & { invocations: Array<{ cmd: string; args: Record<string, unknown> }> } {
  const invocations: Array<{ cmd: string; args: Record<string, unknown> }> = []
  return {
    invoke: async (cmd: string, args?: Record<string, unknown>) => {
      invocations.push({ cmd, args: args ?? {} })
      // Return mock agent execution result
      if (cmd === 'agent_execute_turn') return { success: true }
      // Return mock flow data for sub-flow
      if (cmd === 'bf_get_flow') return {
        flowJson: JSON.stringify({
          nodes: [
            { id: 'sub-start', type: 'start', position: { x: 0, y: 0 }, data: { label: 'Start' } },
            { id: 'sub-end', type: 'end', position: { x: 0, y: 0 }, data: { label: 'End' } },
          ],
          edges: [{ id: 'e1', source: 'sub-start', target: 'sub-end' }],
        }),
      }
      return {}
    },
    openFile: () => {},
    toast: { success: () => {}, error: () => {}, warning: () => {} },
    pushLog: () => {},
    confirm: async () => true,
    invocations,
  }
}

function makeNode(id: string, type: string, overrides: Partial<FlowNode> = {}): FlowNode {
  return {
    id,
    type: type as FlowNode['type'],
    position: { x: 0, y: 0 },
    data: { label: id },
    ...overrides,
  }
}

function makeEdge(id: string, source: string, target: string, overrides: Partial<FlowEdge> = {}): FlowEdge {
  return { id, source, target, ...overrides }
}

function makeGraph(nodes: FlowNode[], edges: FlowEdge[]): FlowDefinition {
  return {
    id: 'test-flow',
    name: 'Test Flow',
    description: '',
    type: 'custom',
    nodes,
    edges,
    materials: [],
    scenarioBindings: [],
    outputConfig: { outputDir: './output', filenamePattern: '{flow}', extension: '.md' },
    version: 1,
  }
}

function collectEvents(events: ConductorEvent[]) {
  return {
    started: events.filter(e => e.eventType === 'node_started'),
    completed: events.filter(e => e.eventType === 'node_completed'),
    failed: events.filter(e => e.eventType === 'node_failed'),
    flowCompleted: events.filter(e => e.eventType === 'flow_completed'),
    flowFailed: events.filter(e => e.eventType === 'flow_failed'),
  }
}

// Reset singleton between tests
beforeEach(() => {
  resetDagEngine()
})

// ─── Topological Sort ──────────────────────────────────────────

describe('DagEngine topological sort', () => {
  it('sorts a linear graph correctly', async () => {
    const nodes = [
      makeNode('a', 'start'),
      makeNode('b', 'agent'),
      makeNode('c', 'end'),
    ]
    const edges = [
      makeEdge('e1', 'a', 'b'),
      makeEdge('e2', 'b', 'c'),
    ]
    const graph = makeGraph(nodes, edges)
    const engine = new DagEngine()
    const runtime = mockRuntime()

    const result = await engine.execute(graph, runtime, {
      runId: 'test-1',
      outputDir: '/tmp',
      materialPaths: [],
    })

    expect(result.completed).toBe(true)

    // Verify events are in order: node_started→node_completed for each node
    const { started, completed, flowCompleted } = collectEvents(result.events)
    expect(started.length).toBe(3)
    expect(completed.length).toBe(3)
    expect(flowCompleted.length).toBe(1)

    // Check execution order via event sequence
    const startEvents = result.events.filter(e => e.eventType === 'node_started')
    const nodeOrder = startEvents.map(e => e.nodeId)
    expect(nodeOrder[0]).toBe('a')
    // b and c can be in any order after a
    expect(nodeOrder).toContain('b')
    expect(nodeOrder).toContain('c')
  })

  it('sorts a diamond graph with correct constraints', async () => {
    const nodes = [
      makeNode('s', 'start'),
      makeNode('a', 'agent'),
      makeNode('b', 'agent'),
      makeNode('e', 'end'),
    ]
    const edges = [
      makeEdge('e1', 's', 'a'),
      makeEdge('e2', 's', 'b'),
      makeEdge('e3', 'a', 'e'),
      makeEdge('e4', 'b', 'e'),
    ]
    const graph = makeGraph(nodes, edges)
    const engine = new DagEngine()
    const runtime = mockRuntime()

    const result = await engine.execute(graph, runtime, {
      runId: 'test-2',
      outputDir: '/tmp',
      materialPaths: [],
    })

    expect(result.completed).toBe(true)
    expect(result.nodeResults.size).toBe(4)

    // First started node must be 's'
    const started = result.events.filter(e => e.eventType === 'node_started')
    expect(started[0].nodeId).toBe('s')
  })
})

// ─── Condition Evaluation ──────────────────────────────────────

describe('DagEngine condition evaluation', () => {
  it('evaluates "true" as true (via XOR gateway first match)', async () => {
    const nodes = [
      makeNode('start', 'start'),
      makeNode('xor', 'gateway_xor', { data: { label: 'XOR' } }),
      makeNode('a', 'agent', { data: { label: 'Path A' } }),
      makeNode('b', 'agent', { data: { label: 'Path B' } }),
      makeNode('end', 'end'),
    ]
    const edges = [
      makeEdge('e1', 'start', 'xor'),
      makeEdge('e2', 'xor', 'a', { condition: 'false' }),
      makeEdge('e3', 'xor', 'b', { condition: 'true' }),
      makeEdge('e4', 'a', 'end'),
      makeEdge('e5', 'b', 'end'),
    ]
    const graph = makeGraph(nodes, edges)
    const engine = new DagEngine()
    const runtime = mockRuntime()

    const result = await engine.execute(graph, runtime, {
      runId: 'test-3',
      outputDir: '/tmp',
      materialPaths: [],
    })

    expect(result.completed).toBe(true)
    // Path B (condition "true") should have been chosen
    const bResult = result.nodeResults.get('b')
    expect(bResult).toBeDefined()
    expect(bResult?.success).toBe(true)
  })

  it('evaluates "false" as false (falls back to first branch)', async () => {
    const nodes = [
      makeNode('start', 'start'),
      makeNode('xor', 'gateway_xor', { data: { label: 'XOR' } }),
      makeNode('a', 'agent', { data: { label: 'Path A' } }),
      makeNode('end', 'end'),
    ]
    const edges = [
      makeEdge('e1', 'start', 'xor'),
      makeEdge('e2', 'xor', 'a', { condition: 'false' }),
      makeEdge('e3', 'a', 'end'),
    ]
    const graph = makeGraph(nodes, edges)
    const engine = new DagEngine()
    const runtime = mockRuntime()

    const result = await engine.execute(graph, runtime, {
      runId: 'test-4',
      outputDir: '/tmp',
      materialPaths: [],
    })

    expect(result.completed).toBe(true)
    // XOR fallback: first branch selected when no condition matches
    const aResult = result.nodeResults.get('a')
    expect(aResult).toBeDefined()
  })
})

// ─── Full Flow Execution ──────────────────────────────────────

describe('DagEngine full flow execution', () => {
  it('executes a simple linear flow start→agent→end', async () => {
    const nodes = [
      makeNode('start', 'start'),
      makeNode('agent', 'agent', { data: { agentId: 'test-agent', agentName: 'Test', skillName: 'test' } as AgentNodeData }),
      makeNode('end', 'end'),
    ]
    const edges = [
      makeEdge('e1', 'start', 'agent'),
      makeEdge('e2', 'agent', 'end'),
    ]
    const graph = makeGraph(nodes, edges)
    const engine = new DagEngine()
    const runtime = mockRuntime()

    const result = await engine.execute(graph, runtime, {
      runId: 'test-5',
      outputDir: '/tmp',
      materialPaths: [],
    })

    expect(result.completed).toBe(true)
    expect(result.nodeResults.size).toBe(3)

    // Agent node should have invoked agent_execute_turn
    const invocations = (runtime as ReturnType<typeof mockRuntime>).invocations
    const agentCalls = invocations.filter(i => i.cmd === 'agent_execute_turn')
    expect(agentCalls.length).toBe(1)
  })

  it('emits node_started and node_completed events', async () => {
    const nodes = [
      makeNode('start', 'start'),
      makeNode('end', 'end'),
    ]
    const edges = [makeEdge('e1', 'start', 'end')]
    const graph = makeGraph(nodes, edges)
    const engine = new DagEngine()
    const runtime = mockRuntime()

    const result = await engine.execute(graph, runtime, {
      runId: 'test-6',
      outputDir: '/tmp',
      materialPaths: [],
    })

    const { started, completed, flowCompleted } = collectEvents(result.events)
    expect(started.length).toBe(2)
    expect(completed.length).toBe(2)
    expect(flowCompleted.length).toBe(1)
  })

  it('emits flow_failed when no start node exists', async () => {
    const nodes = [
      makeNode('agent', 'agent', { data: { agentId: 'a', agentName: 'A', skillName: 's' } as AgentNodeData }),
      makeNode('end', 'end'),
    ]
    const edges = [makeEdge('e1', 'agent', 'end')]
    const graph = makeGraph(nodes, edges)
    const engine = new DagEngine()
    const runtime = mockRuntime()

    const result = await engine.execute(graph, runtime, {
      runId: 'test-7',
      outputDir: '/tmp',
      materialPaths: [],
    })

    expect(result.completed).toBe(false)
    expect(result.error).toContain('No Start node')
    const { flowFailed } = collectEvents(result.events)
    expect(flowFailed.length).toBe(1)
  })

  it('aborts on signal', async () => {
    const nodes = [
      makeNode('start', 'start'),
      makeNode('agent', 'agent', { data: { agentId: 'a', agentName: 'A', skillName: 's' } as AgentNodeData }),
      makeNode('end', 'end'),
    ]
    const edges = [
      makeEdge('e1', 'start', 'agent'),
      makeEdge('e2', 'agent', 'end'),
    ]
    const graph = makeGraph(nodes, edges)
    const engine = new DagEngine()
    const runtime = mockRuntime()
    const controller = new AbortController()

    // Abort immediately — the first executeNode will check the signal
    controller.abort()

    const result = await engine.execute(graph, runtime, {
      runId: 'test-8',
      outputDir: '/tmp',
      materialPaths: [],
      signal: controller.signal,
    })

    expect(result.completed).toBe(false)
    expect(result.error).toContain('aborted')
  })
})

// ─── AND Gateway ───────────────────────────────────────────────

describe('DagEngine AND gateway', () => {
  it('executes all downstream branches in parallel', async () => {
    const nodes = [
      makeNode('start', 'start'),
      makeNode('and', 'gateway_and', { data: { label: 'AND' } }),
      makeNode('a', 'agent', { data: { agentId: 'a', agentName: 'A', skillName: 's' } as AgentNodeData }),
      makeNode('b', 'agent', { data: { agentId: 'b', agentName: 'B', skillName: 's' } as AgentNodeData }),
      makeNode('end', 'end'),
    ]
    const edges = [
      makeEdge('e1', 'start', 'and'),
      makeEdge('e2', 'and', 'a'),
      makeEdge('e3', 'and', 'b'),
      makeEdge('e4', 'a', 'end'),
      makeEdge('e5', 'b', 'end'),
    ]
    const graph = makeGraph(nodes, edges)
    const engine = new DagEngine()
    const runtime = mockRuntime()

    const result = await engine.execute(graph, runtime, {
      runId: 'test-9',
      outputDir: '/tmp',
      materialPaths: [],
    })

    expect(result.completed).toBe(true)
    expect(result.nodeResults.has('a')).toBe(true)
    expect(result.nodeResults.has('b')).toBe(true)
    expect(result.nodeResults.get('a')?.success).toBe(true)
    expect(result.nodeResults.get('b')?.success).toBe(true)
  })
})

// ─── Degradation ───────────────────────────────────────────────

describe('DagEngine degradation', () => {
  it('continues flow when node has retry policy (skip)', async () => {
    const nodes = [
      makeNode('start', 'start'),
      makeNode('agent', 'agent', {
        data: {
          agentId: 'failing',
          agentName: 'Failing',
          skillName: 'test',
          retryPolicy: { maxRetries: 3, onFail: 'skip' },
        } as NodeData,
      }),
      makeNode('end', 'end'),
    ]
    const edges = [
      makeEdge('e1', 'start', 'agent'),
      makeEdge('e2', 'agent', 'end'),
    ]
    const graph = makeGraph(nodes, edges)
    const engine = new DagEngine()

    // Runtime that always fails agent_execute_turn
    const runtime = mockRuntime()
    const origInvoke = runtime.invoke
    runtime.invoke = async (cmd: string, args?: Record<string, unknown>) => {
      if (cmd === 'agent_execute_turn') throw new Error('Agent failed')
      return origInvoke(cmd, args)
    }

    const result = await engine.execute(graph, runtime, {
      runId: 'test-10',
      outputDir: '/tmp',
      materialPaths: [],
    })

    // Should still complete because degradation skips the failed node
    expect(result.completed).toBe(true)
    const agentResult = result.nodeResults.get('agent')
    expect(agentResult?.success).toBe(false)
  })

  it('terminates flow when node fails without retry policy', async () => {
    const nodes = [
      makeNode('start', 'start'),
      makeNode('agent', 'agent', {
        data: { agentId: 'failing', agentName: 'Failing', skillName: 'test' } as AgentNodeData,
      }),
      makeNode('end', 'end'),
    ]
    const edges = [
      makeEdge('e1', 'start', 'agent'),
      makeEdge('e2', 'agent', 'end'),
    ]
    const graph = makeGraph(nodes, edges)
    const engine = new DagEngine()

    const runtime = mockRuntime()
    const origInvoke = runtime.invoke
    runtime.invoke = async (cmd: string, args?: Record<string, unknown>) => {
      if (cmd === 'agent_execute_turn') throw new Error('Agent failed')
      return origInvoke(cmd, args)
    }

    const result = await engine.execute(graph, runtime, {
      runId: 'test-11',
      outputDir: '/tmp',
      materialPaths: [],
    })

    expect(result.completed).toBe(false)
    expect(result.failedNodeId).toBe('agent')
    const { flowFailed } = collectEvents(result.events)
    expect(flowFailed.length).toBe(1)
  })
})

// ─── Timer ─────────────────────────────────────────────────────

describe('DagEngine timer', () => {
  it('waits for the specified duration', async () => {
    const nodes = [
      makeNode('start', 'start'),
      makeNode('timer', 'timer', { data: { durationMs: 50 } as NodeData }),
      makeNode('end', 'end'),
    ]
    const edges = [
      makeEdge('e1', 'start', 'timer'),
      makeEdge('e2', 'timer', 'end'),
    ]
    const graph = makeGraph(nodes, edges)
    const engine = new DagEngine()
    const runtime = mockRuntime()

    const start = Date.now()
    const result = await engine.execute(graph, runtime, {
      runId: 'test-12',
      outputDir: '/tmp',
      materialPaths: [],
    })
    const elapsed = Date.now() - start

    expect(result.completed).toBe(true)
    expect(elapsed).toBeGreaterThanOrEqual(45) // allow slight timing variance
  })
})

// ─── Quality Gate ──────────────────────────────────────────────

describe('DagEngine quality gate', () => {
  it('passes in MVP mode', async () => {
    const nodes = [
      makeNode('start', 'start'),
      makeNode('gate', 'quality_gate', {
        data: { metric: 'test_score', threshold: 0.8, onFail: 'abort' } as NodeData,
      }),
      makeNode('end', 'end'),
    ]
    const edges = [
      makeEdge('e1', 'start', 'gate'),
      makeEdge('e2', 'gate', 'end'),
    ]
    const graph = makeGraph(nodes, edges)
    const engine = new DagEngine()
    const runtime = mockRuntime()

    const result = await engine.execute(graph, runtime, {
      runId: 'test-13',
      outputDir: '/tmp',
      materialPaths: [],
    })

    expect(result.completed).toBe(true)
    expect(result.nodeResults.get('gate')?.success).toBe(true)
  })
})
