<script setup lang="ts">
import { Handle, Position } from '@vue-flow/core'

defineProps<{
  data: {
    agentName?: string
    skillName?: string
    label?: string
  }
}>()
</script>

<template>
  <div
    class="flex flex-col items-center gap-1 px-4 py-2.5 min-w-[100px]
           bg-surface-0 dark:bg-surface-0 border-2 border-blue-400 dark:border-blue-500
           rounded-lg shadow-sm"
  >
    <Handle type="target" :position="Position.Top" class="!bg-blue-500 !w-2 !h-2" />
    <div class="text-lg">👤</div>
    <span class="text-xs font-semibold text-text-primary text-center truncate max-w-[90px]">
      {{ data.agentName || data.label || 'Agent' }}
    </span>
    <span v-if="data.skillName" class="text-[10px] text-text-secondary truncate max-w-[90px]">
      {{ data.skillName }}
    </span>
    <Handle type="source" :position="Position.Bottom" class="!bg-blue-500 !w-2 !h-2" />
  </div>
</template>
