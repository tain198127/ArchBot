// ═══════════════════════════════════════════════════════════════
// ParameterResolver — 解析 ${context.xxx} / ${state.xxx} / ${steps.xxx}
// ═══════════════════════════════════════════════════════════════

import type { RuntimeState, ContextObject } from './RuntimeContext'

export function resolveParams(
  params: Record<string, unknown> | undefined,
  state: RuntimeState,
  context?: ContextObject,
  steps?: Record<string, unknown>,
): Record<string, unknown> {
  if (!params) return {}

  const resolved: Record<string, unknown> = {}
  for (const [key, value] of Object.entries(params)) {
    resolved[key] = resolveValue(value, state, context, steps)
  }
  return resolved
}

function resolveValue(
  value: unknown,
  state: RuntimeState,
  context?: ContextObject,
  steps?: Record<string, unknown>,
): unknown {
  if (typeof value !== 'string') return value

  // Replace ${...} patterns
  return value.replace(/\$\{([^}]+)\}/g, (_, path: string) => {
    return resolvePath(path.trim(), state, context, steps)
  })
}

function resolvePath(
  path: string,
  state: RuntimeState,
  context?: ContextObject,
  steps?: Record<string, unknown>,
): string {
  const parts = path.split('.')

  // context.xxx
  if (parts[0] === 'context' && context) {
    const ctx = context as unknown as Record<string, unknown>
    const res = ctx.resource as Record<string, unknown>
    if (parts[1] === 'resource' && parts[2] && res) {
      return String(res[parts[2]] ?? '')
    }
    return String(ctx[parts[1]] ?? '')
  }

  // state.xxx
  if (parts[0] === 'state') {
    const st = state as unknown as Record<string, unknown>
    const sub = st[parts[1]] as Record<string, unknown> | undefined
    if (sub && parts[2]) {
      return String(sub[parts[2]] ?? '')
    }
    return ''
  }

  // steps.xxx (flow step output)
  if (parts[0] === 'steps' && steps) {
    return String(steps[parts[1]] ?? '')
  }

  // input.xxx (flow input)
  if (parts[0] === 'input' && steps) {
    return String(steps[parts[1]] ?? '')
  }

  // params.xxx (direct param reference)
  if (parts[0] === 'params' && steps) {
    return String(steps[parts[1]] ?? '')
  }

  return ''
}
