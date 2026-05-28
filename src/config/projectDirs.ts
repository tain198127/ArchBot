export interface ProjectDirChild {
  key: string
  labelKey: string
  color: string
}

export interface ProjectDirCategory {
  key: string
  labelKey: string
  color: string
  children: ProjectDirChild[]
}

export const projectCategories: ProjectDirCategory[] = [
  {
    key: 'requirements', labelKey: 'requirements', color: '#409eff',
    children: [
      { key: 'bizReq',       labelKey: 'bizReq',       color: '#5cadff' },
      { key: 'specReq',      labelKey: 'specReq',      color: '#2d8cf0' },
      { key: 'prototype',    labelKey: 'prototype',    color: '#69c0ff' }
    ]
  },
  {
    key: 'design', labelKey: 'design', color: '#67c23a',
    children: [
      { key: 'dataStandard', labelKey: 'dataStandard', color: '#e6a23c' },
      { key: 'metadata',    labelKey: 'metadata',     color: '#909399' },
      { key: 'architecture', labelKey: 'architecture', color: '#67c23a' },
      { key: 'detailDesign', labelKey: 'detailDesign', color: '#9b59b6' }
    ]
  },
  {
    key: 'development', labelKey: 'development', color: '#e6a23c',
    children: [
      { key: 'frontendCode', labelKey: 'frontendCode', color: '#e74c3c' },
      { key: 'backendCode',  labelKey: 'backendCode',  color: '#3498db' },
      { key: 'database',     labelKey: 'database',     color: '#f39c12' },
      { key: 'config',       labelKey: 'config',       color: '#95a5a6' },
      { key: 'defectFix',    labelKey: 'defectFix',    color: '#e67e22' }
    ]
  },
  {
    key: 'testing', labelKey: 'testing', color: '#f39c12',
    children: [
      { key: 'testCases',    labelKey: 'testCases',    color: '#1abc9c' },
      { key: 'businessTest', labelKey: 'businessTest', color: '#2ecc71' },
      { key: 'stressTest',   labelKey: 'stressTest',   color: '#e67e22' },
      { key: 'chaosTest',    labelKey: 'chaosTest',    color: '#c0392b' },
      { key: 'securityTest', labelKey: 'securityTest', color: '#d35400' },
      { key: 'defect',       labelKey: 'defect',       color: '#c0392b' }
    ]
  },
  {
    key: 'deployment', labelKey: 'deployment', color: '#8e44ad',
    children: [
      { key: 'cicd',         labelKey: 'cicd',         color: '#9b59b6' },
      { key: 'versionMgmt',  labelKey: 'versionMgmt',  color: '#7d3c98' },
      { key: 'envMgmt',      labelKey: 'envMgmt',      color: '#6c3483' },
      { key: 'artifactMgmt', labelKey: 'artifactMgmt', color: '#5b2c6f' }
    ]
  }
]

// flat accessor for backward compat
export const projectDirs: ProjectDirChild[] = projectCategories.flatMap(c => c.children)
