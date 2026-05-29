<script setup lang="ts">
import { inject } from 'vue'
import { InputText } from 'primevue'
import { FORM_ITEM_ID } from './formKeys'

const inputId = inject(FORM_ITEM_ID, undefined)

interface Props {
  modelValue?: string | null
  placeholder?: string
  disabled?: boolean
  type?: 'text' | 'password' | 'email' | 'number'
}

withDefaults(defineProps<Props>(), {
  type: 'text',
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>

<template>
  <InputText
    :id="inputId"
    :class="[
      'w-full px-3 py-2 text-sm rounded border border-border-default bg-surface-0 text-text-primary',
      'placeholder:text-text-muted',
      'focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500',
      'disabled:opacity-50 disabled:cursor-not-allowed disabled:bg-surface-50 dark:disabled:bg-surface-100',
      'dark:bg-surface-50 dark:text-text-primary',
    ]"
    :value="modelValue"
    :placeholder="placeholder"
    :disabled="disabled"
    :type="type"
    @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
  />
</template>
