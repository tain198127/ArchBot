<script setup lang="ts">
import { Handle, Position } from '@vue-flow/core'

defineProps<{
  data: {
    label?: string
    fileName?: string
    filePath?: string
    fileType?: string
  }
}>()
</script>

<template>
  <div
    class="flex flex-col items-center gap-1 px-3 py-2 min-w-[90px]
           bg-surface-0 dark:bg-surface-0 border-2 border-emerald-400 dark:border-emerald-500
           rounded-lg shadow-sm"
  >
    <Handle type="source" :position="Position.Bottom" class="!bg-emerald-500 !w-2 !h-2" />
    <div class="text-lg">📄</div>
    <span class="text-xs font-medium text-text-primary text-center truncate max-w-[80px]">
      {{ data.fileName || data.label || 'Material' }}
    </span>
    <span v-if="data.fileType" class="text-[10px] text-text-secondary">
      {{ data.fileType }}
    </span>
  </div>
</template>
