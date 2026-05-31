import { defineConfig } from 'playwright/test'

export default defineConfig({
  testDir: './e2e/native',
  timeout: 30000,
  retries: 1,
  use: {
    viewport: { width: 1280, height: 800 },
    actionTimeout: 5000,
  },
  // Native mode: Tauri app must already be running (cargo tauri dev)
  // The custom fixture will wait for both Vite (1420) and Axum (1421)
})
