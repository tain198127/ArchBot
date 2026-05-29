<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from '../../i18n'

const { t } = useI18n()

const props = withDefaults(defineProps<{
  direction?: 'horizontal' | 'vertical'
  initialSizes?: number[]
  minSizes?: number[]
  collapsible?: boolean[]
  collapseIcons?: string[]
  collapseLabels?: string[]
}>(), {
  direction: 'horizontal',
  initialSizes: () => [250, -1],
  minSizes: () => [150, 200],
  collapsible: () => [],
  collapseIcons: () => [],
  collapseLabels: () => []
})

const emit = defineEmits<{ collapse: [index: number]; expand: [index: number] }>()

const containerRef = ref<HTMLElement | null>(null)
const sizes = ref<number[]>([...props.initialSizes])
const dragging = ref(false)
const collapsed = ref<boolean[]>(props.initialSizes.map(() => false))
const sizesBeforeCollapse = ref<number[]>([...props.initialSizes])

function collapsePanel(index: number) {
  sizesBeforeCollapse.value[index] = sizes.value[index]
  collapsed.value[index] = true
  emit('collapse', index)
}

function expandPanel(index: number) {
  collapsed.value[index] = false
  sizes.value[index] = sizesBeforeCollapse.value[index]
  emit('expand', index)
}

function startDrag(splitterIndex: number, event: MouseEvent) {
  dragging.value = true
  event.preventDefault()
  const container = containerRef.value
  if (!container) return
  const containerSize = props.direction === 'horizontal' ? container.offsetWidth : container.offsetHeight
  const isLastPanel = splitterIndex === props.initialSizes.length - 2
  const panelIndex = isLastPanel ? splitterIndex + 1 : splitterIndex
  const startPos = props.direction === 'horizontal' ? event.clientX : event.clientY
  const startSize = sizes.value[panelIndex]
  const minSize = props.minSizes[panelIndex] || 100
  const maxSize = containerSize - 300
  const isCollapsiblePanel = props.collapsible[panelIndex]

  function onMouseMove(e: MouseEvent) {
    const currentPos = props.direction === 'horizontal' ? e.clientX : e.clientY
    let delta: number; let newSize: number
    if (isLastPanel) { delta = startPos - currentPos; newSize = startSize + delta }
    else { delta = currentPos - startPos; newSize = startSize + delta }
    if (isCollapsiblePanel && newSize < minSize / 2) { collapsePanel(panelIndex); onMouseUp(); return }
    if (newSize >= minSize && newSize <= maxSize) { sizes.value[panelIndex] = newSize }
  }

  function onMouseUp() {
    dragging.value = false
    document.removeEventListener('mousemove', onMouseMove)
    document.removeEventListener('mouseup', onMouseUp)
  }
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}

function getPanelStyle(index: number): Record<string, string> {
  if (collapsed.value[index]) return { width: '0px', height: '0px', overflow: 'hidden', flex: '0' }
  const size = sizes.value[index]
  if (size === -1) return { flex: '1', minWidth: '0', minHeight: '0' }
  const prop = props.direction === 'horizontal' ? 'width' : 'height'
  return { [prop]: `${size}px`, flexShrink: '0' }
}

const collapseButtonDirection = computed(() => props.direction === 'horizontal' ? '→' : '↓')
</script>

<template>
  <div
    ref="containerRef"
    class="flex w-full h-full overflow-hidden"
    :class="[direction === 'horizontal' ? 'flex-row' : 'flex-col', dragging ? (direction === 'horizontal' ? 'cursor-col-resize select-none' : 'cursor-row-resize select-none') : '']"
  >
    <template v-for="(_, index) in initialSizes" :key="index">
      <div v-show="!collapsed[index]" class="overflow-hidden relative" :style="getPanelStyle(index)">
        <slot :name="`panel-${index}`" />
      </div>

      <div
        v-if="index < initialSizes.length - 1"
        class="splitter shrink-0 bg-border-default hover:bg-primary-500 transition-colors relative"
        :class="direction === 'horizontal' ? 'w-px cursor-col-resize' : 'h-px cursor-row-resize'"
        @mousedown="startDrag(index, $event)"
      >
        <button
          v-if="collapsible[index + 1] && !collapsed[index + 1]"
          class="collapse-btn absolute z-20 flex items-center justify-center bg-surface-200 dark:bg-surface-200 border border-border-default text-text-secondary text-[10px] rounded hover:bg-primary-500 hover:text-white hover:border-primary-500 transition-all p-0"
          :class="direction === 'horizontal' ? 'top-1/2 -translate-y-1/2 w-3.5 h-7' : 'left-1/2 -translate-x-1/2 w-7 h-3.5'"
          :title="`${t.panel.collapse}${collapseLabels[index + 1] || ''}`"
          @click.stop="collapsePanel(index + 1)"
          @mousedown.stop
        >
          {{ collapseButtonDirection }}
        </button>
      </div>
    </template>

    <div
      v-for="(isCollapsed, idx) in collapsed" :key="`collapsed-${idx}`"
      v-show="isCollapsed"
      class="collapsed-bar flex items-center justify-center cursor-pointer bg-surface-200 dark:bg-surface-200 border border-border-default transition-colors hover:bg-surface-100 dark:hover:bg-surface-100 hover:border-primary-500 shrink-0"
      :class="direction === 'horizontal' ? 'w-7 flex-col gap-1 py-2' : 'h-7 flex-row gap-1.5 px-2'"
      @click="expandPanel(idx)"
    >
      <span class="text-sm text-text-secondary">{{ collapseIcons[idx] || '◫' }}</span>
      <span
        v-if="collapseLabels[idx]"
        class="text-xs text-text-secondary"
        :class="direction === 'horizontal' ? '[writing-mode:vertical-rl]' : ''"
      >{{ collapseLabels[idx] }}</span>
    </div>
  </div>
</template>
