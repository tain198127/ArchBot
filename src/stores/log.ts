import { reactive } from 'vue'

export interface LogEntry {
  id: number
  timestamp: string
  level: 'error' | 'warn' | 'info'
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
