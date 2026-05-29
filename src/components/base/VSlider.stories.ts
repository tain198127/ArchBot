import type { Meta, StoryObj } from '@storybook/vue3'
import VSlider from './VSlider.vue'

const meta: Meta<typeof VSlider> = { title: 'Base/VSlider', component: VSlider, tags: ['autodocs'], argTypes: { disabled: { control: 'boolean' } }, args: { min: 12, max: 24, step: 1, modelValue: 14 } }
export default meta
type Story = StoryObj<typeof meta>

const render = (args: any) => ({ components: { VSlider }, setup: () => ({ args }), template: '<div class="w-[400px] p-4"><VSlider v-bind="args" /></div>' })

export const Default: Story = { args: { modelValue: 14 }, render }
export const MaxValue: Story = { args: { modelValue: 24 }, render }
export const Disabled: Story = { args: { disabled: true, modelValue: 16 }, render }
export const ChineseLabel: Story = { args: { modelValue: 18 }, render: (args: any) => ({ components: { VSlider }, setup: () => ({ args }), template: '<div class="w-[400px] p-4"><label class="text-sm text-text-primary mb-2 block">字体大小（12-24像素）</label><VSlider v-bind="args" /></div>' }) }
