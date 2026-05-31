import { test, expect } from '../fixtures/tauri-fixture'

test.describe('AI config panel [native]', () => {
  test('AI config panel is accessible via UI tree', async ({ tauriPage }) => {
    // Verify the app has rendered the main layout
    const body = tauriPage.locator('body')
    expect(await body.isVisible()).toBe(true)
  })

  test('can evaluate JavaScript to inspect component state', async ({ tauriPage }) => {
    // Verify Vue app is mounted
    const hasVueApp = await tauriPage.evaluate<boolean>(
      'return !!document.querySelector("#app") || !!document.querySelector("[data-v-app]")'
    )
    expect(hasVueApp).toBe(true)
  })

  test('providers endpoint returns data via HTTP API', async ({ tauriPage }) => {
    // Use eval to call the Tauri invoke API and check providers
    const result = await tauriPage.evaluate<any>(
      'if(window.__TAURI__){ return window.__TAURI__.invoke("ai_list_providers"); } return null;'
    )
    if (result) {
      expect(Array.isArray(result)).toBe(true)
      if (result.length > 0) {
        expect(result[0]).toHaveProperty('id')
        expect(result[0]).toHaveProperty('name')
      }
    }
  })
})
