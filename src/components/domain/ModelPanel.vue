<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue'
import VSelect from '../base/VSelect.vue'
import VButton from '../base/VButton.vue'
import { useI18n } from '../../i18n'
import { invoke } from '@tauri-apps/api/core'
import { homeDir } from '@tauri-apps/api/path'
import { pushLog } from '../../stores/log'

const { t } = useI18n()
const chatWorkspace = ref('')

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

interface ConversationMessage {
  id: number
  role: 'user' | 'assistant'
  content: string
  model?: string
  loading?: boolean
  error?: boolean
}

// ── State ──
const currentMode = ref('chat')
const currentModel = ref('')
const inputText = ref('')
const sending = ref(false)
const messages = ref<ConversationMessage[]>([])
const providers = ref<AIProvider[]>([])
const chatContainer = ref<HTMLDivElement | null>(null)
let msgId = 0

const modes = [
  { value: 'chat', labelKey: 'chatMode' },
  { value: 'plan', labelKey: 'planMode' },
  { value: 'expert', labelKey: 'expertMode' },
]

const modelMsgs = t.value.model as Record<string, string>
const modeOptions = computed(() =>
  modes.map((m) => ({ value: m.value, label: modelMsgs[m.labelKey] || m.labelKey }))
)

// Collect all models from all verified providers
const modelOptions = computed(() => {
  const seen = new Set<string>()
  const opts: { value: string; label: string }[] = []
  for (const p of providers.value) {
    if (!p.has_api_key) continue
    for (const m of p.models) {
      if (seen.has(m)) continue
      seen.add(m)
      opts.push({ value: m, label: `${m} (${p.name})` })
    }
  }
  return opts
})

// ── Load providers ──
async function loadProviders() {
  try {
    providers.value = await invoke('ai_list_providers')
    if (!currentModel.value && modelOptions.value.length > 0) {
      // Default to the claude_code configured model if available
      try {
        const status: any = await invoke('agent_get_status', { runtime: 'claude_code' })
        if (status?.config?.model_name) {
          currentModel.value = status.config.model_name
        } else if (status?.config?.model_default) {
          currentModel.value = status.config.model_default
        }
      } catch { /* use first available */ }
      if (!currentModel.value || !modelOptions.value.some(o => o.value === currentModel.value)) {
        currentModel.value = modelOptions.value[0]?.value || ''
      }
    }
  } catch (e: any) {
    pushLog('error', 'model:providers', String(e))
  }
}

onMounted(async () => {
  chatWorkspace.value = (await homeDir()) + '/.archbot/chat-workspace'
  await loadProviders()
})

// ── Auto-scroll ──
watch(messages, () => {
  nextTick(() => {
    if (chatContainer.value) {
      chatContainer.value.scrollTop = chatContainer.value.scrollHeight
    }
  })
}, { deep: true })

// ── Send ──
async function handleSend() {
  const text = inputText.value.trim()
  if (!text || sending.value) return

  const userMsg: ConversationMessage = {
    id: ++msgId,
    role: 'user',
    content: text,
  }
  messages.value.push(userMsg)
  inputText.value = ''
  sending.value = true

  const assistantMsg: ConversationMessage = {
    id: ++msgId,
    role: 'assistant',
    content: '',
    model: currentModel.value,
    loading: true,
  }
  messages.value.push(assistantMsg)

  // Flush Vue DOM + yield to browser event loop so the loading UI actually paints
  // before the 20+ second Claude Code call blocks the Tauri IPC channel.
  await nextTick()
  await new Promise(r => setTimeout(r, 100))

  pushLog('info', 'model:chat', `Sending to Claude Code — model=${currentModel.value}`)

  try {
    const result: any = await invoke('agent_execute_turn', {
      runtime: 'claude_code',
      workspaceRoot: chatWorkspace.value,
      userMessage: text,
      contextFiles: [],
      modelOverride: currentModel.value || null,
    })

    const content = result.result_content
      || result.stdout_tail
      || result.status
      || 'No output'

    assistantMsg.content = content
    assistantMsg.loading = false
    pushLog('info', 'model:chat', `Turn completed — status=${result.status} duration=${result.duration_ms}ms`)
  } catch (e: any) {
    assistantMsg.content = `Error: ${String(e)}`
    assistantMsg.loading = false
    assistantMsg.error = true
    pushLog('error', 'model:chat', String(e))
  } finally {
    sending.value = false
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    handleSend()
  }
}
</script>

<template>
  <div class="flex flex-col h-full bg-surface-50 dark:bg-surface-50">
    <!-- Header: mode + model selectors -->
    <div class="flex items-center gap-2 px-3 py-2 border-b border-border-default shrink-0">
      <VSelect v-model="currentMode" :options="modeOptions" class="!w-[120px]" />
      <VSelect v-model="currentModel" :options="modelOptions" placeholder="Select model" class="!w-[200px]" />
    </div>

    <!-- Conversation area -->
    <div ref="chatContainer" class="flex-1 overflow-y-auto p-4 space-y-3">
      <div v-if="messages.length === 0" class="flex items-center justify-center h-full">
        <p class="text-sm text-text-muted">
          {{ currentMode === 'chat' ? t.model.chatPlaceholder : currentMode === 'plan' ? t.model.planPlaceholder : t.model.expertPlaceholder }}
        </p>
      </div>

      <div
        v-for="msg in messages"
        :key="msg.id"
        class="flex gap-3"
        :class="msg.role === 'user' ? 'justify-end' : 'justify-start'"
      >
        <!-- Assistant avatar -->
        <div v-if="msg.role === 'assistant'" class="shrink-0 w-7 h-7 rounded-full bg-primary-500/20 flex items-center justify-center text-[11px] text-primary-500 font-semibold mt-0.5">
          🤖
        </div>

        <!-- Message bubble -->
        <div
          class="max-w-[80%] rounded-lg px-3 py-2 text-[13px] leading-relaxed"
          :class="msg.role === 'user'
            ? 'bg-primary-500 text-white rounded-br-sm'
            : msg.error
              ? 'bg-red-500/10 text-red-500 border border-red-500/30 rounded-bl-sm'
              : 'bg-surface-0 border border-border-default text-text-primary rounded-bl-sm'"
        >
          <div v-if="msg.loading" class="flex items-center gap-2 text-text-muted">
            <span class="inline-block w-2 h-2 rounded-full bg-primary-500 animate-pulse" />
            <span class="inline-block w-2 h-2 rounded-full bg-primary-500 animate-pulse" style="animation-delay:0.15s" />
            <span class="inline-block w-2 h-2 rounded-full bg-primary-500 animate-pulse" style="animation-delay:0.3s" />
          </div>
          <pre v-else class="whitespace-pre-wrap font-sans text-[13px]">{{ msg.content }}</pre>
          <div v-if="msg.model" class="text-[10px] text-text-muted/60 mt-1">{{ msg.model }}</div>
        </div>

        <!-- User avatar -->
        <div v-if="msg.role === 'user'" class="shrink-0 w-7 h-7 rounded-full bg-text-muted/20 flex items-center justify-center text-[11px] text-text-secondary font-semibold mt-0.5">
          👤
        </div>
      </div>
    </div>

    <!-- Input area with Send button -->
    <div class="px-3 py-2 border-t border-border-default shrink-0 flex items-end gap-2">
      <textarea
        v-model="inputText"
        :placeholder="t.model.inputPlaceholder"
        :disabled="sending"
        :rows="2"
        class="flex-1 px-3 py-2 text-[13px] rounded-md border bg-surface-0 text-text-primary resize-none border-border-default hover:border-primary-300 placeholder:text-text-muted transition-all duration-150 focus:outline-none focus:border-primary-500 focus:ring-2 focus:ring-primary-500/20 disabled:opacity-40 disabled:cursor-not-allowed dark:bg-surface-100 dark:text-text-primary"
        @keydown="handleKeydown"
      />
      <VButton
        size="sm"
        :loading="sending"
        :disabled="!inputText.trim() || sending"
        @click="handleSend"
      >
        Send
      </VButton>
    </div>
  </div>
</template>
