<script setup lang="ts">
/**
 * BusinessFlowListPanel
 *
 * Displays a table of all business flows (built-in + custom).
 * Click a row → opens the flow editor as a new tab.
 * "New Flow" button creates a blank flow and opens it.
 */
import { onMounted } from 'vue'
import { useI18n } from '../../i18n'
import { useFlowStore } from '../../stores/flowStore'
import { useMenuAction } from '../../composables/useMenuAction'
import type { FlowSummary } from '../../types/businessFlow'

const { tt } = useI18n()
const store = useFlowStore()
const { emit } = useMenuAction()

onMounted(() => {
  store.init()
})

function handleRowClick(flow: FlowSummary) {
  // Open flow editor tab via menu action system
  emit({
    action: 'open.businessFlowEditor',
    payload: { flowId: flow.id, flowName: flow.name },
  })
}

async function handleNewFlow() {
  try {
    const flow = await store.createFlow('Untitled Flow')
    emit({
      action: 'open.businessFlowEditor',
      payload: { flowId: flow.id, flowName: flow.name },
    })
  } catch (e) {
    console.error('Failed to create flow:', e)
  }
}

function formatBindings(bindingsJson: string): string {
  try {
    const bindings = JSON.parse(bindingsJson)
    if (!Array.isArray(bindings) || bindings.length === 0) return '—'
    const labels = bindings.map((b: { label: string }) => b.label).filter(Boolean)
    if (labels.length <= 2) return labels.join(', ')
    return `${labels[0]}, ${labels[1]}… +${labels.length - 2}`
  } catch {
    return '—'
  }
}
</script>

<template>
  <div class="flex flex-col h-full bg-surface-0 dark:bg-surface-0">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-border-default">
      <h2 class="text-lg font-semibold text-text-primary">
        {{ tt('businessFlow.list.title') }}
      </h2>
      <button
        class="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-lg
               bg-primary-500 text-white hover:bg-primary-600
               dark:bg-primary-500 dark:hover:bg-primary-600
               transition-colors cursor-pointer"
        @click="handleNewFlow"
      >
        <span class="text-base leading-none">+</span>
        {{ tt('businessFlow.list.newFlow') }}
      </button>
    </div>

    <!-- Loading -->
    <div v-if="store.loading.value" class="flex items-center justify-center flex-1">
      <span class="text-text-secondary text-sm animate-pulse">Loading…</span>
    </div>

    <!-- Empty state -->
    <div
      v-else-if="store.flows.value.length === 0"
      class="flex flex-col items-center justify-center flex-1 select-none"
    >
      <div class="text-center">
        <div class="text-4xl mb-3 opacity-30">🔄</div>
        <p class="text-text-secondary text-sm">{{ tt('businessFlow.list.empty') }}</p>
      </div>
    </div>

    <!-- Flow table -->
    <div v-else class="flex-1 overflow-y-auto">
      <table class="w-full text-sm">
        <thead class="sticky top-0 bg-surface-50 dark:bg-surface-100 z-10">
          <tr class="text-left text-text-secondary">
            <th class="px-4 py-2 font-medium">{{ tt('businessFlow.list.colName') }}</th>
            <th class="px-4 py-2 font-medium">{{ tt('businessFlow.list.colType') }}</th>
            <th class="px-4 py-2 font-medium">{{ tt('businessFlow.list.colBindings') }}</th>
            <th class="px-4 py-2 font-medium">{{ tt('businessFlow.list.colStatus') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="flow in store.flows.value"
            :key="flow.id"
            class="border-t border-border-default cursor-pointer
                   hover:bg-surface-50 dark:hover:bg-surface-100 transition-colors"
            @click="handleRowClick(flow)"
          >
            <td class="px-4 py-2.5 text-text-primary font-medium">
              {{ flow.name }}
            </td>
            <td class="px-4 py-2.5">
              <span
                class="inline-flex items-center px-2 py-0.5 text-xs font-medium rounded-full"
                :class="flow.type === 'builtin'
                  ? 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400'
                  : 'bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-400'"
              >
                {{ flow.type === 'builtin'
                  ? tt('businessFlow.list.builtin')
                  : tt('businessFlow.list.custom') }}
              </span>
            </td>
            <td class="px-4 py-2.5 text-text-secondary text-xs">
              {{ formatBindings(flow.scenarioBindings) }}
            </td>
            <td class="px-4 py-2.5">
              <span
                v-if="flow.published"
                class="inline-flex items-center gap-1 text-xs text-green-600 dark:text-green-400"
              >
                <span class="w-1.5 h-1.5 rounded-full bg-green-500" />
                {{ tt('businessFlow.list.published') }}
              </span>
              <span
                v-else
                class="inline-flex items-center gap-1 text-xs text-text-muted"
              >
                <span class="w-1.5 h-1.5 rounded-full bg-gray-400" />
                {{ tt('businessFlow.list.draft') }}
              </span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Error -->
    <div v-if="store.error.value" class="px-4 py-2 text-xs text-red-500 bg-red-50 dark:bg-red-900/10">
      {{ store.error.value }}
    </div>
  </div>
</template>
