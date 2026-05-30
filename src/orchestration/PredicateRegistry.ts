// ═══════════════════════════════════════════════════════════════
// PredicateRegistry — 复杂条件注册为可测试函数
// ═══════════════════════════════════════════════════════════════

import type { RuntimeState, ContextObject } from './RuntimeContext'

export type PredicateFn = (state: RuntimeState, context?: ContextObject) => boolean

export class PredicateRegistry {
  private predicates = new Map<string, PredicateFn>()

  register(name: string, fn: PredicateFn): void {
    if (this.predicates.has(name)) {
      throw new Error(`[PredicateRegistry] Duplicate predicate: ${name}`)
    }
    this.predicates.set(name, fn)
  }

  evaluate(name: string, state: RuntimeState, context?: ContextObject): boolean {
    const fn = this.predicates.get(name)
    if (!fn) {
      throw new Error(`[PredicateRegistry] Unknown predicate: ${name}`)
    }
    return fn(state, context)
  }

  has(name: string): boolean {
    return this.predicates.has(name)
  }
}

/** Global singleton */
let globalPredicateRegistry: PredicateRegistry | null = null

export function getPredicateRegistry(): PredicateRegistry {
  if (!globalPredicateRegistry) {
    globalPredicateRegistry = new PredicateRegistry()
  }
  return globalPredicateRegistry
}

export function resetPredicateRegistry(): void {
  globalPredicateRegistry = null
}
