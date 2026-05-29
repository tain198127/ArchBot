import type { Meta, StoryObj } from '@storybook/vue3'
import { ref } from 'vue'
import VRadioGroup from './VRadioGroup.vue'
import VRadio from './VRadio.vue'

const meta: Meta<typeof VRadioGroup> = { title: 'Base/VRadioGroup', component: VRadioGroup, tags: ['autodocs'] }
export default meta
type Story = StoryObj<typeof meta>

export const WithRadios: Story = {
  render: () => ({
    components: { VRadioGroup, VRadio },
    setup: () => { const v = ref('opt1'); return { v } },
    template: `
      <VRadioGroup v-model="v">
        <VRadio value="opt1" label="Option 1" />
        <VRadio value="opt2" label="Option 2" />
        <VRadio value="opt3" label="Option 3 (disabled)" disabled />
      </VRadioGroup>
    `,
  }),
}
