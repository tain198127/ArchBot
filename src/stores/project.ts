import { ref } from 'vue'

export interface ProjectInfo {
  name: string
  path: string
  content: string
}

const currentProject = ref<ProjectInfo | null>(null)

export function useProject() {
  function setProject(project: ProjectInfo) {
    currentProject.value = project
  }

  function closeProject() {
    currentProject.value = null
  }

  return {
    currentProject,
    setProject,
    closeProject
  }
}
