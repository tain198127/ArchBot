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
  background: #fff;
}

.panel-tabs {
  display: flex;
  align-items: center;
  height: 30px;
  background: #f0f0f0;
  border-bottom: 1px solid #e0e0e0;
  flex-shrink: 0;
}

.panel-tab {
  padding: 0 14px;
  height: 100%;
  display: flex;
  align-items: center;
  font-size: 12px;
  color: #666;
  cursor: pointer;
  border-right: 1px solid #e0e0e0;
  transition: background 0.15s;
}

.panel-tab:hover {
  background: #e8e8e8;
}

.panel-tab.active {
  background: #fff;
  color: #333;
  border-bottom: 2px solid #409eff;
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
  color: #999;
}

@media (prefers-color-scheme: dark) {
  .bottom-panel {
    background: #1e1e1e;
  }

  .panel-tabs {
    background: #252525;
    border-bottom-color: #3c3c3c;
  }

  .panel-tab {
    color: #999;
    border-right-color: #3c3c3c;
  }

  .panel-tab:hover {
    background: #2d2d2d;
  }

  .panel-tab.active {
    background: #1e1e1e;
    color: #ddd;
  }

  .placeholder-text {
    color: #666;
  }
}
</style>
