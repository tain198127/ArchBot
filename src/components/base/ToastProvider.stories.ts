import type { Meta, StoryObj } from '@storybook/vue3'
import ToastProvider from './ToastProvider.vue'
import VButton from './VButton.vue'
import { useToast } from '../../composables/useToast'

const meta: Meta<typeof ToastProvider> = { title: 'Base/ToastProvider', component: ToastProvider, tags: ['autodocs'] }
export default meta
type Story = StoryObj<typeof meta>

export const Demo: Story = {
  render: () => ({
    components: { ToastProvider, VButton },
    setup: () => {
      const toast = useToast()
      return { toast }
    },
    template: `
      <div class="p-8 flex gap-2">
        <ToastProvider />
        <VButton @click="toast.success('Operation completed')">Success</VButton>
        <VButton variant="secondary" @click="toast.info('New update available')">Info</VButton>
        <VButton variant="danger" @click="toast.error('Something went wrong')">Error</VButton>
        <VButton variant="ghost" @click="toast.warning('Please check input')">Warning</VButton>
        <VButton variant="secondary" @click="toast.confirm('Delete', 'Are you sure?').then(ok => ok && toast.success('Deleted'))">Confirm</VButton>
      </div>
    `,
  }),
}
