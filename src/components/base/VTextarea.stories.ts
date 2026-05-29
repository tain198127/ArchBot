import type { Meta, StoryObj } from '@storybook/vue3'
import VTextarea from './VTextarea.vue'
import VFormItem from './VFormItem.vue'

const meta: Meta<typeof VTextarea> = { title: 'Base/VTextarea', component: VTextarea, tags: ['autodocs'], argTypes: { rows: { control: 'number' }, disabled: { control: 'boolean' } }, args: { rows: 3, placeholder: 'Enter description...' } }
export default meta
type Story = StoryObj<typeof meta>

const render = (args: any) => ({ components: { VTextarea, VFormItem }, setup: () => ({ args }), template: '<VFormItem label="Description"><VTextarea v-bind="args" /></VFormItem>' })

export const Default: Story = { args: {}, render }
export const Disabled: Story = { args: { disabled: true, modelValue: 'Read only content' }, render }
export const ChineseLongText: Story = { args: { placeholder: '请详细描述该功能模块的业务背景、核心需求、验收标准以及相关依赖项' }, render }
export const EnglishLongText: Story = { args: { placeholder: 'Please describe the business context, core requirements, acceptance criteria, and related dependencies for this feature module in detail' }, render }
