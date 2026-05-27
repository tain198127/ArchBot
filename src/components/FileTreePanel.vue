<script setup lang="ts">
import { useI18n } from '../i18n'

const { t } = useI18n()
</script>

<template>
  <div class="file-tree-panel">
    <div class="panel-header">
      <span class="panel-title">{{ t.panel.project }}</span>
    </div>
    <div class="panel-body">
      <div class="tree-placeholder">
        <p class="placeholder-text">{{ t.panel.openProjectHint }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.file-tree-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-panel);
}

.panel-header {
  display: flex;
  align-items: center;
  height: 32px;
  padding: 0 12px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.panel-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
}

.panel-body {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.tree-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.placeholder-text {
  font-size: 13px;
  color: var(--text-muted);
}
</style>
