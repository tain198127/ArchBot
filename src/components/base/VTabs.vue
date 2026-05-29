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
    <TabList class="flex border-b border-border-default gap-0 px-1">
      <Tab
        v-for="tab in tabs"
        :key="tab.value"
        :value="tab.value"
        :class="[
          'relative px-3.5 py-2.5 text-[13px] font-medium transition-colors cursor-pointer select-none',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary-500/40 rounded-t-md',
          modelValue === tab.value
            ? 'text-primary-500'
            : 'text-text-secondary hover:text-text-primary',
        ]"
      >
        {{ tab.label }}
        <span
          v-if="modelValue === tab.value"
          class="absolute bottom-0 left-3.5 right-3.5 h-[2px] bg-primary-500 rounded-full"
        />
      </Tab>
    </TabList>
    <TabPanels>
      <TabPanel v-for="tab in tabs" :key="tab.value" :value="tab.value">
        <slot :name="tab.value" />
      </TabPanel>
    </TabPanels>
  </Tabs>
</template>
