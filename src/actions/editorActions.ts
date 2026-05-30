// ═══════════════════════════════════════════════════════════════
// editorActions — 编辑器相关动作注册
// ═══════════════════════════════════════════════════════════════

import { getActionRegistry, type ActionRuntime } from '../orchestration/ActionRegistry'

export function registerEditorActions(runtime: ActionRuntime): void {
  const registry = getActionRegistry()

  registry.register('editor.openFile', async (params) => {
    const filePath = String(params.filePath ?? '')
    if (filePath) {
      runtime.openFile(filePath)
    }
  })

  registry.register('editor.find', async () => {
    runtime.toast.warning('查找功能开发中')
  })

  registry.register('editor.replace', async () => {
    runtime.toast.warning('替换功能开发中')
  })

  registry.register('editor.goTo', async () => {
    runtime.toast.warning('定位功能开发中')
  })

  registry.register('editor.cut', async () => {
    runtime.toast.warning('剪切功能开发中')
  })

  registry.register('editor.copy', async () => {
    runtime.toast.warning('复制功能开发中')
  })

  registry.register('editor.format', async () => {
    runtime.toast.warning('格式化功能开发中')
  })
}
