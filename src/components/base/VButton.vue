<script setup lang="ts">
import { Button } from 'primevue'

interface Props {
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost'
  size?: 'sm' | 'md' | 'lg'
  loading?: boolean
  disabled?: boolean
  type?: 'button' | 'submit' | 'reset'
}

withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  type: 'button',
})

defineEmits<{ click: [e: MouseEvent] }>()
</script>

<template>
  <Button
    :class="[
      'inline-flex items-center justify-center gap-1.5 font-medium rounded-md select-none',
      'transition-all duration-150 ease-out',
      'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500/40 focus-visible:ring-offset-1 focus-visible:ring-offset-surface-0',
      'disabled:opacity-40 disabled:cursor-not-allowed',
      variant === 'primary' && 'bg-primary-500 text-white hover:bg-primary-600 active:bg-primary-700 shadow-sm hover:shadow-md',
      variant === 'secondary' && 'bg-surface-100 dark:bg-surface-200 text-text-primary hover:bg-surface-200 dark:hover:bg-surface-300 active:bg-surface-300 border border-border-default',
      variant === 'danger' && 'bg-danger-500 text-white hover:bg-danger-600 active:opacity-90 shadow-sm',
      variant === 'ghost' && 'text-text-secondary hover:text-text-primary hover:bg-surface-100 dark:hover:bg-surface-200 active:bg-surface-200',
      size === 'sm' && 'px-2.5 py-1 text-xs rounded-sm',
      size === 'md' && 'px-4 py-1.5 text-[13px]',
      size === 'lg' && 'px-5 py-2 text-sm',
    ]"
    :disabled="disabled || loading"
    :type="type"
    @click="$emit('click', $event)"
  >
    <svg
      v-if="loading"
      class="animate-spin -ml-0.5 h-3.5 w-3.5"
      fill="none"
      viewBox="0 0 24 24"
    >
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
    </svg>
    <slot />
  </Button>
</template>
