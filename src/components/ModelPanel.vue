<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from '../i18n'

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

function getModeLabel(key: string): string {
  return (t.value.model as Record<string, string>)[key] || key
}
</script>

<template>
  <div class="model-panel">
    <div class="panel-header">
      <el-select v-model="currentMode" size="small" style="width: 120px">
        <el-option
          v-for="mode in modes"
          :key="mode.value"
          :value="mode.value"
          :label="getModeLabel(mode.labelKey)"
        />
      </el-select>
      <el-select v-model="currentModel" size="small" style="width: 140px">
        <el-option
          v-for="model in models"
          :key="model.value"
          :value="model.value"
          :label="model.label"
        />
      </el-select>
    </div>
    <div class="panel-body">
      <div v-if="currentMode === 'chat'" class="mode-placeholder">
        <p>{{ t.model.chatPlaceholder }}</p>
      </div>
      <div v-else-if="currentMode === 'plan'" class="mode-placeholder">
        <p>{{ t.model.planPlaceholder }}</p>
      </div>
      <div v-else-if="currentMode === 'expert'" class="mode-placeholder">
        <p>{{ t.model.expertPlaceholder }}</p>
      </div>
    </div>
    <div class="panel-input">
      <el-input
        type="textarea"
        :rows="2"
        :placeholder="t.model.inputPlaceholder"
        resize="none"
      />
    </div>
  </div>
</template>

<style scoped>
.model-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-panel);
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.panel-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.mode-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-muted);
  font-size: 13px;
}

.panel-input {
  padding: 8px 12px;
  border-top: 1px solid var(--border-color);
  flex-shrink: 0;
}
</style>
