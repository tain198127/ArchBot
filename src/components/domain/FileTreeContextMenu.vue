<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../i18n'
import { getContextMenuItems, type ContextMenuItem } from '../../orchestration/ContextMenuResolver'
import { evaluateExpression } from '../../orchestration/ExpressionEvaluator'
import { getPredicateRegistry } from '../../orchestration/PredicateRegistry'
import { createRuntimeState, type ContextObject } from '../../orchestration/RuntimeContext'
import { useProject } from '../../stores/project'
import { getActionRegistry } from '../../orchestration/ActionRegistry'

interface Props {
  visible: boolean
  x: number
  y: number
  context: ContextObject | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'action': [actionType: string]
  'close': []
}>()

const { tt } = useI18n()
const { currentProject } = useProject()

const items = computed<ContextMenuItem[]>(() => {
  if (!props.context) return []
  return getContextMenuItems(props.context)
})

const filteredItems = computed(() => {
  const state = createRuntimeState(currentProject.value)
  return items.value.filter(item => {
    // Filter out predicate items that evaluate to false
    if (item.predicate) {
      const pr = getPredicateRegistry()
      if (pr.has(item.predicate)) {
        return pr.evaluate(item.predicate, state, props.context ?? undefined)
      }
    }
    // Filter by visibleWhen expression
    if (item.visibleWhen) {
      return evaluateExpression(item.visibleWhen, state, props.context ?? undefined)
    }
    return true
  })
})

function handleItemClick(item: ContextMenuItem) {
  const actionId = item.action
  if (!actionId) return
  const registry = getActionRegistry()
  if (registry.has(actionId)) {
    import('../../composables/useToast').then(({ useToast }) => {
      const toast = useToast()
      registry.execute(actionId, (item.params ?? {}) as Record<string, unknown>, {
        invoke: async () => {},
        openFile: () => {},
        toast: {
          success: toast.success,
          error: toast.error,
          warning: toast.warning,
        },
        pushLog: () => {},
        confirm: async () => false,
      }).catch(e => toast.error(String(e)))
    })
  }
  emit('close')
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible && filteredItems.length > 0"
      class="fixed z-[9999] min-w-[140px] bg-white dark:bg-surface-0 border border-border-default rounded-lg shadow-lg py-1"
      :style="{ left: x + 'px', top: y + 'px' }"
    >
      <template v-for="(item, idx) in filteredItems" :key="item.id ?? idx">
        <div v-if="item.type === 'separator'" class="h-px mx-2 my-1 bg-border-default" />
        <button
          v-else
          class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200"
          @click="handleItemClick(item)"
        >
          {{ tt(item.label ?? '') }}
        </button>
      </template>
    </div>
  </Teleport>
</template>
