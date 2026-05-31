import { reactive } from 'vue'
import { listen } from '@tauri-apps/api/event'

export interface LogEntry {
  id: number
  timestamp: string
  level: 'error' | 'warn' | 'info' | 'trace'
  source: string
  message: string
}

let nextId = 1

/** Reactive array of all log entries, newest first. */
export const logEntries = reactive<LogEntry[]>([])

function timestamp(): string {
  return new Date().toLocaleTimeString('zh-CN', { hour12: false })
}

export function pushLog(level: LogEntry['level'], source: string, message: string) {
  logEntries.unshift({ id: nextId++, timestamp: timestamp(), level, source, message })
  // Keep last 500 entries
  if (logEntries.length > 500) {
    logEntries.length = 500
  }
}

export function clearLog() {
  logEntries.length = 0
}

// ── Backend trace bridge ──
// Subscribes to `archbot:trace` events emitted by the Rust trace module.
// Each trace event appears as a 'trace'-level entry in the log panel,
// showing the exact execution path of backend code.
interface TracePayload {
  timestamp: string
  category: string
  message: string
}

let traceListenerUnlisten: (() => void) | null = null

export async function startTraceListener() {
  if (traceListenerUnlisten) return // already listening
  try {
    traceListenerUnlisten = await listen<TracePayload>('archbot:trace', (event) => {
      const p = event.payload
      pushLog('trace', `backend:${p.category}`, p.message)
    })
  } catch {
    // listen() may fail outside Tauri (e.g., browser dev mode) — ignore
  }
}

export function stopTraceListener() {
  traceListenerUnlisten?.()
  traceListenerUnlisten = null
}
