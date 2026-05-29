<script setup lang="ts">
import { inject } from 'vue'
import { Textarea } from 'primevue'
import { FORM_ITEM_ID } from './formKeys'

const inputId = inject(FORM_ITEM_ID, undefined)

interface Props {
  modelValue?: string
  placeholder?: string
  rows?: number
  disabled?: boolean
}

withDefaults(defineProps<Props>(), {
  rows: 3,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>

<template>
  <Textarea
    :class="[
      'w-full px-3 py-2 text-sm rounded border border-border-default bg-surface-0 text-text-primary resize-none',
      'placeholder:text-text-muted',
      'focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500',
      'disabled:opacity-50 disabled:cursor-not-allowed',
      'dark:bg-surface-50 dark:text-text-primary',
    ]"
    :id="inputId"
    :value="modelValue"
    :placeholder="placeholder"
    :rows="rows"
    :disabled="disabled"
    :auto-resize="false"
    @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
  />
</template>
