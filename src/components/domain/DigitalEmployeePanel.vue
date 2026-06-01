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
import { useProject } from '../../stores/project'
import { useScenario } from '../../composables/useScenario'

const { t, tt } = useI18n()
const toast = useToast()
const { currentProject } = useProject()
const { preset, effectiveDimension } = useScenario()
const de = computed(() => t.value.digitalEmployee as unknown as Record<string, string>)

const DB_TYPE = 'local'

// ── list state ──
const employees = ref<any[]>([])
const searchText = ref('')
const selectedIds = ref<Set<number>>(new Set())
const loading = ref(false)

// ── edit state ──
const editMode = ref<'list' | 'edit'>('list')
const isNew = ref(false)
const editForm = ref<any>({})
const editSkills = ref<string[]>([])
const skillPick = ref('')

function onPickSkill(val: string | number) {
  const v = String(val)
  if (v && !editSkills.value.includes(v)) editSkills.value.push(v)
  skillPick.value = ''
}
function toSelectOptions(arr: any[]) {
  return arr.map((x: any) => ({ value: x.code || x.id, label: x.name || x.label || String(x.code || x.id) }))
}
// ── lookup data ──
const skillOptions = ref<any[]>([])

const commStyleOptions = ['正式', '亲和', '幽默', '严谨', '睿智']
const decisionPrefOptions = ['逻辑+规则', '经验+直觉', '数据+分析', '协作+共识']
const focusAreaOptions = ['需求分析', '系统设计', '代码开发', '测试验证', '部署运维', '项目管理', '数据治理', '安全管理']
const deliverableOptions = ['PRD', 'SRS', '架构设计', '详细设计', '代码实现', '测试用例', '部署方案', '运维手册']

const scenarioRoles = computed<string[]>(() => {
  return effectiveDimension.value?.digitalEmployees || []
})

function isScenarioRole(code: string): boolean {
  return scenarioRoles.value.includes(code)
}

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
    await invoke('de_init', { dbType: DB_TYPE, projectPath: currentProject.value?.path || '' })
    employees.value = await invoke<any[]>('de_list', { dbType: DB_TYPE })
  } catch (e) {
    toast.error(String(e))
  } finally {
    loading.value = false
  }
}

async function loadLookups() {
  try {
    const skills = await invoke<any[]>('db_find_all', { table: 'skills', params: { filters: [], orderBy: [], limit: null, offset: null }, dbType: DB_TYPE })
    skillOptions.value = (skills as any).rows || skills
  } catch { /* ignore */ }
}

// ── list actions ──
function handleNew() {
  isNew.value = true; editMode.value = 'edit'
  editForm.value = { code: '', name: '', is_builtin: false, role: '', personality_tags: '[]', personality_desc: '', comm_style: '', decision_pref: '', focus_areas: '[]', deliverable_groups: '[]', default_capability: '', avatar: '' }
  editSkills.value = []
}

function handleEdit() {
  if (selectedIds.value.size !== 1) { toast.warning(de.value.selectOneHint || '请选择一个员工'); return }
  const id = [...selectedIds.value][0]
  const emp = employees.value.find((e: any) => e.id === id)
  if (!emp) return
  isNew.value = false; editMode.value = 'edit'
  editForm.value = { ...emp }
  editSkills.value = parseJsonArray(emp.skills)
}

function handleRowClick(row: any) {
  selectedIds.value = new Set([row.id])
  isNew.value = false; editMode.value = 'edit'
  editForm.value = { ...row }
  editSkills.value = parseJsonArray(row.skills)
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
// ── capability dropdown (Task 7) ──
interface SkillCommand {
  package: string
  skill_name: string
  command: string
  display_name_en: string
}

interface CapabilityOption {
  value: string
  label: string
  group: string
  command: string
}

const PACKAGE_DISPLAY_NAMES: Record<string, string> = {
  'superpowers':            'Super Power',
  'gstack':                 'Gstack',
  'everything-claude-code': 'Everything Claude Code',
  'super-claude':           'Super Claude',
  'claude-official-skills': 'OpenSpec',
}

const SKILL_NAME_MAP: Record<string, string> = {
  'superpowers/brainstorming':              'de.cap.brainstorming',
  'superpowers/writing-plans':              'de.cap.writingPlans',
  'superpowers/executing-plans':            'de.cap.executingPlans',
  'superpowers/test-driven-development':    'de.cap.testDrivenDevelopment',
  'superpowers/systematic-debugging':       'de.cap.systematicDebugging',
  'superpowers/subagent-driven-development':'de.cap.subagentDrivenDev',
  'superpowers/verification-before-completion':'de.cap.verificationCheck',
  'superpowers/requesting-code-review':     'de.cap.requestingCodeReview',
  'superpowers/dispatching-parallel-agents':'de.cap.dispatchingAgents',
  'superpowers/using-git-worktrees':       'de.cap.gitWorktrees',
  'superpowers/graphify':                  'de.cap.graphify',
  'gstack/browse':              'de.cap.browse',
  'gstack/qa':                  'de.cap.qa',
  'gstack/ship':                'de.cap.ship',
  'gstack/plan-eng-review':     'de.cap.planEngReview',
  'gstack/cso':                 'de.cap.cso',
  'gstack/document-generate':   'de.cap.documentGenerate',
  'gstack/office-hours':        'de.cap.officeHours',
  'gstack/review':              'de.cap.gstackReview',
  'gstack/land-and-deploy':     'de.cap.landAndDeploy',
  'gstack/design-consultation': 'de.cap.designConsultation',
  'gstack/investigate':         'de.cap.investigate',
  'gstack/retro':               'de.cap.retro',
  'everything-claude-code/code-review':      'de.cap.codeReview',
  'everything-claude-code/arch-review':      'de.cap.archReview',
  'everything-claude-code/security-review':  'de.cap.securityReview',
  'everything-claude-code/sc:design':        'de.cap.scDesign',
  'everything-claude-code/sc:implement':     'de.cap.scImplement',
  'everything-claude-code/sc:analyze':       'de.cap.scAnalyze',
  'everything-claude-code/e2e':              'de.cap.e2e',
  'everything-claude-code/deep-research':    'de.cap.deepResearch',
  'everything-claude-code/orchestrate':      'de.cap.orchestrate',
  'everything-claude-code/tdd':              'de.cap.tdd',
  'everything-claude-code/docs':             'de.cap.docs',
  'everything-claude-code/plan':             'de.cap.plan',
  'everything-claude-code/loop':             'de.cap.loop',
  'claude-official-skills/openspec-propose':       'de.cap.openspecPropose',
  'claude-official-skills/openspec-apply-change':  'de.cap.openspecApply',
  'claude-official-skills/openspec-explore':       'de.cap.openspecExplore',
  'claude-official-skills/openspec-new-change':    'de.cap.openspecNewChange',
  'claude-official-skills/openspec-archive-change':'de.cap.openspecArchive',
  'claude-official-skills/openspec-verify-change': 'de.cap.openspecVerify',
}

const capabilityOptions = ref<CapabilityOption[]>([])
const capabilityLoading = ref(false)

async function loadCapabilities() {
  capabilityLoading.value = true
  try {
    const commands = await invoke<SkillCommand[]>('agent_list_skill_commands', { runtime: 'claude_code' })
    capabilityOptions.value = buildGroupedOptions(commands, skillOptions.value)
  } catch {
    capabilityOptions.value = buildGroupedOptions([], skillOptions.value)
  } finally {
    capabilityLoading.value = false
  }
}

function buildGroupedOptions(commands: SkillCommand[], customSkills: any[]): CapabilityOption[] {
  const options: CapabilityOption[] = []
  const guerrillasGroup = tt('digitalEmployee.guerrillas') || 'Guerrillas'

  // Installed skill commands grouped by package
  const grouped = new Map<string, SkillCommand[]>()
  for (const cmd of commands) {
    if (!grouped.has(cmd.package)) grouped.set(cmd.package, [])
    grouped.get(cmd.package)!.push(cmd)
  }
  for (const [pkg, cmds] of grouped) {
    const pkgDisplayName = PACKAGE_DISPLAY_NAMES[pkg] || pkg
    for (const cmd of cmds) {
      const mapKey = `${cmd.package}/${cmd.skill_name}`
      const i18nKey = SKILL_NAME_MAP[mapKey]
      const displayName = i18nKey ? tt(i18nKey) : (cmd.display_name_en || cmd.skill_name)
      options.push({
        value: mapKey,
        label: `${displayName} (${cmd.command})`,
        group: pkgDisplayName,
        command: cmd.command,
      })
    }
  }

  // Custom skills from DB → 游击队 (Guerrillas)
  for (const cs of customSkills) {
    const code = cs.code || cs.id || ''
    const name = cs.name || cs.label || code
    options.push({
      value: `guerrillas/${code}`,
      label: `${name} (/${code})`,
      group: guerrillasGroup,
      command: `/${code}`,
    })
  }

  return options
}

onMounted(async () => { await loadEmployees(); await loadLookups(); await loadCapabilities() })
</script>

<template>
  <div class="flex flex-col h-full bg-surface-50 dark:bg-surface-50 p-4 overflow-y-auto">
    <!-- List View -->
    <template v-if="editMode === 'list'">
      <!-- Scenario info bar -->
      <div v-if="preset" class="flex items-center gap-2 mb-3 px-3 py-2 bg-primary-50 dark:bg-primary-950 border border-primary-200 dark:border-primary-800 rounded-lg text-xs">
        <span class="font-medium text-primary-600">{{ t.scenario.title }}: {{ preset ? tt(preset.nameKey) : '' }}</span>
        <span class="text-text-muted">|</span>
        <span class="text-text-secondary">{{ t.scenario.dimension.digitalEmployees }}:</span>
        <span v-for="role in scenarioRoles" :key="role" class="px-1.5 py-0.5 bg-primary-100 dark:bg-primary-900 text-primary-700 rounded font-mono">{{ role }}</span>
        <span v-if="scenarioRoles.length === 0" class="text-text-muted">—</span>
      </div>
      <div class="flex gap-2 mb-3 items-center">
        <div class="flex-1 max-w-[240px]">
          <VInput v-model="searchText" :placeholder="de.searchPlaceholder || '查找员工...'" />
        </div>
        <VButton size="sm" @click="handleNew">{{ de.new || '新增' }}</VButton>
        <VButton size="sm" variant="secondary" @click="handleEdit">{{ de.edit || '编辑' }}</VButton>
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
            <td class="px-3 py-2 border-b border-border-default text-primary-500 cursor-pointer" @click="handleRowClick(row)">
              {{ row.name }}
              <span v-if="isScenarioRole(row.code)" class="ml-1 text-[10px] text-success-500" title="Scenario recommended">&#9679;</span>
            </td>
            <td class="px-3 py-2 border-b border-border-default">
              {{ row.code }}
              <span v-if="isScenarioRole(row.code)" class="ml-1 px-1 py-0.5 text-[10px] bg-success-100 dark:bg-success-900 text-success-700 rounded">scenario</span>
            </td>
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
        <span class="text-base font-semibold text-text-primary">{{ isNew ? de.newEmployee : de.editEmployee }}</span>
      </div>

      <div class="flex-1 overflow-y-auto">
        <!-- Basic Info -->
        <fieldset class="border border-border-default rounded-lg px-4 py-3 mb-3">
          <legend class="text-sm font-semibold text-text-secondary px-1.5">{{ de.basicInfo || '基础信息' }}</legend>
          <div class="flex items-center gap-3 mb-2">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.name || '姓名' }}</label>
            <div class="flex-1 max-w-[360px]"><VInput v-model="editForm.name" /></div>
          </div>
          <div class="flex items-center gap-3 mb-2">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.code || '编码' }}</label>
            <div class="flex-1 max-w-[360px]"><VInput v-model="editForm.code" :disabled="!isNew && editForm.is_builtin" /></div>
          </div>
          <div class="flex items-center gap-3 mb-2">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.type || '类型' }}</label>
            <span class="text-sm text-text-primary">{{ editForm.is_builtin ? (de.builtin || '内置') : (de.custom || '自定义') }}</span>
          </div>
          <div class="flex items-center gap-3">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.avatar || '头像' }}</label>
            <div class="flex-1 max-w-[120px]"><VInput v-model="editForm.avatar" /></div>
          </div>
        </fieldset>

        <!-- Personality -->
        <fieldset class="border border-border-default rounded-lg px-4 py-3 mb-3">
          <legend class="text-sm font-semibold text-text-secondary px-1.5">{{ de.personality || '人格特征' }}</legend>
          <div class="flex items-center gap-3 mb-2">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.personalityTags || '性格标签' }}</label>
            <div class="flex-1 max-w-[360px]"><VInput v-model="editForm.personality_tags" /></div>
          </div>
          <div class="flex items-center gap-3 mb-2">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.personalityDesc || '性格描述' }}</label>
            <div class="flex-1 max-w-[360px]"><VTextarea v-model="editForm.personality_desc" /></div>
          </div>
          <div class="flex items-center gap-3 mb-2">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.commStyle || '沟通风格' }}</label>
            <VSelect v-model="editForm.comm_style" :options="commStyleOptions.map(s => ({ value: s, label: s }))" class="!w-[200px]" />
          </div>
          <div class="flex items-center gap-3">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.decisionPref || '决策偏好' }}</label>
            <VSelect v-model="editForm.decision_pref" :options="decisionPrefOptions.map(s => ({ value: s, label: s }))" class="!w-[200px]" />
          </div>
        </fieldset>

        <!-- Capabilities -->
        <fieldset class="border border-border-default rounded-lg px-4 py-3 mb-3">
          <legend class="text-sm font-semibold text-text-secondary px-1.5">{{ de.capabilities || '能力配置' }}</legend>
          <div class="flex items-center gap-3 mb-2">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.focusAreas || '专注领域' }}</label>
            <div class="flex gap-3 flex-wrap">
              <VCheckbox v-for="a in focusAreaOptions" :key="a" :model-value="parseJsonArray(editForm.focus_areas).includes(a)" :label="a"
                                @update:model-value="(v: boolean) => { const arr = parseJsonArray(editForm.focus_areas); if (v) arr.push(a); else arr.splice(arr.indexOf(a), 1); editForm.focus_areas = toJsonArray(arr) }" />
            </div>
          </div>
          <div class="flex items-center gap-3 mb-2">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.deliverableGroups || '交付物组' }}</label>
            <div class="flex gap-3 flex-wrap">
              <VCheckbox v-for="d in deliverableOptions" :key="d" :model-value="parseJsonArray(editForm.deliverable_groups).includes(d)" :label="d"
                                @update:model-value="(v: boolean) => { const arr = parseJsonArray(editForm.deliverable_groups); if (v) arr.push(d); else arr.splice(arr.indexOf(d), 1); editForm.deliverable_groups = toJsonArray(arr) }" />
            </div>
          </div>
          <div class="flex items-center gap-3">
            <label class="w-[100px] text-sm text-text-secondary shrink-0 text-right">{{ de.defaultCapability || '默认能力' }}</label>
            <VSelect v-model="editForm.default_capability" :options="capabilityOptions" option-group-label="group" :loading="capabilityLoading" class="!w-[200px]" />
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
              <VSelect
                v-if="editMode === 'edit'"
                v-model="skillPick"
                :options="toSelectOptions(skillOptions)"
                placeholder="+ 添加"
                class="w-[140px]"
                @update:model-value="onPickSkill"
              />
            </div>
          </div>
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
