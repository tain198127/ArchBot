<script setup lang="ts">
import { reactive, ref, computed, onMounted } from 'vue'
import VTabs from '../base/VTabs.vue'
import VButton from '../base/VButton.vue'
import VInput from '../base/VInput.vue'
import VSelect from '../base/VSelect.vue'
import { useI18n } from '../../i18n'
import { useToast } from '../../composables/useToast'
import { pushLog } from '../../stores/log'
import { invoke } from '@tauri-apps/api/core'

const { t } = useI18n()
const toast = useToast()

interface AIProvider {
  id: string
  name: string
  protocol: string
  base_url: string
  models: string[]
  default_model: string | null
  is_builtin: boolean
  has_api_key: boolean
}

function versionOptions(versions: string[]) {
  return versions.map(v => ({ value: v, label: v }))
}

function modelOptions(models: string[]) {
  return models.map(m => ({ value: m, label: m }))
}

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
  selectedProviderId: string
  selectedModel: string
  apiKey: string
  installLoading: boolean
  updateLoading: boolean
  modelSaving: boolean
  validateLoading: boolean
  validateResult: string
}

const initState = (): AgentState => ({
  installed: false,
  installedVersion: '',
  availableVersions: [],
  selectedVersion: '',
  selectedProviderId: '',
  selectedModel: '',
  apiKey: '',
  installLoading: false,
  updateLoading: false,
  modelSaving: false,
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

// ── AI Providers ──
const providers = ref<AIProvider[]>([])
const providerLoading = ref(false)

// 只显示已配置 API Key (验证过) 的 Provider
const verifiedProviders = computed(() =>
  providers.value.filter(p => p.has_api_key)
)

const currentProvider = computed(() =>
  providers.value.find(p => p.id === current.value.selectedProviderId)
)

const runtimeDefaultProviders: Record<string, string> = {
  claude_code: 'anthropic',
  hermes: 'openai',
  opencode: 'openai',
  openclaw: 'openai',
}

async function loadProviders() {
  providerLoading.value = true
  try {
    providers.value = await invoke('ai_list_providers')
  } catch { /* keep defaults */ }
  finally { providerLoading.value = false }
}

onMounted(async () => {
  await loadProviders()
  // 初始加载当前 tab 的状态，不需要等待切换
  await refreshRuntimeStatus(activeTab.value)
})

// ── Lifecycle ──
async function refreshRuntimeStatus(runtime: string) {
  try {
    const result: any = await invoke('agent_get_status', { runtime })
    state[runtime].installed = result.installed ?? false
    state[runtime].installedVersion = result.installed_version ?? ''
    state[runtime].availableVersions = result.available_versions ?? []
    state[runtime].selectedVersion = result.installed_version || state[runtime].selectedVersion
    if (result.config) {
      state[runtime].selectedModel = result.config.model_name || result.config.model_default || ''
    }
  } catch { /* backend not ready */ }

  // Set default provider if none selected
  if (!state[runtime].selectedProviderId) {
    const defaultProvider = runtimeDefaultProviders[runtime] || 'anthropic'
    if (providers.value.some(p => p.id === defaultProvider)) {
      state[runtime].selectedProviderId = defaultProvider
    } else if (providers.value.length > 0) {
      state[runtime].selectedProviderId = providers.value[0].id
    }
    // Set default model from provider
    const provider = providers.value.find(p => p.id === state[runtime].selectedProviderId)
    if (provider && !state[runtime].selectedModel && provider.default_model) {
      state[runtime].selectedModel = provider.default_model
    }
  }
}

function switchTab(runtime: string) {
  activeTab.value = runtime
  refreshRuntimeStatus(runtime)
}

// ── Actions ──
async function installRuntime(runtime: string) {
  const s = state[runtime]
  if (!s.selectedVersion) { toast.error('Please select a version to install'); return }
  s.installLoading = true
  try {
    await invoke('agent_install_runtime', { runtime, version: s.selectedVersion })
    s.installed = true
    s.installedVersion = s.selectedVersion
    toast.success(`${tabs.find(x => x.value === runtime)?.label} installed`)
  } catch (e: any) {
    const msg = String(e); toast.error(msg); pushLog('error', 'agent:install', msg)
  } finally { s.installLoading = false }
}

async function updateRuntime(runtime: string) {
  const s = state[runtime]
  if (!s.selectedVersion) { toast.error('Please select a version to upgrade to'); return }
  s.updateLoading = true
  try {
    await invoke('agent_update_runtime', { runtime, version: s.selectedVersion })
    s.installedVersion = s.selectedVersion
    toast.success(`${tabs.find(x => x.value === runtime)?.label} updated`)
  } catch (e: any) {
    const msg = String(e); toast.error(msg); pushLog('error', 'agent:update', msg)
  } finally { s.updateLoading = false }
}

async function rollbackRuntime(runtime: string) {
  const s = state[runtime]
  s.updateLoading = true
  try {
    const prevVersion: string = await invoke('agent_rollback_runtime', { runtime })
    s.installedVersion = prevVersion
    toast.success(`${tabs.find(x => x.value === runtime)?.label} rolled back to ${prevVersion}`)
  } catch (e: any) {
    const msg = String(e); toast.error(msg); pushLog('error', 'agent:rollback', msg)
  } finally { s.updateLoading = false }
}

async function saveModelConfig(runtime: string) {
  const s = state[runtime]
  const provider = providers.value.find(p => p.id === s.selectedProviderId)
  if (!provider) { toast.error('Please select a provider'); return }
  s.modelSaving = true
  try {
    await invoke('agent_save_config', {
      runtime,
      config: {
        protocol: provider.protocol,
        base_url: provider.base_url,
        model_default: provider.protocol === 'anthropic' ? s.selectedModel : '',
        model_small: '',
        model_large: '',
        model_name: provider.protocol === 'openai' ? s.selectedModel : '',
        extra_args: '',
      },
    })
    toast.success('Model configuration saved')
  } catch (e: any) {
    toast.error(String(e))
  } finally { s.modelSaving = false }
}

async function saveApiKey(runtime: string) {
  const s = state[runtime]
  if (!s.apiKey.trim()) return
  try {
    await invoke('ai_save_provider_secret', { id: s.selectedProviderId, key: s.apiKey })
    s.apiKey = ''
    toast.success('API key saved')
    await loadProviders() // refresh has_api_key flag
  } catch (e: any) { toast.error(String(e)) }
}

async function validateRuntime(runtime: string) {
  const s = state[runtime]
  const provider = providers.value.find(p => p.id === s.selectedProviderId)
  if (!provider) { toast.error('Please select a provider first'); return }
  s.validateLoading = true
  s.validateResult = ''
  try {
    const result: any = await invoke('ai_validate_provider', {
      id: s.selectedProviderId,
      providerId: null,
      protocol: provider.protocol,
      baseUrl: provider.base_url,
      model: s.selectedModel,
    })
    s.validateResult = result.valid ? '✅ Connected' : `❌ ${result.error || 'Validation failed'}`
  } catch (e: any) {
    s.validateResult = `❌ ${e}`
  } finally { s.validateLoading = false }
}
</script>

<template>
  <div class="flex flex-col h-full bg-surface-0">
    <div class="px-5 py-3 border-b border-border-default">
      <h2 class="text-[15px] font-semibold text-text-primary">{{ t.menuConfig.agentConfig }}</h2>
      <p class="text-xs text-text-secondary mt-0.5">{{ t.agentConfig.subtitle }}</p>
    </div>

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
                  <span :class="current.installed ? 'text-emerald-500' : 'text-amber-500'">
                    {{ current.installed ? `${t.agentConfig.installed} (${current.installedVersion})` : `${t.agentConfig.notInstalled}${current.installedVersion ? ` (available: ${current.installedVersion})` : ''}` }}
                  </span>
                </span>
              </div>
              <div class="flex items-center gap-2 mt-2">
                <VSelect
                  v-model="current.selectedVersion"
                  :options="versionOptions(current.availableVersions)"
                  :placeholder="t.agentConfig.selectVersion"
                  class="min-w-[180px]"
                />
                <VButton size="sm" :loading="current.installLoading" @click="installRuntime(rt)">
                  {{ t.agentConfig.install }}
                </VButton>
              </div>
            </section>

            <!-- ========== Section 2: Update & Rollback ========== -->
            <section>
              <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.agentConfig.updateTitle }}</h3>
              <div class="flex items-center gap-2">
                <VButton size="sm" variant="secondary" :loading="current.updateLoading" :disabled="!current.installed" @click="updateRuntime(rt)">
                  {{ t.agentConfig.checkUpdate }}
                </VButton>
                <VButton size="sm" variant="secondary" :disabled="!current.installed" @click="rollbackRuntime(rt)">
                  Rollback
                </VButton>
              </div>
            </section>

            <!-- ========== Section 3: AI Provider & Model ========== -->
            <section>
              <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.agentConfig.modelTitle }}</h3>

              <div class="space-y-2 max-w-[560px]">
                <!-- Empty state when no verified providers -->
                <div v-if="verifiedProviders.length === 0 && !providerLoading" class="text-[12px] text-text-muted py-2 border border-dashed border-border-default rounded-lg px-4 py-6 text-center">
                  No verified AI providers yet.<br/>
                  Go to <span class="text-primary-500 font-medium">AI Settings</span> to configure an API key for a provider.
                </div>

                <!-- Provider list — only verified (has_api_key) -->
                <div
                  v-for="provider in verifiedProviders"
                  :key="provider.id"
                  class="border rounded-lg transition-colors cursor-pointer"
                  :class="current.selectedProviderId === provider.id
                    ? 'border-primary-500 bg-primary-500/5'
                    : 'border-border-default hover:border-text-muted bg-surface-0'"
                  @click="state[rt].selectedProviderId = provider.id"
                >
                  <!-- Provider header -->
                  <div class="flex items-center gap-3 px-4 py-3">
                    <span class="w-3.5 h-3.5 rounded-full shrink-0"
                      :class="current.selectedProviderId === provider.id ? 'bg-primary-500 ring-2 ring-primary-500/30' : 'bg-text-muted/30'" />
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center gap-2">
                        <span class="text-[13px] font-medium text-text-primary">{{ provider.name }}</span>
                        <span v-if="provider.is_builtin" class="text-[10px] px-1 py-px rounded bg-text-muted/10 text-text-muted">builtin</span>
                        <span v-if="provider.has_api_key" class="text-[10px] px-1 py-px rounded bg-emerald-500/10 text-emerald-500">key set</span>
                      </div>
                      <div class="text-[11px] text-text-muted truncate">{{ provider.protocol }} · {{ provider.base_url }}</div>
                    </div>
                  </div>

                  <!-- Model selector (shown when provider selected) -->
                  <div
                    v-if="current.selectedProviderId === provider.id"
                    class="px-4 pb-3 border-t border-border-default/50"
                  >
                    <div class="flex items-center gap-2 mt-2">
                      <span class="text-[12px] text-text-secondary shrink-0">Model:</span>
                      <VSelect
                        :model-value="current.selectedModel"
                        :options="modelOptions(provider.models)"
                        placeholder="Select model"
                        class="flex-1"
                        @update:model-value="state[rt].selectedModel = $event as string"
                      />
                      <VButton size="sm" :loading="current.modelSaving" @click="saveModelConfig(rt)">Save</VButton>
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <!-- ========== Section 4: API Key ========== -->
            <section v-if="current.selectedProviderId">
              <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.agentConfig.apiKeyTitle }}</h3>
              <div class="text-[11px] text-text-muted mb-2">
                Provider: <span class="text-text-primary font-medium">{{ currentProvider?.name || current.selectedProviderId }}</span>
              </div>
              <div class="flex items-center gap-2 max-w-[520px]">
                <VInput v-model="current.apiKey" type="password" :placeholder="t.agentConfig.apiKeyPlaceholder" size="sm" class="flex-1" />
                <VButton size="sm" @click="saveApiKey(rt)">{{ t.agentConfig.saveApiKey }}</VButton>
              </div>
            </section>

            <!-- ========== Section 5: Validate ========== -->
            <section>
              <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.agentConfig.validateTitle }}</h3>
              <div class="flex items-center gap-3">
                <VButton size="sm" variant="secondary" :loading="current.validateLoading" @click="validateRuntime(rt)">
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
