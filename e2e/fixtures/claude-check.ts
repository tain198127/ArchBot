import { execFileSync } from 'child_process'

export interface ClaudeStatus {
  installed: boolean
  version?: string
  error?: string
}

export function checkClaudeCode(): ClaudeStatus {
  try {
    const version = execFileSync('claude', ['--version'], { encoding: 'utf-8', timeout: 8000 }).trim()
    return { installed: true, version }
  } catch {
    // Check if npm package exists even if binary isn't in PATH
    try {
      const npmList = execFileSync('npm', ['list', '-g', '@anthropic-ai/claude-code', '--depth=0'], { encoding: 'utf-8', timeout: 10000 })
      if (npmList.includes('@anthropic-ai/claude-code')) {
        return { installed: false, error: 'claude binary not in PATH but npm package found — check your PATH' }
      }
    } catch { /* not installed */ }
    return { installed: false, error: 'Claude Code CLI not found' }
  }
}

export function ensureClaudeCode(): void {
  const status = checkClaudeCode()
  if (status.installed) {
    console.log(`[E2E] Claude Code CLI found: ${status.version}`)
    return
  }
  console.log('[E2E] Claude Code CLI not found. Installing via npm...')
  try {
    execFileSync('npm', ['install', '-g', '@anthropic-ai/claude-code'], { stdio: 'inherit', timeout: 120000 })
    const retry = checkClaudeCode()
    if (!retry.installed) {
      console.warn(`[E2E] Claude Code installation may have failed: ${retry.error}`)
      console.warn('[E2E] Agent-related tests will be skipped.')
    } else {
      console.log(`[E2E] Claude Code CLI installed: ${retry.version}`)
    }
  } catch (e: any) {
    console.warn(`[E2E] Failed to install Claude Code: ${e.message}`)
    console.warn('[E2E] Agent-related tests will be skipped.')
  }
}
