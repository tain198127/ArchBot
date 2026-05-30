export type RuntimeEnv = 'tauri' | 'browser'

export function detectEnv(): RuntimeEnv {
  if (typeof window !== 'undefined' && '__TAURI__' in window) {
    return 'tauri'
  }
  return 'browser'
}

/** Resolved once at module load — stable for the lifetime of the page. */
export const runtimeEnv: RuntimeEnv = detectEnv()

export const isTauri = runtimeEnv === 'tauri'
export const isBrowser = runtimeEnv === 'browser'
