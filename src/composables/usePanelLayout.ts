import { ref } from 'vue'

const leftCollapsed = ref(false)
const rightCollapsed = ref(false)
const bottomCollapsed = ref(false)
const defaultProviderId = ref<string | null>(null)

export function usePanelLayout() {
  return { leftCollapsed, rightCollapsed, bottomCollapsed, defaultProviderId }
}
