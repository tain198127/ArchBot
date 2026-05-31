import { test, expect } from '../fixtures/tauri-fixture'

test.describe('Toolbar panel toggles [native]', () => {
  test('app loads and renders split panels', async ({ tauriPage }) => {
    const count = await tauriPage.locator('.splitter').count()
    expect(count).toBeGreaterThanOrEqual(2)
    const title = await tauriPage.title()
    expect(title).toBeTruthy()
  })

  test('toolbar toggle buttons exist with correct titles', async ({ tauriPage }) => {
    const btnLoc = tauriPage.locator('button[title*="Toggle"]')
    expect(await btnLoc.count()).toBe(3)
    expect(await btnLoc.nth(0).getAttribute('title')).toBe('Toggle Left Panel')
    expect(await btnLoc.nth(1).getAttribute('title')).toBe('Toggle Right Panel')
    expect(await btnLoc.nth(2).getAttribute('title')).toBe('Toggle Bottom Panel')
  })

  test('left panel toggle changes visual state', async ({ tauriPage }) => {
    const leftBtn = tauriPage.locator('button[title="Toggle Left Panel"]')
    expect(await leftBtn.isVisible()).toBe(true)
    // Expanded by default → muted
    expect(await leftBtn.toHaveClass('text-text-muted')).toBe(true)

    await leftBtn.click()
    await tauriPage.waitForTimeout(500)
    // Collapsed → highlighted
    expect(await leftBtn.toHaveClass('text-primary-500')).toBe(true)

    await leftBtn.click()
    await tauriPage.waitForTimeout(500)
    expect(await leftBtn.toHaveClass('text-text-muted')).toBe(true)
  })

  test('bottom panel toggle changes visual state', async ({ tauriPage }) => {
    const bottomBtn = tauriPage.locator('button[title="Toggle Bottom Panel"]')
    expect(await bottomBtn.isVisible()).toBe(true)

    await bottomBtn.click()
    await tauriPage.waitForTimeout(500)
    expect(await bottomBtn.toHaveClass('text-primary-500')).toBe(true)

    await bottomBtn.click()
    await tauriPage.waitForTimeout(500)
    expect(await bottomBtn.toHaveClass('text-text-muted')).toBe(true)
  })

  test('splitter handles are present', async ({ tauriPage }) => {
    const handles = tauriPage.locator('.splitter-handle')
    const count = await handles.count()
    expect(count).toBeGreaterThanOrEqual(2)
  })
})
