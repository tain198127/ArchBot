<script setup lang="ts">
import { reactive, ref, computed, onMounted } from 'vue'
import VTabs from '../base/VTabs.vue'
import VButton from '../base/VButton.vue'
import VSelect from '../base/VSelect.vue'
import { useI18n } from '../../i18n'
import { useToast } from '../../composables/useToast'
import { pushLog } from '../../stores/log'
import { testRuntime, installSkills, listInstalledSkills, updateSkills } from '../../stores/agentStore'
import type { InstalledSkill, SkillInstallSummary } from '../../stores/agentStore'
import { invoke } from '@tauri-apps/api/core'

interface SkillBundleInfo {
  name: string
  repo: string
  ref: string
  description: string
  installed: boolean
}

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
  installLoading: boolean
  updateLoading: boolean
  validateLoading: boolean
  validateResult: string
  testRuntimeLoading: boolean
  testRuntimeResult: string
  testRuntimeDetail: string
  envVars: Record<string, string>
  newEnvKey: string
  newEnvValue: string
  // Extra CLI args appended at runtime (space-separated)
  extraArgs: string
  // Skill bundle
  skillBundles: SkillBundleInfo[]
  skills: InstalledSkill[]
  skillsLoading: boolean
  skillsSummary: SkillInstallSummary | null
}

const initState = (): AgentState => ({
  installed: false,
  installedVersion: '',
  availableVersions: [],
  selectedVersion: '',
  selectedProviderId: '',
  selectedModel: '',
  installLoading: false,
  updateLoading: false,
  validateLoading: false,
  validateResult: '',
  testRuntimeLoading: false,
  testRuntimeResult: '',
  testRuntimeDetail: '',
  envVars: {},
  newEnvKey: '',
  newEnvValue: '',
  extraArgs: '--dangerously-skip-permissions',
  skillBundles: [],
  skills: [],
  skillsLoading: false,
  skillsSummary: null,
})

const state = reactive<Record<string, AgentState>>({
  claude_code: initState(),
  hermes: initState(),
  opencode: initState(),
  openclaw: initState(),
})

const current = computed(() => state[activeTab.value])

// Detect dangerous permission flags in extra_args
const hasDangerousFlags = computed(() => {
  const args = current.value.extraArgs || ''
  return /\b(--dangerously-skip-permissions|--permission-mode\s+bypassPermissions)\b/.test(args)
})

// ── AI Providers ──
const providers = ref<AIProvider[]>([])
const providerLoading = ref(false)

// 只显示已配置 API Key (验证过) 的 Provider
const verifiedProviders = computed(() =>
  providers.value.filter(p => p.has_api_key)
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
  } catch (e: any) {
    pushLog('error', 'agent:providers', String(e))
  } finally { providerLoading.value = false }
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
      // 恢复上次保存的 provider 选择（仅当该 provider 仍存在时）
      if (result.config.provider_id && providers.value.some(p => p.id === result.config.provider_id)) {
        state[runtime].selectedProviderId = result.config.provider_id
      }
      // Load persisted env vars
      if (result.config.env_vars) {
        state[runtime].envVars = { ...result.config.env_vars }
      }
      // Load persisted extra_args
      if (result.config.extra_args) {
        state[runtime].extraArgs = result.config.extra_args
      }
    }
  } catch (e: any) {
    pushLog('warn', 'agent:status', `Failed to load ${runtime} status: ${String(e)}`)
  }

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

  // Load skill bundles + installed skills
  await loadSkillBundles(runtime)
  await loadInstalledSkills(runtime)
}

async function loadSkillBundles(runtime: string) {
  try {
    state[runtime].skillBundles = await invoke<SkillBundleInfo[]>('agent_get_skill_bundles', { runtime })
  } catch {
    state[runtime].skillBundles = []
  }
}

async function loadInstalledSkills(runtime: string) {
  try {
    state[runtime].skills = await listInstalledSkills(runtime)
  } catch {
    // Skills might not be installed yet — that's fine
    state[runtime].skills = []
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
    const msg: string = await invoke('agent_install_runtime', { runtime, version: s.selectedVersion })
    s.installed = true
    s.installedVersion = s.selectedVersion
    toast.success(msg)
    // Reload skills after install (backend installs them as post-install hook)
    await loadSkillBundles(runtime)
    await loadInstalledSkills(runtime)
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

// ── Skill Actions ──

async function installSkillsForRuntime(runtime: string) {
  const s = state[runtime]
  s.skillsLoading = true
  try {
    s.skillsSummary = await installSkills(runtime)
    await loadSkillBundles(runtime)
    await loadInstalledSkills(runtime)
    if (s.skillsSummary.failed > 0) {
      toast.error(`${s.skillsSummary.succeeded}/${s.skillsSummary.total} skills installed (${s.skillsSummary.failed} failed)`)
    } else if (s.skillsSummary.succeeded > 0) {
      toast.success(`${s.skillsSummary.succeeded} skills installed`)
    }
  } catch (e: any) {
    toast.error(String(e))
  } finally { s.skillsLoading = false }
}

async function updateSkillsForRuntime(runtime: string) {
  const s = state[runtime]
  s.skillsLoading = true
  try {
    s.skillsSummary = await updateSkills(runtime)
    await loadSkillBundles(runtime)
    await loadInstalledSkills(runtime)
    toast.success(`${s.skillsSummary.succeeded} skills updated`)
  } catch (e: any) {
    toast.error(String(e))
  } finally { s.skillsLoading = false }
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

// 选择 provider 或模型时自动保存，无需手动点击保存按钮
async function autoSaveConfig(runtime: string) {
  const s = state[runtime]
  const provider = providers.value.find(p => p.id === s.selectedProviderId)
  if (!provider) return
  try {
    await invoke('agent_save_config', {
      runtime,
      config: {
        provider_id: s.selectedProviderId,
        protocol: provider.protocol,
        base_url: provider.base_url,
        model_default: provider.protocol === 'anthropic' ? s.selectedModel : '',
        model_small: '',
        model_large: '',
        model_name: provider.protocol === 'openai' ? s.selectedModel : '',
        extra_args: s.extraArgs,
        env_vars: s.envVars,
      },
    })
  } catch (e: any) {
    pushLog('error', 'agent:save_config', String(e))
  }
}

function selectProvider(rt: string, providerId: string) {
  state[rt].selectedProviderId = providerId
  // 自动选择该 provider 的默认模型
  const provider = providers.value.find(p => p.id === providerId)
  if (provider && provider.default_model) {
    state[rt].selectedModel = provider.default_model
  }
  autoSaveConfig(rt)
}

function selectModel(rt: string, model: string) {
  state[rt].selectedModel = model
  autoSaveConfig(rt)
}

async function validateRuntime(runtime: string) {
  const s = state[runtime]
  const provider = providers.value.find(p => p.id === s.selectedProviderId)
  if (!provider) {
    const msg = 'Please select a provider first'
    toast.error(msg)
    pushLog('error', 'agent:validate', msg)
    return
  }
  s.validateLoading = true
  s.validateResult = ''
  pushLog('info', 'agent:validate', `Validating ${runtime} via ${provider.name} (${provider.protocol})...`)
  try {
    const result: any = await invoke('ai_validate_provider', {
      id: s.selectedProviderId,
      providerId: null,
      protocol: provider.protocol,
      baseUrl: provider.base_url,
      model: s.selectedModel,
    })
    // Backend returns `ok`, not `valid`
    if (result.ok) {
      const modelInfo = result.response ? ` — ${result.response}` : ''
      s.validateResult = `✅ Connected${modelInfo}`
      pushLog('info', 'agent:validate', `${runtime} connection validated successfully`)
    } else {
      const errMsg = result.error || 'Validation failed'
      s.validateResult = `❌ ${errMsg}`
      pushLog('error', 'agent:validate', errMsg)
    }
  } catch (e: any) {
    const msg = String(e)
    s.validateResult = `❌ ${msg}`
    pushLog('error', 'agent:validate', msg)
  } finally { s.validateLoading = false }
}

// Actually spawn the CLI binary (e.g. `claude --version`) to prove it's the real runtime
async function testRuntimeAction(runtime: string) {
  const s = state[runtime]
  s.testRuntimeLoading = true
  s.testRuntimeResult = ''
  s.testRuntimeDetail = ''
  pushLog('info', 'agent:test-runtime', `Testing ${runtime} binary...`)
  try {
    const result = await testRuntime(runtime)
    if (!result.found) {
      s.testRuntimeResult = `❌ Not found`
      pushLog('error', 'agent:test-runtime', result.stderr || `${runtime} executable not found`)
      return
    }
    if (result.exit_code === 0 && result.stdout) {
      const versionLine = result.stdout.trim().split('\n')[0]
      s.testRuntimeResult = `✅ ${versionLine}`
      s.testRuntimeDetail = `Executable: ${result.executable}\nExit code: ${result.exit_code}\n\n${result.stdout}${result.stderr ? '\n[stderr]\n' + result.stderr : ''}`
      pushLog('info', 'agent:test-runtime', `Real binary confirmed: ${versionLine}`)
    } else {
      s.testRuntimeResult = `❌ Exit code: ${result.exit_code}`
      s.testRuntimeDetail = `Executable: ${result.executable}\nExit code: ${result.exit_code}\n${result.stderr || result.stdout}`
      pushLog('error', 'agent:test-runtime', `Binary failed: exit=${result.exit_code} ${result.stderr}`)
    }
  } catch (e: any) {
    const msg = String(e)
    s.testRuntimeResult = `❌ ${msg}`
    s.testRuntimeDetail = msg
    pushLog('error', 'agent:test-runtime', msg)
  } finally { s.testRuntimeLoading = false }
}

function addEnvVar(rt: string) {
  const s = state[rt]
  const key = s.newEnvKey.trim()
  if (!key) return
  s.envVars = { ...s.envVars, [key]: s.newEnvValue }
  s.newEnvKey = ''
  s.newEnvValue = ''
  autoSaveConfig(rt)
}

function removeEnvVar(rt: string, key: string) {
  const s = state[rt]
  const next = { ...s.envVars }
  delete next[key]
  s.envVars = next
  autoSaveConfig(rt)
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

            <!-- ========== Section 2.5: Skills ========== -->
            <section>
              <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.agentConfig.skillsTitle }}</h3>

              <!-- Configured skill bundles -->
              <div v-if="current.skillBundles.length > 0" class="space-y-2 max-w-[560px] mb-3">
                <div
                  v-for="bundle in current.skillBundles"
                  :key="bundle.name"
                  class="flex items-center justify-between text-xs py-2 px-3 rounded-lg border"
                  :class="bundle.installed ? 'bg-emerald-50 dark:bg-emerald-950/30 border-emerald-200 dark:border-emerald-800' : 'bg-surface-1 border-border-default'"
                >
                  <div class="flex items-center gap-2 min-w-0">
                    <span class="w-2 h-2 rounded-full flex-shrink-0" :class="bundle.installed ? 'bg-emerald-500' : 'bg-amber-400'" />
                    <div class="min-w-0">
                      <div class="font-medium text-text-primary truncate">{{ bundle.name }}</div>
                      <div class="text-text-muted truncate text-[11px]">{{ bundle.description }}</div>
                    </div>
                  </div>
                  <VButton
                    size="sm"
                    variant="secondary"
                    :loading="current.skillsLoading"
                    :disabled="bundle.installed"
                    @click="installSkillsForRuntime(rt)"
                  >
                    {{ bundle.installed ? '✓ Installed' : 'Install' }}
                  </VButton>
                </div>
              </div>
              <div v-else class="text-xs text-text-muted py-2">
                No skill bundles configured
              </div>

              <!-- Installed skill details -->
              <div v-if="current.skills.length > 0" class="space-y-1 max-w-[560px] mt-3">
                <div class="text-xs text-text-muted font-medium mb-1">Installed skill details</div>
                <div
                  v-for="skill in current.skills"
                  :key="skill.name"
                  class="flex items-center justify-between text-xs py-1 px-2 rounded bg-surface-1 border border-border-default"
                >
                  <div class="flex items-center gap-1.5 min-w-0">
                    <span class="text-text-muted flex-shrink-0">{{ skill.name }}</span>
                    <span class="text-text-muted/50">{{ skill.ref }}@{{ skill.commit }}</span>
                  </div>
                  <span class="text-text-muted/50 flex-shrink-0 ml-2">{{ skill.last_updated?.slice(0, 10) || '' }}</span>
                </div>
              </div>

              <div class="flex items-center gap-2 mt-2">
                <VButton size="sm" variant="secondary" :loading="current.skillsLoading" @click="installSkillsForRuntime(rt)">
                  {{ t.agentConfig.skillsReinstall }}
                </VButton>
                <VButton size="sm" variant="secondary" :loading="current.skillsLoading" :disabled="current.skills.length === 0" @click="updateSkillsForRuntime(rt)">
                  {{ t.agentConfig.skillsUpdate }}
                </VButton>
              </div>
              <!-- Install summary -->
              <div v-if="current.skillsSummary" class="mt-2 text-xs text-text-muted">
                {{ current.skillsSummary.succeeded }}/{{ current.skillsSummary.total }} succeeded<span v-if="current.skillsSummary.failed > 0">, {{ current.skillsSummary.failed }} failed</span>
                <div v-if="current.skillsSummary.failed > 0" class="mt-1 space-y-0.5">
                  <div v-for="r in current.skillsSummary.results.filter(x => x.status === 'failed')" :key="r.name" class="text-red-400">
                    ✕ {{ r.name }}: {{ r.error_message }}
                  </div>
                </div>
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
                  @click="selectProvider(rt, provider.id)"
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
                        @update:model-value="selectModel(rt, $event as string)"
                      />
                    </div>
                  </div>
                </div>
              </div>
            </section>

            <!-- ========== Section 3.5: Extra Arguments ========== -->
            <section>
              <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.agentConfig.extraArgs }}</h3>
              <div class="max-w-[560px]">
                <input
                  :value="current.extraArgs"
                  :placeholder="t.agentConfig.extraArgsPlaceholder"
                  class="w-full px-3 py-2 text-sm rounded-lg border border-border-default bg-surface-1 text-text-primary placeholder-text-muted focus:outline-none focus:border-primary-500 transition-colors font-mono"
                  @input="(e: Event) => { current.extraArgs = (e.target as HTMLInputElement).value; autoSaveConfig(rt) }"
                />
                <p class="text-[11px] text-text-muted mt-1">
                  Space-separated CLI flags appended when the runtime launches. Example: <code class="text-primary-400">--verbose --debug</code>
                </p>
                <!-- ⚠️ Security: warn when dangerous permission flags are active -->
                <div v-if="hasDangerousFlags" class="mt-2 px-3 py-2 rounded-lg bg-amber-500/10 border border-amber-500/30 text-xs text-amber-600 dark:text-amber-400 flex items-start gap-2">
                  <span class="mt-0.5 flex-shrink-0">⚠️</span>
                  <div>
                    <p class="font-medium">Permission checks bypassed</p>
                    <p class="mt-0.5 text-amber-500">The runtime will skip all permission prompts. File edits, shell commands, and network requests will execute without confirmation. Only use this in isolated or trusted environments.</p>
                  </div>
                </div>
              </div>
            </section>

            <!-- ========== Section 4: Validate ========== -->
            <section>
              <h3 class="text-sm font-semibold text-text-primary mb-3">{{ t.agentConfig.validateTitle }}</h3>

              <!-- API connectivity test — direct HTTP call to provider -->
              <div class="flex items-center gap-3 mb-2">
                <VButton size="sm" variant="secondary" :loading="current.validateLoading" @click="validateRuntime(rt)">
                  {{ t.agentConfig.validate }}
                </VButton>
                <span v-if="current.validateResult" class="text-[13px]" :class="current.validateResult.startsWith('✅') ? 'text-emerald-500' : 'text-red-400'">
                  {{ current.validateResult }}
                </span>
                <span v-else class="text-[11px] text-text-muted">Test API connectivity</span>
              </div>

              <!-- Runtime binary test — actually spawns the CLI tool -->
              <div class="flex items-center gap-2">
                <VButton size="sm" variant="secondary" :loading="current.testRuntimeLoading" @click="testRuntimeAction(rt)">
                  Test Runtime Binary
                </VButton>
                <span v-if="current.testRuntimeResult" class="text-[11px] font-mono" :class="current.testRuntimeResult.startsWith('✅') ? 'text-emerald-500' : 'text-red-400'">
                  {{ current.testRuntimeResult }}
                </span>
              </div>
              <div v-if="current.testRuntimeDetail" class="mt-2 p-2 rounded bg-surface-50 border border-border-default font-mono text-[11px] text-text-secondary whitespace-pre-wrap max-w-[560px] max-h-[120px] overflow-auto">{{ current.testRuntimeDetail }}</div>
            </section>

            <!-- ========== Section 5: Environment Variables ========== -->
            <section>
              <h3 class="text-sm font-semibold text-text-primary mb-3">Environment Variables</h3>
              <p class="text-[11px] text-text-muted mb-2">
                These are injected into the {{ tabs.find(t => t.value === rt)?.label }} subprocess at launch.
                Use for API keys, base URLs, model overrides, and runtime-specific settings.
              </p>

              <!-- Existing env vars -->
              <div v-if="Object.keys(current.envVars).length > 0" class="space-y-1.5 mb-3 max-w-[560px]">
                <div
                  v-for="(value, key) in current.envVars"
                  :key="key"
                  class="flex items-center gap-2 px-3 py-1.5 rounded bg-surface-50 border border-border-default"
                >
                  <code class="text-[12px] font-semibold text-text-primary shrink-0 min-w-[140px]">{{ key }}</code>
                  <input
                    :value="value"
                    class="flex-1 px-2 py-0.5 text-[12px] bg-surface-0 border border-border-default rounded outline-none focus:border-primary-500 font-mono"
                    placeholder="value"
                    @change="(e: Event) => { const t = e.target as HTMLInputElement; current.envVars = { ...current.envVars, [key]: t.value }; autoSaveConfig(rt) }"
                  />
                  <button
                    class="shrink-0 px-1.5 py-0.5 text-[11px] text-text-muted hover:text-red-500 transition-colors"
                    @click="removeEnvVar(rt, key)"
                  >✕</button>
                </div>
              </div>
              <div v-else class="text-[12px] text-text-muted mb-3">No custom environment variables configured.</div>

              <!-- Add new env var -->
              <div class="flex items-center gap-2 max-w-[560px]">
                <input
                  v-model="current.newEnvKey"
                  placeholder="KEY"
                  class="w-[180px] px-2.5 py-1.5 text-[12px] bg-surface-0 border border-border-default rounded outline-none focus:border-primary-500 font-mono"
                  @keyup.enter="addEnvVar(rt)"
                />
                <input
                  v-model="current.newEnvValue"
                  placeholder="value"
                  class="flex-1 px-2.5 py-1.5 text-[12px] bg-surface-0 border border-border-default rounded outline-none focus:border-primary-500 font-mono"
                  @keyup.enter="addEnvVar(rt)"
                />
                <VButton size="sm" variant="secondary" @click="addEnvVar(rt)">Add</VButton>
              </div>
            </section>

          </div>
        </template>
      </VTabs>
    </div>
  </div>
</template>
