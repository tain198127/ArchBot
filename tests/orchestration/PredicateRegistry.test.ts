// ═══════════════════════════════════════════════════════════════
// PredicateRegistry 单元测试
// ═══════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from 'vitest'
import { PredicateRegistry, resetPredicateRegistry } from '../../src/orchestration/PredicateRegistry'
import { createRuntimeState } from '../../src/orchestration/RuntimeContext'

describe('PredicateRegistry', () => {
  let registry: PredicateRegistry

  beforeEach(() => {
    resetPredicateRegistry()
    registry = new PredicateRegistry()
  })

  it('registers and evaluates a predicate', () => {
    registry.register('isLoaded', (state) => state.project.loaded)
    expect(registry.evaluate('isLoaded', createRuntimeState({ name: 'x', path: '/x.ab', content: '' }))).toBe(true)
    expect(registry.evaluate('isLoaded', createRuntimeState(null))).toBe(false)
  })

  it('throws on duplicate predicate', () => {
    registry.register('dup', () => true)
    expect(() => registry.register('dup', () => false)).toThrow('Duplicate')
  })

  it('throws on unknown predicate', () => {
    expect(() => registry.evaluate('unknown', createRuntimeState(null))).toThrow('Unknown predicate')
  })

  it('has() returns correct boolean', () => {
    registry.register('foo', () => true)
    expect(registry.has('foo')).toBe(true)
    expect(registry.has('bar')).toBe(false)
  })

  it('receives context object', () => {
    registry.register('isController', (_state, ctx) => ctx?.semanticType === 'controller')
    expect(registry.evaluate('isController', createRuntimeState(null), {
      resourceType: 'file', resource: { name: 'test.java' }, semanticType: 'controller',
    })).toBe(true)
  })
})
