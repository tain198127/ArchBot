<script setup lang="ts">
import { ref, watch, onUnmounted } from 'vue'
import { agentStore } from '../../stores/agentStore'

interface StreamEvent {
  id: string
  type: string
  data: string
  timestamp: string
}

const events = ref<StreamEvent[]>([])
const filterType = ref<string | null>(null)
const connected = ref(false)
let eventSource: EventSource | null = null
let idCounter = 0

const eventTypes = [
  'turn.started', 'turn.delta', 'turn.tool_started', 'turn.tool_finished',
  'turn.file_changed', 'turn.completed', 'turn.failed', 'turn.error',
]

function connect() {
  if (!agentStore.selectedSessionId || !agentStore.selectedTurnId) return
  const url = `http://127.0.0.1:1421/api/agent/sessions/${agentStore.selectedSessionId}/turns/${agentStore.selectedTurnId}/stream`

  eventSource = new EventSource(url)
  connected.value = true

  eventSource.onmessage = (e) => {
    try {
      const data = JSON.parse(e.data)
      events.value.push({
        id: data.event_id || String(++idCounter),
        type: data.event_type || e.type,
        data: JSON.stringify(data.payload || data, null, 0),
        timestamp: data.timestamp || new Date().toISOString(),
      })
    } catch {
      events.value.push({
        id: String(++idCounter),
        type: 'unknown',
        data: e.data,
        timestamp: new Date().toISOString(),
      })
    }
  }

  eventSource.onerror = () => {
    connected.value = false
  }

  // Listen for named events
  eventTypes.forEach((et) => {
    eventSource?.addEventListener(et, (e: MessageEvent) => {
      try {
        events.value.push({
          id: String(++idCounter),
          type: et,
          data: e.data,
          timestamp: new Date().toISOString(),
        })
      } catch { /* skip malformed events */ }
    })
  })
}

function disconnect() {
  eventSource?.close()
  eventSource = null
  connected.value = false
}

watch(
  () => [agentStore.selectedSessionId, agentStore.selectedTurnId],
  () => {
    disconnect()
    events.value = []
    if (agentStore.selectedSessionId && agentStore.selectedTurnId) {
      connect()
    }
  },
)

onUnmounted(() => disconnect())

function filteredEvents() {
  if (!filterType.value) return events.value
  return events.value.filter((e) => e.type === filterType.value)
}

function eventColor(type: string): string {
  if (type.includes('.error') || type.includes('.failed')) return 'text-red-500'
  if (type.includes('.warning')) return 'text-amber-500'
  if (type.includes('.completed')) return 'text-green-500'
  if (type.includes('.tool_')) return 'text-blue-500'
  if (type.includes('.delta')) return 'text-text-primary'
  return 'text-text-muted'
}
</script>

<template>
  <div class="flex flex-col h-full bg-surface-0">
    <!-- Toolbar -->
    <div class="flex items-center gap-2 px-3 py-1.5 border-b border-border-default">
      <span class="w-2 h-2 rounded-full shrink-0" :class="connected ? 'bg-green-500' : 'bg-text-muted'" />
      <span class="text-[11px] text-text-secondary">{{ connected ? 'Connected' : 'Disconnected' }}</span>
      <select
        v-model="filterType"
        class="ml-auto px-1.5 py-0.5 text-[10px] bg-surface-50 border border-border-default rounded"
      >
        <option :value="null">All events</option>
        <option v-for="et in eventTypes" :key="et" :value="et">{{ et }}</option>
      </select>
      <button
        class="px-1.5 py-0.5 text-[10px] text-text-muted hover:text-text-primary"
        @click="events = []"
      >Clear</button>
    </div>

    <!-- Event list -->
    <div class="flex-1 overflow-auto font-mono text-[11px]">
      <div
        v-for="evt in filteredEvents()"
        :key="evt.id"
        class="flex gap-2 px-3 py-1 border-b border-border-default/30 hover:bg-surface-50"
      >
        <span class="shrink-0 text-text-muted w-[65px]">{{ evt.timestamp.slice(11, 19) }}</span>
        <span class="shrink-0 w-[130px] truncate" :class="eventColor(evt.type)">{{ evt.type }}</span>
        <span class="text-text-secondary truncate">{{ evt.data.slice(0, 200) }}</span>
      </div>
      <div v-if="events.length === 0" class="flex items-center justify-center h-full">
        <p class="text-[12px] text-text-muted">{{ agentStore.selectedTurnId ? 'Waiting for events...' : 'Select a turn to view events.' }}</p>
      </div>
    </div>
  </div>
</template>
