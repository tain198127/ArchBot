<script setup lang="ts">
import { inject } from 'vue'
import { Select } from 'primevue'
import { ChevronDown } from '@lucide/vue'
import { FORM_ITEM_ID } from './formKeys'

const inputId = inject(FORM_ITEM_ID, undefined)

interface Props {
  modelValue?: string | number | null
  options: { value: string | number; label: string }[]
  placeholder?: string
  disabled?: boolean
}

defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: string | number]
}>()
</script>

<template>
  <div class="relative">
    <Select
      :class="[
        'w-full px-3 py-2 pr-8 text-sm rounded border border-border-default bg-surface-0 text-text-primary',
        'focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500',
        'disabled:opacity-50 disabled:cursor-not-allowed',
        'dark:bg-surface-50 dark:text-text-primary',
      ]"
      :id="inputId"
      :model-value="modelValue"
      :options="options"
      option-label="label"
      option-value="value"
      :placeholder="placeholder"
      :disabled="disabled"
      @update:model-value="emit('update:modelValue', $event)"
    />
    <ChevronDown
      :size="14"
      class="pointer-events-none absolute right-2.5 top-1/2 -translate-y-1/2 text-text-muted"
    />
  </div>
</template>
