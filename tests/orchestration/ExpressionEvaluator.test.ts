// ═══════════════════════════════════════════════════════════════
// ExpressionEvaluator 单元测试
// ═══════════════════════════════════════════════════════════════

import { describe, it, expect } from 'vitest'
import { evaluateExpression } from '../../src/orchestration/ExpressionEvaluator'
import { createRuntimeState, type ContextObject } from '../../src/orchestration/RuntimeContext'

describe('ExpressionEvaluator', () => {
  const loadedState = createRuntimeState({ name: 'test', path: '/test.ab', content: '' })
  const emptyState = createRuntimeState(null)

  it('returns true for empty expression', () => {
    expect(evaluateExpression(undefined, loadedState)).toBe(true)
    expect(evaluateExpression('', loadedState)).toBe(true)
  })

  it('evaluates project.loaded == true', () => {
    expect(evaluateExpression('project.loaded == true', loadedState)).toBe(true)
    expect(evaluateExpression('project.loaded == true', emptyState)).toBe(false)
  })

  it('evaluates project.loaded == false', () => {
    expect(evaluateExpression('project.loaded == false', loadedState)).toBe(false)
    expect(evaluateExpression('project.loaded == false', emptyState)).toBe(true)
  })

  it('evaluates logical AND', () => {
    expect(evaluateExpression('project.loaded == true && project.loaded == true', loadedState)).toBe(true)
    expect(evaluateExpression('project.loaded == true && project.loaded == false', loadedState)).toBe(false)
  })

  it('evaluates logical OR', () => {
    expect(evaluateExpression('project.loaded == true || project.loaded == false', loadedState)).toBe(true)
    expect(evaluateExpression('project.loaded == false || project.loaded == false', loadedState)).toBe(false)
  })

  it('evaluates logical NOT', () => {
    // ! binds tighter than ==, so this is (!project.loaded) == false
    expect(evaluateExpression('!project.loaded == false', loadedState)).toBe(true)
    // Use parentheses for intuitive grouping: NOT (loaded == true) = loaded != true
    expect(evaluateExpression('!(project.loaded == true)', loadedState)).toBe(false)
  })

  it('evaluates parenthesized expressions', () => {
    expect(evaluateExpression('!(project.loaded == true)', loadedState)).toBe(false)
    expect(evaluateExpression('(project.loaded == true) && (project.loaded == true)', loadedState)).toBe(true)
  })

  it('evaluates context.resourceType', () => {
    const ctx: ContextObject = { resourceType: 'file', resource: { name: 'test.java' } }
    expect(evaluateExpression('context.resourceType == "file"', loadedState, ctx)).toBe(true)
    expect(evaluateExpression('context.resourceType == "directory"', loadedState, ctx)).toBe(false)
  })

  it('evaluates context.semanticType', () => {
    const ctx: ContextObject = { resourceType: 'file', resource: { name: 'test.java' }, semanticType: 'controller' }
    expect(evaluateExpression('context.semanticType == "controller"', loadedState, ctx)).toBe(true)
    expect(evaluateExpression('context.semanticType == "service"', loadedState, ctx)).toBe(false)
  })

  it('returns false for invalid syntax', () => {
    expect(evaluateExpression('invalid === syntax', loadedState)).toBe(false)
  })

  it('returns false for disallowed characters', () => {
    expect(evaluateExpression('project.loaded == true ; alert(1)', loadedState)).toBe(false)
  })
})
