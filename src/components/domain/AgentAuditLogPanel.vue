<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { agentStore, loadAuditLog } from '../../stores/agentStore'

const severityFilter = ref<string | null>(null)
const searchText = ref('')

onMounted(() => {
  loadAuditLog().catch(() => {})
})

function severityColor(severity: string): string {
  const map: Record<string, string> = {
    info: 'text-text-muted',
    warning: 'text-amber-500',
    high: 'text-orange-500',
    critical: 'text-red-500',
  }
  return map[severity] || 'text-text-muted'
}

const filteredEntries = () => {
  let entries = agentStore.auditLog
  if (severityFilter.value) {
    entries = entries.filter((e) => e.severity === severityFilter.value)
  }
  if (searchText.value.trim()) {
    const q = searchText.value.toLowerCase()
    entries = entries.filter((e) => e.action.toLowerCase().includes(q) || e.detail.toLowerCase().includes(q))
  }
  return entries
}
</script>

<template>
  <div class="flex flex-col h-full bg-surface-0">
    <!-- Toolbar -->
    <div class="flex items-center gap-2 px-3 py-1.5 border-b border-border-default">
      <span class="text-[12px] font-semibold text-text-primary">Audit Log</span>
      <input
        v-model="searchText"
        placeholder="Search..."
        class="flex-1 px-2 py-0.5 text-[11px] bg-surface-50 border border-border-default rounded outline-none focus:border-primary-500"
      />
      <select
        v-model="severityFilter"
        class="px-1.5 py-0.5 text-[11px] bg-surface-50 border border-border-default rounded"
      >
        <option :value="null">All</option>
        <option value="info">Info</option>
        <option value="warning">Warning</option>
        <option value="high">High</option>
        <option value="critical">Critical</option>
      </select>
      <button
        class="px-1.5 py-0.5 text-[10px] text-text-muted hover:text-text-primary"
        @click="loadAuditLog()"
      >Refresh</button>
    </div>

    <!-- Log entries -->
    <div class="flex-1 overflow-auto">
      <div
        v-for="(entry, idx) in filteredEntries()"
        :key="entry.log_id || idx"
        class="px-3 py-1.5 border-b border-border-default/30 hover:bg-surface-50 font-mono text-[11px]"
      >
        <div class="flex items-center gap-2">
          <span class="text-text-muted w-[70px]">{{ entry.created_at?.slice(11, 19) || '' }}</span>
          <span class="shrink-0 w-[55px]" :class="severityColor(entry.severity)">{{ entry.severity }}</span>
          <span class="text-text-secondary truncate">{{ entry.action }}</span>
        </div>
        <div v-if="entry.detail" class="text-text-muted/70 mt-0.5 truncate">{{ entry.detail }}</div>
      </div>
      <div v-if="filteredEntries().length === 0" class="flex items-center justify-center h-full">
        <p class="text-[12px] text-text-muted">No audit entries found.</p>
      </div>
    </div>
  </div>
</template>
