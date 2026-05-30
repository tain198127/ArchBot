<script setup lang="ts">
import { computed } from 'vue'
import VInput from '../base/VInput.vue'
import VSelect from '../base/VSelect.vue'
import VSlider from '../base/VSlider.vue'
import VSwitch from '../base/VSwitch.vue'
import VRadio from '../base/VRadio.vue'
import VFormItem from '../base/VFormItem.vue'
import VButton from '../base/VButton.vue'
import { useI18n } from '../../i18n'
import { useSettings } from '../../stores/settings'
import { useToast } from '../../composables/useToast'

const { t } = useI18n()
const toast = useToast()
const { settings, saveSettings, resetSettings } = useSettings()

const fontFamilyOptions = computed(() => [
  { value: '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif', label: t.value.settings.fontInter },
  { value: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif', label: t.value.settings.fontSystem },
  { value: '"JetBrains Mono", monospace', label: t.value.settings.fontJetBrains },
  { value: '"Fira Code", monospace', label: t.value.settings.fontFiraCode },
  { value: '"Cascadia Code", "Consolas", monospace', label: t.value.settings.fontCascadia },
  { value: '"Microsoft YaHei", "PingFang SC", sans-serif', label: t.value.settings.fontYaHei },
])

const protocolOptions = [
  { value: 'http', label: 'HTTP' },
  { value: 'https', label: 'HTTPS' },
  { value: 'socks5', label: 'SOCKS5' },
]

function handleSave() {
  saveSettings()
  toast.success(t.value.settings.saved)
}

async function handleReset() {
  const ok = await toast.confirm('', t.value.settings.resetConfirm)
  if (!ok) return
  resetSettings()
  toast.success(t.value.settings.resetDone)
}
</script>

<template>
  <div class="max-w-2xl mx-auto py-8 px-6">
    <h2 class="text-xl font-medium mb-7">{{ t.settings.title }}</h2>

    <!-- Language -->
    <section class="mb-7 pb-5 border-b border-border-default">
      <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.settings.language }}</h3>
      <div class="flex gap-4">
        <VRadio v-model="settings.locale" value="zh-CN" :label="t.settings.languageZh" />
        <VRadio v-model="settings.locale" value="en-US" :label="t.settings.languageEn" />
      </div>
    </section>

    <!-- Font Size -->
    <section class="mb-7 pb-5 border-b border-border-default">
      <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.settings.fontSize }}</h3>
      <VSlider v-model="settings.fontSize" :min="12" :max="24" :step="1" class="max-w-[400px]" />
    </section>

    <!-- Font Family -->
    <section class="mb-7 pb-5 border-b border-border-default">
      <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.settings.fontFamily }}</h3>
      <VSelect v-model="settings.fontFamily" :options="fontFamilyOptions" class="!w-[360px]" />
    </section>

    <!-- Theme -->
    <section class="mb-7 pb-5 border-b border-border-default">
      <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.settings.theme }}</h3>
      <div class="flex gap-4">
        <VRadio v-model="settings.theme" value="light" :label="t.settings.themeLight" />
        <VRadio v-model="settings.theme" value="dark" :label="t.settings.themeDark" />
      </div>
    </section>

    <!-- AI Language -->
    <section class="mb-7 pb-5 border-b border-border-default">
      <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.settings.aiLanguage }}</h3>
      <div class="flex gap-4">
        <VRadio v-model="settings.aiLanguage" value="auto" :label="t.settings.aiLanguageAuto" />
        <VRadio v-model="settings.aiLanguage" value="zh-CN" :label="t.settings.aiLanguageZh" />
        <VRadio v-model="settings.aiLanguage" value="en-US" :label="t.settings.aiLanguageEn" />
      </div>
    </section>

    <!-- Proxy -->
    <section class="mb-7 pb-5 border-b border-border-default">
      <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.settings.proxy }}</h3>
      <div class="flex flex-col gap-4">
        <label class="inline-flex items-center gap-2 text-sm text-text-primary cursor-pointer">
          <VSwitch v-model="settings.proxy.enabled" />
          <span>{{ t.settings.proxyEnable }}</span>
        </label>
        <div v-if="settings.proxy.enabled" class="pl-1">
          <VFormItem :label="t.settings.proxyProtocol">
            <VSelect v-model="settings.proxy.protocol" :options="protocolOptions" class="!w-[140px]" />
          </VFormItem>
          <VFormItem :label="t.settings.proxyAddress">
            <VInput v-model="settings.proxy.address" placeholder="127.0.0.1" class="!w-[260px]" />
          </VFormItem>
          <VFormItem :label="t.settings.proxyPort">
            <VInput v-model="settings.proxy.port" placeholder="7897" class="!w-[140px]" />
          </VFormItem>
        </div>
      </div>
    </section>

    <!-- Update -->
    <section class="mb-7 pb-5 border-b border-border-default">
      <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.settings.update }}</h3>
      <div class="flex items-center gap-4">
        <span class="text-sm text-text-secondary">{{ t.settings.updateCurrent }}: v0.1.0</span>
        <VButton>{{ t.settings.updateCheck }}</VButton>
      </div>
    </section>

    <!-- Actions -->
    <div class="mt-8 pt-5 border-t border-border-default">
      <VButton @click="handleSave">{{ t.settings.save }}</VButton>
      <VButton variant="secondary" class="ml-3" @click="handleReset">{{ t.settings.reset }}</VButton>
    </div>
  </div>
</template>
