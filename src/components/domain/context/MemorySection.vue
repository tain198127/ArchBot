<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from '../../../i18n'
import { useProject } from '../../../stores/project'
import VButton from '../../base/VButton.vue'
import VInput from '../../base/VInput.vue'
import VTextarea from '../../base/VTextarea.vue'

const { tt } = useI18n()
const { currentProject } = useProject()

interface MemoryItem {
  name: string
  description: string
  content: string
}

const items = ref<MemoryItem[]>([])
const loading = ref(false)
const editing = ref(false)
const editItem = ref<MemoryItem>({ name: '', description: '', content: '' })

async function loadItems() {
  const path = currentProject.value?.path
  if (!path) return
  loading.value = true
  try {
    const names = await invoke<string[]>('list_context_entries', { projectPath: path, section: 'memory' })
    const results: MemoryItem[] = []
    for (const name of names) {
      const entry = await invoke<MemoryItem>('get_context_entry', { projectPath: path, section: 'memory', name })
      results.push(entry)
    }
    items.value = results
  } catch { /* no data yet */ }
  finally { loading.value = false }
}

function startNew() {
  editItem.value = { name: '', description: '', content: '' }
  editing.value = true
}

function startEdit(item: MemoryItem) {
  editItem.value = { ...item }
  editing.value = true
}

async function saveItem() {
  const path = currentProject.value?.path
  if (!path || !editItem.value.name.trim()) return
  await invoke('save_context_entry', { projectPath: path, section: 'memory', entry: editItem.value })
  editing.value = false
  await loadItems()
}

async function deleteItem(name: string) {
  const path = currentProject.value?.path
  if (!path) return
  await invoke('delete_context_entry', { projectPath: path, section: 'memory', name })
  await loadItems()
}

onMounted(loadItems)
</script>

<template>
  <div class="p-4 h-full overflow-y-auto">
    <div class="flex items-center justify-between mb-3">
      <span class="text-sm font-medium text-text-primary">{{ tt('context.memory') }}</span>
      <VButton variant="primary" size="sm" @click="startNew">+ Add</VButton>
    </div>

    <div v-if="loading" class="text-xs text-text-muted">Loading...</div>

    <div v-else-if="editing" class="space-y-3">
      <VInput v-model="editItem.name" placeholder="Title" />
      <VInput v-model="editItem.description" placeholder="Description" />
      <VTextarea v-model="editItem.content" placeholder="Content" :rows="6" />
      <div class="flex gap-2">
        <VButton variant="primary" size="sm" @click="saveItem">Save</VButton>
        <VButton variant="secondary" size="sm" @click="editing = false">Cancel</VButton>
      </div>
    </div>

    <div v-else-if="items.length === 0" class="text-xs text-text-muted py-4 text-center">
      {{ tt('context.noData') }}
    </div>

    <div v-else class="space-y-2">
      <div v-for="item in items" :key="item.name"
        class="border border-border-default rounded p-2 cursor-pointer hover:border-primary-300"
        @click="startEdit(item)">
        <div class="flex items-center justify-between">
          <span class="text-sm text-text-primary font-medium">{{ item.name }}</span>
          <button class="text-xs text-danger-500 hover:underline" @click.stop="deleteItem(item.name)">Delete</button>
        </div>
        <p class="text-xs text-text-secondary mt-0.5 truncate">{{ item.description }}</p>
      </div>
    </div>
  </div>
</template>
