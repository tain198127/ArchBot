import type { Meta, StoryObj } from '@storybook/vue3'
import { ref } from 'vue'
import VRadio from './VRadio.vue'
import VRadioGroup from './VRadioGroup.vue'

const meta: Meta<typeof VRadio> = { title: 'Base/VRadio', component: VRadio, tags: ['autodocs'] }
export default meta
type Story = StoryObj<typeof meta>

export const RadioGroup: Story = {
  render: () => ({
    components: { VRadio, VRadioGroup },
    setup: () => {
      const theme = ref('light')
      return { theme }
    },
    template: `
      <VRadioGroup v-model="theme">
        <VRadio value="light" label="Light" :model-value="theme" @update:model-value="theme = $event" />
        <VRadio value="dark" label="Dark" :model-value="theme" @update:model-value="theme = $event" />
      </VRadioGroup>
    `,
  }),
}

export const ChineseLabels: Story = {
  render: () => ({
    components: { VRadio, VRadioGroup },
    setup: () => {
      const lang = ref('auto')
      return { lang }
    },
    template: `
      <VRadioGroup v-model="lang">
        <VRadio value="auto" label="自动检测（根据系统语言自动切换）" :model-value="lang" @update:model-value="lang = $event" />
        <VRadio value="zh-CN" label="简体中文" :model-value="lang" @update:model-value="lang = $event" />
        <VRadio value="en-US" label="English (United States)" :model-value="lang" @update:model-value="lang = $event" />
      </VRadioGroup>
    `,
  }),
}

export const Disabled: Story = {
  render: () => ({
    components: { VRadio, VRadioGroup },
    setup: () => ({ val: ref('a') }),
    template: `
      <VRadioGroup v-model="val">
        <VRadio value="a" label="Available" />
        <VRadio value="b" label="Disabled" disabled />
      </VRadioGroup>
    `,
  }),
}
