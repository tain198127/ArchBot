import { test as base, expect } from 'playwright/test'
import { TauriPage } from './tauri-page'

async function waitForNativeApp(timeoutMs = 60000): Promise<void> {
  const deadline = Date.now() + timeoutMs
  while (Date.now() < deadline) {
    try {
      const res = await fetch('http://127.0.0.1:1421/api/playwright/info')
      if (res.ok) {
        // Also verify the Vite dev server is responsive
        try {
          const vRes = await fetch('http://localhost:1420')
          if (vRes.ok) return
        } catch { /* Vite not up yet */ }
      }
    } catch { /* app not started yet */ }
    await new Promise(r => setTimeout(r, 1500))
  }
  throw new Error('Tauri app did not start within timeout — ensure `cargo tauri dev` is running')
}

export type TauriFixtures = {
  tauriPage: TauriPage
}

export const test = base.extend<TauriFixtures>({
  tauriPage: async ({}, use) => {
    await waitForNativeApp()
    const page = new TauriPage()
    await page.waitForSelector('.splitter', { timeout: 10000 })
    await page.waitForTimeout(500)
    await use(page)
  },
})

export { expect }
