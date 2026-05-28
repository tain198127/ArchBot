import { ref } from 'vue'

export interface MenuActionPayload {
  action: string
  payload?: Record<string, string>
}

export type MenuAction = string | MenuActionPayload

const listeners = ref<Array<(action: MenuAction) => void>>([])

export function useMenuAction() {
  function emit(action: MenuAction) {
    listeners.value.forEach(fn => fn(action))
  }

  function on(fn: (action: MenuAction) => void) {
    listeners.value.push(fn)
    return () => {
      listeners.value = listeners.value.filter(l => l !== fn)
    }
  }

  return { emit, on }
}

export function getAction(action: MenuAction): string {
  return typeof action === 'string' ? action : action.action
}

export function getPayload(action: MenuAction): Record<string, string> | undefined {
  return typeof action === 'string' ? undefined : action.payload
}
