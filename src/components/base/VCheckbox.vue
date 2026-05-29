<script setup lang="ts">
import { Check } from '@lucide/vue'

interface Props {
  modelValue?: boolean
  label?: string
  disabled?: boolean
}

defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

</script>

<template>
  <label
    :class="[
      'inline-flex items-center gap-2 text-sm text-text-primary cursor-pointer',
      disabled && 'opacity-50 cursor-not-allowed',
    ]"
    @click.prevent="emit('update:modelValue', !modelValue)"
  >
    <input
      type="checkbox"
      :checked="modelValue"
      :disabled="disabled"
      class="sr-only"
      @change="emit('update:modelValue', ($event.target as HTMLInputElement).checked)"
    />
    <span
      :class="[
        'inline-flex items-center justify-center h-4 w-4 rounded border shrink-0 transition-colors',
        modelValue
          ? 'bg-primary-500 border-primary-500'
          : 'border-border-default dark:border-border-light bg-surface-0 dark:bg-surface-50',
        !disabled && 'group-hover:border-primary-400',
      ]"
    >
      <Check v-if="modelValue" :size="12" class="text-white" />
    </span>
    <span v-if="label">{{ label }}</span>
    <slot v-else />
  </label>
</template>
