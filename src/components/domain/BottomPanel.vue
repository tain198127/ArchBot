<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from '../../i18n'

const { t } = useI18n()

const activeTab = ref('log')

const tabKeys = ['log', 'terminal', 'analysis'] as const
</script>

<template>
  <div class="flex flex-col h-full bg-surface-0 dark:bg-surface-0">
    <div class="flex items-center h-[32px] bg-surface-50 dark:bg-surface-100 border-b border-border-default shrink-0">
      <div
        v-for="key in tabKeys"
        :key="key"
        class="relative px-3.5 h-full flex items-center text-[12px] cursor-pointer border-r border-border-default transition-colors select-none"
        :class="activeTab === key
          ? 'bg-surface-0 dark:bg-surface-0 text-text-primary'
          : 'text-text-secondary hover:bg-surface-50 dark:hover:bg-surface-50 hover:text-text-primary'"
        @click="activeTab = key"
      >
        {{ (t.bottom as Record<string, string>)[key] }}
        <span
          v-if="activeTab === key"
          class="absolute bottom-0 left-3.5 right-3.5 h-[2px] bg-primary-500 rounded-full"
        />
      </div>
    </div>
    <div class="flex-1 overflow-auto bg-surface-0 dark:bg-surface-0">
      <div class="flex items-center justify-center h-full">
        <p class="text-[12px] text-text-muted font-mono">
          <template v-if="activeTab === 'log'">{{ t.bottom.logPlaceholder }}</template>
          <template v-else-if="activeTab === 'terminal'">{{ t.bottom.terminalPlaceholder }}</template>
          <template v-else>{{ t.bottom.analysisPlaceholder }}</template>
        </p>
      </div>
    </div>
  </div>
</template>
