<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from '../../i18n'
import { logEntries, clearLog } from '../../stores/log'

const { t } = useI18n()

const activeTab = ref('log')

const tabKeys = ['log', 'terminal', 'analysis'] as const

const levelColors: Record<string, string> = {
  error: 'text-red-500',
  warn: 'text-amber-500',
  info: 'text-text-muted',
}
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
          v-if="logEntries.length > 0 && key === 'log'"
          class="ml-1.5 min-w-[18px] h-[16px] flex items-center justify-center rounded-full bg-red-500 text-[10px] text-white font-semibold leading-none"
        >{{ logEntries.length > 99 ? '99+' : logEntries.length }}</span>
        <span
          v-if="activeTab === key"
          class="absolute bottom-0 left-3.5 right-3.5 h-[2px] bg-primary-500 rounded-full"
        />
      </div>
      <div class="flex-1" />
      <button
        v-if="activeTab === 'log' && logEntries.length > 0"
        class="mr-2 text-[11px] text-text-muted hover:text-text-primary transition-colors"
        @click="clearLog"
      >{{ (t.bottom as Record<string, string>).clear || 'Clear' }}</button>
    </div>

    <!-- Log tab -->
    <div v-if="activeTab === 'log'" class="flex-1 overflow-auto bg-surface-0 dark:bg-surface-0 font-mono text-[11px] leading-relaxed">
      <div v-if="logEntries.length === 0" class="flex items-center justify-center h-full">
        <p class="text-[12px] text-text-muted">{{ t.bottom.logPlaceholder }}</p>
      </div>
      <div v-else class="flex flex-col">
        <div
          v-for="entry in logEntries"
          :key="entry.id"
          class="flex gap-2 px-3 py-[3px] border-b border-border-default/50 hover:bg-surface-50 dark:hover:bg-surface-50"
        >
          <span class="shrink-0 text-text-muted w-[70px]">{{ entry.timestamp }}</span>
          <span class="shrink-0 w-[36px]" :class="levelColors[entry.level]">{{ entry.level.toUpperCase() }}</span>
          <span class="shrink-0 text-text-muted w-[56px] truncate">{{ entry.source }}</span>
          <span class="text-text-primary break-all">{{ entry.message }}</span>
        </div>
      </div>
    </div>

    <!-- Terminal tab -->
    <div v-else-if="activeTab === 'terminal'" class="flex-1 overflow-auto bg-surface-0 dark:bg-surface-0">
      <div class="flex items-center justify-center h-full">
        <p class="text-[12px] text-text-muted font-mono">{{ t.bottom.terminalPlaceholder }}</p>
      </div>
    </div>

    <!-- Analysis tab -->
    <div v-else class="flex-1 overflow-auto bg-surface-0 dark:bg-surface-0">
      <div class="flex items-center justify-center h-full">
        <p class="text-[12px] text-text-muted font-mono">{{ t.bottom.analysisPlaceholder }}</p>
      </div>
    </div>
  </div>
</template>
