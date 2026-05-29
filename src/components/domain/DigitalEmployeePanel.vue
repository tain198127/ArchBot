<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import VButton from '../base/VButton.vue'
import VInput from '../base/VInput.vue'
import VSelect from '../base/VSelect.vue'
import VTextarea from '../base/VTextarea.vue'
import VCheckbox from '../base/VCheckbox.vue'
import { useToast } from '../../composables/useToast'
import { useI18n } from '../../i18n'

const { t } = useI18n()
const toast = useToast()
const de = computed(() => t.value.digitalEmployee as Record<string, string>)

const DB_TYPE = 'local'

// ── list state ──
const employees = ref<any[]>([])
const searchText = ref('')
const selectedIds = ref<Set<number>>(new Set())
const loading = ref(false)

// ── edit state ──
const editMode = ref<'list' | 'edit' | 'view'>('list')
const isNew = ref(false)
const editForm = ref<any>({})
const editSkills = ref<string[]>([])
const editAgents = ref<string[]>([])
const editMcps = ref<string[]>([])
const editHandoffs = ref<any[]>([])

// ── lookup data ──
const employeeOptions = ref<any[]>([])
const skillOptions = ref<any[]>([])
const agentOptions = ref<any[]>([])
const mcpOptions = ref<any[]>([])

const commStyleOptions = ['正式', '亲和', '幽默', '严谨', '睿智']
const decisionPrefOptions = ['逻辑+规则', '经验+直觉', '数据+分析', '协作+共识']
const focusAreaOptions = ['需求分析', '系统设计', '代码开发', '测试验证', '部署运维', '项目管理', '数据治理', '安全管理']
const deliverableOptions = ['PRD', 'SRS', '架构设计', '详细设计', '代码实现', '测试用例', '部署方案', '运维手册']
const operationOptions = ['read', 'write', 'review', 'analyze', 'brainstorm', 'design', 'code', 'test', 'deploy']
const transferModeOptions = [
  { value: 'notify', label: '通知' },
  { value: 'consult', label: '咨询' },
  { value: 'delegate', label: '委托' },
]

const filteredEmployees = computed(() => {
  const q = searchText.value.toLowerCase()
  if (!q) return employees.value
  return employees.value.filter((e: any) =>
    (e.name || '').toLowerCase().includes(q) ||
    (e.code || '').toLowerCase().includes(q) ||
    (e.role || '').toLowerCase().includes(q)
  )
})

function parseJsonArray(val: any): string[] {
  if (!val) return []
  if (Array.isArray(val)) return val
  try { const p = JSON.parse(val); return Array.isArray(p) ? p : [] } catch { return String(val).split(',').map(s => s.trim()).filter(Boolean) }
}

function toJsonArray(arr: string[]): string {
  return JSON.stringify(arr)
}

// ── data loading ──
async function loadEmployees() {
  loading.value = true
  try {
    await invoke('de_init', { dbType: DB_TYPE })
    employees.value = await invoke<any[]>('de_list', { dbType: DB_TYPE })
  } catch (e) {
    toast.error(String(e))
  } finally {
    loading.value = false
  }
}

async function loadLookups() {
  try {
    const [emps, skills, agents, mcps] = await Promise.all([
      invoke<any[]>('de_list', { dbType: DB_TYPE }),
      invoke<any[]>('db_find_all', { table: 'skills', params: { filters: [], orderBy: [], limit: null, offset: null }, dbType: DB_TYPE }),
      invoke<any[]>('db_find_all', { table: 'agents', params: { filters: [], orderBy: [], limit: null, offset: null }, dbType: DB_TYPE }),
      invoke<any[]>('db_find_all', { table: 'mcps', params: { filters: [], orderBy: [], limit: null, offset: null }, dbType: DB_TYPE }),
    ])
    employeeOptions.value = emps
    skillOptions.value = (skills as any).rows || skills
    agentOptions.value = (agents as any).rows || agents
    mcpOptions.value = (mcps as any).rows || mcps
  } catch { /* ignore */ }
}

// ── list actions ──
function handleNew() {
  isNew.value = true; editMode.value = 'edit'
  editForm.value = { code: '', name: '', is_builtin: false, role: '', personality_tags: '[]', personality_desc: '', comm_style: '', decision_pref: '', focus_areas: '[]', deliverable_groups: '[]', default_op: 'read', avatar: '' }
  editSkills.value = []; editAgents.value = []; editMcps.value = []; editHandoffs.value = []
}

function handleEdit() {
  if (selectedIds.value.size !== 1) { toast.warning(de.value.selectOneHint || '请选择一个员工'); return }
  const id = [...selectedIds.value][0]
  const emp = employees.value.find((e: any) => e.id === id)
  if (!emp) return
  isNew.value = false; editMode.value = 'edit'
  editForm.value = { ...emp }
  editSkills.value = parseJsonArray(emp.skills)
  editAgents.value = parseJsonArray(emp.agents)
  editMcps.value = parseJsonArray(emp.mcps)
  editHandoffs.value = (parseJsonArray(emp.handoffs) as any[]).map((h: any) => typeof h === 'string' ? JSON.parse(h) : h)
}

function handleView() {
  if (selectedIds.value.size !== 1) { toast.warning(de.value.selectOneHint || '请选择一个员工'); return }
  const id = [...selectedIds.value][0]
  const emp = employees.value.find((e: any) => e.id === id)
  if (!emp) return
  isNew.value = false; editMode.value = 'view'
  editForm.value = { ...emp }
  editSkills.value = parseJsonArray(emp.skills)
  editAgents.value = parseJsonArray(emp.agents)
  editMcps.value = parseJsonArray(emp.mcps)
  editHandoffs.value = (parseJsonArray(emp.handoffs) as any[]).map((h: any) => typeof h === 'string' ? JSON.parse(h) : h)
}

function handleRowClick(row: any) {
  selectedIds.value = new Set([row.id])
  isNew.value = false; editMode.value = 'view'
  editForm.value = { ...row }
  editSkills.value = parseJsonArray(row.skills)
  editAgents.value = parseJsonArray(row.agents)
  editMcps.value = parseJsonArray(row.mcps)
  editHandoffs.value = (parseJsonArray(row.handoffs) as any[]).map((h: any) => typeof h === 'string' ? JSON.parse(h) : h)
}

// ── edit actions ──
async function handleSave() {
  if (!editForm.value.code || !editForm.value.name) {
    toast.warning(de.value.nameOrCodeRequired)
    return
  }
  editForm.value.updated_at = new Date().toISOString()
  try {
    await invoke('de_save', { employee: editForm.value, dbType: DB_TYPE })
    toast.success(isNew.value ? de.value.createSuccess : de.value.saveSuccess)
    backToList()
  } catch (e) {
    toast.error(String(e))
  }
}

function handleCopy() {
  isNew.value = true
  editForm.value = { ...editForm.value, id: undefined, code: '', name: editForm.value.name + ' (副本)', is_builtin: false }
  editSkills.value = [...editSkills.value]
  editAgents.value = [...editAgents.value]
  editMcps.value = [...editMcps.value]
  editHandoffs.value = editHandoffs.value.map((h: any) => ({ ...h, id: undefined }))
}

function handleDelete() {
  if (editForm.value.is_builtin) {
    toast.warning(de.value.builtinCannotDelete)
    return
  }
  toast.confirm(de.value.deleteConfirmTitle, de.value.deleteConfirmMessage.replace('{name}', editForm.value.name)).then(async (confirmed) => {
    if (!confirmed) return
    try {
      await invoke('de_delete', { id: editForm.value.id, dbType: DB_TYPE })
      toast.success(de.value.deleted)
      backToList()
    } catch (e) { toast.error(String(e)) }
  })
}

function backToList() {
  editMode.value = 'list'
  selectedIds.value = new Set()
  loadEmployees()
}

// ── handoff helpers ──
function addHandoff() {
  editHandoffs.value.push({ employee_code: editForm.value.code, trigger_op: 'write', target_employee_code: '', transfer_data: '[]', transfer_mode: 'notify', context_ref: '' })
}

function removeHandoff(idx: number) {
  editHandoffs.value = editHandoffs.value.filter((_: any, i: number) => i !== idx)
}

const opOptions = computed(() => operationOptions.map(o => ({ value: o, label: o })))
const empOptions = computed(() => employeeOptions.value.map((e: any) => ({ value: e.code, label: e.name })))
const tmOptions = computed(() => transferModeOptions.map(t => ({ value: t.value, label: t.label })))
onMounted(async () => { await loadEmployees(); await loadLookups() })
</script>

<template>
  <div class="flex flex-col h-full bg-surface-50 dark:bg-surface-50 p-4 overflow-y-auto">
    <!-- List View -->
    <template v-if="editMode === 'list'">
      <div class="flex gap-2 mb-3 items-center">
        <div class="flex-1 max-w-[240px]">
          <VInput v-model="searchText" :placeholder="de.searchPlaceholder || '查找员工...'" />
        </div>
        <VButton size="sm" @click="handleNew">{{ de.new || '新增' }}</VButton>
        <VButton size="sm" variant="secondary" @click="handleEdit">{{ de.edit || '编辑' }}</VButton>
        <VButton size="sm" variant="ghost" @click="handleView">{{ de.view || '查看' }}</VButton>
      </div>
      <table class="w-full border-collapse text-sm">
        <thead>
          <tr>
            <th class="text-left px-3 py-2 border-b-2 border-border-default text-text-secondary font-semibold w-[50px]">#</th>
            <th class="text-left px-3 py-2 border-b-2 border-border-default text-text-secondary font-semibold w-[140px]">{{ de.name || '姓名' }}</th>
            <th class="text-left px-3 py-2 border-b-2 border-border-default text-text-secondary font-semibold w-[140px]">{{ de.role || '角色' }}</th>
            <th class="text-left px-3 py-2 border-b-2 border-border-default text-text-secondary font-semibold w-[160px]">{{ de.personality || '性格' }}</th>
            <th class="text-left px-3 py-2 border-b-2 border-border-default text-text-secondary font-semibold w-[90px]">{{ de.type || '类型' }}</th>
            <th class="text-left px-3 py-2 border-b-2 border-border-default text-text-secondary font-semibold">{{ de.focusAreas || '专注领域' }}</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(row, idx) in filteredEmployees" :key="row.id"
            class="cursor-pointer" :class="selectedIds.has(row.id) ? 'bg-blue-100 dark:bg-blue-900/20' : ''"
            @click="selectedIds = new Set([row.id])"
            @dblclick="handleRowClick(row)"
          >
            <td class="px-3 py-2 border-b border-border-default">{{ idx + 1 }}</td>
            <td class="px-3 py-2 border-b border-border-default text-primary-500 cursor-pointer" @click="handleRowClick(row)">{{ row.name }}</td>
            <td class="px-3 py-2 border-b border-border-default">{{ row.code }}</td>
            <td class="px-3 py-2 border-b border-border-default">{{ parseJsonArray(row.personality_tags).join('·') }}</td>
            <td class="px-3 py-2 border-b border-border-default">{{ row.is_builtin ? (de.builtin || '内置') : (de.custom || '自定义') }}</td>
            <td class="px-3 py-2 border-b border-border-default">{{ parseJsonArray(row.focus_areas).join('+') }}</td>
          </tr>
          <tr v-if="filteredEmployees.length === 0">
            <td colspan="6" class="text-center text-text-muted py-6">{{ de.noData || '暂无数据' }}</td>
          </tr>
        </tbody>
      </table>
    </template>

    <!-- Edit / View -->
    <template v-else>
      <div class="flex gap-3 items-center mb-4">
        <VButton size="sm" variant="ghost" @click="backToList">&larr; {{ de.back || '返回列表' }}</VButton>
        <span class="text-base font-semibold text-text-primary">{{ isNew ? (de.newEmployee || '新增数字员工') : (editMode === 'view' ? (de.viewEmployee || '查看数字员工') : (de.editEmployee || '编辑数字员工')) }}</span>
      </div>

      <div class="flex-1 overflow-y-auto">
        <!-- Basic Info -->
        <fieldset class="border border-border-default rounded-lg px-4 py-3 mb-3">
          <legend class="text-sm font-semibold text-text-secondary px-1.5">{{ de.basicInfo || '基础信息' }}</legend>
          <div class="flex items-center gap-3 mb-2">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.name || '姓名' }}</label>
            <div class="flex-1 max-w-[360px]"><VInput v-model="editForm.name" :disabled="editMode === 'view'" /></div>
          </div>
          <div class="flex items-center gap-3 mb-2">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.code || '编码' }}</label>
            <div class="flex-1 max-w-[360px]"><VInput v-model="editForm.code" :disabled="editMode === 'view' || (!isNew && editForm.is_builtin)" /></div>
          </div>
          <div class="flex items-center gap-3 mb-2">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.type || '类型' }}</label>
            <span class="text-sm text-text-primary">{{ editForm.is_builtin ? (de.builtin || '内置') : (de.custom || '自定义') }}</span>
          </div>
          <div class="flex items-center gap-3">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.avatar || '头像' }}</label>
            <div class="flex-1 max-w-[120px]"><VInput v-model="editForm.avatar" :disabled="editMode === 'view'" /></div>
          </div>
        </fieldset>

        <!-- Personality -->
        <fieldset class="border border-border-default rounded-lg px-4 py-3 mb-3">
          <legend class="text-sm font-semibold text-text-secondary px-1.5">{{ de.personality || '人格特征' }}</legend>
          <div class="flex items-center gap-3 mb-2">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.personalityTags || '性格标签' }}</label>
            <div class="flex-1 max-w-[360px]"><VInput v-model="editForm.personality_tags" :disabled="editMode === 'view'" /></div>
          </div>
          <div class="flex items-center gap-3 mb-2">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.personalityDesc || '性格描述' }}</label>
            <div class="flex-1 max-w-[360px]"><VTextarea v-model="editForm.personality_desc" :disabled="editMode === 'view'" /></div>
          </div>
          <div class="flex items-center gap-3 mb-2">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.commStyle || '沟通风格' }}</label>
            <VSelect v-model="editForm.comm_style" :options="commStyleOptions.map(s => ({ value: s, label: s }))" class="!w-[200px]" :disabled="editMode === 'view'" />
          </div>
          <div class="flex items-center gap-3">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.decisionPref || '决策偏好' }}</label>
            <VSelect v-model="editForm.decision_pref" :options="decisionPrefOptions.map(s => ({ value: s, label: s }))" class="!w-[200px]" :disabled="editMode === 'view'" />
          </div>
        </fieldset>

        <!-- Capabilities -->
        <fieldset class="border border-border-default rounded-lg px-4 py-3 mb-3">
          <legend class="text-sm font-semibold text-text-secondary px-1.5">{{ de.capabilities || '能力配置' }}</legend>
          <div class="flex items-center gap-3 mb-2">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.focusAreas || '专注领域' }}</label>
            <div class="flex gap-3 flex-wrap">
              <VCheckbox v-for="a in focusAreaOptions" :key="a" :model-value="parseJsonArray(editForm.focus_areas).includes(a)" :label="a"
                :disabled="editMode === 'view'"
                @update:model-value="(v: boolean) => { const arr = parseJsonArray(editForm.focus_areas); if (v) arr.push(a); else arr.splice(arr.indexOf(a), 1); editForm.focus_areas = toJsonArray(arr) }" />
            </div>
          </div>
          <div class="flex items-center gap-3 mb-2">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.deliverableGroups || '交付物组' }}</label>
            <div class="flex gap-3 flex-wrap">
              <VCheckbox v-for="d in deliverableOptions" :key="d" :model-value="parseJsonArray(editForm.deliverable_groups).includes(d)" :label="d"
                :disabled="editMode === 'view'"
                @update:model-value="(v: boolean) => { const arr = parseJsonArray(editForm.deliverable_groups); if (v) arr.push(d); else arr.splice(arr.indexOf(d), 1); editForm.deliverable_groups = toJsonArray(arr) }" />
            </div>
          </div>
          <div class="flex items-center gap-3">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.defaultOp || '默认操作' }}</label>
            <VSelect v-model="editForm.default_op" :options="opOptions" class="!w-[200px]" :disabled="editMode === 'view'" />
          </div>
        </fieldset>

        <!-- Skill Stack -->
        <fieldset class="border border-border-default rounded-lg px-4 py-3 mb-3">
          <legend class="text-sm font-semibold text-text-secondary px-1.5">{{ de.skillStack || '技能栈' }}</legend>
          <div class="flex items-center gap-3 mb-2">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">Skill</label>
            <div class="flex gap-1.5 flex-wrap items-center">
              <span v-for="s in editSkills" :key="s" class="inline-flex items-center gap-1 px-2 py-0.5 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded text-xs text-primary-500">
                {{ s }}
                <button v-if="editMode === 'edit'" class="border-0 bg-transparent text-text-muted cursor-pointer text-xs px-0.5 hover:text-danger-500" @click="editSkills = editSkills.filter(x => x !== s)">&#10005;</button>
              </span>
              <select v-if="editMode === 'edit'" class="h-7 px-1.5 border border-border-default rounded text-xs" @change="(e: any) => { if (e.target.value && !editSkills.includes(e.target.value)) editSkills.push(e.target.value); e.target.value = '' }">
                <option value="">+ 添加</option>
                <option v-for="sk in skillOptions" :key="sk.code" :value="sk.code">{{ sk.name }}</option>
              </select>
            </div>
          </div>
          <div class="flex items-center gap-3 mb-2">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">Agent</label>
            <div class="flex gap-1.5 flex-wrap items-center">
              <span v-for="a in editAgents" :key="a" class="inline-flex items-center gap-1 px-2 py-0.5 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded text-xs text-primary-500">
                {{ a }}
                <button v-if="editMode === 'edit'" class="border-0 bg-transparent text-text-muted cursor-pointer text-xs px-0.5 hover:text-danger-500" @click="editAgents = editAgents.filter(x => x !== a)">&#10005;</button>
              </span>
              <select v-if="editMode === 'edit'" class="h-7 px-1.5 border border-border-default rounded text-xs" @change="(e: any) => { if (e.target.value && !editAgents.includes(e.target.value)) editAgents.push(e.target.value); e.target.value = '' }">
                <option value="">+ 添加</option>
                <option v-for="ag in agentOptions" :key="ag.code" :value="ag.code">{{ ag.name }}</option>
              </select>
            </div>
          </div>
          <div class="flex items-center gap-3">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">MCP</label>
            <div class="flex gap-1.5 flex-wrap items-center">
              <span v-for="m in editMcps" :key="m" class="inline-flex items-center gap-1 px-2 py-0.5 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded text-xs text-primary-500">
                {{ m }}
                <button v-if="editMode === 'edit'" class="border-0 bg-transparent text-text-muted cursor-pointer text-xs px-0.5 hover:text-danger-500" @click="editMcps = editMcps.filter(x => x !== m)">&#10005;</button>
              </span>
              <select v-if="editMode === 'edit'" class="h-7 px-1.5 border border-border-default rounded text-xs" @change="(e: any) => { if (e.target.value && !editMcps.includes(e.target.value)) editMcps.push(e.target.value); e.target.value = '' }">
                <option value="">+ 添加</option>
                <option v-for="mc in mcpOptions" :key="mc.code" :value="mc.code">{{ mc.name }}</option>
              </select>
            </div>
          </div>
        </fieldset>

        <!-- Handoff Rules -->
        <fieldset class="border border-border-default rounded-lg px-4 py-3 mb-3">
          <legend class="text-sm font-semibold text-text-secondary px-1.5">{{ de.handoffRules || '交互规则' }}</legend>
          <div v-for="(h, idx) in editHandoffs" :key="idx" class="flex gap-2 items-center mb-1.5">
            <VSelect v-model="h.trigger_op" :options="opOptions" class="!h-7 !text-xs !w-[100px]" :disabled="editMode === 'view'" />
            <span class="text-text-muted text-sm">&rarr;</span>
            <VSelect v-model="h.target_employee_code" :options="empOptions" class="!h-7 !text-xs !w-[140px]" :disabled="editMode === 'view'" placeholder="选择角色" />
            <VSelect v-model="h.transfer_mode" :options="tmOptions" class="!h-7 !text-xs !w-[100px]" :disabled="editMode === 'view'" />
            <button v-if="editMode === 'edit'" class="px-1.5 py-0.5 text-xs border border-border-default rounded cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-100" @click="removeHandoff(idx)">&#10005;</button>
          </div>
          <VButton v-if="editMode === 'edit'" size="sm" variant="secondary" @click="addHandoff">+ {{ de.addHandoff || '添加交互规则' }}</VButton>
        </fieldset>

        <!-- Footer -->
        <div class="flex gap-2 py-3 border-t border-border-default">
          <VButton @click="handleSave">{{ de.save || '保存' }}</VButton>
          <VButton variant="secondary" @click="handleCopy" :disabled="isNew">{{ de.copy || '复制' }}</VButton>
          <VButton variant="danger" @click="handleDelete" :disabled="isNew || editForm.is_builtin">{{ de.delete || '删除' }}</VButton>
          <VButton variant="ghost" @click="backToList">{{ de.cancel || '取消' }}</VButton>
        </div>
      </div>
    </template>
  </div>
</template>
