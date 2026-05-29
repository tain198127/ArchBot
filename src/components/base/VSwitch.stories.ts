import type { Meta, StoryObj } from '@storybook/vue3'
import VSwitch from './VSwitch.vue'

const meta: Meta<typeof VSwitch> = { title: 'Base/VSwitch', component: VSwitch, tags: ['autodocs'], argTypes: { disabled: { control: 'boolean' } } }
export default meta
type Story = StoryObj<typeof meta>

const render = (args: any) => ({ components: { VSwitch }, setup: () => ({ args }), template: '<label class="flex items-center gap-2 text-sm"><VSwitch v-bind="args" /><span>Enable proxy</span></label>' })

export const Off: Story = { args: { modelValue: false }, render }
export const On: Story = { args: { modelValue: true }, render }
export const Disabled: Story = { args: { disabled: true, modelValue: true }, render }
