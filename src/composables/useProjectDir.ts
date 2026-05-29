import { useProject } from '../stores/project'

export function useProjectDir(): () => string {
  const { currentProject } = useProject()

  return function projectDir(): string {
    if (!currentProject.value) return ''
    const p = currentProject.value.path
    const idx = Math.max(p.lastIndexOf('/'), p.lastIndexOf('\\'))
    return idx >= 0 ? p.substring(0, idx) : p
  }
}
