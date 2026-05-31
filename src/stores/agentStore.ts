import { reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface AgentSession {
  session_id: string
  title: string
  goal: string
  project_id: string
  runtime_type: string
  default_model: string
  current_state: string
  status: string
  created_at: string
  updated_at: string
}

export interface AgentTurnInfo {
  turn_id: string
  session_id: string
  sequence_number: number
  user_message: string
  status: string
  runtime_type: string
  runtime_version: string
  model: string
  started_at: string
  finished_at: string
  error_message: string
  duration_ms: number
}

export interface FileChange {
  path: string
  change_type: string
}

export interface TurnResult {
  turn_id: string
  runtime: string
  status: string
  stdout_tail: string
  result_md_path: string | null
  file_changes: FileChange[]
  audit_violations: string[]
  duration_ms: number
}

export interface AuditEntry {
  log_id: string
  action: string
  severity: string
  detail: string
  created_at: string
}

export interface AdapterHealth {
  runtime_type: string
  available: boolean
  version: string
  capabilities: string[]
}

interface AgentEvent {
  event_id: string
  session_id: string
  turn_id: string
  runtime: string
  event_type: string
  timestamp: string
  payload: Record<string, unknown>
}

interface AgentState {
  sessions: AgentSession[]
  selectedSessionId: string | null
  turns: AgentTurnInfo[]
  selectedTurnId: string | null
  events: AgentEvent[]
  fileChanges: FileChange[]
  auditLog: AuditEntry[]
  loading: boolean
  error: string | null
}

export const agentStore = reactive<AgentState>({
  sessions: [],
  selectedSessionId: null,
  turns: [],
  selectedTurnId: null,
  events: [],
  fileChanges: [],
  auditLog: [],
  loading: false,
  error: null,
})

export async function createSession(
  title: string,
  goal?: string,
  projectId?: string,
  runtimeType?: string,
): Promise<AgentSession> {
  const session: AgentSession = await invoke('agent_create_session', {
    title,
    goal,
    projectId,
    runtimeType,
  })
  agentStore.sessions.push(session)
  return session
}

export async function listSessions(): Promise<AgentSession[]> {
  const sessions: AgentSession[] = await invoke('agent_list_sessions')
  agentStore.sessions = sessions
  return sessions
}

export async function updateSessionStatus(
  sessionId: string,
  status: string,
): Promise<void> {
  await invoke('agent_update_session_status', { sessionId, status })
  const session = agentStore.sessions.find((s) => s.session_id === sessionId)
  if (session) session.status = status
}

export async function createTurn(
  sessionId: string,
  userMessage: string,
  contextFiles: string[] = [],
  runtimeType?: string,
  workspaceRoot?: string,
): Promise<TurnResult> {
  const result: TurnResult = await invoke('agent_create_turn', {
    sessionId,
    userMessage,
    contextFiles,
    runtimeType,
    workspaceRoot,
  })
  return result
}

export async function loadTurns(sessionId: string): Promise<void> {
  // Via fetch to the agent API
  agentStore.loading = true
  try {
    const resp = await fetch(`http://127.0.0.1:1421/api/agent/sessions/${sessionId}/turns`)
    const json = await resp.json()
    if (json.success) {
      agentStore.turns = json.data
    }
  } catch (e) {
    agentStore.error = String(e)
  } finally {
    agentStore.loading = false
  }
}

export async function loadFileChanges(turnId: string): Promise<void> {
  agentStore.loading = true
  try {
    const resp = await fetch(`http://127.0.0.1:1421/api/agent/turns/${turnId}/file-changes`)
    const json = await resp.json()
    if (json.success) {
      agentStore.fileChanges = json.data.changes || []
    }
  } catch (e) {
    agentStore.error = String(e)
  } finally {
    agentStore.loading = false
  }
}

export async function loadAuditLog(): Promise<void> {
  agentStore.loading = true
  try {
    const resp = await fetch('http://127.0.0.1:1421/api/agent/audit-log')
    const json = await resp.json()
    if (json.success) {
      agentStore.auditLog = json.data.entries || []
    }
  } catch (e) {
    agentStore.error = String(e)
  } finally {
    agentStore.loading = false
  }
}

export interface RuntimeTestResult {
  found: boolean
  executable: string
  exit_code: number
  stdout: string
  stderr: string
}

export async function testRuntime(runtime: string): Promise<RuntimeTestResult> {
  return await invoke('agent_test_runtime', { runtime })
}

export async function checkRuntimeHealth(runtime: string): Promise<AdapterHealth> {
  return await invoke('agent_check_runtime_health', { runtime })
}

export function selectSession(sessionId: string | null): void {
  agentStore.selectedSessionId = sessionId
  if (sessionId) {
    loadTurns(sessionId)
  } else {
    agentStore.turns = []
  }
}

export function selectTurn(turnId: string | null): void {
  agentStore.selectedTurnId = turnId
  if (turnId) {
    loadFileChanges(turnId)
  } else {
    agentStore.fileChanges = []
  }
}
