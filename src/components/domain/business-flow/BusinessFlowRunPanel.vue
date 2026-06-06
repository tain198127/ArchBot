<script setup lang="ts">
/**
 * BusinessFlowRunPanel
 *
 * Displays streaming output from the Conductor during flow execution.
 * Listens for archbot:trace events and shows per-node progress.
 */
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from '../../i18n'

const props = defineProps<{ flowId: string }>()

const { tt } = useI18n()
const runId = ref<string | null>(null)
const status = ref<string>('idle') // idle | running | completed | failed | aborted
const events = ref<Array<{ timestamp: string; message: string; category: string }>>([])
const error = ref<string | null>(null)
let unlisten: (() => void) | null = null

onMounted(async () => {
  // Listen for trace events from the conductor
  unlisten = await listen<{ timestamp: string; category: string; message: string }>(
    'archbot:trace',
    (event) => {
      if (event.payload.category === 'conductor') {
        events.value.push(event.payload)
        // Auto-scroll
        if (events.value.length > 0) {
          // Parse status from messages
          if (event.payload.message.includes('Flow execution completed')) {
            status.value = 'completed'
          } else if (event.payload.message.includes('Flow run failed') || event.payload.message.includes('flow_failed')) {
            status.value = 'failed'
          } else if (event.payload.message.includes('aborted')) {
            status.value = 'aborted'
          }
        }
      }
    }
  )
})

onUnmounted(() => {
  unlisten?.()
})

async function handleRun() {
  if (!props.flowId) return

  status.value = 'running'
  events.value = []
  error.value = null

  try {
    const id = await invoke<string>('bf_run_flow', {
      flowId: props.flowId,
      materialPaths: [] as string[],
      outputDirOverride: null,
    })
    runId.value = id
  } catch (e: unknown) {
    error.value = String(e)
    status.value = 'failed'
  }
}

async function handleAbort() {
  if (!runId.value) return
  try {
    await invoke('bf_abort_run', { runId: runId.value })
    status.value = 'aborted'
  } catch (e: unknown) {
    error.value = String(e)
  }
}

const statusLabel = (s: string) => {
  const map: Record<string, string> = {
    idle: tt('businessFlow.run.statusPending'),
    running: tt('businessFlow.run.statusRunning'),
    completed: tt('businessFlow.run.statusCompleted'),
    failed: tt('businessFlow.run.statusFailed'),
    aborted: tt('businessFlow.run.statusAborted'),
  }
  return map[s] || s
}

const statusColor = (s: string) => {
  const map: Record<string, string> = {
    running: 'text-blue-600 dark:text-blue-400',
    completed: 'text-green-600 dark:text-green-400',
    failed: 'text-red-600 dark:text-red-400',
    aborted: 'text-amber-600 dark:text-amber-400',
    idle: 'text-text-secondary',
  }
  return map[s] || 'text-text-secondary'
}
</script>

<template>
  <div class="flex flex-col h-full bg-surface-0 dark:bg-surface-0">
    <!-- Header -->
    <div class="flex items-center justify-between px-3 py-2 border-b border-border-default shrink-0">
      <span class="text-sm font-semibold text-text-primary">
        {{ tt('businessFlow.run.title') }}
      </span>
      <div class="flex items-center gap-2">
        <span class="text-xs" :class="statusColor(status)">
          {{ statusLabel(status) }}
        </span>
        <button
          v-if="status === 'idle' || status === 'completed' || status === 'failed' || status === 'aborted'"
          class="px-2.5 py-1 text-xs font-medium rounded-md
                 bg-primary-500 text-white hover:bg-primary-600
                 dark:bg-primary-500 dark:hover:bg-primary-600
                 transition-colors cursor-pointer"
          @click="handleRun"
        >
          {{ tt('businessFlow.run.start') }}
        </button>
        <button
          v-if="status === 'running'"
          class="px-2.5 py-1 text-xs font-medium rounded-md
                 bg-red-500 text-white hover:bg-red-600
                 dark:bg-red-500 dark:hover:bg-red-600
                 transition-colors cursor-pointer"
          @click="handleAbort"
        >
          {{ tt('businessFlow.run.abort') }}
        </button>
      </div>
    </div>

    <!-- Output log -->
    <div class="flex-1 overflow-y-auto p-3 font-mono text-xs">
      <div v-if="events.length === 0 && status === 'idle'" class="text-text-secondary text-center mt-8">
        {{ tt('businessFlow.run.configure') }}
      </div>
      <div
        v-for="(evt, i) in events"
        :key="i"
        class="py-0.5 text-text-primary"
      >
        <span class="text-text-muted mr-2">[{{ evt.timestamp?.slice(11, 19) || '' }}]</span>
        <span>{{ evt.message }}</span>
      </div>
    </div>

    <!-- Error -->
    <div v-if="error" class="px-3 py-2 text-xs text-red-500 bg-red-50 dark:bg-red-900/10 border-t border-border-default">
      {{ error }}
    </div>
  </div>
</template>
