import { ref, watch } from 'vue'

const leftCollapsed = ref(false)
const rightCollapsed = ref(false)
const bottomCollapsed = ref(false)
const defaultProviderId = ref<string | null>(null)

const SETTINGS_URL = 'http://127.0.0.1:1421/api/settings'

// 通过 HTTP API 直接读写 settings.json，避免 Vue composable 上下文限制
async function loadSettingsJson(): Promise<Record<string, unknown>> {
  try {
    const resp = await fetch(SETTINGS_URL)
    const json = await resp.json()
    if (json.success && json.data) {
      return JSON.parse(json.data)
    }
  } catch { /* server not ready */ }
  return {}
}

async function saveSettingsJson(data: Record<string, unknown>): Promise<void> {
  try {
    await fetch(SETTINGS_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ content: JSON.stringify(data) }),
    })
  } catch { /* server not ready */ }
}

// 变更 defaultProviderId → 持久化到 settings.json（异步不阻塞）
watch(defaultProviderId, async (val) => {
  const settings = await loadSettingsJson()
  settings.defaultProviderId = val
  await saveSettingsJson(settings)
})

// 启动时从 settings.json 恢复
let initPromise: Promise<void> | null = null
function ensureInit(): Promise<void> {
  if (!initPromise) {
    initPromise = loadSettingsJson().then((settings) => {
      if (settings.defaultProviderId) {
        defaultProviderId.value = settings.defaultProviderId as string
      }
    })
  }
  return initPromise
}

export function usePanelLayout() {
  ensureInit()
  return { leftCollapsed, rightCollapsed, bottomCollapsed, defaultProviderId }
}
