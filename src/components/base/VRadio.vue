<script setup lang="ts">
import { inject } from 'vue'
import { RADIO_GROUP_NAME } from './formKeys'

const groupName = inject(RADIO_GROUP_NAME, undefined)

interface Props {
  value: string
  modelValue?: string
  label?: string
  disabled?: boolean
}

defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>

<template>
  <label
    :class="[
      'inline-flex items-center gap-2 text-[13px] text-text-primary cursor-pointer select-none',
      disabled && 'opacity-40 cursor-not-allowed',
    ]"
  >
    <input
      type="radio"
      :name="groupName"
      :value="value"
      :checked="modelValue === value"
      :disabled="disabled"
      class="sr-only"
      @change="emit('update:modelValue', value)"
    />
    <span
      :class="[
        'inline-flex h-[18px] w-[18px] rounded-full border-2 items-center justify-center shrink-0 transition-all duration-150',
        modelValue === value
          ? 'border-primary-500'
          : 'border-border-default',
      ]"
    >
      <span
        v-if="modelValue === value"
        class="inline-block h-2 w-2 rounded-full bg-primary-500 transition-transform duration-150 scale-100"
      />
    </span>
    <span v-if="label">{{ label }}</span>
    <slot v-else />
  </label>
</template>
