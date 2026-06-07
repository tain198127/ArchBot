<script setup lang="ts">
import { Handle, Position } from '@vue-flow/core'

defineProps<{
  data: {
    refType?: string
    refName?: string
    filePath?: string
    label?: string
  }
}>()

function refIcon(type?: string): string {
  switch (type) {
    case 'file': return '📁'
    case 'agent': return '🤖'
    case 'skill': return '⚡'
    case 'employee': return '🧑‍💼'
    default: return '🔗'
  }
}
</script>

<template>
  <div
    class="flex flex-col items-center gap-1 px-4 py-2.5 min-w-[100px]
           bg-gray-50 dark:bg-gray-900 border-2 border-dashed border-gray-400 dark:border-gray-500
           rounded-lg shadow-sm"
  >
    <Handle type="target" :position="Position.Top" class="!bg-gray-500 !w-2 !h-2" />
    <div class="text-lg">{{ refIcon(data.refType) }}</div>
    <span class="text-[10px] text-text-secondary uppercase tracking-wider">
      {{ data.refType ?? 'ref' }}
    </span>
    <span class="text-xs font-semibold text-text-primary text-center truncate max-w-[100px]">
      {{ data.refName || data.label || data.filePath?.split('/').pop() || 'Reference' }}
    </span>
    <Handle type="source" :position="Position.Bottom" class="!bg-gray-500 !w-2 !h-2" />
  </div>
</template>
