import { reactive, watch, toRefs } from 'vue'
import { useI18n } from '../i18n'
import type { Locale } from '../i18n'

export interface ProxyConfig {
  enabled: boolean
  address: string
  port: string
  protocol: 'http' | 'https' | 'socks5'
}

export interface AppSettings {
  locale: Locale
  fontSize: number
  fontFamily: string
  theme: 'light' | 'dark'
  aiLanguage: 'zh-CN' | 'en-US' | 'auto'
  proxy: ProxyConfig
}

const STORAGE_KEY = 'archbot-settings'

function loadSettings(): AppSettings {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) {
      return { ...getDefaults(), ...JSON.parse(stored) }
    }
  } catch {
    // ignore
  }
  return getDefaults()
}

function getDefaults(): AppSettings {
  return {
    locale: 'zh-CN',
    fontSize: 14,
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    theme: 'light',
    aiLanguage: 'auto',
    proxy: {
      enabled: false,
      address: '',
      port: '',
      protocol: 'http'
    }
  }
}

const state = reactive<AppSettings>(loadSettings())

function applyTheme(theme: 'light' | 'dark') {
  document.documentElement.setAttribute('data-theme', theme)
}

function applyFont(fontSize: number, fontFamily: string) {
  document.documentElement.style.fontSize = `${fontSize}px`
  document.documentElement.style.fontFamily = fontFamily
}

function saveSettings() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(state))
}

export function useSettings() {
  const { setLocale } = useI18n()

  watch(() => state.locale, (locale) => {
    setLocale(locale)
    saveSettings()
  })

  watch(() => state.theme, (theme) => {
    applyTheme(theme)
    saveSettings()
  })

  watch(() => state.fontSize, (size) => {
    applyFont(size, state.fontFamily)
    saveSettings()
  })

  watch(() => state.fontFamily, (family) => {
    applyFont(state.fontSize, family)
    saveSettings()
  })

  watch(() => state.aiLanguage, () => {
    saveSettings()
  })

  watch(() => state.proxy, () => {
    saveSettings()
  }, { deep: true })

  function initSettings() {
    applyTheme(state.theme)
    applyFont(state.fontSize, state.fontFamily)
    setLocale(state.locale)
  }

  return {
    settings: state,
    ...toRefs(state),
    initSettings,
    saveSettings
  }
}
