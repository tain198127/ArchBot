<script setup lang="ts">
import { Handle, Position } from '@vue-flow/core'

defineProps<{
  data: {
    operation?: string
    threshold?: number
    label?: string
  }
}>()

function opSymbol(op?: string): string {
  switch (op) {
    case 'add': return '+'
    case 'subtract': return '−'
    case 'multiply': return '×'
    case 'divide': return '÷'
    case 'compare': return '≥'
    case 'aggregate': return 'Σ'
    default: return '#'
  }
}
</script>

<template>
  <div
    class="flex flex-col items-center justify-center gap-0.5
           w-[56px] h-[56px] rotate-45
           bg-indigo-50 dark:bg-indigo-950 border-2 border-indigo-400 dark:border-indigo-500
           shadow-sm"
  >
    <Handle type="target" :position="Position.Top" class="!bg-indigo-500 !w-2 !h-2" />
    <div class="-rotate-45 flex flex-col items-center">
      <span class="text-lg font-bold text-indigo-600 dark:text-indigo-400 leading-none">
        {{ opSymbol(data.operation) }}
      </span>
      <span v-if="data.threshold != null" class="text-[9px] font-mono text-indigo-500 dark:text-indigo-400">
        {{ data.threshold }}
      </span>
    </div>
    <Handle type="source" :position="Position.Bottom" class="!bg-indigo-500 !w-2 !h-2" />
  </div>
</template>
