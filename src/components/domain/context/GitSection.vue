<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../../../i18n'
import { useProject } from '../../../stores/project'
import VButton from '../../base/VButton.vue'
import VTextarea from '../../base/VTextarea.vue'

const { tt } = useI18n()
const { currentProject } = useProject()

const content = ref('')
const loading = ref(false)
const saved = ref(false)

async function load() {
  const path = currentProject.value?.path
  if (!path) return
  loading.value = true
  try {
    content.value = await invoke<string>('get_context_config', { projectPath: path, section: 'git' })
  } catch { content.value = '' }
  finally { loading.value = false }
}

async function saveConfig() {
  const path = currentProject.value?.path
  if (!path) return
  await invoke('save_context_config', { projectPath: path, section: 'git', content: content.value })
  saved.value = true
  setTimeout(() => (saved.value = false), 2000)
}

onMounted(load)
</script>

<template>
  <div class="p-4 h-full overflow-y-auto flex flex-col">
    <div class="flex items-center justify-between mb-3">
      <span class="text-sm font-medium text-text-primary">{{ tt('context.git') }}</span>
      <div class="flex items-center gap-2">
        <span v-if="saved" class="text-xs text-success-500">Saved</span>
        <VButton variant="primary" size="sm" @click="saveConfig">Save</VButton>
      </div>
    </div>
    <div v-if="loading" class="text-xs text-text-muted">Loading...</div>
    <VTextarea v-model="content" placeholder="Git context configuration (YAML)" :rows="12" class="flex-1" />
  </div>
</template>
