<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from '../i18n'

const { t } = useI18n()

const activeTab = ref('log')

const tabKeys = ['log', 'terminal', 'analysis'] as const
</script>

<template>
  <div class="bottom-panel">
    <div class="panel-tabs">
      <div
        v-for="key in tabKeys"
        :key="key"
        class="panel-tab"
        :class="{ active: activeTab === key }"
        @click="activeTab = key"
      >
        {{ (t.bottom as Record<string, string>)[key] }}
      </div>
    </div>
    <div class="panel-content">
      <div v-if="activeTab === 'log'" class="tab-content">
        <p class="placeholder-text">{{ t.bottom.logPlaceholder }}</p>
      </div>
      <div v-else-if="activeTab === 'terminal'" class="tab-content">
        <p class="placeholder-text">{{ t.bottom.terminalPlaceholder }}</p>
      </div>
      <div v-else-if="activeTab === 'analysis'" class="tab-content">
        <p class="placeholder-text">{{ t.bottom.analysisPlaceholder }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.bottom-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
}

.panel-tabs {
  display: flex;
  align-items: center;
  height: 30px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.panel-tab {
  padding: 0 14px;
  height: 100%;
  display: flex;
  align-items: center;
  font-size: 12px;
  color: var(--text-secondary);
  cursor: pointer;
  border-right: 1px solid var(--border-color);
  transition: background 0.15s;
}

.panel-tab:hover {
  background: var(--bg-hover);
}

.panel-tab.active {
  background: var(--bg-primary);
  color: var(--text-primary);
  border-bottom: 2px solid var(--accent-color);
}

.panel-content {
  flex: 1;
  overflow: auto;
}

.tab-content {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 16px;
}

.placeholder-text {
  font-size: 13px;
  color: var(--text-muted);
}
</style>
