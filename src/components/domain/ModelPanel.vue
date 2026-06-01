<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue'
import VSelect from '../base/VSelect.vue'
import VButton from '../base/VButton.vue'
import VInput from '../base/VInput.vue'
import { useI18n } from '../../i18n'
import { invoke } from '@tauri-apps/api/core'
import { homeDir } from '@tauri-apps/api/path'
import { pushLog } from '../../stores/log'
import { useProject } from '../../stores/project'
import { useToast } from '../../composables/useToast'

const { t, tt } = useI18n()
const { currentProject } = useProject()
const toast = useToast()
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
  loadEmployees()
  loadFiles()
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

// ── @ mention — Silicon Corps roles ──
const showAtMention = ref(false)
const atMentionFilter = ref('')
const atMentionEmployees = ref<any[]>([])
const textareaRef = ref<HTMLTextAreaElement | null>(null)

async function loadEmployees() {
  try {
    atMentionEmployees.value = await invoke<any[]>('de_list', { dbType: 'local' })
  } catch { atMentionEmployees.value = [] }
}

function filteredEmployees() {
  const q = atMentionFilter.value.toLowerCase()
  return atMentionEmployees.value.filter((e: any) =>
    (e.name || '').toLowerCase().includes(q) || (e.code || '').toLowerCase().includes(q)
  ).slice(0, 8)
}

function insertAtMention(emp: any) {
  const textarea = textareaRef.value
  if (!textarea) return
  const pos = textarea.selectionStart
  // Find the @ that started this
  const before = inputText.value.slice(0, pos)
  const atIdx = before.lastIndexOf('@')
  if (atIdx === -1) return
  const before2 = inputText.value.slice(0, atIdx)
  const after = inputText.value.slice(pos)
  inputText.value = before2 + '@' + emp.name + ' ' + after
  showAtMention.value = false
  atMentionFilter.value = ''
  // Focus back
  nextTick(() => {
    textarea.focus()
    const newPos = atIdx + emp.name.length + 2
    textarea.setSelectionRange(newPos, newPos)
  })
}

function onInput(e: Event) {
  const textarea = e.target as HTMLTextAreaElement
  const pos = textarea.selectionStart
  const before = inputText.value.slice(0, pos)
  // Check if we just typed @
  const atMatch = before.match(/@([^\s@]*)$/)
  if (atMatch) {
    atMentionFilter.value = atMatch[1]
    showAtMention.value = true
  } else {
    showAtMention.value = false
    atMentionFilter.value = ''
  }
}

// ── # file reference ──
const showHashRef = ref(false)
const hashRefFilter = ref('')
const hashRefFiles = ref<string[]>([])

async function loadFiles() {
  try {
    const projPath = currentProject.value?.path
    if (!projPath) { hashRefFiles.value = []; return }
    const result: any = await invoke('fs_list_tree', { root: projPath, maxDepth: 3 })
    // Flatten tree to file paths
    const paths: string[] = []
    function walk(nodes: any[]) {
      for (const n of nodes) {
        if (n.type === 'file') paths.push(n.path || n.name)
        if (n.children?.length) walk(n.children)
      }
    }
    walk(result?.tree || result || [])
    hashRefFiles.value = paths.slice(0, 100)
  } catch { hashRefFiles.value = [] }
}

function filteredFiles() {
  const q = hashRefFilter.value.toLowerCase()
  if (!q) return hashRefFiles.value.slice(0, 12)
  return hashRefFiles.value.filter(f => f.toLowerCase().includes(q)).slice(0, 12)
}

function insertHashRef(path: string) {
  const textarea = textareaRef.value
  if (!textarea) return
  const pos = textarea.selectionStart
  const before = inputText.value.slice(0, pos)
  const hashIdx = before.lastIndexOf('#')
  if (hashIdx === -1) return
  const before2 = inputText.value.slice(0, hashIdx)
  const after = inputText.value.slice(pos)
  inputText.value = before2 + '#[' + path + '] ' + after
  showHashRef.value = false
  hashRefFilter.value = ''
  nextTick(() => {
    textarea.focus()
    const newPos = hashIdx + path.length + 4
    textarea.setSelectionRange(newPos, newPos)
  })
}

function onInputHash(e: Event) {
  const textarea = e.target as HTMLTextAreaElement
  const pos = textarea.selectionStart
  const before = inputText.value.slice(0, pos)
  const hashMatch = before.match(/#([^\s#]*)$/)
  if (hashMatch) {
    hashRefFilter.value = hashMatch[1]
    showHashRef.value = true
    showAtMention.value = false
  } else {
    showHashRef.value = false
    hashRefFilter.value = ''
  }
}

function onCombinedInput(e: Event) {
  onInput(e)
  onInputHash(e)
}

// ── 炼魂 (Soul Refining) ──
const refining = ref(false)
const showRefineDialog = ref(false)
const refineSkillName = ref('')

function openRefineDialog() {
  if (messages.value.length === 0) {
    toast.warning(tt('model.noMessagesToRefine') || 'No conversation to refine')
    return
  }
  refineSkillName.value = ''
  showRefineDialog.value = true
}

async function handleRefine() {
  if (!refineSkillName.value.trim() && messages.value.length < 2) {
    toast.warning('Please enter a skill name')
    return
  }
  refining.value = true
  showRefineDialog.value = false
  try {
    // Collect all conversation content
    const convo = messages.value
      .map(m => `[${m.role === 'user' ? 'User' : 'Assistant'}]: ${m.content}`)
      .join('\n\n')

    const nameHint = refineSkillName.value.trim()
      ? `Use this name for the skill: ${refineSkillName.value.trim()}`
      : 'Generate a concise, descriptive skill name based on the conversation content.'

    const prompt = [
      'You are distilling a conversation into a reusable Claude Code skill (SKILL.md).',
      '',
      nameHint,
      '',
      '<conversation>',
      convo.slice(0, 8000), // Truncate to avoid token overflow
      '</conversation>',
      '',
      'Output ONLY valid SKILL.md content (YAML frontmatter + markdown body).',
      'The skill should capture the methodology, patterns, and workflows discussed.',
      'Include: name, description, usage examples, and tool coordination in the SKILL.md.',
    ].join('\n')

    const result: any = await invoke('agent_execute_turn', {
      runtime: 'claude_code',
      workspaceRoot: chatWorkspace.value,
      userMessage: prompt,
      contextFiles: [],
      modelOverride: currentModel.value || null,
    })

    const skillContent = result.result_content || result.stdout_tail || ''

    // Save the distilled skill to DB
    const skillName = refineSkillName.value.trim() || 'distilled-skill-' + Date.now()
    const skillCode = skillName.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '')
    await invoke('db_insert', {
      table: 'skills',
      data: {
        code: skillCode,
        name: skillName,
        command: '/' + skillCode,
        description: 'Distilled from conversation on ' + new Date().toLocaleDateString(),
        body: skillContent,
        group_name: 'guerrillas',
        updated_at: new Date().toISOString(),
      },
      dbType: 'local',
    }).catch(async () => {
      // If exists, update
      await invoke('db_update', {
        table: 'skills',
        id: skillCode,
        data: {
          name: skillName,
          command: '/' + skillCode,
          body: skillContent,
          updated_at: new Date().toISOString(),
        },
        dbType: 'local',
      })
    })

    // Add a system message about the distillation
    messages.value.push({
      id: ++msgId,
      role: 'assistant',
      content: `🔮 **炼魂完成** — 已生成技能「${skillName}」(/${skillCode})\n\n对话中的方法论已萃取并保存到 Skill 配置中，可在硅基军团的默认能力中使用。`,
    })
    toast.success(`Skill "${skillName}" created from conversation`)
  } catch (e: any) {
    toast.error(String(e))
  } finally {
    refining.value = false
  }
}
</script>

<template>
  <div class="flex flex-col h-full bg-surface-50 dark:bg-surface-50">
    <!-- Header: mode + model selectors + 炼魂 -->
    <div class="flex items-center gap-2 px-3 py-2 border-b border-border-default shrink-0">
      <VSelect v-model="currentMode" :options="modeOptions" class="!w-[120px]" />
      <VSelect v-model="currentModel" :options="modelOptions" placeholder="Select model" class="!w-[200px]" />
      <VButton
        size="sm"
        variant="secondary"
        :loading="refining"
        :disabled="messages.length === 0"
        :title="tt('model.refineTitle') || '炼魂 — 将对话蒸馏为可复用技能'"
        @click="openRefineDialog"
      >
        🔮 {{ tt('model.refine') || '炼魂' }}
      </VButton>
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
    <div class="px-3 py-2 border-t border-border-default shrink-0 relative">
      <!-- @ mention popup -->
      <div v-if="showAtMention && filteredEmployees().length > 0" class="absolute bottom-full left-3 mb-1 z-50 w-60 bg-surface-0 dark:bg-surface-50 border border-border-default rounded-lg shadow-xl max-h-48 overflow-y-auto">
        <div class="px-2 py-1 text-[10px] text-text-muted font-medium border-b border-border-default">@ Silicon Corps</div>
        <div
          v-for="emp in filteredEmployees()"
          :key="emp.code"
          class="px-3 py-1.5 text-xs cursor-pointer hover:bg-primary-50 dark:hover:bg-primary-900/20 flex items-center gap-2"
          @click="insertAtMention(emp)"
        >
          <span class="text-text-primary font-medium">{{ emp.name }}</span>
          <span class="text-text-muted text-[11px]">{{ emp.code }}</span>
        </div>
      </div>

      <!-- # file reference popup -->
      <div v-if="showHashRef && filteredFiles().length > 0" class="absolute bottom-full left-3 mb-1 z-50 w-72 bg-surface-0 dark:bg-surface-50 border border-border-default rounded-lg shadow-xl max-h-48 overflow-y-auto">
        <div class="px-2 py-1 text-[10px] text-text-muted font-medium border-b border-border-default"># Files</div>
        <div
          v-for="f in filteredFiles()"
          :key="f"
          class="px-3 py-1 text-xs cursor-pointer hover:bg-primary-50 dark:hover:bg-primary-900/20 font-mono text-[11px] text-text-primary"
          @click="insertHashRef(f)"
        >
          {{ f }}
        </div>
      </div>

      <div class="flex items-end gap-2">
        <textarea
          ref="textareaRef"
          v-model="inputText"
          :placeholder="t.model.inputPlaceholder"
          :disabled="sending"
          :rows="2"
          class="flex-1 px-3 py-2 text-[13px] rounded-md border bg-surface-0 text-text-primary resize-none border-border-default hover:border-primary-300 placeholder:text-text-muted transition-all duration-150 focus:outline-none focus:border-primary-500 focus:ring-2 focus:ring-primary-500/20 disabled:opacity-40 disabled:cursor-not-allowed dark:bg-surface-100 dark:text-text-primary"
          @keydown="handleKeydown"
          @input="onCombinedInput"
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

    <!-- 炼魂 Dialog -->
    <div v-if="showRefineDialog" class="fixed inset-0 z-50 flex items-center justify-center bg-black/40" @click.self="showRefineDialog = false">
      <div class="bg-surface-0 dark:bg-surface-50 rounded-xl shadow-2xl p-6 w-[400px] border border-border-default">
        <h3 class="text-base font-semibold text-text-primary mb-2">🔮 {{ tt('model.refine') || '炼魂' }}</h3>
        <p class="text-sm text-text-secondary mb-4">{{ tt('model.refineDesc') || '将对话中的方法论、流程和知识蒸馏为一个可复用的 Skill。' }}</p>
        <label class="text-sm text-text-secondary mb-1 block">{{ tt('model.refineNameLabel') || 'Skill 名称（留空自动生成）' }}</label>
        <VInput v-model="refineSkillName" placeholder="e.g. code-review-workflow" class="mb-4" />
        <div class="flex gap-2 justify-end">
          <VButton variant="ghost" @click="showRefineDialog = false">{{ tt('model.cancel') || '取消' }}</VButton>
          <VButton :loading="refining" @click="handleRefine">{{ tt('model.refineStart') || '开始蒸馏' }}</VButton>
        </div>
      </div>
    </div>
  </div>
</template>
