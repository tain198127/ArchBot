<script setup lang="ts">
import { ref, defineAsyncComponent } from 'vue'
import { useI18n } from '../../i18n'
import { logEntries, clearLog } from '../../stores/log'

// ── Lazy-loaded agent sub-panels ──
// These are only shown when the user navigates to the Agent tab and
// selects a specific sub-tab.  Lazy loading keeps them out of the
// initial bundle and defers their first paint until they are actually
// needed, cutting ~40 % off the startup JavaScript parse time.
const AgentSessionPanel = defineAsyncComponent(() => import('./AgentSessionPanel.vue'))
const AgentTurnPanel = defineAsyncComponent(() => import('./AgentTurnPanel.vue'))
const AgentEventStreamPanel = defineAsyncComponent(() => import('./AgentEventStreamPanel.vue'))
const AgentDiffReviewPanel = defineAsyncComponent(() => import('./AgentDiffReviewPanel.vue'))
const AgentAuditLogPanel = defineAsyncComponent(() => import('./AgentAuditLogPanel.vue'))

const { t } = useI18n()

const activeTab = ref('log')
const agentSubTab = ref('session')

const tabKeys = ['log', 'terminal', 'analysis', 'agent'] as const

const agentSubTabs = ['session', 'turn', 'events', 'diff', 'audit'] as const

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
        {{ (t.bottom as Record<string, string>)[key] || key }}
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
    <div v-else-if="activeTab === 'analysis'" class="flex-1 overflow-auto bg-surface-0 dark:bg-surface-0">
      <div class="flex items-center justify-center h-full">
        <p class="text-[12px] text-text-muted font-mono">{{ t.bottom.analysisPlaceholder }}</p>
      </div>
    </div>

    <!-- Agent tab group -->
    <div v-else-if="activeTab === 'agent'" class="flex flex-col flex-1 overflow-hidden">
      <!-- Agent sub-tabs -->
      <div class="flex items-center h-[28px] border-b border-border-default shrink-0">
        <div
          v-for="st in agentSubTabs"
          :key="st"
          class="px-3 h-full flex items-center text-[11px] cursor-pointer transition-colors select-none"
          :class="agentSubTab === st
            ? 'text-text-primary border-b-2 border-primary-500'
            : 'text-text-secondary hover:text-text-primary'"
          @click="agentSubTab = st"
        >{{ (t.bottom as Record<string, string>)[st] || st }}</div>
      </div>

      <div class="flex-1 overflow-hidden">
        <AgentSessionPanel v-if="agentSubTab === 'session'" class="h-full" />
        <AgentTurnPanel v-else-if="agentSubTab === 'turn'" class="h-full" />
        <AgentEventStreamPanel v-else-if="agentSubTab === 'events'" class="h-full" />
        <AgentDiffReviewPanel v-else-if="agentSubTab === 'diff'" class="h-full" />
        <AgentAuditLogPanel v-else-if="agentSubTab === 'audit'" class="h-full" />
      </div>
    </div>
  </div>
</template>
