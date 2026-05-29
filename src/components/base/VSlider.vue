<script setup lang="ts">
interface Props {
  modelValue?: number
  min?: number
  max?: number
  step?: number
  disabled?: boolean
}

withDefaults(defineProps<Props>(), {
  min: 0,
  max: 100,
  step: 1,
})

const emit = defineEmits<{
  'update:modelValue': [value: number]
}>()
</script>

<template>
  <div class="flex items-center gap-3">
    <input
      type="range"
      :min="min"
      :max="max"
      :step="step"
      :value="modelValue"
      :disabled="disabled"
      class="w-full h-1.5 rounded appearance-none bg-surface-200 dark:bg-surface-300 cursor-pointer
             focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-500
             disabled:opacity-50 disabled:cursor-not-allowed
             [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:h-3.5 [&::-webkit-slider-thumb]:w-3.5
             [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-primary-500
             [&::-webkit-slider-thumb]:cursor-pointer"
      @input="emit('update:modelValue', Number(($event.target as HTMLInputElement).value))"
    />
    <span class="text-xs text-text-muted w-10 text-right tabular-nums">{{ modelValue }}</span>
  </div>
</template>
