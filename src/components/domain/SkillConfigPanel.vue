<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import VButton from '../base/VButton.vue'
import VInput from '../base/VInput.vue'
import VSelect from '../base/VSelect.vue'
import VTextarea from '../base/VTextarea.vue'
import { useI18n } from '../../i18n'
import { useToast } from '../../composables/useToast'

const { t } = useI18n()
const toast = useToast()
const DB_TYPE = 'local'

const skills = ref<any[]>([])
const loading = ref(false)
const editMode = ref(false)
const isNew = ref(true)
const editForm = ref({ code: '', name: '', label: '', command: '', description: '', body: '', group_name: '' })
const skillBodyInput = ref('')

async function loadSkills() {
  loading.value = true
  try {
    const result = await invoke<any>('db_find_all', {
      table: 'skills',
      params: { filters: [], orderBy: [], limit: null, offset: null },
      dbType: DB_TYPE,
    })
    skills.value = (result as any).rows || result
  } catch (e) {
    toast.error(String(e))
  } finally {
    loading.value = false
  }
}

function handleNew() {
  isNew.value = true
  editMode.value = true
  editForm.value = { code: '', name: '', label: '', command: '', description: '', body: '', group_name: '' }
  skillBodyInput.value = ''
}

function handleEdit(skill: any) {
  isNew.value = false
  editMode.value = true
  editForm.value = { ...skill, body: skill.body || '', group_name: skill.group_name || '' }
  skillBodyInput.value = skill.body || ''
}

function backToList() {
  editMode.value = false
  loadSkills()
}

async function handleSave() {
  if (!editForm.value.code || !editForm.value.name) {
    toast.warning('Code and Name are required')
    return
  }
  editForm.value.command = editForm.value.command || `/${editForm.value.code}`
  // Build body from markdown content
  if (skillBodyInput.value) {
    editForm.value.body = buildSkillBody()
  }
  try {
    await invoke('db_insert', {
      table: 'skills',
      data: {
        ...editForm.value,
        updated_at: new Date().toISOString(),
      },
      dbType: DB_TYPE,
    })
    toast.success(isNew.value ? 'Skill created' : 'Skill saved')
    backToList()
  } catch (e: any) {
    // Try update if insert fails (for existing records)
    if (String(e).includes('UNIQUE')) {
      try {
        await invoke('db_update', {
          table: 'skills',
          id: editForm.value.code,
          data: { ...editForm.value, updated_at: new Date().toISOString() },
          dbType: DB_TYPE,
        })
        toast.success('Skill updated')
        backToList()
      } catch (e2: any) {
        toast.error(String(e2))
      }
    } else {
      toast.error(String(e))
    }
  }
}

async function handleDelete(skill: any) {
  toast.confirm('Delete Skill', `Delete "${skill.name}"?`).then(async (confirmed) => {
    if (!confirmed) return
    try {
      await invoke('db_delete', { table: 'skills', id: skill.code, dbType: DB_TYPE })
      toast.success('Deleted')
      loadSkills()
    } catch (e: any) {
      toast.error(String(e))
    }
  })
}

function buildSkillBody(): string {
  const lines: string[] = []
  lines.push('---')
  lines.push(`name: ${editForm.value.name}`)
  if (editForm.value.description) lines.push(`description: ${editForm.value.description}`)
  if (editForm.value.group_name) lines.push(`group: ${editForm.value.group_name}`)
  lines.push('---')
  lines.push('')
  lines.push(skillBodyInput.value)
  return lines.join('\n')
}

// ── Generate using skill-create ──
const generating = ref(false)
async function handleGenerate() {
  if (!editForm.value.name) {
    toast.warning('Please enter a skill name first')
    return
  }
  generating.value = true
  try {
    // Use the everything-claude-code skill-create to generate SKILL.md content
    const prompt = `Create a Claude Code skill with the following details:
Name: ${editForm.value.name}
Command: ${editForm.value.command || '/' + editForm.value.code}
Description: ${editForm.value.description || 'No description provided'}
${skillBodyInput.value ? 'Current draft content:\n' + skillBodyInput.value : 'Please generate the skill body content.'}

Generate comprehensive SKILL.md content for this skill including usage instructions, examples, and tool coordination.`

    const result = await invoke<string>('agent_execute_turn', {
      runtime: 'claude_code',
      sessionId: '',
      userMessage: prompt,
      contextFiles: [],
      workspaceRoot: '',
    })
    // Parse result — the skill-create response should contain the generated content
    skillBodyInput.value = result || skillBodyInput.value
    toast.success('Skill content generated')
  } catch (e: any) {
    toast.error(String(e))
  } finally {
    generating.value = false
  }
}

onMounted(() => { loadSkills() })
</script>

<template>
  <div class="flex flex-col h-full bg-surface-50 dark:bg-surface-50 p-4 overflow-y-auto">
    <!-- List View -->
    <template v-if="!editMode">
      <div class="flex gap-2 mb-3 items-center">
        <h3 class="text-base font-semibold text-text-primary flex-1">{{ t.menuConfig.skill || 'Skill Config' }}</h3>
        <VButton size="sm" @click="handleNew">+ New Skill</VButton>
      </div>
      <table class="w-full border-collapse text-sm">
        <thead>
          <tr>
            <th class="text-left px-3 py-2 border-b-2 border-border-default text-text-secondary font-semibold w-[140px]">Name</th>
            <th class="text-left px-3 py-2 border-b-2 border-border-default text-text-secondary font-semibold w-[120px]">Command</th>
            <th class="text-left px-3 py-2 border-b-2 border-border-default text-text-secondary font-semibold">Description</th>
            <th class="text-left px-3 py-2 border-b-2 border-border-default text-text-secondary font-semibold w-[100px]">Group</th>
            <th class="text-left px-3 py-2 border-b-2 border-border-default text-text-secondary font-semibold w-[80px]">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="s in skills"
            :key="s.code"
            class="cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-100"
            @click="handleEdit(s)"
          >
            <td class="px-3 py-2 border-b border-border-default font-medium text-primary-500">{{ s.name }}</td>
            <td class="px-3 py-2 border-b border-border-default font-mono text-xs">{{ s.command || '/' + s.code }}</td>
            <td class="px-3 py-2 border-b border-border-default text-text-secondary text-xs">{{ s.description?.slice(0, 80) || '' }}</td>
            <td class="px-3 py-2 border-b border-border-default text-xs">{{ s.group_name || '游击队' }}</td>
            <td class="px-3 py-2 border-b border-border-default" @click.stop>
              <VButton size="sm" variant="danger" @click="handleDelete(s)">Del</VButton>
            </td>
          </tr>
          <tr v-if="skills.length === 0">
            <td colspan="5" class="text-center text-text-muted py-6">No skills defined yet</td>
          </tr>
        </tbody>
      </table>
    </template>

    <!-- Edit View -->
    <template v-else>
      <div class="flex gap-3 items-center mb-4">
        <VButton size="sm" variant="ghost" @click="backToList">&larr; Back</VButton>
        <span class="text-base font-semibold text-text-primary">{{ isNew ? 'Create Skill' : 'Edit Skill' }}</span>
      </div>

      <div class="flex-1 overflow-y-auto space-y-3 max-w-[640px]">
        <!-- Required fields -->
        <div class="flex items-center gap-3">
          <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">Code *</label>
          <div class="flex-1"><VInput v-model="editForm.code" placeholder="skill-code" /></div>
        </div>
        <div class="flex items-center gap-3">
          <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">Name *</label>
          <div class="flex-1"><VInput v-model="editForm.name" placeholder="Display name" /></div>
        </div>
        <div class="flex items-center gap-3">
          <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">Command</label>
          <div class="flex-1"><VInput v-model="editForm.command" placeholder="/my-command" /></div>
        </div>
        <div class="flex items-center gap-3">
          <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">Group</label>
          <div class="flex-1">
            <VSelect
              v-model="editForm.group_name"
              :options="[
                { value: '', label: '游击队 (Guerrillas)' },
                { value: 'superpowers', label: 'Super Power' },
                { value: 'gstack', label: 'Gstack' },
                { value: 'everything-claude-code', label: 'Everything Claude Code' },
                { value: 'super-claude', label: 'Super Claude' },
                { value: 'claude-official-skills', label: 'OpenSpec' },
              ]"
              placeholder="Select group"
            />
          </div>
        </div>
        <div class="flex items-start gap-3">
          <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right pt-1">Description</label>
          <div class="flex-1"><VTextarea v-model="editForm.description" placeholder="Brief description" /></div>
        </div>

        <!-- SKILL.md body -->
        <fieldset class="border border-border-default rounded-lg px-4 py-3">
          <legend class="text-sm font-semibold text-text-secondary px-1.5">SKILL.md Content</legend>
          <VTextarea
            v-model="skillBodyInput"
            placeholder="# Skill Name&#10;&#10;## Usage&#10;/my-command&#10;&#10;## Description&#10;..."
            class="!h-[200px] font-mono text-xs"
          />
          <div class="flex items-center gap-2 mt-2">
            <VButton size="sm" variant="secondary" :loading="generating" @click="handleGenerate">
              ⚡ Generate via skill-create
            </VButton>
            <span class="text-[11px] text-text-muted">Uses Everything Claude Code to auto-generate content</span>
          </div>
        </fieldset>

        <!-- Footer -->
        <div class="flex gap-2 py-3 border-t border-border-default">
          <VButton @click="handleSave">Save</VButton>
          <VButton variant="ghost" @click="backToList">Cancel</VButton>
        </div>
      </div>
    </template>
  </div>
</template>
