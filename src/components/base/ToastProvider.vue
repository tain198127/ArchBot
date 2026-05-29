<script setup lang="ts">
import { provide, ref } from 'vue'
import { TOAST_KEY, getToastState } from '../../composables/useToast'
import type { ToastItem, ToastAPI } from '../../composables/useToast'
import { useI18n } from '../../i18n'
import { X, CheckCircle, AlertCircle, Info, AlertTriangle } from '@lucide/vue'

const { t } = useI18n()
const cm = t.value.common as Record<string, string>

const { toasts, pushToast } = getToastState()

const confirmResolve = ref<((v: boolean) => void) | null>(null)
const confirmVisible = ref(false)
const confirmTitle = ref('')
const confirmMessage = ref('')

function confirm(title: string, message: string): Promise<boolean> {
  confirmTitle.value = title
  confirmMessage.value = message
  confirmVisible.value = true
  return new Promise((resolve) => { confirmResolve.value = resolve })
}

function onConfirm(result: boolean) {
  confirmVisible.value = false
  confirmResolve.value?.(result)
  confirmResolve.value = null
}

const api: ToastAPI = {
  success: (m) => pushToast('success', m),
  error: (m) => pushToast('error', m),
  info: (m) => pushToast('info', m),
  warning: (m) => pushToast('warning', m),
  confirm,
  toasts,
}
provide(TOAST_KEY, api)

const iconMap: Record<ToastItem['type'], typeof CheckCircle> = {
  success: CheckCircle,
  error: AlertCircle,
  info: Info,
  warning: AlertTriangle,
}
</script>

<template>
  <div
    v-if="toasts.length"
    class="fixed top-4 right-4 z-[9999] flex flex-col gap-2 max-w-sm"
    role="alert"
    aria-live="polite"
  >
    <div
      v-for="t in toasts"
      :key="t.id"
      :class="[
        'flex items-center gap-2 px-4 py-3 rounded-lg shadow-lg text-sm font-medium animate-slide-in',
        t.type === 'success' && 'bg-green-600 text-white dark:bg-green-700',
        t.type === 'error' && 'bg-red-600 text-white dark:bg-red-700',
        t.type === 'info' && 'bg-blue-600 text-white dark:bg-blue-700',
        t.type === 'warning' && 'bg-yellow-500 text-white dark:bg-yellow-600',
      ]"
    >
      <component :is="iconMap[t.type]" :size="16" />
      <span>{{ t.message }}</span>
      <button
        class="ml-auto p-1 rounded hover:bg-white/20 focus-visible:ring-2 focus-visible:ring-white"
        @click="toasts = toasts.filter((x) => x.id !== t.id)"
        :aria-label="cm.dismiss"
      >
        <X :size="14" />
      </button>
    </div>
  </div>

  <Teleport to="body">
    <div
      v-if="confirmVisible"
      class="fixed inset-0 z-[9998] flex items-center justify-center bg-black/40"
      @keydown.escape="onConfirm(false)"
    >
      <div
        class="bg-white dark:bg-surface-100 rounded-lg shadow-xl p-6 max-w-sm w-full mx-4"
        role="alertdialog"
        aria-modal="true"
        :aria-label="confirmTitle"
      >
        <h3 class="text-base font-semibold text-text-primary">{{ confirmTitle }}</h3>
        <p class="mt-2 text-sm text-text-secondary">{{ confirmMessage }}</p>
        <div class="mt-4 flex justify-end gap-2">
          <button
            class="px-4 py-2 text-sm rounded bg-surface-100 dark:bg-surface-200 text-text-primary hover:bg-surface-200 dark:hover:bg-surface-300 focus-visible:ring-2 focus-visible:ring-primary-500"
            @click="onConfirm(false)"
          >
            {{ cm.cancel }}
          </button>
          <button
            class="px-4 py-2 text-sm rounded bg-primary-500 text-white hover:bg-primary-600 focus-visible:ring-2 focus-visible:ring-primary-500"
            @click="onConfirm(true)"
          >
            {{ cm.confirm }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
