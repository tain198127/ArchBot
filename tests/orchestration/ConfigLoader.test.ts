// ═══════════════════════════════════════════════════════════════
// ConfigLoader 集成测试 — YML 加载与 Schema 校验
// ═══════════════════════════════════════════════════════════════

import { describe, it, expect } from 'vitest'
import { readFileSync } from 'fs'
import { resolve } from 'path'
import yaml from 'js-yaml'

import {
  menusConfigSchema,
  contextMenusConfigSchema,
  actionsConfigSchema,
  flowsConfigSchema,
} from '../../src/orchestration/ConfigValidator'

const configDir = resolve(__dirname, '../../src/config')

function loadYml(filename: string): unknown {
  const raw = readFileSync(resolve(configDir, filename), 'utf-8')
  return yaml.load(raw)
}

describe('ConfigLoader — YML Schema Validation', () => {
  it('menus.yml passes schema validation', () => {
    const data = loadYml('menus.yml')
    const parsed = menusConfigSchema.parse(data)
    expect(parsed.menus.length).toBeGreaterThanOrEqual(5)
    // Verify all menu items have required fields
    for (const cat of parsed.menus) {
      expect(cat.id).toBeTruthy()
      expect(cat.label).toBeTruthy()
      for (const grp of cat.groups) {
        for (const item of grp.items) {
          expect(item.id).toBeTruthy()
          expect(item.label).toBeTruthy()
        }
      }
    }
  })

  it('menus.yml i18n labels follow key format', () => {
    const data = loadYml('menus.yml')
    const parsed = menusConfigSchema.parse(data)
    for (const cat of parsed.menus) {
      expect(cat.label).toMatch(/^[a-zA-Z][a-zA-Z0-9_.]*$/)
      for (const grp of cat.groups) {
        for (const item of grp.items) {
          expect(item.label).toMatch(/^[a-zA-Z][a-zA-Z0-9_.]*$/)
        }
      }
    }
  })

  it('context-menus.yml passes schema validation', () => {
    const data = loadYml('context-menus.yml')
    const parsed = contextMenusConfigSchema.parse(data)
    expect(parsed.contextMenus.length).toBeGreaterThanOrEqual(5)
    for (const ctx of parsed.contextMenus) {
      expect(ctx.appliesTo.resourceType).toBeTruthy()
      expect(ctx.items.length).toBeGreaterThan(0)
    }
  })

  it('actions.yml passes schema validation', () => {
    const data = loadYml('actions.yml')
    const parsed = actionsConfigSchema.parse(data)
    expect(parsed.actions.length).toBeGreaterThanOrEqual(30)
    for (const action of parsed.actions) {
      expect(action.id).toBeTruthy()
      expect(['frontend', 'backend']).toContain(action.executor)
    }
  })

  it('actions.yml — every action id is unique', () => {
    const data = loadYml('actions.yml')
    const parsed = actionsConfigSchema.parse(data)
    const ids = parsed.actions.map(a => a.id)
    const dupes = ids.filter((id, i) => ids.indexOf(id) !== i)
    expect(dupes).toEqual([])
  })

  it('flows.yml passes schema validation', () => {
    const data = loadYml('flows.yml')
    const parsed = flowsConfigSchema.parse(data)
    for (const flow of parsed.flows) {
      expect(flow.id).toBeTruthy()
      expect(flow.steps.length).toBeGreaterThan(0)
      expect(['abort']).toContain(flow.onError)
    }
  })

  it('actions referenced in menus.yml exist in actions.yml', () => {
    const menus = menusConfigSchema.parse(loadYml('menus.yml'))
    const actions = actionsConfigSchema.parse(loadYml('actions.yml'))
    const actionIds = new Set(actions.actions.map(a => a.id))

    for (const cat of menus.menus) {
      for (const grp of cat.groups) {
        for (const item of grp.items) {
          if (item.action && item.type !== 'submenu') {
            expect(actionIds.has(item.action)).toBe(true)
          }
        }
      }
    }
  })

  it('actions referenced in context-menus.yml exist in actions.yml', () => {
    const ctxMenus = contextMenusConfigSchema.parse(loadYml('context-menus.yml'))
    const actions = actionsConfigSchema.parse(loadYml('actions.yml'))
    const actionIds = new Set(actions.actions.map(a => a.id))

    for (const ctx of ctxMenus.contextMenus) {
      for (const item of ctx.items) {
        if (item.action) {
          expect(actionIds.has(item.action)).toBe(true)
        }
      }
    }
  })
})
