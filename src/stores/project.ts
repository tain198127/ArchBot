import { ref } from 'vue'

export interface ProjectInfo {
  name: string
  path: string
  content: string
}

export interface RecentProject {
  name: string
  path: string
}

const STORAGE_KEY = 'archbot_recent_projects'
const MAX_RECENT = 5

const currentProject = ref<ProjectInfo | null>(null)

function loadRecent(): RecentProject[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) {
      return JSON.parse(raw) as RecentProject[]
    }
  } catch { /* ignore */ }
  return []
}

const recentProjects = ref<RecentProject[]>(loadRecent())

function persistRecent() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(recentProjects.value))
}

export function useProject() {
  function setProject(project: ProjectInfo) {
    currentProject.value = project
    addRecentProject({ name: project.name, path: project.path })
  }

  function closeProject() {
    currentProject.value = null
  }

  function addRecentProject(project: RecentProject) {
    const filtered = recentProjects.value.filter(p => p.path !== project.path)
    recentProjects.value = [{ ...project }, ...filtered].slice(0, MAX_RECENT)
    persistRecent()
  }

  function clearRecentProjects() {
    recentProjects.value = []
    persistRecent()
  }

  return {
    currentProject,
    recentProjects,
    setProject,
    closeProject,
    addRecentProject,
    clearRecentProjects
  }
}
