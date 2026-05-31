import { defineConfig } from 'playwright/test'

export default defineConfig({
  testDir: './e2e',
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
