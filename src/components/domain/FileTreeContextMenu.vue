<script setup lang="ts">
import { useI18n } from '../../i18n'

interface Props {
  visible: boolean
  x: number
  y: number
  type: 'dataStandard' | 'domain' | 'category' | 'group' | 'item'
  domain?: string
  groupKey?: string
  itemKey?: string
}

defineProps<Props>()

const emit = defineEmits<{
  'new-domain': []
  'new-entity': []
  'new-dict': []
  'import-file': []
  'reverse-db': []
  'reverse-ddl': []
  'reverse-code': []
  'action': [actionType: string]
}>()

const { t } = useI18n()
const ctx = t.value.contextMenu as Record<string, string>

function emitAction(type: string) {
  emit('action', type)
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="fixed z-[9999] min-w-[140px] bg-white dark:bg-surface-0 border border-border-default rounded-lg shadow-lg py-1"
      :style="{ left: x + 'px', top: y + 'px' }"
    >
      <!-- dataStandard group context menu -->
      <template v-if="type === 'dataStandard'">
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emit('new-domain')">{{ ctx.newDomain }}</button>
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emit('new-entity')">{{ ctx.newEntity }}</button>
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emit('new-dict')">{{ ctx.newDictionary }}</button>
        <div class="h-px mx-2 my-1 bg-border-default" />
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emit('import-file')">{{ ctx.importFile }}</button>
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emit('reverse-db')">{{ ctx.reverseDb }}</button>
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emit('reverse-ddl')">{{ ctx.reverseDdl }}</button>
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emit('reverse-code')">{{ ctx.reverseCode }}</button>
      </template>

      <!-- domain context menu -->
      <template v-else-if="type === 'domain'">
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emit('new-entity')">{{ ctx.newEntity }}</button>
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emit('new-dict')">{{ ctx.newDictionary }}</button>
      </template>

      <!-- category context menu -->
      <template v-else-if="type === 'category'">
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('brainstorm')">{{ ctx.brainstorm }}</button>
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('generateSRS')">{{ ctx.generateSRS }}</button>
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('exportHTML')">{{ ctx.exportHTML }}</button>
        <div class="h-px mx-2 my-1 bg-border-default" />
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('sealAll')">{{ ctx.sealAll }}</button>
        <div class="h-px mx-2 my-1 bg-border-default" />
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('importProject')">{{ ctx.importProject }}</button>
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('exportPackage')">{{ ctx.exportPackage }}</button>
      </template>

      <!-- group context menu -->
      <template v-else-if="type === 'group'">
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('analyze')">{{ ctx.analyze }}</button>
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('review')">{{ ctx.review }}</button>
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('write')">{{ ctx.write }}</button>
        <div class="h-px mx-2 my-1 bg-border-default" />
        <template v-if="groupKey === 'bizContext'">
          <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('brainstorm')">{{ ctx.brainstorm }}</button>
        </template>
        <template v-else-if="groupKey === 'dataStandard'">
          <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('reverse')">{{ ctx.reverseDb }}</button>
        </template>
        <template v-else-if="groupKey === 'funcSpec'">
          <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('preview')">{{ ctx.preview }}</button>
        </template>
        <div class="h-px mx-2 my-1 bg-border-default" />
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('seal')">{{ ctx.seal }}</button>
        <div class="h-px mx-2 my-1 bg-border-default" />
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('import')">{{ ctx.import }}</button>
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('export')">{{ ctx.export }}</button>
      </template>

      <!-- item context menu -->
      <template v-else-if="type === 'item'">
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('analyze')">{{ ctx.analyze }}</button>
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('review')">{{ ctx.review }}</button>
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('write')">{{ ctx.write }}</button>
        <div class="h-px mx-2 my-1 bg-border-default" />
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('seal')">{{ ctx.seal }}</button>
        <div class="h-px mx-2 my-1 bg-border-default" />
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('import')">{{ ctx.import }}</button>
        <button class="block w-full text-left px-4 py-1.5 text-sm text-text-primary cursor-pointer hover:bg-surface-100 dark:hover:bg-surface-200" @click="emitAction('export')">{{ ctx.export }}</button>
      </template>
    </div>
  </Teleport>
</template>

