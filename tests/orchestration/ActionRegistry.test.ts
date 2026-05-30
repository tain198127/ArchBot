// ═══════════════════════════════════════════════════════════════
// ActionRegistry 单元测试
// ═══════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from 'vitest'
import { ActionRegistry, resetActionRegistry, type ActionRuntime } from '../../src/orchestration/ActionRegistry'

function stubRuntime(): ActionRuntime {
  return {
    invoke: async () => ({}),
    openFile: () => {},
    toast: { success: () => {}, error: () => {}, warning: () => {} },
    pushLog: () => {},
    confirm: async () => false,
  }
}

describe('ActionRegistry', () => {
  let registry: ActionRegistry

  beforeEach(() => {
    resetActionRegistry()
    registry = new ActionRegistry()
  })

  it('registers and executes an action', async () => {
    registry.register('test.ping', async (params) => {
      return { pong: params.x }
    })
    const result = await registry.execute('test.ping', { x: 1 }, stubRuntime())
    expect(result).toEqual({ pong: 1 })
  })

  it('throws on duplicate registration', () => {
    registry.register('test.dup', async () => {})
    expect(() => registry.register('test.dup', async () => {})).toThrow('Duplicate')
  })

  it('throws on unknown action', async () => {
    await expect(() => registry.execute('test.unknown', {}, stubRuntime())).rejects.toThrow('Unknown action')
  })

  it('has() returns correct boolean', () => {
    registry.register('test.foo', async () => {})
    expect(registry.has('test.foo')).toBe(true)
    expect(registry.has('test.bar')).toBe(false)
  })

  it('listActions returns all registered ids', () => {
    registry.register('a.one', async () => {})
    registry.register('b.two', async () => {})
    expect(registry.listActions()).toEqual(['a.one', 'b.two'])
  })
})
