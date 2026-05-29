import type { Meta, StoryObj } from '@storybook/vue3'
import { ref } from 'vue'
import VDialog from './VDialog.vue'
import VButton from './VButton.vue'

const meta: Meta<typeof VDialog> = {
  title: 'Base/VDialog',
  component: VDialog,
  tags: ['autodocs'],
  argTypes: {
    visible: { control: 'boolean' },
    title: { control: 'text' },
    width: { control: 'text' },
  },
  args: { title: 'Dialog Title', width: '480px' },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: { visible: true },
  render: (args) => ({
    components: { VDialog, VButton },
    setup: () => {
      const visible = ref(true)
      return { args, visible }
    },
    template: `
      <VDialog :visible="visible" v-bind="args" @update:visible="visible = $event">
        <p class="text-text-secondary">Dialog content goes here.</p>
        <template #footer>
          <VButton variant="secondary" @click="visible = false">Cancel</VButton>
          <VButton @click="visible = false">Confirm</VButton>
        </template>
      </VDialog>
    `,
  }),
}

export const ChineseLongText: Story = {
  args: { visible: true, title: '确认删除数据标准实体及其所有关联字段和索引' },
  render: (args) => ({
    components: { VDialog, VButton },
    setup: () => {
      const visible = ref(true)
      return { args, visible }
    },
    template: `
      <VDialog :visible="visible" v-bind="args" @update:visible="visible = $event">
        <p class="text-text-secondary">此操作不可撤销，删除后将同时移除该实体下的所有字段定义、索引配置和关联关系数据。</p>
        <template #footer>
          <VButton variant="secondary" @click="visible = false">取消操作</VButton>
          <VButton variant="danger" @click="visible = false">确认删除</VButton>
        </template>
      </VDialog>
    `,
  }),
}
