<script setup lang="ts">
import { Tabs, TabList, Tab, TabPanels, TabPanel } from 'primevue'

interface TabDef {
  value: string
  label: string
}

interface Props {
  modelValue?: string
  tabs: TabDef[]
}

defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>

<template>
  <Tabs :value="modelValue ?? tabs[0]?.value ?? ''" @update:value="emit('update:modelValue', $event as string)">
    <TabList class="flex border-b border-border-default gap-0">
      <Tab
        v-for="tab in tabs"
        :key="tab.value"
        :value="tab.value"
        :class="[
          'px-4 py-2.5 text-sm font-medium border-b-2 -mb-px transition-colors cursor-pointer',
          'focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:outline-none',
          modelValue === tab.value
            ? 'border-primary-500 text-primary-500'
            : 'border-transparent text-text-secondary hover:text-text-primary hover:border-border-default',
        ]"
      >
        {{ tab.label }}
      </Tab>
    </TabList>
    <TabPanels>
      <TabPanel v-for="tab in tabs" :key="tab.value" :value="tab.value">
        <div class="pt-4">
          <slot :name="tab.value" />
        </div>
      </TabPanel>
    </TabPanels>
  </Tabs>
</template>
