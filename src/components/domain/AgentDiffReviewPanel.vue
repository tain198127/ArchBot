<script setup lang="ts">
import { watch } from 'vue'
import { agentStore, loadFileChanges } from '../../stores/agentStore'
import '../../stores/agentStore'

watch(
  () => agentStore.selectedTurnId,
  (tid) => {
    if (tid) loadFileChanges(tid)
  },
)

function changeIcon(type: string): string {
  const map: Record<string, string> = { created: '+', modified: '~', deleted: '-', renamed: '>' }
  return map[type] || '?'
}

function changeColor(type: string): string {
  const map: Record<string, string> = { created: 'text-green-500', modified: 'text-amber-500', deleted: 'text-red-500', renamed: 'text-blue-500' }
  return map[type] || 'text-text-muted'
}
</script>

<template>
  <div class="flex flex-col h-full bg-surface-0">
    <div class="px-3 py-1.5 border-b border-border-default">
      <span class="text-[12px] font-semibold text-text-primary">File Changes</span>
      <span class="text-[11px] text-text-muted ml-2">{{ agentStore.fileChanges.length }} files</span>
    </div>

    <div class="flex-1 overflow-auto">
      <div
        v-for="(change, idx) in agentStore.fileChanges"
        :key="idx"
        class="px-3 py-2 border-b border-border-default/30 hover:bg-surface-50"
      >
        <div class="flex items-center gap-2">
          <span class="w-5 h-5 flex items-center justify-center rounded text-[11px] font-bold"
            :class="changeColor(change.change_type)"
          >{{ changeIcon(change.change_type) }}</span>
          <span class="text-[12px] text-text-primary truncate flex-1">{{ change.path }}</span>
          <span class="text-[10px] shrink-0" :class="changeColor(change.change_type)">{{ change.change_type }}</span>
        </div>
      </div>
      <div v-if="agentStore.fileChanges.length === 0" class="flex items-center justify-center h-full">
        <p class="text-[12px] text-text-muted">{{ agentStore.selectedTurnId ? 'No file changes for this turn.' : 'Select a turn to see file changes.' }}</p>
      </div>
    </div>
  </div>
</template>
