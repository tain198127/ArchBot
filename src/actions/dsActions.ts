// ═══════════════════════════════════════════════════════════════
// dsActions — 数据标准相关动作注册
// ═══════════════════════════════════════════════════════════════

import { getActionRegistry, type ActionRuntime } from '../orchestration/ActionRegistry'

export function registerDsActions(runtime: ActionRuntime): void {
  const registry = getActionRegistry()

  // UI actions (ds.newDomain, ds.newEntity, etc.) are registered in
  // FileTreePanel.vue's onMounted — they need component ref access.
  // Only flow-support actions registered here:

  registry.register('ds.validateFormat', async (params) => {
    const data = params.data
    return { valid: true, data }
  })

  registry.register('ds.parseEntities', async (params) => {
    return { entities: [], domain: params.domain }
  })

  registry.register('ds.saveEntities', async (params) => {
    runtime.pushLog('info', 'ds', `Saving ${(params.entities as unknown[])?.length ?? 0} entities`)
  })
}
