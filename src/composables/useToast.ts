import { inject, ref } from 'vue'
import type { InjectionKey, Ref } from 'vue'

export interface ToastItem {
  id: number
  type: 'success' | 'error' | 'info' | 'warning'
  message: string
}

export interface ToastAPI {
  success: (msg: string) => void
  error: (msg: string) => void
  info: (msg: string) => void
  warning: (msg: string) => void
  confirm: (title: string, message: string) => Promise<boolean>
  toasts: Ref<ToastItem[]>
}

export const TOAST_KEY: InjectionKey<ToastAPI> = Symbol('toast')

const toasts = ref<ToastItem[]>([])
let nextId = 0

export function useToast(): ToastAPI {
  const api = inject(TOAST_KEY, null)
  if (api) return api

  return {
    success: (msg) => pushToast('success', msg),
    error: (msg) => pushToast('error', msg),
    info: (msg) => pushToast('info', msg),
    warning: (msg) => pushToast('warning', msg),
    confirm: (_title, _message) => Promise.resolve(false),
    toasts,
  }
}

function pushToast(type: ToastItem['type'], message: string) {
  const id = ++nextId
  toasts.value = [...toasts.value, { id, type, message }]
  setTimeout(() => {
    toasts.value = toasts.value.filter((t) => t.id !== id)
  }, 4000)
}

export function getToastState() {
  return { toasts, nextId, pushToast }
}
