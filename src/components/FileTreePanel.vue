<script setup lang="ts">
</script>

<template>
  <div class="file-tree-panel">
    <div class="panel-header">
      <span class="panel-title">项目</span>
    </div>
    <div class="panel-body">
      <div class="tree-placeholder">
        <p class="placeholder-text">打开项目以查看文件树</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.file-tree-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #f9f9f9;
}

.panel-header {
  display: flex;
  align-items: center;
  height: 32px;
  padding: 0 12px;
  border-bottom: 1px solid #e0e0e0;
  flex-shrink: 0;
}

.panel-title {
  font-size: 12px;
  font-weight: 600;
  color: #666;
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
  color: #999;
}

@media (prefers-color-scheme: dark) {
  .file-tree-panel {
    background: #252525;
  }

  .panel-header {
    border-bottom-color: #3c3c3c;
  }

  .panel-title {
    color: #aaa;
  }

  .placeholder-text {
    color: #666;
  }
}
</style>
