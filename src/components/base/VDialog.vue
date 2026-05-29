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
      mask: { class: 'fixed inset-0 bg-black/50 backdrop-blur-[2px]' },
    }"
    @update:visible="emit('update:visible', $event)"
  >
    <Transition name="dialog">
      <div v-if="visible" class="dialog-panel bg-white dark:bg-surface-50 rounded-xl shadow-xl max-h-[85vh] flex flex-col ring-1 ring-border-default/50">
        <div class="flex items-center justify-between px-5 py-3.5 border-b border-border-default">
          <h2 class="text-sm font-semibold text-text-primary">{{ title }}</h2>
          <button
            class="p-1.5 rounded-md hover:bg-surface-100 dark:hover:bg-surface-200 transition-colors text-text-muted hover:text-text-primary"
            @click="close"
            aria-label="Close"
          >
            <X :size="16" />
          </button>
        </div>
        <div class="px-5 py-4 overflow-y-auto flex-1">
          <slot />
        </div>
        <div v-if="$slots.footer" class="px-5 py-3 border-t border-border-default flex justify-end gap-2">
          <slot name="footer" />
        </div>
      </div>
    </Transition>
  </Dialog>
</template>
