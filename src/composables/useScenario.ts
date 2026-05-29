import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useProject } from '../stores/project'
import { SCENARIO_PRESETS, type ScenarioType, type ScenarioPreset } from '../config/scenarios'

export interface ScenarioDimension {
  digitalEmployees: string[]
  businessFlow: string[]
  skills: string[]
  collaborationMode: string
  dirStructure: string[]
}

export interface ProjectScenario {
  type: string
  overrides: Record<string, unknown>
  appliedAt: string
}

const currentScenario = ref<ProjectScenario>({ type: '', overrides: {}, appliedAt: '' })
const loading = ref(false)
const lastSaved = ref<ProjectScenario | null>(null)

export function useScenario() {
  const { currentProject } = useProject()

  const preset = computed<ScenarioPreset | null>(() => {
    if (!currentScenario.value.type) return null
    return SCENARIO_PRESETS.find(p => p.type === currentScenario.value.type) || null
  })

  const effectiveDimension = computed<ScenarioDimension | null>(() => {
    const p = preset.value
    if (!p) return null
    return {
      ...p.defaults,
      ...(currentScenario.value.overrides as Partial<ScenarioDimension>),
    }
  })

  async function load() {
    const path = currentProject.value?.path
    if (!path) return
    loading.value = true
    try {
      const result = await invoke<ProjectScenario>('get_scenario', { projectPath: path })
      currentScenario.value = result
      lastSaved.value = { ...result }
    } catch {
      currentScenario.value = { type: '', overrides: {}, appliedAt: '' }
    } finally {
      loading.value = false
    }
  }

  async function save(type: ScenarioType, overrides: Partial<ScenarioDimension> = {}) {
    const path = currentProject.value?.path
    if (!path) return

    const newScenario: ProjectScenario = {
      type,
      overrides,
      appliedAt: new Date().toISOString(),
    }

    await invoke('save_scenario', {
      projectPath: path,
      scenario: newScenario,
    })

    const prevType = currentScenario.value.type
    currentScenario.value = newScenario
    lastSaved.value = { ...newScenario }

    return prevType !== type
  }

  function hasChanges(): boolean {
    if (!lastSaved.value) return currentScenario.value.type !== ''
    return JSON.stringify(currentScenario.value) !== JSON.stringify(lastSaved.value)
  }

  async function switchScenario(newType: ScenarioType) {
    const currentOverrides = currentScenario.value.overrides
    const newPreset = SCENARIO_PRESETS.find(p => p.type === newType)
    if (!newPreset) return false

    // Preserve user overrides that are still applicable
    const merged: Partial<ScenarioDimension> = {}
    const dimKeys: (keyof ScenarioDimension)[] = [
      'digitalEmployees', 'businessFlow', 'skills', 'collaborationMode', 'dirStructure',
    ]
    for (const key of dimKeys) {
      const val = currentOverrides[key]
      if (val !== undefined && val !== null) {
        ;(merged as Record<string, unknown>)[key] = val
      }
    }

    return save(newType, merged as Partial<ScenarioDimension>)
  }

  function computeDiff(oldType: ScenarioType, newType: ScenarioType) {
    const oldPreset = SCENARIO_PRESETS.find(p => p.type === oldType)
    const newPreset = SCENARIO_PRESETS.find(p => p.type === newType)
    if (!oldPreset || !newPreset) return null

    const diff: Record<string, { from: unknown; to: unknown }> = {}
    const keys = [...new Set([
      ...Object.keys(oldPreset.defaults),
      ...Object.keys(newPreset.defaults),
    ])] as (keyof ScenarioDimension)[]

    for (const key of keys) {
      const from = oldPreset.defaults[key]
      const to = newPreset.defaults[key]
      if (JSON.stringify(from) !== JSON.stringify(to)) {
        diff[key] = { from, to }
      }
    }
    return Object.keys(diff).length > 0 ? diff : null
  }

  return {
    currentScenario,
    preset,
    effectiveDimension,
    loading,
    load,
    save,
    switchScenario,
    computeDiff,
    hasChanges,
  }
}
