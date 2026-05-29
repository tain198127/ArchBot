<script setup lang="ts">
import { Dialog } from 'primevue'
import { X } from '@lucide/vue'

interface Props {
  visible: boolean
  title?: string
  width?: string
}

withDefaults(defineProps<Props>(), {
  width: '480px',
})

const emit = defineEmits<{
  'update:visible': [value: boolean]
}>()

function close() {
  emit('update:visible', false)
}
</script>

<template>
  <Dialog
    :visible="visible"
    :modal="true"
    :draggable="false"
    :style="{ width }"
    :pt="{
      root: { class: 'fixed inset-0 z-50 flex items-center justify-center' },
      mask: { class: 'fixed inset-0 bg-black/40' },
    }"
    @update:visible="emit('update:visible', $event)"
  >
    <div class="bg-white dark:bg-surface-0 rounded-lg shadow-xl max-h-[90vh] flex flex-col">
      <div class="flex items-center justify-between px-6 py-4 border-b border-border-default">
        <h2 class="text-base font-semibold text-text-primary">{{ title }}</h2>
        <button
          class="p-1 rounded hover:bg-surface-100 dark:hover:bg-surface-200 focus-visible:ring-2 focus-visible:ring-primary-500"
          @click="close"
          aria-label="Close"
        >
          <X :size="18" />
        </button>
      </div>
      <div class="px-6 py-4 overflow-y-auto flex-1">
        <slot />
      </div>
      <div v-if="$slots.footer" class="px-6 py-3 border-t border-border-default flex justify-end gap-2">
        <slot name="footer" />
      </div>
    </div>
  </Dialog>
</template>
