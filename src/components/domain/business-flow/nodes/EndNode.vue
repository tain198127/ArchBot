<script setup lang="ts">
import { Handle, Position } from '@vue-flow/core'

defineProps<{ data: { label: string } }>()
</script>

<template>
  <div
    class="flex items-center justify-center w-12 h-12 rounded-full
           bg-red-500 text-white text-sm font-bold shadow-md
           border-2 border-red-600 dark:border-red-400"
  >
    <Handle type="target" :position="Position.Top" class="!bg-red-700 !w-2 !h-2" />
    {{ data.label || 'End' }}
  </div>
</template>
