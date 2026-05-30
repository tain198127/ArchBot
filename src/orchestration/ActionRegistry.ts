// ═══════════════════════════════════════════════════════════════
// ActionRegistry — 动作注册与执行
// register(id, handler) → execute(id, params, runtime)
// ═══════════════════════════════════════════════════════════════

export interface ActionRuntime {
  /** Tauri invoke (desktop) or HTTP fetch (browser) */
  invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown>
  /** Open a file in the editor */
  openFile: (path: string) => void
  /** Show a toast notification */
  toast: {
    success: (msg: string) => void
    error: (msg: string) => void
    warning: (msg: string) => void
  }
  /** Push a log entry to the bottom panel */
  pushLog: (level: 'info' | 'warn' | 'error', source: string, msg: string) => void
  /** Confirm dialog */
  confirm: (msg: string) => Promise<boolean>
}

export type ActionHandler = (
  params: Record<string, unknown>,
  runtime: ActionRuntime,
) => Promise<unknown>

export class ActionRegistry {
  private handlers = new Map<string, ActionHandler>()

  register(actionId: string, handler: ActionHandler): void {
    if (this.handlers.has(actionId)) {
      throw new Error(`[ActionRegistry] Duplicate action id: ${actionId}`)
    }
    this.handlers.set(actionId, handler)
  }

  has(actionId: string): boolean {
    return this.handlers.has(actionId)
  }

  async execute(
    actionId: string,
    params: Record<string, unknown>,
    runtime: ActionRuntime,
  ): Promise<unknown> {
    const handler = this.handlers.get(actionId)
    if (!handler) {
      throw new Error(`[ActionRegistry] Unknown action: ${actionId}`)
    }
    return handler(params, runtime)
  }

  /** List all registered action ids (for debugging) */
  listActions(): string[] {
    return Array.from(this.handlers.keys())
  }
}

/** Global singleton */
let globalRegistry: ActionRegistry | null = null

export function getActionRegistry(): ActionRegistry {
  if (!globalRegistry) {
    globalRegistry = new ActionRegistry()
  }
  return globalRegistry
}

export function resetActionRegistry(): void {
  globalRegistry = null
}
