<script setup lang="ts">
import { inject } from 'vue'
import { Select } from 'primevue'
import { ChevronDown } from '@lucide/vue'
import { FORM_ITEM_ID } from './formKeys'

const inputId = inject(FORM_ITEM_ID, undefined)

interface Option {
  value: string | number
  label: string
  group?: string
}

interface Props {
  modelValue?: string | number | null
  options: Option[]
  placeholder?: string
  disabled?: boolean
  loading?: boolean
  optionGroupLabel?: string
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
        'w-full pl-3 pr-8 py-1.5 text-[13px] rounded-md border bg-surface-0 text-text-primary',
        'border-border-default hover:border-primary-300',
        'transition-all duration-150',
        'focus:outline-none focus:border-primary-500 focus:ring-2 focus:ring-primary-500/20',
        'disabled:opacity-40 disabled:cursor-not-allowed',
        'dark:bg-surface-100 dark:text-text-primary',
      ]"
      :pt="{
        overlay: { class: 'bg-surface-0 dark:bg-surface-50 border border-border-default rounded-lg shadow-lg py-1' },
        list: { class: 'm-0 p-0 list-none' },
        option: { class: 'px-3 py-1.5 text-[13px] text-text-primary cursor-pointer hover:bg-surface-50 dark:hover:bg-surface-100 transition-colors' },
        optionGroup: { class: '' },
        emptyMessage: { class: 'px-3 py-2 text-[12px] text-text-muted' },
        dropdownIcon: { class: 'hidden' },
      }"
      :id="inputId"
      :model-value="modelValue"
      :options="options"
      option-label="label"
      option-value="value"
      :option-group-label="optionGroupLabel"
      :loading="loading"
      :placeholder="placeholder"
      :disabled="disabled"
      @update:model-value="emit('update:modelValue', $event)"
    />
    <ChevronDown
      :size="14"
      class="pointer-events-none absolute right-2.5 top-1/2 -translate-y-1/2 text-text-muted transition-transform duration-150"
    />
  </div>
</template>
