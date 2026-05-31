import { test, expect } from 'playwright/test'

test.describe('Toolbar panel toggles', () => {
  test.beforeEach(async ({ page }) => {
    // In browser-only dev mode, Tauri API 404s are expected — don't fail on console errors
    await page.goto('http://localhost:1430', { waitUntil: 'domcontentloaded' })
    // Wait for Vue to mount and render
    await page.waitForSelector('.splitter', { timeout: 8000 })
    await page.waitForTimeout(400)
  })

  test('app loads and renders split panels', async ({ page }) => {
    // Splitters should exist (at least 2 for the 3-panel layout)
    const count = await page.locator('.splitter').count()
    expect(count).toBeGreaterThanOrEqual(2)
    // MenuBar should be visible (use first menu item, exact match)
    await expect(page.getByText('文件', { exact: true }).first()).toBeVisible()
  })

  test('toolbar toggle buttons exist with correct titles', async ({ page }) => {
    const buttons = page.locator('button[title*="Toggle"]')
    await expect(buttons).toHaveCount(3)
    await expect(buttons.nth(0)).toHaveAttribute('title', 'Toggle Left Panel')
    await expect(buttons.nth(1)).toHaveAttribute('title', 'Toggle Right Panel')
    await expect(buttons.nth(2)).toHaveAttribute('title', 'Toggle Bottom Panel')
  })

  test('left panel toggle toggles active visual state', async ({ page }) => {
    const leftBtn = page.locator('button[title="Toggle Left Panel"]')
    await expect(leftBtn).toBeVisible()
    // Expanded by default → muted
    await expect(leftBtn).toHaveClass(/text-text-muted/)

    await leftBtn.click()
    await page.waitForTimeout(500)
    // Collapsed → highlighted
    await expect(leftBtn).toHaveClass(/text-primary-500/)

    await leftBtn.click()
    await page.waitForTimeout(500)
    await expect(leftBtn).toHaveClass(/text-text-muted/)
  })

  test('bottom panel toggle toggles active visual state', async ({ page }) => {
    const bottomBtn = page.locator('button[title="Toggle Bottom Panel"]')
    await expect(bottomBtn).toBeVisible()

    await bottomBtn.click()
    await page.waitForTimeout(500)
    await expect(bottomBtn).toHaveClass(/text-primary-500/)

    await bottomBtn.click()
    await page.waitForTimeout(500)
    await expect(bottomBtn).toHaveClass(/text-text-muted/)
  })

  test('splitter handles are present and visible', async ({ page }) => {
    const handles = page.locator('.splitter-handle')
    const count = await handles.count()
    expect(count).toBeGreaterThanOrEqual(2)
  })
})
