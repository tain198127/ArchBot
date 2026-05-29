import type { Meta, StoryObj } from '@storybook/vue3'
import VCheckbox from './VCheckbox.vue'

const meta: Meta<typeof VCheckbox> = { title: 'Base/VCheckbox', component: VCheckbox, tags: ['autodocs'], argTypes: { disabled: { control: 'boolean' } } }
export default meta
type Story = StoryObj<typeof meta>

const render = (args: any) => ({ components: { VCheckbox }, setup: () => ({ args }), template: '<VCheckbox v-bind="args" />' })

export const Default: Story = { args: { label: 'Enable feature', modelValue: false }, render }
export const Checked: Story = { args: { label: 'Checked state', modelValue: true }, render }
export const Disabled: Story = { args: { label: 'Disabled', disabled: true }, render }
export const ChineseLongText: Story = { args: { label: '同意《用户服务协议》和《隐私政策》并确认已阅读所有条款内容' }, render }
export const EnglishLongText: Story = { args: { label: 'I agree to the Terms of Service and Privacy Policy and confirm that I have read and understood all provisions' }, render }
