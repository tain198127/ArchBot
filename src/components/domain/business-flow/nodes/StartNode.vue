<script setup lang="ts">
import { Handle, Position } from '@vue-flow/core'

defineProps<{ data: { label: string } }>()
</script>

<template>
  <div
    class="flex items-center justify-center w-12 h-12 rounded-full
           bg-green-500 text-white text-sm font-bold shadow-md
           border-2 border-green-600 dark:border-green-400"
  >
    <Handle type="source" :position="Position.Bottom" class="!bg-green-700 !w-2 !h-2" />
    {{ data.label || 'Start' }}
  </div>
</template>
