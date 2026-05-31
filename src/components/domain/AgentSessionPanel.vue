<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  agentStore,
  createSession,
  listSessions,
  selectSession,
} from '../../stores/agentStore'

const showCreate = ref(false)
const newTitle = ref('')
const newGoal = ref('')
const newRuntime = ref('claude_code')

onMounted(() => {
  listSessions().catch(() => {})
})

async function handleCreate() {
  if (!newTitle.value.trim()) return
  await createSession(newTitle.value.trim(), newGoal.value.trim(), undefined, newRuntime.value)
  showCreate.value = false
  newTitle.value = ''
  newGoal.value = ''
}

function statusColor(status: string): string {
  const map: Record<string, string> = {
    active: 'text-green-500',
    paused: 'text-amber-500',
    closed: 'text-text-muted',
    archived: 'text-text-muted/50',
  }
  return map[status] || 'text-text-muted'
}
</script>

<template>
  <div class="flex flex-col h-full bg-surface-0">
    <!-- Header -->
    <div class="flex items-center justify-between px-3 py-2 border-b border-border-default">
      <span class="text-[12px] font-semibold text-text-primary">Agent Sessions</span>
      <button
        class="px-2 py-0.5 text-[11px] bg-primary-500 text-white rounded hover:bg-primary-600 transition-colors"
        @click="showCreate = !showCreate"
      >+ New</button>
    </div>

    <!-- Create form -->
    <div v-if="showCreate" class="px-3 py-2 border-b border-border-default bg-surface-50 space-y-1.5">
      <input
        v-model="newTitle"
        placeholder="Session title"
        class="w-full px-2 py-1 text-[12px] bg-surface-0 border border-border-default rounded outline-none focus:border-primary-500"
        @keyup.enter="handleCreate"
      />
      <input
        v-model="newGoal"
        placeholder="Goal (optional)"
        class="w-full px-2 py-1 text-[12px] bg-surface-0 border border-border-default rounded outline-none focus:border-primary-500"
      />
      <select
        v-model="newRuntime"
        class="w-full px-2 py-1 text-[12px] bg-surface-0 border border-border-default rounded outline-none"
      >
        <option value="claude_code">Claude Code</option>
        <option value="opencode">OpenCode</option>
        <option value="hermes">Hermes</option>
        <option value="openclaw">OpenClaw</option>
      </select>
      <div class="flex gap-2">
        <button
          class="px-2.5 py-0.5 text-[11px] bg-primary-500 text-white rounded hover:bg-primary-600"
          @click="handleCreate"
        >Create</button>
        <button
          class="px-2.5 py-0.5 text-[11px] text-text-secondary hover:text-text-primary"
          @click="showCreate = false"
        >Cancel</button>
      </div>
    </div>

    <!-- Session list -->
    <div class="flex-1 overflow-auto">
      <div
        v-for="session in agentStore.sessions"
        :key="session.session_id"
        class="px-3 py-2 border-b border-border-default/50 cursor-pointer hover:bg-surface-50 transition-colors"
        :class="{ 'bg-primary-500/10 border-l-2 border-l-primary-500': agentStore.selectedSessionId === session.session_id }"
        @click="selectSession(session.session_id)"
      >
        <div class="flex items-center justify-between">
          <span class="text-[12px] font-medium text-text-primary truncate">{{ session.title }}</span>
          <span class="text-[10px] shrink-0 ml-2" :class="statusColor(session.status)">{{ session.status }}</span>
        </div>
        <div class="flex items-center gap-2 mt-0.5">
          <span class="text-[10px] text-text-muted">{{ session.runtime_type }}</span>
          <span class="text-[10px] text-text-muted/60">{{ session.created_at.slice(0, 10) }}</span>
        </div>
        <div v-if="session.goal" class="text-[10px] text-text-muted/70 mt-0.5 truncate">{{ session.goal }}</div>
      </div>
      <div v-if="agentStore.sessions.length === 0" class="flex items-center justify-center h-full">
        <p class="text-[12px] text-text-muted">No sessions yet. Create one to start.</p>
      </div>
    </div>
  </div>
</template>
