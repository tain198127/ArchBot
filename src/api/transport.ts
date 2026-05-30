import { invoke } from '@tauri-apps/api/core'
import { runtimeEnv } from './env'
import type { ApiResponse } from './types'

/** IPC transport — calls Tauri backend directly. */
export async function ipcCall<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(cmd, args)
}

/** HTTP transport — calls Axum backend through Vite proxy (dev) or same-origin (prod). */
const API_BASE = '/api'

export async function httpGet<T>(path: string): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`)
  const json: ApiResponse<T> = await res.json()
  if (!json.success) throw new Error(json.error || 'Unknown error')
  return json.data
}

export async function httpPost<T>(path: string, body?: unknown): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: body ? JSON.stringify(body) : undefined,
  })
  const json: ApiResponse<T> = await res.json()
  if (!json.success) throw new Error(json.error || 'Unknown error')
  return json.data
}

export async function httpDelete<T>(path: string, body?: unknown): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    method: 'DELETE',
    headers: { 'Content-Type': 'application/json' },
    body: body ? JSON.stringify(body) : undefined,
  })
  const json: ApiResponse<T> = await res.json()
  if (!json.success) throw new Error(json.error || 'Unknown error')
  return json.data
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
