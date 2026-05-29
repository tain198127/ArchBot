import type { Meta, StoryObj } from '@storybook/vue3'
import VFormItem from './VFormItem.vue'
import VInput from './VInput.vue'

const meta: Meta<typeof VFormItem> = { title: 'Base/VFormItem', component: VFormItem, tags: ['autodocs'], argTypes: { required: { control: 'boolean' } }, args: { label: 'Field Label' } }
export default meta
type Story = StoryObj<typeof meta>

const render = (args: any) => ({ components: { VFormItem, VInput }, setup: () => ({ args }), template: '<VFormItem v-bind="args"><VInput placeholder="Input..." /></VFormItem>' })

export const Default: Story = { args: { label: 'Username' }, render }
export const Required: Story = { args: { label: 'Email Address', required: true }, render }
export const ChineseLabel: Story = { args: { label: '需求描述（请详细填写业务场景和功能要点）', required: true }, render }
export const EnglishLongLabel: Story = { args: { label: 'Requirement Description (Please provide detailed business context and functional requirements)', required: true }, render }
