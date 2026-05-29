import { createI18n, useI18n as useVueI18n } from 'vue-i18n'
import { computed } from 'vue'
import type { Ref, ComputedRef } from 'vue'
import zhCN from './zh-CN'
import enUS from './en-US'

type Messages = typeof zhCN
type Locale = 'zh-CN' | 'en-US'

const i18n = createI18n({
  legacy: false,
  locale: 'en-US',
  fallbackLocale: 'zh-CN',
  messages: { 'zh-CN': zhCN, 'en-US': enUS },
})

const rawMessages: Record<Locale, Messages> = {
  'zh-CN': zhCN,
  'en-US': enUS,
}

export function useI18n(): {
  t: ComputedRef<Messages>
  tt: (key: string) => string
  currentLocale: Ref<Locale>
  setLocale: (locale: Locale) => void
} {
  const { t: vut, locale } = useVueI18n()

  const t = computed<Messages>(() => rawMessages[(locale.value as Locale) || 'en-US'])

  function tt(key: string): string {
    const result = vut(key)
    return typeof result === 'string' ? result : key
  }

  function setLocale(loc: Locale) {
    (locale as Ref<Locale>).value = loc
  }

  return {
    t,
    tt,
    currentLocale: locale as Ref<Locale>,
    setLocale,
  }
}

export { i18n }
export type { Locale, Messages }
