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

// ── Lazy-load guards for @/# mention data ──
// Employee list and file tree are only fetched when the user actually
// types @ or # in the input area.  This avoids 2-3 Tauri IPC round-trips
// (including a potentially expensive de_list DB query) at app startup.
let employeesLoaded = false
let filesLoaded = false

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

// @input fires AFTER DOM update — always reads current textarea value
function handleInput() {
  const ta = textareaRef.value
  if (!ta) return
  const pos = ta.selectionStart
  const before = ta.value.slice(0, pos)

  // Find last @ or # before cursor
  const atIdx = before.lastIndexOf('@')
  const hashIdx = before.lastIndexOf('#')

  if (atIdx === -1 && hashIdx === -1) {
    showAtMention.value = false
    showHashRef.value = false
    return
  }

  if (atIdx > hashIdx) {
    // @ mention active — lazy-load employee list on first use
    if (!employeesLoaded) { employeesLoaded = true; loadEmployees() }
    const filter = before.slice(atIdx + 1)
    if (filter.includes(' ') || filter.includes('\n')) {
      showAtMention.value = false
    } else {
      showHashRef.value = false
      showAtMention.value = true
      atMentionFilter.value = filter
    }
  } else {
    // # file reference active — lazy-load file tree on first use
    if (!filesLoaded) { filesLoaded = true; loadFiles() }
    const filter = before.slice(hashIdx + 1)
    if (filter.includes(' ') || filter.includes('\n')) {
      showHashRef.value = false
    } else {
      showAtMention.value = false
      showHashRef.value = true
      hashRefFilter.value = filter
    }
  }
}

// ── Shared state ──
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const showAtMention = ref(false)
const atMentionFilter = ref('')
const atMentionEmployees = ref<any[]>([])
const showHashRef = ref(false)
const hashRefFilter = ref('')
const hashRefTree = ref<FileRefNode[]>([])
const expandedDirs = ref<Set<string>>(new Set())

// ── File tree types and helpers ──
interface FileRefNode {
  name: string
  type: 'file' | 'dir'
  path?: string
  children?: FileRefNode[]
}

interface VisibleNode {
  node: FileRefNode
  depth: number
}

function filterTree(nodes: FileRefNode[], q: string): FileRefNode[] {
  if (!q) return nodes
  const lower = q.toLowerCase()
  const result: FileRefNode[] = []
  for (const n of nodes) {
    const match = n.name.toLowerCase().includes(lower)
    if (n.type === 'file') {
      if (match) result.push(n)
    } else {
      const filteredChildren = n.children ? filterTree(n.children, q) : []
      if (match || filteredChildren.length > 0) {
        result.push({ ...n, children: filteredChildren })
      }
    }
  }
  return result
}

const filteredFileTree = computed(() => filterTree(hashRefTree.value, hashRefFilter.value))

const visibleNodes = computed(() => {
  const result: VisibleNode[] = []
  function walk(nodes: FileRefNode[], depth: number) {
    for (const n of nodes) {
      result.push({ node: n, depth })
      if (n.type === 'dir' && n.children && expandedDirs.value.has(n.path || n.name)) {
        walk(n.children, depth + 1)
      }
    }
  }
  walk(filteredFileTree.value, 0)
  return result
})

function toggleDir(path: string) {
  const next = new Set(expandedDirs.value)
  if (next.has(path)) next.delete(path)
  else next.add(path)
  expandedDirs.value = next
}

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
  const fullText = textarea.value
  const before = fullText.slice(0, pos)
  const atIdx = before.lastIndexOf('@')
  if (atIdx === -1) return
  inputText.value = textarea.value = fullText.slice(0, atIdx) + '@' + emp.name + ' ' + fullText.slice(pos)
  showAtMention.value = false
  nextTick(() => { textarea.focus(); textarea.setSelectionRange(atIdx + emp.name.length + 2, atIdx + emp.name.length + 2) })
}

async function loadFiles() {
  try {
    const projPath = currentProject.value?.path
    if (!projPath) { hashRefTree.value = []; return }
    const result: any = await invoke('fs_list_tree', { root: projPath, maxDepth: 3 })
    hashRefTree.value = (result?.tree || result || []) as FileRefNode[]
  } catch { hashRefTree.value = [] }
}

function insertHashRef(node: FileRefNode) {
  const textarea = textareaRef.value
  if (!textarea) return
  const filePath = node.path || node.name
  const pos = textarea.selectionStart
  const fullText = textarea.value
  const before = fullText.slice(0, pos)
  const hashIdx = before.lastIndexOf('#')
  if (hashIdx === -1) return
  inputText.value = textarea.value = fullText.slice(0, hashIdx) + '#[' + filePath + '] ' + fullText.slice(pos)
  showHashRef.value = false
  nextTick(() => { textarea.focus(); textarea.setSelectionRange(hashIdx + filePath.length + 4, hashIdx + filePath.length + 4) })
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

      <!-- # file reference tree popup -->
      <div v-if="showHashRef && visibleNodes.length > 0" class="absolute bottom-full left-3 mb-1 z-50 w-80 h-72 bg-surface-0 dark:bg-surface-50 border border-border-default rounded-lg shadow-xl flex flex-col">
        <div class="px-2 py-1 text-[10px] text-text-muted font-medium border-b border-border-default shrink-0"># Project Files</div>
        <div class="flex-1 overflow-y-auto p-1">
          <div
            v-for="item in visibleNodes"
            :key="(item.node.path || item.node.name) + '@' + item.depth"
            :style="{ paddingLeft: (item.depth * 14 + 8) + 'px' }"
            class="flex items-center gap-1 py-0.5 text-xs cursor-pointer hover:bg-primary-50 dark:hover:bg-primary-900/20 rounded mr-0.5"
            :class="item.node.type === 'file' ? 'font-mono text-[11px]' : ''"
            @click="item.node.type === 'dir' ? toggleDir(item.node.path || item.node.name) : insertHashRef(item.node)"
          >
            <!-- Dir: expand/collapse icon -->
            <template v-if="item.node.type === 'dir'">
              <span class="w-3 text-center text-[10px] shrink-0">{{ expandedDirs.has(item.node.path || item.node.name) ? '▾' : '▸' }}</span>
              <span class="text-amber-500 shrink-0">{{ expandedDirs.has(item.node.path || item.node.name) ? '📂' : '📁' }}</span>
              <span class="text-text-primary truncate">{{ item.node.name }}</span>
            </template>
            <!-- File: click to insert -->
            <template v-else>
              <span class="w-3 shrink-0" />
              <span class="text-text-muted shrink-0">📄</span>
              <span class="text-text-primary truncate">{{ item.node.name }}</span>
            </template>
          </div>
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
          @input="handleInput"
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
