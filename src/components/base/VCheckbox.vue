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
      'inline-flex items-center gap-2 text-[13px] text-text-primary cursor-pointer select-none group',
      disabled && 'opacity-40 cursor-not-allowed',
    ]"
    @click.prevent="!disabled && emit('update:modelValue', !modelValue)"
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
        'inline-flex items-center justify-center h-[18px] w-[18px] rounded-sm border shrink-0 transition-all duration-150',
        modelValue
          ? 'bg-primary-500 border-primary-500 shadow-sm shadow-primary-500/20'
          : 'border-border-default bg-surface-0 dark:bg-surface-100',
        !disabled && 'group-hover:border-primary-400',
      ]"
    >
      <Transition name="fade">
        <Check v-if="modelValue" :size="13" class="text-white" stroke-width="3" />
      </Transition>
    </span>
    <span v-if="label">{{ label }}</span>
    <slot v-else />
  </label>
</template>
