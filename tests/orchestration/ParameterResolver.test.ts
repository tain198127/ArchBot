// ═══════════════════════════════════════════════════════════════
// ParameterResolver 单元测试
// ═══════════════════════════════════════════════════════════════

import { describe, it, expect } from 'vitest'
import { resolveParams } from '../../src/orchestration/ParameterResolver'
import { createRuntimeState, type ContextObject } from '../../src/orchestration/RuntimeContext'

describe('ParameterResolver', () => {
  const state = createRuntimeState({ name: 'myproject', path: '/home/proj.ab', content: '' })
  const context: ContextObject = { resourceType: 'file', resource: { name: 'test.java', path: '/src/test.java' } }
  const steps = { controllerResult: { endpoints: 5 }, serviceResult: { services: 3 } }

  it('returns empty object for undefined params', () => {
    expect(resolveParams(undefined, state)).toEqual({})
  })

  it('passes through non-string values unchanged', () => {
    expect(resolveParams({ num: 42 }, state)).toEqual({ num: 42 })
  })

  it('resolves ${context.resource.path}', () => {
    const result = resolveParams({ filePath: '${context.resource.path}' }, state, context)
    expect(result.filePath).toBe('/src/test.java')
  })

  it('resolves ${state.project.name}', () => {
    const result = resolveParams({ name: '${state.project.name}' }, state)
    expect(result.name).toBe('myproject')
  })

  it('resolves ${steps.xxx}', () => {
    const result = resolveParams({ data: '${steps.controllerResult}' }, state, undefined, steps)
    expect(result.data).toBe('[object Object]')
  })

  it('returns empty string for missing variable', () => {
    const result = resolveParams({ missing: '${context.resource.missing}' }, state, context)
    expect(result.missing).toBe('')
  })

  it('handles multiple replacements in one string', () => {
    const result = resolveParams({ msg: '${state.project.name} at ${context.resource.path}' }, state, context)
    expect(result.msg).toBe('myproject at /src/test.java')
  })
})
