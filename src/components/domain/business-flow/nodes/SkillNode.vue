<script setup lang="ts">
import { Handle, Position } from '@vue-flow/core'

defineProps<{
  data: {
    skillCode?: string
    skillName?: string
    command?: string
    label?: string
  }
}>()
</script>

<template>
  <div
    class="flex flex-col items-center gap-1 px-4 py-2.5 min-w-[100px]
           bg-cyan-50 dark:bg-cyan-950 border-2 border-cyan-400 dark:border-cyan-500
           rounded-lg shadow-sm"
  >
    <Handle type="target" :position="Position.Top" class="!bg-cyan-500 !w-2 !h-2" />
    <div class="text-lg">⚡</div>
    <span class="text-xs font-semibold text-text-primary text-center truncate max-w-[100px]">
      {{ data.skillName || data.label || 'Skill' }}
    </span>
    <span v-if="data.command" class="text-[10px] font-mono text-cyan-600 dark:text-cyan-400 truncate max-w-[100px]">
      {{ data.command }}
    </span>
    <Handle type="source" :position="Position.Bottom" class="!bg-cyan-500 !w-2 !h-2" />
  </div>
</template>
