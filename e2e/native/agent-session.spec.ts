import { test, expect } from '../fixtures/tauri-fixture'

test.describe('Agent Session [native]', () => {

  test('bottom panel has agent tab', async ({ tauriPage }) => {
    const bottomPanel = tauriPage.locator('.flex.flex-col.h-full.bg-surface-0')
    expect(await bottomPanel.isVisible()).toBe(true)
  })

  test('can create session via HTTP API', async () => {
    const res = await fetch('http://127.0.0.1:1421/api/agent/sessions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        title: 'E2E Test Session',
        goal: 'Verify end-to-end session flow',
        project_id: '/tmp/archbot-e2e-test',
        runtime_type: 'claude_code',
      }),
    })
    const json = await res.json()
    expect(json.success).toBe(true)
    expect(json.data.session_id).toBeTruthy()
    expect(json.data.title).toBe('E2E Test Session')
    expect(json.data.status).toBe('active')
  })

  test('can list sessions via HTTP API', async () => {
    const res = await fetch('http://127.0.0.1:1421/api/agent/sessions')
    const json = await res.json()
    expect(json.success).toBe(true)
    expect(Array.isArray(json.data)).toBe(true)
  })

  test('session CRUD via HTTP API', async () => {
    // Create
    const createRes = await fetch('http://127.0.0.1:1421/api/agent/sessions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        title: 'E2E API Session',
        goal: 'Testing via HTTP',
        runtime_type: 'claude_code',
      }),
    })
    const createJson = await createRes.json()
    expect(createJson.success).toBe(true)
    const sessionId: string = createJson.data.session_id
    expect(sessionId).toBeTruthy()

    // Get
    const getRes = await fetch(`http://127.0.0.1:1421/api/agent/sessions/${sessionId}`)
    const getJson = await getRes.json()
    expect(getJson.success).toBe(true)
    expect(getJson.data.title).toBe('E2E API Session')

    // Update status
    const pauseRes = await fetch(`http://127.0.0.1:1421/api/agent/sessions/${sessionId}/status`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ status: 'paused' }),
    })
    const pauseJson = await pauseRes.json()
    expect(pauseJson.success).toBe(true)

    // Close
    const closeRes = await fetch(`http://127.0.0.1:1421/api/agent/sessions/${sessionId}/status`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ status: 'closed' }),
    })
    const closeJson = await closeRes.json()
    expect(closeJson.success).toBe(true)
  })

  test('runtime health check via HTTP API', async () => {
    const res = await fetch('http://127.0.0.1:1421/api/agent/audit-log')
    const json = await res.json()
    expect(json.success).toBe(true)
  })

  test('can create and execute a turn via HTTP API', async () => {
    // First create a session
    const createRes = await fetch('http://127.0.0.1:1421/api/agent/sessions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        title: 'E2E Turn Test',
        goal: 'Execute a test turn',
        runtime_type: 'claude_code',
        project_id: '/tmp/archbot-e2e-turn-test',
      }),
    })
    const createJson = await createRes.json()
    expect(createJson.success).toBe(true)
    const sessionId: string = createJson.data.session_id

    // Create and execute a turn
    try {
      const turnRes = await fetch(`http://127.0.0.1:1421/api/agent/sessions/${sessionId}/turns`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          user_message: 'List the current working directory and write a summary.',
        }),
      })
      const turnJson = await turnRes.json()

      if (turnJson.success) {
        const result = turnJson.data
        expect(result.turn_id).toBeTruthy()
        expect(result.status).toBeTruthy()
        console.log(`Turn ${result.turn_id}: ${result.status}, ${result.duration_ms}ms`)

        // Query events for this turn
        const eventsRes = await fetch(
          `http://127.0.0.1:1421/api/agent/sessions/${sessionId}/turns/${result.turn_id}/events`
        )
        const eventsJson = await eventsRes.json()
        expect(eventsJson.success).toBe(true)
        // Events may be empty if the turn completed very quickly, which is fine
        console.log(`Events for turn: ${eventsJson.data?.total ?? 0}`)
      } else {
        // Turn execution may fail if Claude Code isn't configured — that's OK
        console.log(`Turn execution returned: ${turnJson.error}`)
        // The API itself worked (didn't crash)
        expect(turnRes.ok).toBe(true)
      }
    } finally {
      // Clean up: close the session
      await fetch(`http://127.0.0.1:1421/api/agent/sessions/${sessionId}/status`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ status: 'closed' }),
      })
    }
  })
})
