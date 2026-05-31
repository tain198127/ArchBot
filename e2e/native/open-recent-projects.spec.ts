import { test, expect } from '../fixtures/tauri-fixture'

const MOCK_RECENT = [
  { name: 'demo-project-alpha', path: '/tmp/demo-alpha.ab' },
  { name: 'demo-project-beta',  path: '/tmp/demo-beta.ab' },
]

test.describe('Open Recent Projects [native]', () => {

  test('menu bar renders with menu items', async ({ tauriPage }) => {
    // Verify the menu bar container exists
    const menuBar = tauriPage.locator('.menu-items')
    expect(await menuBar.isVisible()).toBe(true)
    // Menu items have text content
    const text = await menuBar.textContent()
    expect(text).toBeTruthy()
  })

  test('can seed and read localStorage recent projects', async ({ tauriPage }) => {
    // Seed localStorage with mock data
    await tauriPage.evaluate(
      `localStorage.setItem('archbot_recent_projects', '${JSON.stringify(MOCK_RECENT).replace(/'/g, "\\'")}')`
    )
    // Read it back
    const raw = await tauriPage.evaluate<string>(
      "return localStorage.getItem('archbot_recent_projects')"
    )
    expect(raw).toBeTruthy()
    const parsed = JSON.parse(raw!)
    expect(Array.isArray(parsed)).toBe(true)
    expect(parsed.length).toBe(2)
    expect(parsed[0].name).toBe('demo-project-alpha')
  })

  test('can clear localStorage recent projects', async ({ tauriPage }) => {
    // Seed first
    await tauriPage.evaluate(
      `localStorage.setItem('archbot_recent_projects', '${JSON.stringify(MOCK_RECENT).replace(/'/g, "\\'")}')`
    )
    // Clear
    await tauriPage.evaluate("localStorage.removeItem('archbot_recent_projects')")
    // Verify empty
    const raw = await tauriPage.evaluate<string | null>(
      "return localStorage.getItem('archbot_recent_projects')"
    )
    expect(raw).toBeNull()
  })
})
