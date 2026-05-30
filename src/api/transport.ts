import { invoke } from '@tauri-apps/api/core'
import { pushLog } from '../stores/log'
import { runtimeEnv } from './env'
import type { ApiResponse } from './types'

const API_BASE = '/api'

async function logError(context: string, err: unknown): Promise<never> {
  const msg = err instanceof Error ? err.message : String(err)
  pushLog('error', 'api', `[${context}] ${msg}`)
  throw err
}

/** IPC transport — calls Tauri backend directly. */
export async function ipcCall<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args)
  } catch (e) {
    return logError(`ipc:${cmd}`, e)
  }
}

async function httpRequest<T>(method: string, path: string, body?: unknown): Promise<T> {
  const url = `${API_BASE}${path}`
  let res: Response
  try {
    res = await fetch(url, {
      method,
      headers: { 'Content-Type': 'application/json' },
      body: body && method !== 'GET' ? JSON.stringify(body) : undefined,
    })
  } catch (e) {
    return logError(`${method} ${url}`, e)
  }
  const json: ApiResponse<T> = await res.json()
  if (!json.success) {
    pushLog('error', 'api', `[${method} ${path}] ${json.error || 'Unknown error'}`)
    throw new Error(json.error || 'Unknown error')
  }
  return json.data
}

export function httpGet<T>(path: string): Promise<T> {
  return httpRequest<T>('GET', path)
}

export function httpPost<T>(path: string, body?: unknown): Promise<T> {
  return httpRequest<T>('POST', path, body)
}

export function httpDelete<T>(path: string, body?: unknown): Promise<T> {
  return httpRequest<T>('DELETE', path, body)
}

/** Auto-select the right transport based on runtime environment. */
export function call<T>(
  cmd: string,
  httpMethod: string,
  httpPath: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (runtimeEnv === 'tauri') {
    return ipcCall<T>(cmd, args)
  }
  if (httpMethod === 'GET') {
    return httpGet<T>(httpPath)
  }
  if (httpMethod === 'DELETE') {
    return httpDelete<T>(httpPath, args)
  }
  return httpPost<T>(httpPath, args)
}
