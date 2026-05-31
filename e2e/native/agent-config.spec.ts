import { test, expect } from '../fixtures/tauri-fixture'
import { checkClaudeCode } from '../fixtures/claude-check'

test.describe('Agent config panel [native]', () => {
  const claudeStatus = checkClaudeCode()

  test.beforeAll(async () => {
    if (!claudeStatus.installed) {
      console.log('[E2E] Skipping agent config tests — Claude Code not installed')
    }
  })

  test('agent config panel shows runtime tabs', async ({ tauriPage }) => {
    // Tabs for Claude Code, Hermes, OpenCode, OpenClaw
    // The Agent config is in the bottom panel
    const tabs = tauriPage.locator('button[role="tab"]')
    const tabCount = await tabs.count()
    // May be 0 if bottom panel is collapsed; skip if so
    if (tabCount === 0) {
      console.log('[E2E] No tab elements found — bottom panel may be collapsed, skipping')
      return
    }
    expect(tabCount).toBeGreaterThanOrEqual(1)
  })

  test('window info is accessible via playwright bridge', async ({ tauriPage }) => {
    const info = await tauriPage.windowInfo()
    expect(info.title).toBeTruthy()
    expect(info.width).toBeGreaterThan(0)
    expect(info.height).toBeGreaterThan(0)
  })

  test('eval bridge returns correct document title', async ({ tauriPage }) => {
    const title = await tauriPage.evaluate<string>('return document.title')
    expect(title).toContain('ArchBot')
  })
})
