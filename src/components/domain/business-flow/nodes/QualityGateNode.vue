<script setup lang="ts">
import { Handle, Position } from '@vue-flow/core'

defineProps<{
  data: {
    label?: string
    metric?: string
    threshold?: number
  }
}>()
</script>

<template>
  <div
    class="flex flex-col items-center gap-1 px-3 py-2 min-w-[90px]
           bg-surface-0 dark:bg-surface-0 border-2 border-orange-400 dark:border-orange-500
           rounded-lg shadow-sm"
  >
    <Handle type="target" :position="Position.Top" class="!bg-orange-500 !w-2 !h-2" />
    <div class="text-lg">🛡</div>
    <span class="text-xs font-semibold text-text-primary text-center truncate max-w-[80px]">
      {{ data.label || 'Quality Gate' }}
    </span>
    <span v-if="data.metric" class="text-[10px] text-text-secondary">
      {{ data.metric }} ≥ {{ data.threshold }}
    </span>
    <Handle type="source" :position="Position.Bottom" class="!bg-orange-500 !w-2 !h-2" />
  </div>
</template>
