<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from '../../i18n'
import VTabs from '../base/VTabs.vue'
import RulesSection from './context/RulesSection.vue'
import MemorySection from './context/MemorySection.vue'
import SessionSection from './context/SessionSection.vue'
import CodebaseSection from './context/CodebaseSection.vue'
import GitSection from './context/GitSection.vue'

const { tt } = useI18n()

const activeTab = ref('rules')

const tabs = [
  { value: 'rules', label: tt('context.rules') },
  { value: 'memory', label: tt('context.memory') },
  { value: 'session', label: tt('context.session') },
  { value: 'codebase', label: tt('context.codebase') },
  { value: 'git', label: tt('context.git') },
]
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="px-4 pt-4">
      <h2 class="text-lg font-semibold text-text-primary">{{ tt('context.title') }}</h2>
    </div>
    <VTabs v-model="activeTab" :tabs="tabs">
      <template #rules>
        <RulesSection />
      </template>
      <template #memory>
        <MemorySection />
      </template>
      <template #session>
        <SessionSection />
      </template>
      <template #codebase>
        <CodebaseSection />
      </template>
      <template #git>
        <GitSection />
      </template>
    </VTabs>
  </div>
</template>
