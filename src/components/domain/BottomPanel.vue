<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from '../../i18n'

const { t } = useI18n()

const activeTab = ref('log')

const tabKeys = ['log', 'terminal', 'analysis'] as const
</script>

<template>
  <div class="flex flex-col h-full bg-surface-0 dark:bg-surface-0">
    <div class="flex items-center h-[30px] bg-surface-100 dark:bg-surface-100 border-b border-border-default shrink-0">
      <div
        v-for="key in tabKeys"
        :key="key"
        class="px-3.5 h-full flex items-center text-xs cursor-pointer border-r border-border-default transition-colors select-none"
        :class="activeTab === key
          ? 'bg-surface-0 dark:bg-surface-0 text-text-primary border-b-2 border-primary-500'
          : 'text-text-secondary hover:bg-surface-100 dark:hover:bg-surface-100'"
        @click="activeTab = key"
      >
        {{ (t.bottom as Record<string, string>)[key] }}
      </div>
    </div>
    <div class="flex-1 overflow-auto">
      <div class="flex items-center justify-center h-full p-4">
        <p class="text-sm text-text-muted">
          <template v-if="activeTab === 'log'">{{ t.bottom.logPlaceholder }}</template>
          <template v-else-if="activeTab === 'terminal'">{{ t.bottom.terminalPlaceholder }}</template>
          <template v-else>{{ t.bottom.analysisPlaceholder }}</template>
        </p>
      </div>
    </div>
  </div>
</template>
