import type { Meta, StoryObj } from '@storybook/vue3'
import VButton from './VButton.vue'

const meta: Meta<typeof VButton> = {
  title: 'Base/VButton',
  component: VButton,
  tags: ['autodocs'],
  argTypes: {
    variant: { control: 'select', options: ['primary', 'secondary', 'danger', 'ghost'] },
    size: { control: 'select', options: ['sm', 'md', 'lg'] },
    loading: { control: 'boolean' },
    disabled: { control: 'boolean' },
  },
  args: { variant: 'primary', size: 'md', loading: false, disabled: false },
}

export default meta
type Story = StoryObj<typeof meta>

export const Primary: Story = { args: { variant: 'primary' }, render: (args) => ({ components: { VButton }, setup: () => ({ args }), template: '<VButton v-bind="args">Submit</VButton>' }) }
export const Secondary: Story = { args: { variant: 'secondary' }, render: (args) => ({ components: { VButton }, setup: () => ({ args }), template: '<VButton v-bind="args">Cancel</VButton>' }) }
export const Danger: Story = { args: { variant: 'danger' }, render: (args) => ({ components: { VButton }, setup: () => ({ args }), template: '<VButton v-bind="args">Delete</VButton>' }) }
export const Ghost: Story = { args: { variant: 'ghost' }, render: (args) => ({ components: { VButton }, setup: () => ({ args }), template: '<VButton v-bind="args">Back</VButton>' }) }
export const Loading: Story = { args: { variant: 'primary', loading: true }, render: (args) => ({ components: { VButton }, setup: () => ({ args }), template: '<VButton v-bind="args">Saving...</VButton>' }) }
export const Disabled: Story = { args: { disabled: true }, render: (args) => ({ components: { VButton }, setup: () => ({ args }), template: '<VButton v-bind="args">Disabled</VButton>' }) }

export const ChineseLongText: Story = {
  args: { variant: 'primary', size: 'lg' },
  render: (args) => ({ components: { VButton }, setup: () => ({ args }), template: '<VButton v-bind="args">确认提交并生成需求规格文档</VButton>' }),
}

export const EnglishLongText: Story = {
  args: { variant: 'secondary', size: 'lg' },
  render: (args) => ({ components: { VButton }, setup: () => ({ args }), template: '<VButton v-bind="args">Generate Requirement Specification Document</VButton>' }),
}
