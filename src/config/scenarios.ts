import type { ScenarioDimension } from '../composables/useScenario'

export type ScenarioType = 'greenfield' | 'legacy-modernization' | 'product-customization'

export interface ScenarioPreset {
  type: ScenarioType
  nameKey: string
  descKey: string
  defaults: ScenarioDimension
}

export const SCENARIO_PRESETS: ScenarioPreset[] = [
  {
    type: 'greenfield',
    nameKey: 'scenario.greenfield.name',
    descKey: 'scenario.greenfield.desc',
    defaults: {
      digitalEmployees: ['architect', 'frontend-dev', 'backend-dev', 'tester'],
      businessFlow: ['requirement', 'design', 'development', 'testing', 'deployment'],
      skills: ['scaffold', 'design-gen', 'code-gen'],
      collaborationMode: 'parallel',
      dirStructure: [],
    },
  },
  {
    type: 'legacy-modernization',
    nameKey: 'scenario.legacy.name',
    descKey: 'scenario.legacy.desc',
    defaults: {
      digitalEmployees: ['reverse-analyst', 'migration-engineer', 'tester'],
      businessFlow: ['reverse-analysis', 'assessment', 'refactoring', 'testing', 'deployment'],
      skills: ['reverse-engineering', 'code-migration', 'compatibility-test'],
      collaborationMode: 'sequential',
      dirStructure: ['analysis'],
    },
  },
  {
    type: 'product-customization',
    nameKey: 'scenario.product.name',
    descKey: 'scenario.product.desc',
    defaults: {
      digitalEmployees: ['product-analyst', 'customization-dev', 'tester'],
      businessFlow: ['product-understanding', 'gap-analysis', 'customization', 'testing', 'deployment'],
      skills: ['api-understanding', 'extension-dev', 'upstream-sync'],
      collaborationMode: 'hybrid',
      dirStructure: ['upstream-reference'],
    },
  },
]
