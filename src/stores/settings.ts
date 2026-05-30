import { reactive, watch, toRefs } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../i18n'
import type { Locale } from '../i18n'

export interface ProxyConfig {
  enabled: boolean
  address: string
  port: string
  protocol: 'http' | 'https' | 'socks5'
}

export interface HttpServerConfig {
  enabled: boolean
  port: number
  bindLan: boolean
}

export interface AppSettings {
  locale: Locale
  fontSize: number
  fontFamily: string
  theme: 'light' | 'dark'
  aiLanguage: 'zh-CN' | 'en-US' | 'auto'
  proxy: ProxyConfig
  httpServer: HttpServerConfig
}

function getDefaults(): AppSettings {
  return {
    locale: 'zh-CN',
    fontSize: 13,
    fontFamily: '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    theme: 'dark',
    aiLanguage: 'auto',
    proxy: {
      enabled: false,
      address: '',
      port: '',
      protocol: 'http'
    },
    httpServer: {
      enabled: false,
      port: 1421,
      bindLan: false
    }
  }
}

const state = reactive<AppSettings>(getDefaults())

function applyTheme(theme: 'light' | 'dark') {
  document.documentElement.classList.toggle('dark', theme === 'dark')
}

function applyFont(fontSize: number, fontFamily: string) {
  document.documentElement.style.fontSize = `${fontSize}px`
  document.documentElement.style.fontFamily = fontFamily
}

async function persistSettings() {
  try {
    await invoke('save_settings', { content: JSON.stringify(state) })
  } catch {
    // ignore
  }
}

export function useSettings() {
  const { setLocale } = useI18n()

  watch(() => state.locale, (locale) => {
    setLocale(locale)
    persistSettings()
  })

  watch(() => state.theme, (theme) => {
    applyTheme(theme)
    persistSettings()
  })

  watch(() => state.fontSize, (size) => {
    applyFont(size, state.fontFamily)
    persistSettings()
  })

  watch(() => state.fontFamily, (family) => {
    applyFont(state.fontSize, family)
    persistSettings()
  })

  watch(() => state.aiLanguage, () => {
    persistSettings()
  })

  watch(() => state.proxy, () => {
    persistSettings()
  }, { deep: true })

  watch(() => state.httpServer, () => {
    persistSettings()
  }, { deep: true })

  /**
   * 初始化配置
   *
   * 业务逻辑：
   * 1. 通过 Tauri 命令从 ~/.ArchBot/settings.json 读取配置
   * 2. 配置文件存在且有效则合并到当前 state
   * 3. 配置文件不存在或无效则使用默认值
   * 4. 应用主题、字体、语言等视觉设置
   */
  async function initSettings() {
    try {
      const stored = await invoke<string>('load_settings')
      if (stored) {
        const parsed = JSON.parse(stored) as Partial<AppSettings>
        Object.assign(state, { ...getDefaults(), ...parsed })
      }
    } catch {
      // ignore
    }
    applyTheme(state.theme)
    applyFont(state.fontSize, state.fontFamily)
    setLocale(state.locale)
  }

  function resetSettings() {
    const defaults = getDefaults()
    Object.assign(state, defaults)
    applyTheme(defaults.theme)
    applyFont(defaults.fontSize, defaults.fontFamily)
    setLocale(defaults.locale)
    persistSettings()
  }

  return {
    settings: state,
    ...toRefs(state),
    initSettings,
    resetSettings,
    saveSettings: persistSettings
  }
}
