<script setup lang="ts">
import { ref, watch } from 'vue'
import {
  agentStore,
  createTurn,
  loadTurns,
  selectTurn,
} from '../../stores/agentStore'

const message = ref('')
const executing = ref(false)

watch(
  () => agentStore.selectedSessionId,
  (sid) => {
    if (sid) loadTurns(sid)
  },
)

async function handleSend() {
  if (!message.value.trim() || !agentStore.selectedSessionId) return
  executing.value = true
  try {
    await createTurn(agentStore.selectedSessionId, message.value.trim())
    message.value = ''
    if (agentStore.selectedSessionId) {
      await loadTurns(agentStore.selectedSessionId)
    }
  } finally {
    executing.value = false
  }
}

function statusBadge(status: string): string {
  const map: Record<string, string> = {
    pending: 'bg-text-muted/20 text-text-muted',
    running: 'bg-blue-500/20 text-blue-500',
    streaming: 'bg-cyan-500/20 text-cyan-500',
    completed: 'bg-green-500/20 text-green-500',
    failed: 'bg-red-500/20 text-red-500',
    cancelled: 'bg-amber-500/20 text-amber-500',
    timeout: 'bg-red-500/20 text-red-500',
  }
  return map[status] || 'bg-text-muted/20 text-text-muted'
}
</script>

<template>
  <div class="flex flex-col h-full bg-surface-0">
    <!-- Turn list -->
    <div class="flex-1 overflow-auto">
      <div
        v-for="turn in agentStore.turns"
        :key="turn.turn_id"
        class="px-3 py-2 border-b border-border-default/50 cursor-pointer hover:bg-surface-50 transition-colors"
        :class="{ 'bg-primary-500/10': agentStore.selectedTurnId === turn.turn_id }"
        @click="selectTurn(turn.turn_id)"
      >
        <div class="flex items-center gap-2">
          <span class="px-1.5 py-0.5 text-[10px] rounded font-medium" :class="statusBadge(turn.status)">{{ turn.status }}</span>
          <span class="text-[11px] text-text-secondary">#{{ turn.sequence_number }}</span>
          <span class="text-[10px] text-text-muted ml-auto">{{ turn.runtime_type }}</span>
        </div>
        <div class="text-[12px] text-text-primary mt-1 truncate">{{ turn.user_message }}</div>
        <div v-if="turn.duration_ms > 0" class="text-[10px] text-text-muted mt-0.5">{{ (turn.duration_ms / 1000).toFixed(1) }}s</div>
      </div>
      <div v-if="agentStore.turns.length === 0" class="flex items-center justify-center h-full">
        <p class="text-[12px] text-text-muted">{{ agentStore.selectedSessionId ? 'No turns yet.' : 'Select a session first.' }}</p>
      </div>
    </div>

    <!-- Message input -->
    <div class="flex gap-2 px-3 py-2 border-t border-border-default bg-surface-50">
      <input
        v-model="message"
        placeholder="Type a message..."
        :disabled="executing || !agentStore.selectedSessionId"
        class="flex-1 px-2.5 py-1.5 text-[12px] bg-surface-0 border border-border-default rounded outline-none focus:border-primary-500 disabled:opacity-50"
        @keyup.enter="handleSend"
      />
      <button
        class="px-3 py-1.5 text-[12px] bg-primary-500 text-white rounded hover:bg-primary-600 disabled:opacity-50 transition-colors"
        :disabled="executing || !agentStore.selectedSessionId || !message.trim()"
        @click="handleSend"
      >{{ executing ? '...' : 'Send' }}</button>
    </div>
  </div>
</template>
