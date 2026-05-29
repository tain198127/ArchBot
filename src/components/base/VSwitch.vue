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
      'relative inline-flex h-5 w-9 shrink-0 rounded-full transition-all duration-200',
      'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500/40 focus-visible:ring-offset-1',
      'disabled:opacity-40 disabled:cursor-not-allowed',
      modelValue ? 'bg-primary-500' : 'bg-surface-300 dark:bg-surface-300',
    ]"
    :disabled="disabled"
    @click="emit('update:modelValue', !modelValue)"
  >
    <span
      :class="[
        'inline-block h-[18px] w-[18px] rounded-full bg-white shadow-sm transition-transform duration-200',
        modelValue ? 'translate-x-[17px] mt-px' : 'translate-x-px mt-px',
      ]"
    />
  </button>
</template>
