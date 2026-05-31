const BASE = 'http://127.0.0.1:1421/api/playwright'

interface ApiEnvelope<T> {
  success: boolean
  data?: T
  error?: string
}

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    method,
    headers: body ? { 'Content-Type': 'application/json' } : undefined,
    body: body ? JSON.stringify(body) : undefined,
  })
  if (!res.ok) {
    throw new Error(`Playwright API returned ${res.status}: ${await res.text()}`)
  }
  const json: ApiEnvelope<T> = await res.json()
  if (!json.success) {
    throw new Error(json.error ?? 'Playwright plugin error')
  }
  return json.data as T
}

export interface WindowInfo {
  title: string
  width: number
  height: number
  focused: boolean
}

export const playwright = {
  eval:      (js: string)                    => request<any>('POST', '/eval', { js }),
  evalGet:   (js: string)                    => request<any>('GET', `/eval?js=${encodeURIComponent(js)}`),
  click:     (selector: string)              => request<any>('POST', '/click', { selector }),
  fill:      (selector: string, value: string) => request<any>('POST', '/fill', { selector, value }),
  text:      (selector: string)              => request<string>('GET', `/text?selector=${encodeURIComponent(selector)}`),
  attribute: (selector: string, name: string) => request<string | null>('GET', `/attribute?selector=${encodeURIComponent(selector)}&name=${encodeURIComponent(name)}`),
  count:     (selector: string)              => request<number>('GET', `/count?selector=${encodeURIComponent(selector)}`),
  waitFor:   (selector: string, timeout = 10000) => request<void>('GET', `/wait?selector=${encodeURIComponent(selector)}&timeout=${timeout}`),
  visible:   (selector: string)              => request<boolean>('GET', `/visible?selector=${encodeURIComponent(selector)}`),
  checked:   (selector: string)              => request<boolean>('GET', `/checked?selector=${encodeURIComponent(selector)}`),
  hover:     (selector: string)              => request<any>('POST', '/hover', { selector }),
  screenshot: ()                             => request<{ url: string; title: string; ready: boolean }>('GET', '/screenshot'),
  info:      ()                              => request<WindowInfo>('GET', '/info'),
}
