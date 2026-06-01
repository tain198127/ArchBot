# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: toolbar-panels.spec.ts >> Toolbar panel toggles [native] >> left panel toggle changes visual state
- Location: e2e/native/toolbar-panels.spec.ts:19:3

# Error details

```
Error: expect(received).toBe(expected) // Object.is equality

Expected: true
Received: false
```

# Test source

```ts
  1  | import { test, expect } from '../fixtures/tauri-fixture'
  2  | 
  3  | test.describe('Toolbar panel toggles [native]', () => {
  4  |   test('app loads and renders split panels', async ({ tauriPage }) => {
  5  |     const count = await tauriPage.locator('.splitter').count()
  6  |     expect(count).toBeGreaterThanOrEqual(2)
  7  |     const title = await tauriPage.title()
  8  |     expect(title).toBeTruthy()
  9  |   })
  10 | 
  11 |   test('toolbar toggle buttons exist with correct titles', async ({ tauriPage }) => {
  12 |     const btnLoc = tauriPage.locator('button[title*="Toggle"]')
  13 |     expect(await btnLoc.count()).toBe(3)
  14 |     expect(await btnLoc.nth(0).getAttribute('title')).toBe('Toggle Left Panel')
  15 |     expect(await btnLoc.nth(1).getAttribute('title')).toBe('Toggle Right Panel')
  16 |     expect(await btnLoc.nth(2).getAttribute('title')).toBe('Toggle Bottom Panel')
  17 |   })
  18 | 
  19 |   test('left panel toggle changes visual state', async ({ tauriPage }) => {
  20 |     const leftBtn = tauriPage.locator('button[title="Toggle Left Panel"]')
  21 |     expect(await leftBtn.isVisible()).toBe(true)
  22 |     // Expanded by default → muted
> 23 |     expect(await leftBtn.toHaveClass('text-text-muted')).toBe(true)
     |                                                          ^ Error: expect(received).toBe(expected) // Object.is equality
  24 | 
  25 |     await leftBtn.click()
  26 |     await tauriPage.waitForTimeout(500)
  27 |     // Collapsed → highlighted
  28 |     expect(await leftBtn.toHaveClass('text-primary-500')).toBe(true)
  29 | 
  30 |     await leftBtn.click()
  31 |     await tauriPage.waitForTimeout(500)
  32 |     expect(await leftBtn.toHaveClass('text-text-muted')).toBe(true)
  33 |   })
  34 | 
  35 |   test('bottom panel toggle changes visual state', async ({ tauriPage }) => {
  36 |     const bottomBtn = tauriPage.locator('button[title="Toggle Bottom Panel"]')
  37 |     expect(await bottomBtn.isVisible()).toBe(true)
  38 | 
  39 |     await bottomBtn.click()
  40 |     await tauriPage.waitForTimeout(500)
  41 |     expect(await bottomBtn.toHaveClass('text-primary-500')).toBe(true)
  42 | 
  43 |     await bottomBtn.click()
  44 |     await tauriPage.waitForTimeout(500)
  45 |     expect(await bottomBtn.toHaveClass('text-text-muted')).toBe(true)
  46 |   })
  47 | 
  48 |   test('splitter handles are present', async ({ tauriPage }) => {
  49 |     const handles = tauriPage.locator('.splitter-handle')
  50 |     const count = await handles.count()
  51 |     expect(count).toBeGreaterThanOrEqual(2)
  52 |   })
  53 | })
  54 | 
```