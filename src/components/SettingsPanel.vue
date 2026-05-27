<script setup lang="ts">
import { useI18n } from '../i18n'
import { useSettings } from '../stores/settings'
import { ElMessage, ElMessageBox } from 'element-plus'

const { t } = useI18n()
const { settings, saveSettings, resetSettings } = useSettings()

const fontFamilyOptions = [
  { value: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif', label: 'System Default' },
  { value: '"JetBrains Mono", "Fira Code", monospace', label: 'JetBrains Mono' },
  { value: '"Fira Code", "Cascadia Code", monospace', label: 'Fira Code' },
  { value: '"Cascadia Code", "Consolas", monospace', label: 'Cascadia Code' },
  { value: '"Source Code Pro", monospace', label: 'Source Code Pro' },
  { value: '"Microsoft YaHei", "PingFang SC", sans-serif', label: 'Microsoft YaHei' }
]

function handleSave() {
  saveSettings()
  ElMessage.success(t.value.settings.saved)
}

async function handleReset() {
  try {
    await ElMessageBox.confirm(t.value.settings.resetConfirm, '', { type: 'warning' })
    resetSettings()
    ElMessage.success(t.value.settings.resetDone)
  } catch {
    // cancelled
  }
}
</script>

<template>
  <div class="settings-page">
    <h2 class="settings-title">{{ t.settings.title }}</h2>

    <div class="settings-section">
      <h3>{{ t.settings.language }}</h3>
      <el-radio-group v-model="settings.locale">
        <el-radio value="zh-CN">{{ t.settings.languageZh }}</el-radio>
        <el-radio value="en-US">{{ t.settings.languageEn }}</el-radio>
      </el-radio-group>
    </div>

    <div class="settings-section">
      <h3>{{ t.settings.fontSize }}</h3>
      <div class="setting-row">
        <el-slider
          v-model="settings.fontSize"
          :min="12"
          :max="24"
          :step="1"
          show-input
          style="max-width: 400px"
        />
      </div>
    </div>

    <div class="settings-section">
      <h3>{{ t.settings.fontFamily }}</h3>
      <el-select v-model="settings.fontFamily" style="width: 360px">
        <el-option
          v-for="opt in fontFamilyOptions"
          :key="opt.value"
          :value="opt.value"
          :label="opt.label"
        />
      </el-select>
    </div>

    <div class="settings-section">
      <h3>{{ t.settings.theme }}</h3>
      <el-radio-group v-model="settings.theme">
        <el-radio value="light">{{ t.settings.themeLight }}</el-radio>
        <el-radio value="dark">{{ t.settings.themeDark }}</el-radio>
      </el-radio-group>
    </div>

    <div class="settings-section">
      <h3>{{ t.settings.aiLanguage }}</h3>
      <el-radio-group v-model="settings.aiLanguage">
        <el-radio value="auto">{{ t.settings.aiLanguageAuto }}</el-radio>
        <el-radio value="zh-CN">{{ t.settings.aiLanguageZh }}</el-radio>
        <el-radio value="en-US">{{ t.settings.aiLanguageEn }}</el-radio>
      </el-radio-group>
    </div>

    <div class="settings-section">
      <h3>{{ t.settings.proxy }}</h3>
      <div class="proxy-form">
        <el-switch v-model="settings.proxy.enabled" :active-text="t.settings.proxyEnable" />
        <div v-if="settings.proxy.enabled" class="proxy-fields">
          <el-form label-width="80px" label-position="left">
            <el-form-item :label="t.settings.proxyProtocol">
              <el-select v-model="settings.proxy.protocol" style="width: 140px">
                <el-option value="http" label="HTTP" />
                <el-option value="https" label="HTTPS" />
                <el-option value="socks5" label="SOCKS5" />
              </el-select>
            </el-form-item>
            <el-form-item :label="t.settings.proxyAddress">
              <el-input v-model="settings.proxy.address" placeholder="127.0.0.1" style="width: 260px" />
            </el-form-item>
            <el-form-item :label="t.settings.proxyPort">
              <el-input v-model="settings.proxy.port" placeholder="7897" style="width: 140px" />
            </el-form-item>
          </el-form>
        </div>
      </div>
    </div>

    <div class="settings-section">
      <h3>{{ t.settings.update }}</h3>
      <div class="update-row">
        <span class="version-text">{{ t.settings.updateCurrent }}: v0.1.0</span>
        <el-button type="primary" size="default">{{ t.settings.updateCheck }}</el-button>
      </div>
    </div>

    <div class="settings-actions">
      <el-button type="primary" @click="handleSave">{{ t.settings.save }}</el-button>
      <el-button @click="handleReset">{{ t.settings.reset }}</el-button>
    </div>
  </div>
</template>

<style scoped>
.settings-page {
  max-width: 640px;
  margin: 0 auto;
  padding: 32px 24px;
}

.settings-title {
  font-size: 20px;
  font-weight: 500;
  margin-bottom: 28px;
}

.settings-section {
  margin-bottom: 28px;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--border-color, #eee);
}

.settings-section h3 {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 12px;
  color: var(--text-primary, #333);
}

.setting-row {
  display: flex;
  align-items: center;
}

.proxy-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.proxy-fields {
  padding-left: 4px;
}

.update-row {
  display: flex;
  align-items: center;
  gap: 16px;
}

.version-text {
  font-size: 13px;
  color: var(--text-secondary, #666);
}

.settings-actions {
  margin-top: 32px;
  padding-top: 20px;
  border-top: 1px solid var(--border-color, #eee);
}
</style>
