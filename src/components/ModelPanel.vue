<script setup lang="ts">
import { ref } from 'vue'

const currentMode = ref('chat')
const currentModel = ref('gpt-4')

const modes = [
  { value: 'chat', label: '问答模式' },
  { value: 'plan', label: 'Plan模式' },
  { value: 'expert', label: '专家团模式' }
]

const models = [
  { value: 'gpt-4', label: 'GPT-4' },
  { value: 'claude-sonnet', label: 'Claude Sonnet' },
  { value: 'claude-opus', label: 'Claude Opus' }
]
</script>

<template>
  <div class="model-panel">
    <div class="panel-header">
      <el-select v-model="currentMode" size="small" style="width: 120px">
        <el-option
          v-for="mode in modes"
          :key="mode.value"
          :value="mode.value"
          :label="mode.label"
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
        <p>对话消息将显示在这里</p>
      </div>
      <div v-else-if="currentMode === 'plan'" class="mode-placeholder">
        <p>Plan 步骤将显示在这里</p>
      </div>
      <div v-else-if="currentMode === 'expert'" class="mode-placeholder">
        <p>专家团讨论将显示在这里</p>
      </div>
    </div>
    <div class="panel-input">
      <el-input
        type="textarea"
        :rows="2"
        placeholder="输入消息..."
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
  background: #fafafa;
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid #e0e0e0;
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
  color: #999;
  font-size: 13px;
}

.panel-input {
  padding: 8px 12px;
  border-top: 1px solid #e0e0e0;
  flex-shrink: 0;
}

@media (prefers-color-scheme: dark) {
  .model-panel {
    background: #252525;
  }

  .panel-header {
    border-bottom-color: #3c3c3c;
  }

  .panel-input {
    border-top-color: #3c3c3c;
  }

  .mode-placeholder {
    color: #666;
  }
}
</style>
