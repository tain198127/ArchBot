import type { Meta, StoryObj } from '@storybook/vue3'
import VSelect from './VSelect.vue'
import VFormItem from './VFormItem.vue'

const meta: Meta<typeof VSelect> = {
  title: 'Base/VSelect',
  component: VSelect,
  tags: ['autodocs'],
  argTypes: { disabled: { control: 'boolean' } },
}

export default meta
type Story = StoryObj<typeof meta>

const options = [
  { value: 'gpt-4', label: 'GPT-4' },
  { value: 'claude-sonnet', label: 'Claude Sonnet 4.6' },
  { value: 'claude-opus', label: 'Claude Opus 4.7' },
]

const render = (args: any) => ({
  components: { VSelect, VFormItem },
  setup: () => ({ args }),
  template: '<VFormItem label="AI Model"><VSelect v-bind="args" /></VFormItem>',
})

export const Default: Story = { args: { options, modelValue: 'gpt-4' }, render }
export const Disabled: Story = { args: { options, modelValue: 'gpt-4', disabled: true }, render }
export const WithPlaceholder: Story = { args: { options, placeholder: 'Select a model...' }, render }
