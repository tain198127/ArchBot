import { open } from '@tauri-apps/plugin-dialog'
import { pushLog } from '../stores/log'
import { isTauri } from './env'

export interface FileFilter {
  name: string
  extensions: string[]
}

/**
 * Open a file picker dialog, returning the absolute path or null.
 */
export async function openFileDialog(
  filters: FileFilter[],
): Promise<string | null> {
  if (isTauri) {
    const result = await open({
      multiple: false,
      filters,
    })
    return (result as string) ?? null
  }

  return browserOpenFile(filters)
}

/**
 * Open a directory picker dialog.
 */
export async function openDirectoryDialog(): Promise<string | null> {
  if (isTauri) {
    const result = await open({ directory: true })
    return (result as string) ?? null
  }

  return browserOpenDirectory()
}

async function browserOpenFile(filters: FileFilter[]): Promise<string | null> {
  const accept = filters.flatMap((f) => f.extensions.map((e) => `.${e}`)).join(',')
  pushLog('info', 'filePicker', `Browser file dialog opened (accept="${accept}")`)

  try {
    return await new Promise<string | null>((resolve) => {
      const input = document.createElement('input')
      input.type = 'file'
      input.accept = accept
      input.style.display = 'none'

      let settled = false
      const done = (value: string | null) => {
        if (settled) return
        settled = true
        input.remove()
        pushLog('info', 'filePicker', `Selected: ${value ?? '(cancelled)'}`)
        resolve(value)
      }

      input.onchange = () => {
        const file = input.files?.[0]
        if (!file) { done(null); return }
        const path: string = (file as any).path ?? file.webkitRelativePath ?? file.name
        done(path || null)
      }

      // Fallback: resolve null if dialog closes without selection.
      // Some browsers don't fire `oncancel`; the focus event catches those.
      window.addEventListener('focus', () => {
        setTimeout(() => { if (input.parentNode) done(null) }, 500)
      }, { once: true })

      // Absolute last-resort timeout (5 minutes)
      setTimeout(() => done(null), 300_000)

      document.body.appendChild(input)
      input.click()
    })
  } catch (e) {
    pushLog('error', 'filePicker', `File dialog error: ${e}`)
    return null
  }
}

async function browserOpenDirectory(): Promise<string | null> {
  try {
    const w = window as any
    const handle = await w.showDirectoryPicker()
    return handle.path ?? handle.name
  } catch {
    return null
  }
}
