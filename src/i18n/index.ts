import { ref, computed } from 'vue'
import zhCN from './zh-CN'
import enUS from './en-US'

type Messages = typeof zhCN
type Locale = 'zh-CN' | 'en-US'

const messages: Record<Locale, Messages> = {
  'zh-CN': zhCN,
  'en-US': enUS
}

const currentLocale = ref<Locale>('zh-CN')

export function useI18n() {
  const t = computed(() => messages[currentLocale.value])

  function setLocale(locale: Locale) {
    currentLocale.value = locale
  }

  return { t, currentLocale, setLocale }
}

export type { Locale, Messages }
