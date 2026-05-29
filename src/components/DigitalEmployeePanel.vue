<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useI18n } from '../i18n'

const { t } = useI18n()
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
const editForm = ref<Record<string, any>>({})
const editSkills = ref<string[]>([])
const editAgents = ref<string[]>([])
const editMcps = ref<string[]>([])
const editHandoffs = ref<any[]>([])

// ── lookup data ──
const skillOptions = ref<any[]>([])
const agentOptions = ref<any[]>([])
const mcpOptions = ref<any[]>([])
const employeeOptions = ref<any[]>([])

const operationOptions = ['analyze', 'review', 'write', 'seal', 'reverse', 'import', 'export']
const focusAreaOptions = ['需求', '设计', '开发', '测试', '部署']
const deliverableOptions = ['bizContext', 'userStories', 'dataStandard', 'funcSpec', 'qualityAttr']
const commStyleOptions = ['formal', 'concise', 'humorous', 'academic']
const decisionPrefOptions = ['data_driven', 'aggressive', 'conservative']
const transferModeOptions = [
  { value: 'notify', label: '通知' },
  { value: 'open_editor', label: '自动打开编辑器' },
  { value: 'add_to_queue', label: '加入审批队列' }
]

// ── filter ──
const filteredEmployees = computed(() => {
  const q = searchText.value.toLowerCase()
  if (!q) return employees.value
  return employees.value.filter((e: any) =>
    e.name?.toLowerCase().includes(q) ||
    e.code?.toLowerCase().includes(q) ||
    e.personality_tags?.toLowerCase().includes(q)
  )
})

// ── load ──
async function loadEmployees() {
  loading.value = true
  try {
    await invoke('de_init', { dbType: DB_TYPE })
    employees.value = await invoke<any[]>('de_list', { dbType: DB_TYPE })
  } catch (e) {
    console.error('加载员工列表失败:', e)
    ElMessage.error(String(e))
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
  } catch (e) {
    console.error('加载查找数据失败:', e)
  }
}

// ── list actions ──
function handleNew() {
  isNew.value = true
  editMode.value = 'edit'
  editForm.value = {
    code: '', name: '', is_builtin: false, avatar: '🤖',
    personality_tags: '[]', personality_desc: '',
    comm_style: 'formal', decision_pref: 'data_driven',
    focus_areas: '[]', deliverable_groups: '[]', default_op: 'write',
    sort_order: employees.value.length + 1,
    created_at: new Date().toISOString(), updated_at: new Date().toISOString()
  }
  editSkills.value = []
  editAgents.value = []
  editMcps.value = []
  editHandoffs.value = []
}

function handleEdit() {
  if (selectedIds.value.size !== 1) {
    ElMessage.warning(de.value.selectOneHint || '请选择一个员工')
    return
  }
  const id = [...selectedIds.value][0]
  const emp = employees.value.find((e: any) => e.id === id)
  if (!emp) return
  isNew.value = false
  editMode.value = 'edit'
  editForm.value = { ...emp }
  // TODO: load relations
  editSkills.value = []
  editAgents.value = []
  editMcps.value = []
  editHandoffs.value = []
}

function handleView() {
  if (selectedIds.value.size !== 1) {
    ElMessage.warning(de.value.selectOneHint || '请选择一个员工')
    return
  }
  const id = [...selectedIds.value][0]
  const emp = employees.value.find((e: any) => e.id === id)
  if (!emp) return
  isNew.value = false
  editMode.value = 'view'
  editForm.value = { ...emp }
}

function handleRowClick(row: any) {
  selectedIds.value = new Set([row.id])
  isNew.value = false
  editMode.value = 'edit'
  editForm.value = { ...row }
  editSkills.value = []
  editAgents.value = []
  editMcps.value = []
  editHandoffs.value = []
}

// ── edit actions ──
async function handleSave() {
  if (!editForm.value.code || !editForm.value.name) {
    ElMessage.warning('编码和姓名不能为空')
    return
  }
  editForm.value.updated_at = new Date().toISOString()
  try {
    await invoke('de_save', { employee: editForm.value, dbType: DB_TYPE })
    ElMessage.success(isNew.value ? '创建成功' : '保存成功')
    backToList()
  } catch (e) {
    ElMessage.error(String(e))
  }
}

function handleCopy() {
  isNew.value = true
  editForm.value = {
    ...editForm.value,
    id: undefined, code: '', name: editForm.value.name + ' (副本)',
    is_builtin: false
  }
  editSkills.value = [...editSkills.value]
  editAgents.value = [...editAgents.value]
  editMcps.value = [...editMcps.value]
  editHandoffs.value = editHandoffs.value.map((h: any) => ({ ...h, id: undefined }))
}

function handleDelete() {
  if (editForm.value.is_builtin) {
    ElMessage.warning('内置员工不可删除')
    return
  }
  ElMessageBox.confirm(`确定删除员工「${editForm.value.name}」？`, '确认删除', {
    confirmButtonText: '删除', cancelButtonText: '取消', type: 'warning'
  }).then(async () => {
    try {
      await invoke('de_delete', { id: editForm.value.id, dbType: DB_TYPE })
      ElMessage.success('已删除')
      backToList()
    } catch (e) {
      ElMessage.error(String(e))
    }
  }).catch(() => {})
}

function backToList() {
  editMode.value = 'list'
  selectedIds.value = new Set()
  loadEmployees()
}

// ── handoff helpers ──
function addHandoff() {
  editHandoffs.value.push({
    employee_code: editForm.value.code,
    trigger_op: 'write',
    target_employee_code: '',
    transfer_data: '[]',
    transfer_mode: 'notify',
    context_ref: ''
  })
}

function removeHandoff(index: number) {
  editHandoffs.value.splice(index, 1)
}

// ── JSON helpers ──
function parseJsonArray(val: string): string[] {
  try { return JSON.parse(val) } catch { return [] }
}
function toJsonArray(arr: string[]): string {
  return JSON.stringify(arr)
}

// ── lifecycle ──
onMounted(async () => {
  await loadEmployees()
  await loadLookups()
})

</script>

<template>
  <div class="de-panel">
    <!-- ── List View ── -->
    <template v-if="editMode === 'list'">
      <div class="de-toolbar">
        <input
          v-model="searchText"
          class="de-search"
          :placeholder="de.searchPlaceholder || '查找员工...'"
          @keyup.enter="() => {}"
        />
        <button class="de-btn de-btn-primary" @click="handleNew">{{ de.new || '新增' }}</button>
        <button class="de-btn" @click="handleEdit">{{ de.edit || '编辑' }}</button>
        <button class="de-btn" @click="handleView">{{ de.view || '查看' }}</button>
      </div>
      <table class="de-table" v-loading="loading">
        <thead>
          <tr>
            <th style="width:50px">#</th>
            <th style="width:140px">{{ de.name || '姓名' }}</th>
            <th style="width:140px">{{ de.role || '角色' }}</th>
            <th style="width:160px">{{ de.personality || '性格' }}</th>
            <th style="width:90px">{{ de.type || '类型' }}</th>
            <th>{{ de.focusAreas || '专注领域' }}</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(row, idx) in filteredEmployees"
            :key="row.id"
            :class="{ selected: selectedIds.has(row.id) }"
            @click="selectedIds = new Set([row.id])"
            @dblclick="handleRowClick(row)"
          >
            <td>{{ idx + 1 }}</td>
            <td class="de-name-link" @click="handleRowClick(row)">{{ row.name }}</td>
            <td>{{ row.code }}</td>
            <td>{{ parseJsonArray(row.personality_tags).join('·') }}</td>
            <td>{{ row.is_builtin ? (de.builtin || '内置') : (de.custom || '自定义') }}</td>
            <td>{{ parseJsonArray(row.focus_areas).join('+') }}</td>
          </tr>
          <tr v-if="filteredEmployees.length === 0">
            <td colspan="6" class="de-empty">{{ de.noData || '暂无数据' }}</td>
          </tr>
        </tbody>
      </table>
    </template>

    <!-- ── Edit / View ── -->
    <template v-else>
      <div class="de-edit-header">
        <button class="de-btn" @click="backToList">← {{ de.back || '返回列表' }}</button>
        <span class="de-edit-title">{{ isNew ? (de.newEmployee || '新增数字员工') : (editMode === 'view' ? (de.viewEmployee || '查看数字员工') : (de.editEmployee || '编辑数字员工')) }}</span>
      </div>

      <div class="de-edit-body">
        <!-- 基础信息 -->
        <fieldset class="de-section">
          <legend>{{ de.basicInfo || '基础信息' }}</legend>
          <div class="de-row">
            <label>{{ de.name || '姓名' }}</label>
            <input v-model="editForm.name" class="de-input" :disabled="editMode === 'view'" />
          </div>
          <div class="de-row">
            <label>{{ de.code || '编码' }}</label>
            <input v-model="editForm.code" class="de-input" :disabled="editMode === 'view' || (!isNew && editForm.is_builtin)" />
          </div>
          <div class="de-row">
            <label>{{ de.type || '类型' }}</label>
            <span class="de-static">{{ editForm.is_builtin ? (de.builtin || '内置') : (de.custom || '自定义') }}</span>
          </div>
          <div class="de-row">
            <label>{{ de.avatar || '头像' }}</label>
            <input v-model="editForm.avatar" class="de-input de-input-short" :disabled="editMode === 'view'" />
          </div>
        </fieldset>

        <!-- 人格特征 -->
        <fieldset class="de-section">
          <legend>{{ de.personality || '人格特征' }}</legend>
          <div class="de-row">
            <label>{{ de.personalityTags || '性格标签' }}</label>
            <input v-model="editForm.personality_tags" class="de-input" :disabled="editMode === 'view'" />
          </div>
          <div class="de-row">
            <label>{{ de.personalityDesc || '性格描述' }}</label>
            <textarea v-model="editForm.personality_desc" class="de-textarea" :disabled="editMode === 'view'" />
          </div>
          <div class="de-row">
            <label>{{ de.commStyle || '沟通风格' }}</label>
            <select v-model="editForm.comm_style" class="de-select" :disabled="editMode === 'view'">
              <option v-for="s in commStyleOptions" :key="s" :value="s">{{ s }}</option>
            </select>
          </div>
          <div class="de-row">
            <label>{{ de.decisionPref || '决策偏好' }}</label>
            <select v-model="editForm.decision_pref" class="de-select" :disabled="editMode === 'view'">
              <option v-for="d in decisionPrefOptions" :key="d" :value="d">{{ d }}</option>
            </select>
          </div>
        </fieldset>

        <!-- 能力配置 -->
        <fieldset class="de-section">
          <legend>{{ de.capabilities || '能力配置' }}</legend>
          <div class="de-row">
            <label>{{ de.focusAreas || '专注领域' }}</label>
            <div class="de-check-group">
              <label v-for="a in focusAreaOptions" :key="a" class="de-check">
                <input type="checkbox" :value="a" :disabled="editMode === 'view'"
                  :checked="parseJsonArray(editForm.focus_areas).includes(a)"
                  @change="(e: any) => { const arr = parseJsonArray(editForm.focus_areas); if (e.target.checked) arr.push(a); else arr.splice(arr.indexOf(a), 1); editForm.focus_areas = toJsonArray(arr) }"
                /> {{ a }}
              </label>
            </div>
          </div>
          <div class="de-row">
            <label>{{ de.deliverableGroups || '交付物组' }}</label>
            <div class="de-check-group">
              <label v-for="d in deliverableOptions" :key="d" class="de-check">
                <input type="checkbox" :value="d" :disabled="editMode === 'view'"
                  :checked="parseJsonArray(editForm.deliverable_groups).includes(d)"
                  @change="(e: any) => { const arr = parseJsonArray(editForm.deliverable_groups); if (e.target.checked) arr.push(d); else arr.splice(arr.indexOf(d), 1); editForm.deliverable_groups = toJsonArray(arr) }"
                /> {{ d }}
              </label>
            </div>
          </div>
          <div class="de-row">
            <label>{{ de.defaultOp || '默认操作' }}</label>
            <select v-model="editForm.default_op" class="de-select" :disabled="editMode === 'view'">
              <option v-for="o in operationOptions" :key="o" :value="o">{{ o }}</option>
            </select>
          </div>
        </fieldset>

        <!-- 技能栈 -->
        <fieldset class="de-section">
          <legend>{{ de.skillStack || '技能栈' }}</legend>
          <div class="de-row">
            <label>Skill</label>
            <div class="de-tag-list">
              <span v-for="s in editSkills" :key="s" class="de-tag">{{ s }} <button v-if="editMode === 'edit'" class="de-tag-remove" @click="editSkills = editSkills.filter(x => x !== s)">✕</button></span>
              <select v-if="editMode === 'edit'" class="de-select-inline" @change="(e: any) => { if (e.target.value && !editSkills.includes(e.target.value)) editSkills.push(e.target.value); e.target.value = '' }">
                <option value="">+ 添加</option>
                <option v-for="sk in skillOptions" :key="sk.code" :value="sk.code">{{ sk.name }}</option>
              </select>
            </div>
          </div>
          <div class="de-row">
            <label>Agent</label>
            <div class="de-tag-list">
              <span v-for="a in editAgents" :key="a" class="de-tag">{{ a }} <button v-if="editMode === 'edit'" class="de-tag-remove" @click="editAgents = editAgents.filter(x => x !== a)">✕</button></span>
              <select v-if="editMode === 'edit'" class="de-select-inline" @change="(e: any) => { if (e.target.value && !editAgents.includes(e.target.value)) editAgents.push(e.target.value); e.target.value = '' }">
                <option value="">+ 添加</option>
                <option v-for="ag in agentOptions" :key="ag.code" :value="ag.code">{{ ag.name }}</option>
              </select>
            </div>
          </div>
          <div class="de-row">
            <label>MCP</label>
            <div class="de-tag-list">
              <span v-for="m in editMcps" :key="m" class="de-tag">{{ m }} <button v-if="editMode === 'edit'" class="de-tag-remove" @click="editMcps = editMcps.filter(x => x !== m)">✕</button></span>
              <select v-if="editMode === 'edit'" class="de-select-inline" @change="(e: any) => { if (e.target.value && !editMcps.includes(e.target.value)) editMcps.push(e.target.value); e.target.value = '' }">
                <option value="">+ 添加</option>
                <option v-for="mc in mcpOptions" :key="mc.code" :value="mc.code">{{ mc.name }}</option>
              </select>
            </div>
          </div>
        </fieldset>

        <!-- 交互规则 -->
        <fieldset class="de-section">
          <legend>{{ de.handoffRules || '交互规则' }}</legend>
          <div v-for="(h, idx) in editHandoffs" :key="idx" class="de-handoff-row">
            <select v-model="h.trigger_op" class="de-select-inline" :disabled="editMode === 'view'">
              <option v-for="o in operationOptions" :key="o" :value="o">{{ o }}</option>
            </select>
            <span class="de-arrow">→</span>
            <select v-model="h.target_employee_code" class="de-select-inline" :disabled="editMode === 'view'">
              <option value="">选择角色</option>
              <option v-for="emp in employeeOptions" :key="emp.code" :value="emp.code">{{ emp.name }}</option>
            </select>
            <select v-model="h.transfer_mode" class="de-select-inline" :disabled="editMode === 'view'">
              <option v-for="tm in transferModeOptions" :key="tm.value" :value="tm.value">{{ tm.label }}</option>
            </select>
            <button v-if="editMode === 'edit'" class="de-btn de-btn-sm" @click="removeHandoff(idx)">✕</button>
          </div>
          <button v-if="editMode === 'edit'" class="de-btn" @click="addHandoff">+ {{ de.addHandoff || '添加交互规则' }}</button>
        </fieldset>
      </div>

      <!-- 操作栏 -->
      <div class="de-edit-footer">
        <button v-if="editMode === 'edit'" class="de-btn de-btn-primary" @click="handleSave">{{ isNew ? (de.create || '创建') : (de.save || '保存') }}</button>
        <button v-if="editMode === 'edit' && !isNew" class="de-btn" @click="handleCopy">{{ de.copy || '复制' }}</button>
        <button class="de-btn" @click="backToList">{{ de.cancel || '取消' }}</button>
        <button v-if="editMode === 'edit' && !isNew && !editForm.is_builtin" class="de-btn de-btn-danger" @click="handleDelete">{{ de.delete || '删除' }}</button>
      </div>
    </template>
  </div>
</template>

<style scoped>
.de-panel { display: flex; flex-direction: column; height: 100%; background: var(--bg-panel); padding: 16px; overflow-y: auto; }
.de-toolbar { display: flex; gap: 8px; margin-bottom: 12px; align-items: center; }
.de-search { flex: 1; max-width: 240px; height: 32px; padding: 0 10px; border: 1px solid var(--border-color); border-radius: 4px; font-size: 13px; }
.de-btn { padding: 6px 14px; border: 1px solid var(--border-color); border-radius: 4px; background: var(--bg-primary); color: var(--text-primary); font-size: 13px; cursor: pointer; }
.de-btn:hover { background: var(--bg-hover); }
.de-btn-primary { background: #409eff; color: #fff; border-color: #409eff; }
.de-btn-primary:hover { background: #3a8ee6; }
.de-btn-danger { color: #e74c3c; border-color: #e74c3c; }
.de-btn-danger:hover { background: #fef0f0; }
.de-btn-sm { padding: 2px 6px; font-size: 11px; }
.de-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.de-table th { text-align: left; padding: 8px 12px; border-bottom: 2px solid var(--border-color); color: var(--text-secondary); font-weight: 600; }
.de-table td { padding: 8px 12px; border-bottom: 1px solid var(--border-color); cursor: pointer; }
.de-table tr:hover { background: var(--bg-hover); }
.de-table tr.selected { background: #dbeafe; }
.de-name-link { color: #409eff; cursor: pointer; }
.de-name-link:hover { text-decoration: underline; }
.de-empty { text-align: center; color: var(--text-muted); padding: 24px; }
.de-edit-header { display: flex; gap: 12px; align-items: center; margin-bottom: 16px; }
.de-edit-title { font-size: 16px; font-weight: 600; }
.de-edit-body { flex: 1; overflow-y: auto; }
.de-section { border: 1px solid var(--border-color); border-radius: 6px; padding: 12px 16px; margin-bottom: 12px; }
.de-section legend { font-size: 13px; font-weight: 600; color: var(--text-secondary); padding: 0 6px; }
.de-row { display: flex; align-items: center; gap: 12px; margin-bottom: 8px; }
.de-row label { width: 100px; font-size: 13px; color: var(--text-secondary); flex-shrink: 0; text-align: right; }
.de-input { flex: 1; max-width: 360px; height: 32px; padding: 0 10px; border: 1px solid var(--border-color); border-radius: 4px; font-size: 13px; }
.de-input-short { max-width: 120px; }
.de-textarea { flex: 1; max-width: 360px; height: 72px; padding: 6px 10px; border: 1px solid var(--border-color); border-radius: 4px; font-size: 13px; resize: vertical; }
.de-select { flex: 1; max-width: 200px; height: 32px; padding: 0 8px; border: 1px solid var(--border-color); border-radius: 4px; font-size: 13px; }
.de-select-inline { height: 28px; padding: 0 6px; border: 1px solid var(--border-color); border-radius: 4px; font-size: 12px; }
.de-check-group { display: flex; gap: 12px; flex-wrap: wrap; }
.de-check { display: flex; align-items: center; gap: 4px; font-size: 13px; cursor: pointer; }
.de-static { font-size: 13px; color: var(--text-primary); }
.de-tag-list { display: flex; gap: 6px; flex-wrap: wrap; align-items: center; }
.de-tag { display: inline-flex; align-items: center; gap: 4px; padding: 2px 8px; background: #ecf5ff; border: 1px solid #d9ecff; border-radius: 3px; font-size: 12px; color: #409eff; }
.de-tag-remove { border: none; background: none; color: #909399; cursor: pointer; font-size: 11px; padding: 0 2px; }
.de-tag-remove:hover { color: #e74c3c; }
.de-handoff-row { display: flex; gap: 8px; align-items: center; margin-bottom: 6px; }
.de-arrow { color: var(--text-muted); font-size: 14px; }
.de-edit-footer { display: flex; gap: 8px; padding: 12px 0; border-top: 1px solid var(--border-color); }
</style>
