// ═══════════════════════════════════════════════════════════════
// RuntimeContext — 聚合运行时状态供表达式求值和参数解析使用
// ═══════════════════════════════════════════════════════════════

import type { ProjectInfo } from '../stores/project'

export interface ContextObject {
  resourceType: 'project' | 'directory' | 'file' | 'domain' | 'entity'
    | 'dict' | 'table' | 'dataStandard' | 'category' | 'group' | 'item'
  resource: {
    name: string
    path?: string
    [key: string]: unknown
  }
  semanticType?: string
  attributes?: Record<string, unknown>
}

export interface RuntimeState {
  project: {
    loaded: boolean
    path: string
    name: string
  }
}

export function createRuntimeState(project: ProjectInfo | null): RuntimeState {
  return {
    project: {
      loaded: project !== null,
      path: project?.path ?? '',
      name: project?.name ?? '',
    },
  }
}
