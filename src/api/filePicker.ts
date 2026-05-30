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

async function browserOpenFile(filters: FileFilter[]): Promise<string | null> {
  try {
    const exts = filters.flatMap((f) => f.extensions.map((e) => `.${e}`))
    const types = exts.length > 0
      ? [{ description: filters.map((f) => f.name).join(', '), accept: { 'application/octet-stream': exts } }]
      : undefined

    const w = window as any
    const opts: any = { multiple: false }
    if (types) opts.types = types
    const [handle] = await w.showOpenFilePicker(opts)
    const file = await handle.getFile()
    return file.path ?? file.name
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
