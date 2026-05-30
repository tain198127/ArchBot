// ═══════════════════════════════════════════════════════════════
// uiActions — UI 面板动作注册
// ═══════════════════════════════════════════════════════════════

import { getActionRegistry, type ActionRuntime } from '../orchestration/ActionRegistry'

export function registerUIActions(runtime: ActionRuntime): void {
  const registry = getActionRegistry()

  registry.register('ui.refreshPanel', async (params) => {
    const panelId = String(params.panelId ?? '')
    runtime.pushLog('info', 'ui', `Refreshing panel: ${panelId}`)
  })
}
