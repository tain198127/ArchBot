<script setup lang="ts">
import { ref, computed } from 'vue'
import VSelect from '../base/VSelect.vue'
import VTextarea from '../base/VTextarea.vue'
import { useI18n } from '../../i18n'

const { t } = useI18n()

const currentMode = ref('chat')
const currentModel = ref('gpt-4')

const modes = [
  { value: 'chat', labelKey: 'chatMode' },
  { value: 'plan', labelKey: 'planMode' },
  { value: 'expert', labelKey: 'expertMode' }
]

const models = [
  { value: 'gpt-4', label: 'GPT-4' },
  { value: 'claude-sonnet', label: 'Claude Sonnet' },
  { value: 'claude-opus', label: 'Claude Opus' }
]

const modelMsgs = t.value.model as Record<string, string>

const modeOptions = computed(() =>
  modes.map((m) => ({ value: m.value, label: modelMsgs[m.labelKey] || m.labelKey }))
)
</script>

<template>
  <div class="flex flex-col h-full bg-surface-50 dark:bg-surface-50">
    <div class="flex items-center gap-2 px-3 py-2 border-b border-border-default shrink-0">
      <VSelect v-model="currentMode" :options="modeOptions" class="!w-[120px]" />
      <VSelect v-model="currentModel" :options="models" class="!w-[140px]" />
    </div>
    <div class="flex-1 overflow-y-auto p-4">
      <div class="flex items-center justify-center h-full text-sm text-text-muted">
        <p v-if="currentMode === 'chat'">{{ t.model.chatPlaceholder }}</p>
        <p v-else-if="currentMode === 'plan'">{{ t.model.planPlaceholder }}</p>
        <p v-else-if="currentMode === 'expert'">{{ t.model.expertPlaceholder }}</p>
      </div>
    </div>
    <div class="px-3 py-2 border-t border-border-default shrink-0">
      <VTextarea :rows="2" :placeholder="t.model.inputPlaceholder" />
    </div>
  </div>
</template>
