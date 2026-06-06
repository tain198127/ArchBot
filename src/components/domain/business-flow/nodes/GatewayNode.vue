<script setup lang="ts">
import { Handle, Position } from '@vue-flow/core'
import { computed } from 'vue'

const props = defineProps<{
  data: {
    label?: string
  }
  type?: string
}>()

const symbol = computed(() => {
  switch (props.type) {
    case 'gateway_xor': return '◇'
    case 'gateway_and': return '◈'
    case 'gateway_or': return '◆'
    default: return '◇'
  }
})

const borderColor = computed(() => {
  switch (props.type) {
    case 'gateway_xor': return 'border-amber-400 dark:border-amber-500'
    case 'gateway_and': return 'border-purple-400 dark:border-purple-500'
    case 'gateway_or': return 'border-cyan-400 dark:border-cyan-500'
    default: return 'border-amber-400'
  }
})
</script>

<template>
  <div
    class="flex items-center justify-center w-14 h-14 rotate-45
           bg-surface-0 dark:bg-surface-0 border-2 shadow-md"
    :class="borderColor"
  >
    <Handle type="target" :position="Position.Top" class="!bg-amber-500 !w-2 !h-2 -rotate-45" />
    <span class="text-base font-bold -rotate-45 text-text-primary">
      {{ symbol }}
    </span>
    <Handle type="source" :position="Position.Bottom" class="!bg-amber-500 !w-2 !h-2 -rotate-45" />
  </div>
</template>
