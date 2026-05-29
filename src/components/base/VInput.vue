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
      'w-full px-3 py-1.5 text-[13px] rounded-md border bg-surface-0 text-text-primary',
      'border-border-default hover:border-primary-300',
      'placeholder:text-text-muted',
      'transition-all duration-150',
      'focus:outline-none focus:border-primary-500 focus:ring-2 focus:ring-primary-500/20',
      'disabled:opacity-40 disabled:cursor-not-allowed disabled:bg-surface-50 dark:disabled:bg-surface-100',
      'dark:bg-surface-100 dark:text-text-primary',
    ]"
    :value="modelValue"
    :placeholder="placeholder"
    :disabled="disabled"
    :type="type"
    @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
  />
</template>
