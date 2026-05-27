export interface ProjectDir {
  key: string
  labelKey: string
  color: string
}

export const projectDirs: ProjectDir[] = [
  { key: 'dataStandard',  labelKey: 'dataStandard',  color: '#e6a23c' },
  { key: 'metadata',      labelKey: 'metadata',      color: '#909399' },
  { key: 'requirements',  labelKey: 'requirements',  color: '#409eff' },
  { key: 'architecture',  labelKey: 'architecture',  color: '#67c23a' },
  { key: 'detailDesign',  labelKey: 'detailDesign',  color: '#9b59b6' },
  { key: 'frontendCode',  labelKey: 'frontendCode',  color: '#e74c3c' },
  { key: 'backendCode',   labelKey: 'backendCode',   color: '#3498db' },
  { key: 'database',      labelKey: 'database',      color: '#f39c12' },
  { key: 'testCases',     labelKey: 'testCases',     color: '#1abc9c' },
  { key: 'businessTest',  labelKey: 'businessTest',  color: '#2ecc71' },
  { key: 'stressTest',    labelKey: 'stressTest',    color: '#e67e22' },
  { key: 'chaosTest',     labelKey: 'chaosTest',     color: '#c0392b' },
  { key: 'defect',        labelKey: 'defect',        color: '#d35400' },
  { key: 'deployment',    labelKey: 'deployment',    color: '#8e44ad' }
]
