export interface ProjectDirChild {
  key: string
  labelKey: string
  color: string
}

export interface ProjectDirGroup {
  key: string
  labelKey: string
  color: string
  children: ProjectDirChild[]
}

export interface ProjectDirCategory {
  key: string
  labelKey: string
  color: string
  groups?: ProjectDirGroup[]
  children?: ProjectDirChild[]
}

export const projectCategories: ProjectDirCategory[] = [
  {
    key: 'requirements', labelKey: 'requirements', color: '#409eff',
    groups: [
      {
        key: 'bizContext', labelKey: 'bizContext', color: '#5cadff',
        children: [
          { key: 'bizGoals',        labelKey: 'bizGoals',        color: '#5cadff' },
          { key: 'stakeholders',    labelKey: 'stakeholders',    color: '#4da0f5' },
          { key: 'bizProcess',      labelKey: 'bizProcess',      color: '#3d94eb' },
          { key: 'bizRules',        labelKey: 'bizRules',        color: '#2d88e0' }
        ]
      },
      {
        key: 'userStories', labelKey: 'userStories', color: '#67c23a',
        children: [
          { key: 'storyMap',        labelKey: 'storyMap',        color: '#67c23a' },
          { key: 'epicList',        labelKey: 'epicList',        color: '#5ab22e' },
          { key: 'backlog',         labelKey: 'backlog',         color: '#4da11f' }
        ]
      },
      {
        key: 'dataStandard', labelKey: 'dataStandard', color: '#e6a23c',
        children: [
          { key: 'dataModel',       labelKey: 'dataModel',       color: '#e6a23c' },
          { key: 'dataDictionary',  labelKey: 'dataDictionary',  color: '#d9992a' },
          { key: 'dataFlow',        labelKey: 'dataFlow',        color: '#cc8a1f' },
          { key: 'dataClassify',    labelKey: 'dataClassify',    color: '#bf7a14' }
        ]
      },
      {
        key: 'funcSpec', labelKey: 'funcSpec', color: '#e74c3c',
        children: [
          { key: 'funcTree',        labelKey: 'funcTree',        color: '#e74c3c' },
          { key: 'prototype',       labelKey: 'prototype',       color: '#d94434' },
          { key: 'apiContract',     labelKey: 'apiContract',     color: '#cc3c2d' },
          { key: 'integration',     labelKey: 'integration',     color: '#bf3425' }
        ]
      },
      {
        key: 'qualityAttr', labelKey: 'qualityAttr', color: '#8e44ad',
        children: [
          { key: 'performance',     labelKey: 'performance',     color: '#8e44ad' },
          { key: 'security',        labelKey: 'security',        color: '#7d3c98' },
          { key: 'availability',    labelKey: 'availability',    color: '#6c3483' },
          { key: 'migration',       labelKey: 'migration',       color: '#5b2c6f' }
        ]
      },
      {
        key: 'evidence', labelKey: 'evidence', color: '#2ecc71',
        children: [
          { key: 'reqEvidence',      labelKey: 'reqEvidence',      color: '#2ecc71' },
          { key: 'designRationale',  labelKey: 'designRationale',  color: '#27ae60' },
          { key: 'testEvidence',     labelKey: 'testEvidence',     color: '#1e8449' }
        ]
      },
      {
        key: 'materials', labelKey: 'materials', color: '#3498db',
        children: [
          { key: 'references',       labelKey: 'references',       color: '#3498db' },
          { key: 'meetingNotes',     labelKey: 'meetingNotes',     color: '#2980b9' },
          { key: 'decisionRecords',  labelKey: 'decisionRecords',  color: '#1f6dad' }
        ]
      },
      {
        key: 'risk', labelKey: 'risk', color: '#e67e22',
        children: [
          { key: 'riskRegister',     labelKey: 'riskRegister',     color: '#e67e22' },
          { key: 'issueTracking',    labelKey: 'issueTracking',    color: '#d35400' },
          { key: 'assumptionLog',    labelKey: 'assumptionLog',    color: '#ba4a00' }
        ]
      }
    ]
  },
  {
    key: 'design', labelKey: 'design', color: '#67c23a',
    children: [
      { key: 'metadata',    labelKey: 'metadata',     color: '#909399' },
      { key: 'architecture', labelKey: 'architecture', color: '#67c23a' },
      { key: 'detailDesign', labelKey: 'detailDesign', color: '#9b59b6' },
      { key: 'apiCard',      labelKey: 'apiCard',      color: '#e74c3c' },
      { key: 'errorCode',    labelKey: 'errorCode',    color: '#e67e22' }
    ]
  },
  {
    key: 'development', labelKey: 'development', color: '#e6a23c',
    children: [
      { key: 'frontendCode', labelKey: 'frontendCode', color: '#e74c3c' },
      { key: 'backendCode',  labelKey: 'backendCode',  color: '#3498db' },
      { key: 'database',     labelKey: 'database',     color: '#f39c12' },
      { key: 'config',       labelKey: 'config',       color: '#95a5a6' },
      { key: 'defectFix',       labelKey: 'defectFix',       color: '#e67e22' },
      { key: 'callChain',      labelKey: 'callChain',      color: '#8e44ad' },
      { key: 'templateScaffold', labelKey: 'templateScaffold', color: '#1abc9c' }
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
export const projectDirs: ProjectDirChild[] = projectCategories.flatMap(c => {
  const fromGroups = (c.groups || []).flatMap(g => g.children)
  const fromChildren = c.children || []
  return [...fromGroups, ...fromChildren]
})
