// ═══════════════════════════════════════════════════════════════
// actions/index.ts — 注册所有动作
// ═══════════════════════════════════════════════════════════════

import type { ActionRuntime } from '../orchestration/ActionRegistry'
import { registerProjectActions } from './projectActions'
import { registerEditorActions } from './editorActions'
import { registerUIActions } from './uiActions'
import { registerDsActions } from './dsActions'
import { registerMiscActions } from './miscActions'

/** Register all action handlers. Call once at app startup. */
export function registerAllActions(runtime: ActionRuntime): void {
  registerProjectActions(runtime)
  registerEditorActions(runtime)
  registerUIActions(runtime)
  registerDsActions(runtime)
  registerMiscActions(runtime)
}
