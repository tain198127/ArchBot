// ═══════════════════════════════════════════════════════════════
// ConfigValidator — Zod Schema 校验 4 份 YML 配置
// ═══════════════════════════════════════════════════════════════

import { z } from 'zod'

// ── Menu Item ──
const menuItemSchema = z.object({
  id: z.string().min(1),
  label: z.string().min(1),
  shortcut: z.string().optional(),
  action: z.string().optional(),
  type: z.enum(['submenu']).optional(),
  provider: z.string().optional(),
  enabledWhen: z.string().optional(),
  visibleWhen: z.string().optional(),
})

// ── Menu Group ──
const menuGroupSchema = z.object({
  items: z.array(menuItemSchema),
})

// ── Menu Category ──
const menuCategorySchema = z.object({
  id: z.string().min(1),
  label: z.string().min(1),
  disabledWhen: z.string().optional(),
  groups: z.array(menuGroupSchema),
  note: z.string().optional(),
})

// ── Menus Config ──
export const menusConfigSchema = z.object({
  version: z.string(),
  menus: z.array(menuCategorySchema),
})

// ── Context Menu Item ──
const contextMenuItemSchema = z.object({
  id: z.string().optional(),
  label: z.string().optional(),
  action: z.string().optional(),
  flow: z.string().optional(),
  type: z.enum(['separator']).optional(),
  params: z.record(z.string(), z.unknown()).optional(),
  predicate: z.string().optional(),
  visibleWhen: z.string().optional(),
  enabledWhen: z.string().optional(),
})

// ── Context Menu ──
const contextMenuSchema = z.object({
  id: z.string().min(1),
  appliesTo: z.object({
    resourceType: z.string().min(1),
    fileExtension: z.string().optional(),
    semanticType: z.string().optional(),
  }),
  items: z.array(contextMenuItemSchema),
})

export const contextMenusConfigSchema = z.object({
  version: z.string(),
  contextMenus: z.array(contextMenuSchema),
})

// ── Action ──
const actionSchema = z.object({
  id: z.string().min(1),
  label: z.string(),
  executor: z.enum(['frontend', 'backend']),
  command: z.string().optional(),
  handler: z.string().optional(),
  input: z.record(z.string(), z.string()).optional(),
  result: z.object({
    storeAs: z.string().optional(),
    notify: z.boolean().optional(),
  }).optional(),
  confirm: z.string().optional(),
})

export const actionsConfigSchema = z.object({
  version: z.string(),
  actions: z.array(actionSchema),
})

// ── Flow Step ──
const flowStepSchema = z.object({
  id: z.string().min(1),
  action: z.string().min(1),
  params: z.record(z.string(), z.string()).optional(),
  outputAs: z.string().optional(),
})

// ── Flow ──
const flowSchema = z.object({
  id: z.string().min(1),
  label: z.string(),
  description: z.string().optional(),
  onError: z.enum(['abort']).default('abort'),
  input: z.record(z.string(), z.string()).optional(),
  steps: z.array(flowStepSchema),
})

export const flowsConfigSchema = z.object({
  version: z.string(),
  flows: z.array(flowSchema),
})

// ── Type exports ──
export type MenusConfig = z.infer<typeof menusConfigSchema>
export type ContextMenusConfig = z.infer<typeof contextMenusConfigSchema>
export type ActionsConfig = z.infer<typeof actionsConfigSchema>
export type FlowsConfig = z.infer<typeof flowsConfigSchema>

export type { ContextMenuItem } from './ContextMenuResolver'
