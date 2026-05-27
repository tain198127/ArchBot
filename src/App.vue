<script setup lang="ts">
import MenuBar from './components/MenuBar.vue'
import SplitPanel from './components/SplitPanel.vue'
import FileTreePanel from './components/FileTreePanel.vue'
import EditorPanel from './components/EditorPanel.vue'
import ModelPanel from './components/ModelPanel.vue'
import BottomPanel from './components/BottomPanel.vue'
</script>

<template>
  <div class="app-root">
    <MenuBar />
    <SplitPanel
      direction="vertical"
      :initial-sizes="[-1, 200]"
      :min-sizes="[300, 120]"
      :collapsible="[false, true]"
      :collapse-icons="['', '⌨']"
      :collapse-labels="['', '面板']"
    >
      <template #panel-0>
        <SplitPanel
          direction="horizontal"
          :initial-sizes="[240, -1, 320]"
          :min-sizes="[180, 300, 240]"
          :collapsible="[false, false, true]"
          :collapse-icons="['', '', '🤖']"
          :collapse-labels="['', '', '模型']"
        >
          <template #panel-0>
            <FileTreePanel />
          </template>
          <template #panel-1>
            <EditorPanel />
          </template>
          <template #panel-2>
            <ModelPanel />
          </template>
        </SplitPanel>
      </template>
      <template #panel-1>
        <BottomPanel />
      </template>
    </SplitPanel>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  width: 100%;
  height: 100%;
  overflow: hidden;
}

:root {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
    Ubuntu, Cantarell, 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif;
  font-size: 14px;
  line-height: 1.5;
  color: #333;
  background: #fff;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #ccc;
    background: #1e1e1e;
  }
}
</style>

<style scoped>
.app-root {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  overflow: hidden;
}
</style>
