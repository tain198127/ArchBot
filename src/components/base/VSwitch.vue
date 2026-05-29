<script setup lang="ts">
import { inject } from 'vue'
import { FORM_ITEM_ID } from './formKeys'

const inputId = inject(FORM_ITEM_ID, undefined)

interface Props {
  modelValue?: boolean
  disabled?: boolean
}

defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()
</script>

<template>
  <button
    :id="inputId"
    role="switch"
    :aria-checked="modelValue"
    :class="[
      'relative inline-flex h-5 w-9 shrink-0 rounded-full transition-colors',
      'focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-1 focus-visible:outline-none',
      'disabled:opacity-50 disabled:cursor-not-allowed',
      modelValue ? 'bg-primary-500' : 'bg-surface-300 dark:bg-surface-200',
    ]"
    :disabled="disabled"
    @click="emit('update:modelValue', !modelValue)"
  >
    <span
      :class="[
        'inline-block h-4 w-4 rounded-full bg-white transition-transform mt-0.5',
        modelValue ? 'translate-x-[18px]' : 'translate-x-[2px]',
      ]"
    />
  </button>
</template>
