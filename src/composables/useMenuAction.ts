import { ref } from 'vue'

export type MenuAction = string

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
