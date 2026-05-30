import { open } from '@tauri-apps/plugin-dialog'
import { isTauri } from './env'

export interface FileFilter {
  name: string
  extensions: string[]
}

/**
 * Open a file picker dialog. In Tauri mode uses the native dialog;
 * in browser mode uses the File System Access API.
 *
 * Returns the absolute file path string, or null if cancelled.
 * Browsers only expose the path in Chrome/Edge; other browsers
 * return the filename only (suitable for display but not for
 * backend file operations).
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

async function browserOpenFile(_filters: FileFilter[]): Promise<string | null> {
  try {
    // Use a classic <input type="file"> approach — it exposes the full
    // filesystem path via `webkitRelativePath` in Chromium browsers.
    // File System Access API's showOpenFilePicker() does NOT give us
    // the absolute path (only filename), which the backend needs.
    return await new Promise<string | null>((resolve) => {
      const input = document.createElement('input')
      input.type = 'file'
      input.accept = _filters.flatMap((f) => f.extensions.map((e) => `.${e}`)).join(',')
      input.style.display = 'none'

      input.onchange = () => {
        const file = input.files?.[0]
        resolve(file ? ((file as any).path ?? file.webkitRelativePath ?? file.name) : null)
        input.remove()
      }

      // Handle cancel (no file selected)
      input.oncancel = () => {
        resolve(null)
        input.remove()
      }

      // Fallback: if dialog closes without change or cancel event
      const cleanup = () => {
        setTimeout(() => {
          if (input.parentNode) {
            resolve(null)
            input.remove()
          }
        }, 1000)
      }

      document.body.appendChild(input)
      input.click()
      window.addEventListener('focus', cleanup, { once: true })
    })
  } catch {
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
