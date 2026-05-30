// ═══════════════════════════════════════════════════════════════
// miscActions — 杂项动作注册 (config / settings / license / run / cat / grp / itm / fs)
// ═══════════════════════════════════════════════════════════════

import { getActionRegistry, type ActionRuntime } from '../orchestration/ActionRegistry'

export function registerMiscActions(runtime: ActionRuntime): void {
  const registry = getActionRegistry()
  const stub = (label: string) => async () => { runtime.toast.warning(`${label}功能开发中`) }

  // ── Config ──
  // Config actions — handled by EditorPanel via useMenuAction bus
  // Stubs only for ActionRegistry (EditorPanel handles actual tab opening)

  // ── Settings ──
  registry.register('settings.clearCache', async () => {
    window.location.reload()
  })

  // ── License (registered in App.vue — needs dialog ref) ──

  // ── Run ──
  registry.register('run.terminal', stub('命令行'))
  registry.register('run.genRequirement', stub('生成需规'))
  registry.register('run.genDesign', stub('生成设计'))
  registry.register('run.genCode', stub('生成代码'))
  registry.register('run.genSkeleton', stub('生成代码骨架'))
  registry.register('run.genDbTable', stub('生成数据库表'))
  registry.register('run.genDataStandard', stub('生成数据标准'))
  registry.register('run.genCallChain', stub('生成调用链'))
  registry.register('run.genTestCase', stub('生成测试用例'))
  registry.register('run.genE2eTest', stub('生成E2E测试'))

  // ── Category ──
  registry.register('cat.brainstorm', stub('脑暴'))
  registry.register('cat.generateSRS', stub('生成SRS'))
  registry.register('cat.exportHTML', stub('导出HTML'))
  registry.register('cat.sealAll', stub('归档全部'))
  registry.register('cat.importProject', stub('导入项目'))
  registry.register('cat.exportPackage', stub('导出包'))

  // ── Group ──
  registry.register('grp.analyze', stub('分析'))
  registry.register('grp.review', stub('评审'))
  registry.register('grp.write', stub('写作'))
  registry.register('grp.seal', stub('归档'))
  registry.register('grp.import', stub('导入'))
  registry.register('grp.export', stub('导出'))

  // ── Item ──
  registry.register('itm.analyze', stub('分析'))
  registry.register('itm.review', stub('评审'))
  registry.register('itm.write', stub('写作'))
  registry.register('itm.seal', stub('归档'))
  registry.register('itm.import', stub('导入'))
  registry.register('itm.export', stub('导出'))

  // ── File System ──
  registry.register('fs.deleteFile', async (params) => {
    const filePath = String(params.filePath ?? '')
    runtime.pushLog('warn', 'fs', `File deletion not implemented: ${filePath}`)
    runtime.toast.warning('文件删除功能开发中')
  })

  // ── Flow actions ──
  registry.register('validation.ensureJavaController', async (params) => {
    return { valid: true, filePath: params.filePath }
  })
  registry.register('analysis.controller.single', stub('Controller分析'))
  registry.register('analysis.service.trace', stub('Service追踪'))
  registry.register('analysis.mapper.trace', stub('Mapper追踪'))
  registry.register('analysis.database.identifyTables', stub('表识别'))
  registry.register('workspace.saveAnalysisResult', async (params) => {
    runtime.pushLog('info', 'workspace', `Saving analysis result: ${params.type}`)
  })
}
