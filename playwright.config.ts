import { defineConfig } from 'playwright/test'

const NATIVE = process.env.TAURI_E2E === '1'

export default defineConfig({
  testDir: './e2e',
  testIgnore: ['**/native/**'],
  timeout: 15000,
  retries: 0,
  use: {
    baseURL: 'http://localhost:1430',
    headless: true,
    viewport: { width: 1280, height: 800 },
    actionTimeout: 5000,
  },
  webServer: {
    command: 'npx vite --port 1430',
    url: 'http://localhost:1430',
    reuseExistingServer: true,
  },
})
