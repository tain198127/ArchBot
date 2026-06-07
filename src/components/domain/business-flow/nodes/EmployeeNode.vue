<script setup lang="ts">
import { Handle, Position } from '@vue-flow/core'

defineProps<{
  data: {
    employeeCode?: string
    employeeName?: string
    avatar?: string
    personality?: string
    skillName?: string
    label?: string
  }
}>()
</script>

<template>
  <div
    class="flex flex-col items-center gap-1 px-4 py-2.5 min-w-[100px]
           bg-purple-50 dark:bg-purple-950 border-2 border-purple-400 dark:border-purple-500
           rounded-lg shadow-sm"
  >
    <Handle type="target" :position="Position.Top" class="!bg-purple-500 !w-2 !h-2" />
    <div class="text-lg">{{ data.avatar || '🧑‍💼' }}</div>
    <span class="text-xs font-semibold text-text-primary text-center truncate max-w-[100px]">
      {{ data.employeeName || data.label || 'Employee' }}
    </span>
    <span v-if="data.personality" class="text-[10px] text-purple-600 dark:text-purple-400 truncate max-w-[100px]">
      {{ data.personality }}
    </span>
    <span v-if="data.skillName" class="text-[10px] text-text-secondary truncate max-w-[90px]">
      ⚡ {{ data.skillName }}
    </span>
    <Handle type="source" :position="Position.Bottom" class="!bg-purple-500 !w-2 !h-2" />
  </div>
</template>
