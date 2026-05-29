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
      'w-full px-3 py-2 text-[13px] rounded-md border bg-surface-0 text-text-primary resize-none',
      'border-border-default hover:border-primary-300',
      'placeholder:text-text-muted',
      'transition-all duration-150',
      'focus:outline-none focus:border-primary-500 focus:ring-2 focus:ring-primary-500/20',
      'disabled:opacity-40 disabled:cursor-not-allowed',
      'dark:bg-surface-100 dark:text-text-primary',
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
