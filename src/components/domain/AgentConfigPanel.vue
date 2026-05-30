<script setup lang="ts">
import { reactive, ref, computed } from 'vue'
import VTabs from '../base/VTabs.vue'
import VButton from '../base/VButton.vue'
import VInput from '../base/VInput.vue'
import VFormItem from '../base/VFormItem.vue'
import { useI18n } from '../../i18n'
import { useToast } from '../../composables/useToast'
import { invoke } from '@tauri-apps/api/core'

const { t } = useI18n()
const toast = useToast()

// ── Runtime tabs ──
const tabs = [
  { value: 'claude_code',  label: 'Claude Code' },
  { value: 'hermes',       label: 'Hermes' },
  { value: 'opencode',     label: 'OpenCode' },
  { value: 'openclaw',     label: 'OpenClaw' },
]
const activeTab = ref('claude_code')

// ── Per-runtime state ──
interface AgentState {
  installed: boolean
  installedVersion: string
  availableVersions: string[]
  selectedVersion: string
  protocol: 'anthropic' | 'openai'
  baseUrl: string
  modelDefault: string
  modelSmall: string
  modelLarge: string
  modelName: string
  apiKey: string
  extraArgs: string
  installLoading: boolean
  updateLoading: boolean
  validateLoading: boolean
  validateResult: string
}

const initState = (): AgentState => ({
  installed: false,
  installedVersion: '',
  availableVersions: [],
  selectedVersion: '',
  protocol: 'anthropic',
  baseUrl: '',
  modelDefault: '',
  modelSmall: '',
  modelLarge: '',
  modelName: '',
  apiKey: '',
  extraArgs: '',
  installLoading: false,
  updateLoading: false,
  validateLoading: false,
  validateResult: '',
})

const state = reactive<Record<string, AgentState>>({
  claude_code: initState(),
  hermes: initState(),
  opencode: initState(),
  openclaw: initState(),
})

const current = computed(() => state[activeTab.value])

// ── Defaults per runtime ──
const defaults: Record<string, { protocol: 'anthropic'|'openai'; baseUrl: string } & (
  | { modelDefault: string; modelSmall: string; modelLarge: string }
  | { modelName: string }
)> = {
  claude_code: {
    protocol: 'anthropic',
    baseUrl: 'https://api.anthropic.com',
    modelDefault: 'claude-sonnet-4-6',
    modelSmall: 'claude-haiku-4-5',
    modelLarge: 'claude-opus-4-7',
  },
  hermes:   { protocol: 'openai', baseUrl: 'https://api.openai.com/v1', modelName: 'gpt-5.2' },
  opencode: { protocol: 'openai', baseUrl: 'https://api.openai.com/v1', modelName: 'gpt-5.2' },
  openclaw: { protocol: 'openai', baseUrl: 'https://api.openai.com/v1', modelName: 'gpt-5.2' },
}

// ── Lifecycle: 切换 tab 时加载该 Runtime 的状态 ──
async function refreshRuntimeStatus(runtime: string) {
  const d = defaults[runtime]
  if (d) {
    state[runtime].protocol = d.protocol
    state[runtime].baseUrl = d.baseUrl
    if ('modelDefault' in d) {
      state[runtime].modelDefault = d.modelDefault
      state[runtime].modelSmall = d.modelSmall
      state[runtime].modelLarge = d.modelLarge
    } else {
      state[runtime].modelName = d.modelName || ''
    }
  }

  try {
    const result: any = await invoke('agent_get_status', { runtime })
    state[runtime].installed = result.installed ?? false
    state[runtime].installedVersion = result.installed_version ?? ''
    state[runtime].availableVersions = result.available_versions ?? []
    state[runtime].selectedVersion = result.installed_version ?? ''
    if (result.config) {
      state[runtime].baseUrl = result.config.base_url || state[runtime].baseUrl
      state[runtime].protocol = result.config.protocol || state[runtime].protocol
      state[runtime].extraArgs = result.config.extra_args || ''
      if (result.config.protocol === 'anthropic') {
        state[runtime].modelDefault = result.config.model_default || state[runtime].modelDefault
        state[runtime].modelSmall = result.config.model_small || state[runtime].modelSmall
        state[runtime].modelLarge = result.config.model_large || state[runtime].modelLarge
      } else {
        state[runtime].modelName = result.config.model_name || state[runtime].modelName
      }
    }
  } catch {
    // backend not ready — keep defaults
  }
}

function switchTab(runtime: string) {
  activeTab.value = runtime
  refreshRuntimeStatus(runtime)
}

// ── Actions ──
async function installRuntime(runtime: string) {
  const s = state[runtime]
  s.installLoading = true
  s.validateResult = ''
  try {
    await invoke('agent_install', { runtime, version: s.selectedVersion || undefined })
    s.installed = true
    s.installedVersion = s.selectedVersion
    toast.success(`${tabs.find(x => x.value === runtime)?.label} installed`)
  } catch (e: any) {
    toast.error(String(e))
  } finally {
    s.installLoading = false
  }
}

async function updateRuntime(runtime: string) {
  const s = state[runtime]
  s.updateLoading = true
  s.validateResult = ''
  try {
    await invoke('agent_update', { runtime })
    await refreshRuntimeStatus(runtime)
    toast.success(`${tabs.find(x => x.value === runtime)?.label} updated`)
  } catch (e: any) {
    toast.error(String(e))
  } finally {
    s.updateLoading = false
  }
}

async function saveConfig(runtime: string) {
  const s = state[runtime]
  try {
    await invoke('agent_save_config', {
      runtime,
      config: {
        protocol: s.protocol,
        base_url: s.baseUrl,
        model_default: s.modelDefault,
        model_small: s.modelSmall,
        model_large: s.modelLarge,
        model_name: s.modelName,
        extra_args: s.extraArgs,
      },
    })
    toast.success(t.value.common.confirm)
  } catch (e: any) {
    toast.error(String(e))
  }
}

async function saveApiKey(runtime: string) {
  const s = state[runtime]
  if (!s.apiKey.trim()) return
  try {
    await invoke('agent_save_secret', { runtime, key: 'api_token', value: s.apiKey })
    s.apiKey = ''
    toast.success('API key saved')
  } catch (e: any) {
    toast.error(String(e))
  }
}

async function validateRuntime(runtime: string) {
  const s = state[runtime]
  s.validateLoading = true
  s.validateResult = ''
  try {
    const result: any = await invoke('agent_validate', { runtime })
    s.validateResult = result.ok ? '✅ Connected' : `❌ ${result.error || 'Unknown error'}`
  } catch (e: any) {
    s.validateResult = `❌ ${e}`
  } finally {
    s.validateLoading = false
  }
}
</script>

<template>
  <div class="flex flex-col h-full bg-surface-0">
    <!-- Header -->
    <div class="px-5 py-3 border-b border-border-default">
      <h2 class="text-[15px] font-semibold text-text-primary">{{ t.menuConfig.agentConfig }}</h2>
      <p class="text-xs text-text-secondary mt-0.5">{{ t.agentConfig.subtitle }}</p>
    </div>

    <!-- Runtime tabs -->
    <div class="flex-1 overflow-auto">
      <VTabs v-model="activeTab" :tabs="tabs" @update:model-value="switchTab">
        <template v-for="rt in tabs.map(x => x.value)" :key="rt" #[rt]>
          <div class="p-5 space-y-6">

            <!-- ========== Section 1: Install ========== -->
            <section>
              <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.agentConfig.installTitle }}</h3>
              <div class="flex items-center gap-3">
                <span class="text-sm text-text-secondary">
                  {{ t.agentConfig.status }}:
                  <span :class="current.installed ? 'text-emerald-500' : 'text-text-muted'">
                    {{ current.installed ? `${t.agentConfig.installed} (${current.installedVersion})` : t.agentConfig.notInstalled }}
                  </span>
                </span>
              </div>
              <div class="flex items-center gap-2 mt-2">
                <select
                  v-model="current.selectedVersion"
                  class="h-8 rounded-md border border-border-default bg-surface-100 px-2 text-[13px] text-text-primary"
                >
                  <option value="">{{ t.agentConfig.selectVersion }}</option>
                  <option v-for="v in current.availableVersions" :key="v" :value="v">{{ v }}</option>
                </select>
                <VButton size="sm" :loading="current.installLoading" @click="installRuntime(rt)">
                  {{ t.agentConfig.install }}
                </VButton>
              </div>
            </section>

            <!-- ========== Section 2: Update ========== -->
            <section>
              <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.agentConfig.updateTitle }}</h3>
              <VButton size="sm" variant="secondary" :loading="current.updateLoading" :disabled="!current.installed" @click="updateRuntime(rt)">
                {{ t.agentConfig.checkUpdate }}
              </VButton>
            </section>

            <!-- ========== Section 3: Version ========== -->
            <section>
              <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.agentConfig.versionTitle }}</h3>
              <select
                v-model="current.selectedVersion"
                class="h-8 rounded-md border border-border-default bg-surface-100 px-2 text-[13px] text-text-primary min-w-[200px]"
              >
                <option value="">{{ current.installedVersion || t.agentConfig.selectVersion }}</option>
                <option v-for="v in current.availableVersions" :key="v" :value="v">{{ v }}</option>
              </select>
            </section>

            <!-- ========== Section 4: Model Config ========== -->
            <section>
              <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.agentConfig.modelTitle }}</h3>
              <div class="space-y-3 max-w-[520px]">
                <!-- Protocol -->
                <VFormItem :label="t.agentConfig.protocol">
                  <select
                    v-model="current.protocol"
                    class="h-8 rounded-md border border-border-default bg-surface-100 px-2 text-[13px] text-text-primary w-full"
                  >
                    <option value="anthropic">Anthropic-compatible</option>
                    <option value="openai">OpenAI-compatible</option>
                  </select>
                </VFormItem>

                <!-- Base URL -->
                <VFormItem :label="t.agentConfig.baseUrl">
                  <VInput v-model="current.baseUrl" :placeholder="t.agentConfig.baseUrlPlaceholder" size="sm" />
                </VFormItem>

                <!-- Anthropic: three-tier model config -->
                <template v-if="current.protocol === 'anthropic'">
                  <VFormItem :label="t.agentConfig.modelDefault">
                    <VInput v-model="current.modelDefault" placeholder="claude-sonnet-4-6" size="sm" />
                    <span class="text-[11px] text-text-muted">{{ t.agentConfig.modelDefaultDesc }}</span>
                  </VFormItem>
                  <VFormItem :label="t.agentConfig.modelSmall">
                    <VInput v-model="current.modelSmall" placeholder="claude-haiku-4-5" size="sm" />
                    <span class="text-[11px] text-text-muted">{{ t.agentConfig.modelSmallDesc }}</span>
                  </VFormItem>
                  <VFormItem :label="t.agentConfig.modelLarge">
                    <VInput v-model="current.modelLarge" placeholder="claude-opus-4-7" size="sm" />
                    <span class="text-[11px] text-text-muted">{{ t.agentConfig.modelLargeDesc }}</span>
                  </VFormItem>
                </template>

                <!-- OpenAI: single model -->
                <template v-else>
                  <VFormItem :label="t.agentConfig.modelName">
                    <VInput v-model="current.modelName" :placeholder="t.agentConfig.modelNamePlaceholder" size="sm" />
                  </VFormItem>
                </template>

                <!-- Extra Args -->
                <VFormItem :label="t.agentConfig.extraArgs">
                  <VInput v-model="current.extraArgs" :placeholder="t.agentConfig.extraArgsPlaceholder" size="sm" />
                </VFormItem>

                <VButton size="sm" @click="saveConfig(rt)">{{ t.agentConfig.saveConfig }}</VButton>
              </div>
            </section>

            <!-- ========== Section 5: API Key ========== -->
            <section>
              <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.agentConfig.apiKeyTitle }}</h3>
              <div class="flex items-center gap-2 max-w-[520px]">
                <VInput v-model="current.apiKey" type="password" :placeholder="t.agentConfig.apiKeyPlaceholder" size="sm" class="flex-1" />
                <VButton size="sm" @click="saveApiKey(rt)">{{ t.agentConfig.saveApiKey }}</VButton>
              </div>
            </section>

            <!-- ========== Section 6: Validate ========== -->
            <section>
              <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.agentConfig.validateTitle }}</h3>
              <div class="flex items-center gap-3">
                <VButton size="sm" variant="secondary" :loading="current.validateLoading" :disabled="!current.installed" @click="validateRuntime(rt)">
                  {{ t.agentConfig.validate }}
                </VButton>
                <span v-if="current.validateResult" class="text-[13px]" :class="current.validateResult.startsWith('✅') ? 'text-emerald-500' : 'text-red-400'">
                  {{ current.validateResult }}
                </span>
              </div>
            </section>

          </div>
        </template>
      </VTabs>
    </div>
  </div>
</template>
