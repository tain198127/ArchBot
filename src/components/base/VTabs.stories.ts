import type { Meta, StoryObj } from '@storybook/vue3'
import { ref } from 'vue'
import VTabs from './VTabs.vue'

const meta: Meta<typeof VTabs> = { title: 'Base/VTabs', component: VTabs, tags: ['autodocs'] }
export default meta
type Story = StoryObj<typeof meta>

const tabs = [
  { value: 'fields', label: 'Fields' },
  { value: 'indexes', label: 'Indexes' },
  { value: 'relations', label: 'Relations' },
  { value: 'ddl', label: 'DDL Preview' },
]

export const Default: Story = {
  args: { tabs, modelValue: 'fields' },
  render: (args) => ({
    components: { VTabs },
    setup: () => {
      const active = ref('fields')
      return { args, active }
    },
    template: `
      <VTabs v-model="active" :tabs="args.tabs">
        <div v-if="active === 'fields'" class="text-text-secondary">Fields content</div>
        <div v-else-if="active === 'indexes'" class="text-text-secondary">Indexes content</div>
        <div v-else-if="active === 'relations'" class="text-text-secondary">Relations content</div>
        <div v-else class="text-text-secondary">DDL content</div>
      </VTabs>
    `,
  }),
}

const chTabs = [
  { value: 'basic', label: '基础信息' },
  { value: 'fields', label: '字段定义（含类型、长度、约束）' },
  { value: 'indexes', label: '索引与性能优化' },
]
export const ChineseLongLabels: Story = {
  args: { tabs: chTabs, modelValue: 'basic' },
  render: (args) => ({
    components: { VTabs },
    setup: () => { const active = ref('basic'); return { args, active } },
    template: '<VTabs v-model="active" :tabs="args.tabs"><p class="text-text-secondary p-2">内容区域</p></VTabs>',
  }),
}
