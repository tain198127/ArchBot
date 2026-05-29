<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from '../../i18n'
import { useScenario } from '../../composables/useScenario'
import { SCENARIO_PRESETS, type ScenarioType } from '../../config/scenarios'
import VButton from '../base/VButton.vue'

const { t, tt } = useI18n()
const { currentScenario, preset, loading, load, switchScenario, computeDiff } = useScenario()

const switching = ref(false)
const selectedType = ref<ScenarioType | null>(null)
const showDiff = ref(false)
const diffData = ref<Record<string, { from: unknown; to: unknown }> | null>(null)

onMounted(() => {
  load()
})

function selectScenario(type: ScenarioType) {
  selectedType.value = type
}

async function applyScenario() {
  if (!selectedType.value) return
  switching.value = true
  try {
    if (currentScenario.value.type) {
      const diff = computeDiff(currentScenario.value.type as ScenarioType, selectedType.value)
      if (diff && Object.keys(diff).length > 0) {
        diffData.value = diff
        showDiff.value = true
        switching.value = false
        return
      }
    }
    await switchScenario(selectedType.value)
  } finally {
    switching.value = false
    showDiff.value = false
  }
}

async function confirmSwitch() {
  if (!selectedType.value) return
  showDiff.value = false
  switching.value = true
  try {
    await switchScenario(selectedType.value)
  } finally {
    switching.value = false
  }
}

function dimLabel(key: string): string {
  const dims = t.value.scenario.dimension as Record<string, string>
  return dims?.[key] || key
}

function formatDimValue(key: string, value: unknown): string {
  if (key === 'collaborationMode') {
    const modes = t.value.scenario.modes as Record<string, string>
    return modes?.[value as string] || (value as string)
  }
  if (Array.isArray(value)) {
    return value.join(', ')
  }
  return String(value)
}
</script>

<template>
  <div class="flex flex-col h-full p-4 overflow-y-auto">
    <h2 class="text-lg font-semibold text-text-primary mb-2">{{ tt('scenario.title') }}</h2>
    <p class="text-sm text-text-secondary mb-4">{{ tt('scenario.selectHint') }}</p>

    <div v-if="loading" class="text-sm text-text-muted py-8 text-center">Loading...</div>

    <template v-else>
      <div class="grid gap-3 mb-6">
        <div
          v-for="sp in SCENARIO_PRESETS" :key="sp.type"
          class="border rounded-lg p-4 cursor-pointer transition-colors"
          :class="{
            'border-primary-500 bg-primary-50 dark:bg-primary-950': selectedType === sp.type || (preset?.type === sp.type && !selectedType),
            'border-border-default hover:border-primary-300': selectedType !== sp.type && preset?.type !== sp.type
          }"
          @click="selectScenario(sp.type)"
        >
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2">
              <span class="font-medium text-text-primary">{{ tt(sp.nameKey) }}</span>
              <span v-if="preset?.type === sp.type && !selectedType" class="text-xs bg-primary-500 text-white px-1.5 py-0.5 rounded">
                {{ tt('scenario.current') }}
              </span>
            </div>
          </div>
          <p class="text-xs text-text-secondary leading-relaxed">{{ tt(sp.descKey) }}</p>

          <div v-if="selectedType === sp.type || (preset?.type === sp.type && !selectedType)" class="mt-3 pt-3 border-t border-border-default">
            <div class="grid grid-cols-2 gap-2 text-xs">
              <div v-for="(val, key) in sp.defaults" :key="key" class="flex flex-col">
                <span class="text-text-muted">{{ dimLabel(key) }}</span>
                <span class="text-text-primary font-medium">{{ formatDimValue(key, val) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <VButton
          :disabled="!selectedType || switching"
          :loading="switching"
          variant="primary"
          @click="applyScenario"
        >
          {{ currentScenario.type ? tt('scenario.switch') : 'Apply' }}
        </VButton>
        <span v-if="currentScenario.type && !selectedType" class="text-xs text-text-muted">
          {{ tt('scenario.current') }}: {{ preset ? tt(preset.nameKey) : currentScenario.type }}
        </span>
      </div>
    </template>

    <!-- Diff confirmation dialog -->
    <div v-if="showDiff" class="fixed inset-0 z-[3000] flex items-center justify-center bg-black/40" @click.self="showDiff = false">
      <div class="bg-white dark:bg-surface-0 rounded-xl shadow-2xl p-6 max-w-lg w-full mx-4">
        <h3 class="text-base font-semibold text-text-primary mb-2">{{ tt('scenario.switchConfirm') }}</h3>
        <div class="max-h-60 overflow-y-auto mb-4 text-xs">
          <div v-for="(change, key) in diffData" :key="key" class="py-2 border-b border-border-default">
            <div class="font-medium text-text-primary mb-1">{{ dimLabel(key) }}</div>
            <div class="flex items-center gap-2">
              <span class="text-text-muted line-through">{{ formatDimValue(key, change.from) }}</span>
              <span class="text-text-muted">→</span>
              <span class="text-primary-600">{{ formatDimValue(key, change.to) }}</span>
            </div>
          </div>
        </div>
        <div class="flex justify-end gap-2">
          <VButton variant="secondary" @click="showDiff = false">{{ tt('common.cancel') }}</VButton>
          <VButton variant="primary" @click="confirmSwitch">{{ tt('common.confirm') }}</VButton>
        </div>
      </div>
    </div>
  </div>
</template>
