import type { Meta, StoryObj } from '@storybook/vue3'
import VInput from './VInput.vue'
import VFormItem from './VFormItem.vue'

const meta: Meta<typeof VInput> = {
  title: 'Base/VInput',
  component: VInput,
  tags: ['autodocs'],
  argTypes: {
    type: { control: 'select', options: ['text', 'password', 'email', 'number'] },
    disabled: { control: 'boolean' },
    placeholder: { control: 'text' },
  },
  args: { type: 'text', disabled: false, placeholder: 'Enter text...' },
}

export default meta
type Story = StoryObj<typeof meta>

const renderWithLabel = (args: any) => ({
  components: { VInput, VFormItem },
  setup: () => ({ args }),
  template: '<VFormItem label="Field Label"><VInput v-bind="args" /></VFormItem>',
})

export const Default: Story = { args: {}, render: renderWithLabel }
export const WithPlaceholder: Story = { args: { placeholder: 'Type something...' }, render: renderWithLabel }
export const Password: Story = { args: { type: 'password', modelValue: 'secret123' }, render: renderWithLabel }
export const Disabled: Story = { args: { disabled: true, modelValue: 'Read only' }, render: renderWithLabel }

export const ChineseLongText: Story = {
  args: { placeholder: '请输入完整的需求描述、业务场景、功能要点以及验收标准' },
  render: renderWithLabel,
}

export const EnglishLongText: Story = {
  args: { placeholder: 'Enter the complete requirement description, business scenario, functional points, and acceptance criteria' },
  render: renderWithLabel,
}
