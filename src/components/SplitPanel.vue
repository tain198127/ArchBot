<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from '../i18n'

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

const emit = defineEmits<{
  collapse: [index: number]
  expand: [index: number]
}>()

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

/**
 * 拖拽分割线调整面板大小
 *
 * 业务逻辑：
 * 1. 记录鼠标起始位置和面板初始尺寸
 * 2. 监听 mousemove 计算偏移量，对右侧面板需反向计算（容器尺寸 - 鼠标位置）
 * 3. 当尺寸小于最小值的一半时自动触发折叠
 * 4. 尺寸在最小值和最大值之间正常调整
 * 5. mouseup 时解除监听
 */
function startDrag(splitterIndex: number, event: MouseEvent) {
  dragging.value = true
  event.preventDefault()

  const container = containerRef.value
  if (!container) return

  const containerSize = props.direction === 'horizontal'
    ? container.offsetWidth
    : container.offsetHeight

  const isLastPanel = splitterIndex === props.initialSizes.length - 2
  const panelIndex = isLastPanel ? splitterIndex + 1 : splitterIndex

  const startPos = props.direction === 'horizontal' ? event.clientX : event.clientY
  const startSize = sizes.value[panelIndex]
  const minSize = props.minSizes[panelIndex] || 100
  const maxSize = containerSize - 300
  const isCollapsiblePanel = props.collapsible[panelIndex]

  function onMouseMove(e: MouseEvent) {
    const currentPos = props.direction === 'horizontal' ? e.clientX : e.clientY
    let delta: number
    let newSize: number

    if (isLastPanel) {
      delta = startPos - currentPos
      newSize = startSize + delta
    } else {
      delta = currentPos - startPos
      newSize = startSize + delta
    }

    if (isCollapsiblePanel && newSize < minSize / 2) {
      collapsePanel(panelIndex)
      onMouseUp()
      return
    }

    if (newSize >= minSize && newSize <= maxSize) {
      sizes.value[panelIndex] = newSize
    }
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
  if (collapsed.value[index]) {
    return { width: '0px', height: '0px', overflow: 'hidden', flex: '0' }
  }
  const size = sizes.value[index]
  if (size === -1) {
    return { flex: '1', minWidth: '0', minHeight: '0' }
  }
  const prop = props.direction === 'horizontal' ? 'width' : 'height'
  return { [prop]: `${size}px`, flexShrink: '0' }
}

const collapseButtonDirection = computed(() => {
  return props.direction === 'horizontal' ? '→' : '↓'
})
</script>

<template>
  <div
    ref="containerRef"
    class="split-panel"
    :class="[direction, { dragging }]"
  >
    <template v-for="(_, index) in initialSizes" :key="index">
      <div v-show="!collapsed[index]" class="panel" :style="getPanelStyle(index)">
        <slot :name="`panel-${index}`" />
      </div>

      <div
        v-if="index < initialSizes.length - 1"
        class="splitter"
        :class="[direction, { 'has-collapse': collapsible[index + 1] }]"
        @mousedown="startDrag(index, $event)"
      >
        <button
          v-if="collapsible[index + 1] && !collapsed[index + 1]"
          class="collapse-btn"
          :class="direction"
          :title="`${t.panel.collapse}${collapseLabels[index + 1] || ''}`"
          @click.stop="collapsePanel(index + 1)"
          @mousedown.stop
        >
          {{ collapseButtonDirection }}
        </button>
      </div>
    </template>

    <div
      v-for="(isCollapsed, index) in collapsed"
      :key="`collapsed-${index}`"
      v-show="isCollapsed"
      class="collapsed-bar"
      :class="direction"
      @click="expandPanel(index)"
    >
      <span class="collapsed-icon">{{ collapseIcons[index] || '◫' }}</span>
      <span v-if="collapseLabels[index]" class="collapsed-label">{{ collapseLabels[index] }}</span>
    </div>
  </div>
</template>

<style scoped>
.split-panel {
  display: flex;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.split-panel.horizontal {
  flex-direction: row;
}

.split-panel.vertical {
  flex-direction: column;
}

.split-panel.dragging {
  cursor: col-resize;
  user-select: none;
}

.split-panel.vertical.dragging {
  cursor: row-resize;
}

.panel {
  overflow: hidden;
  position: relative;
}

.splitter {
  flex-shrink: 0;
  background: #e0e0e0;
  position: relative;
  transition: background 0.15s;
}

.splitter::after {
  content: '';
  position: absolute;
  z-index: 10;
}

.splitter.horizontal {
  width: 1px;
  cursor: col-resize;
}

.splitter.horizontal::after {
  top: 0;
  bottom: 0;
  left: -1px;
  right: -1px;
  width: 3px;
}

.splitter.vertical {
  height: 1px;
  cursor: row-resize;
}

.splitter.vertical::after {
  left: 0;
  right: 0;
  top: -1px;
  bottom: -1px;
  height: 3px;
}

.splitter:hover {
  background: #409eff;
}

.collapse-btn {
  position: absolute;
  z-index: 20;
  background: #f5f5f5;
  border: 1px solid #e0e0e0;
  color: #666;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
  box-shadow: none;
  padding: 0;
}

.collapse-btn.horizontal {
  top: 50%;
  left: -7px;
  transform: translateY(-50%);
  width: 14px;
  height: 28px;
  border-radius: 4px;
  font-size: 10px;
}

.collapse-btn.vertical {
  left: 50%;
  top: -7px;
  transform: translateX(-50%);
  width: 28px;
  height: 14px;
  border-radius: 4px;
  font-size: 10px;
}

.collapse-btn:hover {
  background: #409eff;
  color: #fff;
  border-color: #409eff;
}

.collapsed-bar {
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  background: #f5f5f5;
  border: 1px solid #e0e0e0;
  transition: background 0.15s;
  flex-shrink: 0;
}

.collapsed-bar.horizontal {
  width: 28px;
  flex-direction: column;
  gap: 4px;
  padding: 8px 0;
}

.collapsed-bar.vertical {
  height: 28px;
  flex-direction: row;
  gap: 6px;
  padding: 0 8px;
}

.collapsed-bar:hover {
  background: #e8f4ff;
  border-color: #409eff;
}

.collapsed-icon {
  font-size: 14px;
  color: #666;
}

.collapsed-label {
  font-size: 11px;
  color: #666;
  writing-mode: vertical-rl;
  text-orientation: mixed;
}

.collapsed-bar.vertical .collapsed-label {
  writing-mode: horizontal-tb;
}

@media (prefers-color-scheme: dark) {
  .splitter {
    background: #3c3c3c;
  }

  .splitter:hover {
    background: #409eff;
  }

  .collapse-btn {
    background: #2b2b2b;
    border-color: #444;
    color: #aaa;
  }

  .collapse-btn:hover {
    background: #409eff;
    color: #fff;
    border-color: #409eff;
  }

  .collapsed-bar {
    background: #2b2b2b;
    border-color: #3c3c3c;
  }

  .collapsed-bar:hover {
    background: #2d3a4a;
    border-color: #409eff;
  }

  .collapsed-icon {
    color: #aaa;
  }

  .collapsed-label {
    color: #aaa;
  }
}
</style>
