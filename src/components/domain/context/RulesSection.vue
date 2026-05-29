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

interface RuleItem {
  name: string
  description: string
  content: string
}

const rules = ref<RuleItem[]>([])
const loading = ref(false)
const editing = ref(false)
const editItem = ref<RuleItem>({ name: '', description: '', content: '' })

async function loadRules() {
  const path = currentProject.value?.path
  if (!path) return
  loading.value = true
  try {
    const names = await invoke<string[]>('list_context_entries', { projectPath: path, section: 'rules' })
    const items: RuleItem[] = []
    for (const name of names) {
      const entry = await invoke<{ name: string; description: string; content: string }>('get_context_entry', { projectPath: path, section: 'rules', name })
      items.push(entry)
    }
    rules.value = items
  } catch { /* no data yet */ }
  finally { loading.value = false }
}

function startNew() {
  editItem.value = { name: '', description: '', content: '' }
  editing.value = true
}

function startEdit(item: RuleItem) {
  editItem.value = { ...item }
  editing.value = true
}

async function saveRule() {
  const path = currentProject.value?.path
  if (!path || !editItem.value.name.trim()) return
  await invoke('save_context_entry', { projectPath: path, section: 'rules', entry: editItem.value })
  editing.value = false
  await loadRules()
}

async function deleteRule(name: string) {
  const path = currentProject.value?.path
  if (!path) return
  await invoke('delete_context_entry', { projectPath: path, section: 'rules', name })
  await loadRules()
}

onMounted(loadRules)
</script>

<template>
  <div class="p-4 h-full overflow-y-auto">
    <div class="flex items-center justify-between mb-3">
      <span class="text-sm font-medium text-text-primary">{{ tt('context.rules') }}</span>
      <VButton variant="primary" size="sm" @click="startNew">+ Add</VButton>
    </div>

    <div v-if="loading" class="text-xs text-text-muted">Loading...</div>

    <div v-else-if="editing" class="space-y-3">
      <VInput v-model="editItem.name" placeholder="Rule name" />
      <VInput v-model="editItem.description" placeholder="Description" />
      <VTextarea v-model="editItem.content" placeholder="Rule content" :rows="6" />
      <div class="flex gap-2">
        <VButton variant="primary" size="sm" @click="saveRule">Save</VButton>
        <VButton variant="secondary" size="sm" @click="editing = false">Cancel</VButton>
      </div>
    </div>

    <div v-else-if="rules.length === 0" class="text-xs text-text-muted py-4 text-center">
      {{ tt('context.noData') }}
    </div>

    <div v-else class="space-y-2">
      <div v-for="rule in rules" :key="rule.name"
        class="border border-border-default rounded p-2 cursor-pointer hover:border-primary-300"
        @click="startEdit(rule)">
        <div class="flex items-center justify-between">
          <span class="text-sm text-text-primary font-medium">{{ rule.name }}</span>
          <button class="text-xs text-danger-500 hover:underline" @click.stop="deleteRule(rule.name)">Delete</button>
        </div>
        <p class="text-xs text-text-secondary mt-0.5 truncate">{{ rule.description }}</p>
      </div>
    </div>
  </div>
</template>
