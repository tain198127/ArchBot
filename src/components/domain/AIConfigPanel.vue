<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import VButton from '../base/VButton.vue'
import VInput from '../base/VInput.vue'
import VSelect from '../base/VSelect.vue'
import VDialog from '../base/VDialog.vue'
import { useI18n } from '../../i18n'
import { useToast } from '../../composables/useToast'
import { invoke } from '@tauri-apps/api/core'

const { t, tt } = useI18n()
const toast = useToast()

const protocolOptions = [
  { value: 'anthropic', label: 'Anthropic' },
  { value: 'openai', label: 'OpenAI-compatible' },
]

interface Provider {
  id: string
  name: string
  protocol: string
  base_url: string
  models: string[]
  default_model: string | null
  is_builtin: boolean
  has_api_key: boolean
}

const providers = ref<Provider[]>([])
const selectedId = ref<string>('')
const loading = ref(false)
const showAddDialog = ref(false)

// ── Edit form state ──
const editName = ref('')
const editProtocol = ref('anthropic')
const editBaseUrl = ref('')
const editModels = ref<string[]>([])
const editDefaultModel = ref<string | null>(null)
const editApiKey = ref('')
const newModelInput = ref('')
const validating = ref(false)
const validateResult = ref('')
const saving = ref(false)

// ── Remote model IDs returned by validation (per-provider, session-only) ──
// Models in this set are "verified" and shown in bright colors.
const remoteVerifiedModels = ref<Record<string, Set<string>>>({})

// ── Add custom form ──
const newId = ref('')
const newName = ref('')
const newProtocol = ref('openai')
const newBaseUrl = ref('')
const adding = ref(false)

const selected = computed(() => providers.value.find(p => p.id === selectedId.value))
const isBuiltin = computed(() => selected.value?.is_builtin ?? false)
const sortedModels = computed(() => {
  const arr = [...editModels.value]
  if (editDefaultModel.value) {
    arr.sort((a, b) => {
      if (a === editDefaultModel.value) return -1
      if (b === editDefaultModel.value) return 1
      return 0
    })
  }
  return arr
})

onMounted(() => loadProviders())

async function loadProviders() {
  loading.value = true
  try {
    providers.value = await invoke<Provider[]>('ai_list_providers')
    if (!selectedId.value && providers.value.length > 0) {
      selectProvider(providers.value[0].id)
    }
  } catch (e: any) {
    toast.error(String(e))
  } finally {
    loading.value = false
  }
}

function selectProvider(id: string) {
  selectedId.value = id
  const p = providers.value.find(x => x.id === id)
  if (!p) return
  editName.value = p.name
  editProtocol.value = p.protocol
  editBaseUrl.value = p.base_url
  editModels.value = [...p.models]
  editDefaultModel.value = p.default_model
  editApiKey.value = ''
  validateResult.value = ''
  newModelInput.value = ''

  if (!remoteVerifiedModels.value[id]) {
    remoteVerifiedModels.value[id] = new Set()
  }
}

function addModel() {
  const v = newModelInput.value.trim()
  if (!v) return
  if (!editModels.value.includes(v)) {
    editModels.value.push(v)
  }
  newModelInput.value = ''
}

function removeModel(modelName: string) {
  const idx = editModels.value.indexOf(modelName)
  if (idx >= 0) editModels.value.splice(idx, 1)
  if (editDefaultModel.value === modelName) {
    editDefaultModel.value = editModels.value.length > 0 ? editModels.value[0] : null
  }
}

async function saveProvider() {
  saving.value = true
  try {
    console.log('[aiConfig] saving provider:', selectedId.value, 'models:', editModels.value, 'default:', editDefaultModel.value)
    await invoke('ai_save_provider', {
      id: selectedId.value,
      name: editName.value,
      protocol: editProtocol.value,
      baseUrl: editBaseUrl.value,
      models: editModels.value,
      defaultModel: editDefaultModel.value ?? undefined,
    })
    console.log('[aiConfig] save succeeded')
    toast.success(t.value.aiConfig.saveSuccess)
    await loadProviders()
  } catch (e: any) {
    console.error('[aiConfig] save failed:', e)
    const msg = typeof e === 'string' ? e : (e?.message || e?.error || JSON.stringify(e))
    toast.error(String(msg))
  } finally {
    saving.value = false
  }
}

function setDefaultModel(model: string) {
  editDefaultModel.value = model
}

function isDefaultModel(model: string): boolean {
  return editDefaultModel.value === model
}

function isModelVerified(model: string): boolean {
  return remoteVerifiedModels.value[selectedId.value]?.has(model) ?? false
}

async function saveApiKey() {
  if (!editApiKey.value.trim()) return
  try {
    await invoke('ai_save_provider_secret', {
      id: selectedId.value,
      key: editApiKey.value,
    })
    editApiKey.value = ''
    // Immediately update the has_api_key indicator in the list
    const p = providers.value.find(x => x.id === selectedId.value)
    if (p) p.has_api_key = true
    toast.success(t.value.aiConfig.apiKeySaved)
  } catch (e: any) {
    toast.error(String(e))
  }
}

async function doValidate() {
  validating.value = true
  validateResult.value = ''
  try {
    const result: any = await invoke('ai_validate_provider', {
      id: selectedId.value,
      protocol: editProtocol.value,
      baseUrl: editBaseUrl.value,
      model: editModels.value.length > 0 ? editModels.value[0] : '',
    })
    if (result.ok) {
      const reply = result.response || t.value.aiConfig.validated
      validateResult.value = `✅ ${reply}`
      // Populate remote-verified model set from API response
      if (result.remote_models && result.remote_models.length > 0) {
        const set = new Set<string>(result.remote_models)
        remoteVerifiedModels.value[selectedId.value] = set
      }
      // Also mark the tested model individually
      const testedModel = editModels.value.length > 0 ? editModels.value[0] : ''
      if (testedModel && !remoteVerifiedModels.value[selectedId.value]) {
        remoteVerifiedModels.value[selectedId.value] = new Set()
      }
      if (testedModel) {
        remoteVerifiedModels.value[selectedId.value].add(testedModel)
      }
    } else {
      validateResult.value = `❌ ${result.error || t.value.aiConfig.validateFailed}`
    }
  } catch (e: any) {
    validateResult.value = `❌ ${e}`
  } finally {
    validating.value = false
  }
}

async function deleteProvider() {
  if (isBuiltin.value) {
    toast.error(t.value.aiConfig.cannotDeleteBuiltin)
    return
  }
  if (!confirm(tt('aiConfig.confirmDelete') || 'Confirm delete?')) return

  try {
    await invoke('ai_delete_provider', { id: selectedId.value })
    toast.success(t.value.aiConfig.deleteSuccess)
    selectedId.value = ''
    await loadProviders()
    if (providers.value.length > 0) {
      selectProvider(providers.value[0].id)
    }
  } catch (e: any) {
    toast.error(String(e))
  }
}

async function addCustomProvider() {
  const id = newId.value.trim().toLowerCase().replace(/\s+/g, '-')
  if (!id || !newName.value.trim()) return
  adding.value = true
  try {
    await invoke('ai_save_provider', {
      id,
      name: newName.value.trim(),
      protocol: newProtocol.value,
      baseUrl: newBaseUrl.value.trim(),
      models: [] as string[],
      defaultModel: null,
    })
    showAddDialog.value = false
    newId.value = ''
    newName.value = ''
    newBaseUrl.value = ''
    await loadProviders()
    selectProvider(id)
    toast.success(t.value.aiConfig.saveSuccess)
  } catch (e: any) {
    toast.error(String(e))
  } finally {
    adding.value = false
  }
}
</script>

<template>
  <div class="flex flex-col h-full bg-surface-0">
    <!-- Header -->
    <div class="px-5 py-3 border-b border-border-default shrink-0">
      <h2 class="text-[15px] font-semibold text-text-primary">{{ t.aiConfig.title }}</h2>
      <p class="text-xs text-text-secondary mt-0.5">{{ t.aiConfig.subtitle }}</p>
    </div>

    <!-- Body: left list + right detail -->
    <div class="flex flex-1 overflow-hidden">
      <!-- Left: provider list -->
      <div class="w-[240px] shrink-0 border-r border-border-default flex flex-col bg-surface-50 dark:bg-surface-50">
        <div class="px-3 py-2 text-[11px] font-semibold text-text-muted uppercase tracking-wide">
          {{ t.aiConfig.providerList }}
        </div>
        <div class="flex-1 overflow-y-auto">
          <div
            v-for="p in providers" :key="p.id"
            class="flex items-center gap-2 px-3 py-2.5 cursor-pointer border-l-[3px] text-[13px] transition-colors"
            :class="selectedId === p.id
              ? 'bg-surface-0 dark:bg-surface-0 border-primary-500 text-text-primary'
              : 'border-transparent text-text-secondary hover:bg-surface-100 dark:hover:bg-surface-100'"
            @click="selectProvider(p.id)"
          >
            <div class="flex-1 min-w-0">
              <div class="truncate font-medium">{{ p.name }}</div>
              <div class="text-[11px] text-text-muted truncate">{{ p.base_url }}</div>
            </div>
            <div class="flex items-center gap-1 shrink-0">
              <span
                class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium"
                :class="p.protocol === 'anthropic' ? 'bg-purple-100 text-purple-600 dark:bg-purple-900/30 dark:text-purple-400' : 'bg-emerald-100 text-emerald-600 dark:bg-emerald-900/30 dark:text-emerald-400'"
              >{{ p.protocol === 'anthropic' ? 'ANT' : 'OAI' }}</span>
              <span
                class="w-1.5 h-1.5 rounded-full"
                :class="p.has_api_key ? 'bg-emerald-400' : 'bg-surface-300 dark:bg-surface-400'"
                :title="p.has_api_key ? 'API key configured' : 'No API key'"
              />
            </div>
          </div>
        </div>
        <div class="p-2 border-t border-border-default">
          <VButton size="sm" variant="secondary" class="w-full" @click="showAddDialog = true">
            + {{ t.aiConfig.addProvider }}
          </VButton>
        </div>
      </div>

      <!-- Right: detail editor -->
      <div class="flex-1 overflow-y-auto p-5">
        <template v-if="selected">
          <div class="max-w-[540px] space-y-5">
            <!-- Name + Protocol row -->
            <div class="flex items-center gap-3">
              <div class="flex-1">
                <label class="block text-[13px] font-medium text-text-primary mb-1">{{ t.aiConfig.name }}</label>
                <VInput
                  v-model="editName"
                  :placeholder="t.aiConfig.namePlaceholder"
                  :disabled="isBuiltin"
                  size="sm"
                />
              </div>
              <div class="w-[160px]">
                <label class="block text-[13px] font-medium text-text-primary mb-1">{{ t.aiConfig.protocol }}</label>
                <VSelect v-model="editProtocol" :options="protocolOptions" />
              </div>
            </div>

            <!-- Base URL -->
            <div>
              <label class="block text-[13px] font-medium text-text-primary mb-1">{{ t.aiConfig.baseUrl }}</label>
              <VInput v-model="editBaseUrl" :placeholder="t.aiConfig.baseUrlPlaceholder" size="sm" />
            </div>

            <!-- Models list -->
            <div>
              <label class="block text-[13px] font-medium text-text-primary mb-1">{{ t.aiConfig.models }}</label>
              <div class="flex flex-wrap gap-1.5 mb-2">
                <span
                  v-for="m in sortedModels" :key="m"
                  class="group relative inline-flex items-center gap-1 pr-1 pl-2 py-1 rounded-md text-[12px] border transition-all"
                  :class="[
                    isDefaultModel(m)
                      ? 'bg-primary-50 border-primary-400 text-primary-700 dark:bg-primary-950 dark:border-primary-600 dark:text-primary-400 shadow-sm'
                      : isModelVerified(m)
                        ? 'bg-surface-0 dark:bg-surface-0 border-emerald-300 dark:border-emerald-700 text-text-primary'
                        : 'bg-surface-100 dark:bg-surface-100 border-border-default text-text-muted opacity-60',
                  ]"
                >
                  <!-- default badge (top-left corner) -->
                  <span
                    v-if="isDefaultModel(m)"
                    class="absolute -top-1.5 -left-1.5 text-[9px] px-1 py-px rounded font-medium bg-primary-500 text-white shadow-sm"
                  >{{ t.aiConfig.default }}</span>
                  {{ m }}
                  <!-- verified checkmark (remote API confirmed) -->
                  <span
                    v-if="isModelVerified(m)"
                    class="text-emerald-500 text-[10px] leading-none"
                    :title="t.aiConfig.validated"
                  >&#10003;</span>
                  <!-- set-as-default button, visible on hover for non-default models -->
                  <button
                    v-if="!isDefaultModel(m)"
                    class="text-[10px] text-amber-500 hover:text-amber-600 hover:bg-amber-50 dark:hover:bg-amber-950 px-1 rounded leading-none opacity-0 group-hover:opacity-100 transition-all"
                    :title="t.aiConfig.setDefault"
                    @click="setDefaultModel(m)"
                  >&#9733;</button>
                  <button class="text-text-muted hover:text-danger-500 text-sm leading-none ml-0.5" @click="removeModel(m)">&times;</button>
                </span>
                <span v-if="editModels.length === 0" class="text-[12px] text-text-muted">{{ t.aiConfig.notConfigured }}</span>
              </div>
              <div class="flex items-center gap-2">
                <VInput
                  v-model="newModelInput"
                  :placeholder="t.aiConfig.modelsPlaceholder"
                  size="sm"
                  class="flex-1"
                  @keyup.enter="addModel"
                />
                <VButton size="sm" variant="secondary" @click="addModel">{{ t.aiConfig.addModel }}</VButton>
              </div>
            </div>

            <!-- API Key -->
            <div>
              <label class="block text-[13px] font-medium text-text-primary mb-1">
                {{ t.aiConfig.apiKey }}
                <span
                  v-if="selected.has_api_key"
                  class="inline-flex items-center ml-2 px-1.5 py-0.5 rounded text-[10px] bg-emerald-100 text-emerald-600 dark:bg-emerald-900/30 dark:text-emerald-400"
                >{{ t.aiConfig.apiKeySaved }}</span>
              </label>
              <div class="flex items-center gap-2">
                <VInput
                  v-model="editApiKey"
                  type="password"
                  :placeholder="t.aiConfig.apiKeyPlaceholder"
                  size="sm"
                  class="flex-1"
                />
                <VButton size="sm" variant="secondary" :disabled="!editApiKey.trim()" @click="saveApiKey">
                  {{ t.aiConfig.save }}
                </VButton>
              </div>
            </div>

            <!-- Actions: validate, save, delete -->
            <div class="flex items-center gap-2 pt-2 border-t border-border-default">
              <VButton size="sm" :loading="saving" @click="saveProvider">{{ t.aiConfig.save }}</VButton>
              <VButton size="sm" variant="secondary" :loading="validating" @click="doValidate">
                {{ t.aiConfig.validate }}
              </VButton>
              <div class="flex-1" />
              <VButton
                v-if="!isBuiltin"
                size="sm"
                variant="secondary"
                class="!text-red-500 hover:!bg-red-50 dark:hover:!bg-red-950"
                @click="deleteProvider"
              >{{ t.aiConfig.deleteProvider }}</VButton>
            </div>

            <!-- Validate result -->
            <!-- Validate result log -->
            <div
              v-if="validateResult"
              class="rounded-md border overflow-hidden"
              :class="validateResult.startsWith('✅')
                ? 'border-emerald-200 dark:border-emerald-800'
                : 'border-red-200 dark:border-red-800'"
            >
              <div
                class="px-3 py-1.5 text-[11px] font-medium"
                :class="validateResult.startsWith('✅')
                  ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-400'
                  : 'bg-red-50 text-red-700 dark:bg-red-950 dark:text-red-400'"
              >{{ validateResult.startsWith('✅') ? '✅ ' + t.aiConfig.validated : '❌ ' + t.aiConfig.validateFailed }}</div>
              <div
                v-if="validateResult.startsWith('✅')"
                class="px-3 py-2 text-[12px] text-text-primary bg-surface-0 dark:bg-surface-0 leading-relaxed whitespace-pre-wrap max-h-[200px] overflow-y-auto font-mono"
              >{{ validateResult.slice(2).trim() }}</div>
              <div
                v-else
                class="px-3 py-2 text-[12px] text-red-600 dark:text-red-400 bg-red-50/50 dark:bg-red-950/50 leading-relaxed whitespace-pre-wrap max-h-[200px] overflow-y-auto"
              >{{ validateResult.slice(2).trim() }}</div>
            </div>

            <!-- Provider meta -->
            <div class="flex items-center gap-2 text-[11px] text-text-muted">
              <span
                class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium"
                :class="isBuiltin ? 'bg-blue-100 text-blue-600 dark:bg-blue-900/30 dark:text-blue-400' : 'bg-amber-100 text-amber-600 dark:bg-amber-900/30 dark:text-amber-400'"
              >{{ isBuiltin ? t.aiConfig.builtin : t.aiConfig.custom }}</span>
              <span>ID: {{ selected.id }}</span>
              <span>Protocol: {{ selected.protocol }}</span>
            </div>
          </div>
        </template>

        <!-- Empty state -->
        <div v-else class="flex items-center justify-center h-full text-sm text-text-muted">
          {{ t.aiConfig.notConfigured }}
        </div>
      </div>
    </div>

    <!-- Add custom dialog -->
    <VDialog
      :visible="showAddDialog"
      :title="t.aiConfig.addProvider"
      width="420px"
      @update:visible="showAddDialog = $event"
    >
      <div class="space-y-4">
        <div>
          <label class="block text-[13px] font-medium text-text-primary mb-1">{{ t.aiConfig.name }}</label>
          <VInput v-model="newName" :placeholder="t.aiConfig.namePlaceholder" size="sm" />
        </div>
        <div>
          <label class="block text-[13px] font-medium text-text-primary mb-1">{{ t.aiConfig.protocol }}</label>
          <VSelect v-model="newProtocol" :options="protocolOptions" />
        </div>
        <div>
          <label class="block text-[13px] font-medium text-text-primary mb-1">{{ t.aiConfig.baseUrl }}</label>
          <VInput v-model="newBaseUrl" :placeholder="t.aiConfig.baseUrlPlaceholder" size="sm" />
        </div>
      </div>
      <template #footer>
        <div class="flex justify-end gap-2">
          <VButton size="sm" variant="secondary" @click="showAddDialog = false">{{ t.common.cancel }}</VButton>
          <VButton size="sm" :loading="adding" :disabled="!newName.trim()" @click="addCustomProvider">{{ t.common.confirm }}</VButton>
        </div>
      </template>
    </VDialog>
  </div>
</template>
