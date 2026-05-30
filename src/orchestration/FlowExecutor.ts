// ═══════════════════════════════════════════════════════════════
// FlowExecutor — 串行执行多步骤 Flow，步骤间通过 outputAs 传递数据
// onError: abort — 失败即停止，不回滚前序步骤
// ═══════════════════════════════════════════════════════════════

import { getConfig } from './ConfigLoader'
import { getActionRegistry, type ActionRuntime } from './ActionRegistry'
import { resolveParams } from './ParameterResolver'
import type { RuntimeState, ContextObject } from './RuntimeContext'

export interface FlowResult {
  flowId: string
  outputs: Record<string, unknown>
  completed: boolean
  failedStep?: string
  error?: string
}

export async function executeFlow(
  flowId: string,
  state: RuntimeState,
  runtime: ActionRuntime,
  context?: ContextObject,
): Promise<FlowResult> {
  const config = getConfig().flows
  const flow = config.flows.find(f => f.id === flowId)

  if (!flow) {
    throw new Error(`[FlowExecutor] Unknown flow: ${flowId}`)
  }

  const steps: Record<string, unknown> = {}
  const registry = getActionRegistry()

  for (const step of flow.steps) {
    if (!registry.has(step.action)) {
      return {
        flowId, outputs: steps, completed: false,
        failedStep: step.id,
        error: `Unknown action: ${step.action}`,
      }
    }

    try {
      const resolved = resolveParams(
        step.params as Record<string, unknown> ?? {},
        state,
        context,
        steps,
      )
      const result = await registry.execute(step.action, resolved, runtime)
      if (step.outputAs) {
        steps[step.outputAs] = result
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      return {
        flowId, outputs: steps, completed: false,
        failedStep: step.id,
        error: msg,
      }
    }
  }

  return { flowId, outputs: steps, completed: true }
}
