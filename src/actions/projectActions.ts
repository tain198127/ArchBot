// ═══════════════════════════════════════════════════════════════
// projectActions — 项目相关动作注册
// ═══════════════════════════════════════════════════════════════

import { getActionRegistry, type ActionRuntime } from '../orchestration/ActionRegistry'
import { openFileDialog, browserOpenProjectFile } from '../api/filePicker'
import { isTauri } from '../api/env'
import { openProject as apiOpenProject, initArchbotDir, dbDisconnect } from '../api'
import { useProject } from '../stores/project'

export function registerProjectActions(runtime: ActionRuntime): void {
  const registry = getActionRegistry()
  const { closeProject: closeStoreProject } = useProject()

  registry.register('project.open', async () => {
    const result = await openProjectViaDialog(runtime)
    return result
  })

  // project.create is registered in App.vue (needs dialog ref)

  registry.register('project.close', async () => {
    try { await dbDisconnect() } catch { /* ignore */ }
    closeStoreProject()
    runtime.toast.success('项目已关闭')
  })

  registry.register('project.save', async () => {
    runtime.toast.warning('保存功能开发中')
  })

  registry.register('project.saveAs', async () => {
    runtime.toast.warning('另存功能开发中')
  })

  registry.register('project.openRemote', async () => {
    runtime.toast.warning('远程项目打开功能开发中')
  })
}

async function openProjectViaDialog(runtime: ActionRuntime) {
  const filter = [{ name: 'ArchBot 项目文件', extensions: ['ab'] }]

  if (isTauri) {
    const selected = await openFileDialog(filter).catch(() => null)
    if (!selected) return
    runtime.pushLog('info', 'app', `Opening project: ${selected} (mode: tauri)`)
    const result = await apiOpenProject(selected)
    const { setProject } = useProject()
    setProject({ name: result.name, path: selected, content: result.content })
    await initArchbotDir(selected).catch(() => {})
    runtime.toast.success('项目已打开')
  } else {
    const result = await browserOpenProjectFile(filter).catch(() => null)
    if (!result) return
    runtime.pushLog('info', 'app', `Opening project: ${result.path} (mode: browser)`)
    const { setProject } = useProject()
    setProject({ name: result.name, path: result.path, content: result.content })
    runtime.toast.success('项目已打开')
  }
}
