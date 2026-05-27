<script setup lang="ts">
import { ref } from 'vue'

const props = withDefaults(defineProps<{
  direction?: 'horizontal' | 'vertical'
  initialSizes?: number[]
  minSizes?: number[]
}>(), {
  direction: 'horizontal',
  initialSizes: () => [250, -1],
  minSizes: () => [150, 200]
})

const containerRef = ref<HTMLElement | null>(null)
const sizes = ref<number[]>([...props.initialSizes])
const dragging = ref(false)

function startDrag(index: number, event: MouseEvent) {
  dragging.value = true
  event.preventDefault()

  const startPos = props.direction === 'horizontal' ? event.clientX : event.clientY
  const startSize = sizes.value[index]
  const container = containerRef.value
  if (!container) return

  const containerSize = props.direction === 'horizontal'
    ? container.offsetWidth
    : container.offsetHeight

  const minSize = props.minSizes[index] || 100
  const maxSize = containerSize - 200

  function onMouseMove(e: MouseEvent) {
    const currentPos = props.direction === 'horizontal' ? e.clientX : e.clientY
    const delta = currentPos - startPos
    const newSize = startSize + delta

    if (newSize >= minSize && newSize <= maxSize) {
      sizes.value[index] = newSize
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
  const size = sizes.value[index]
  if (size === -1) {
    return { flex: '1', minWidth: '0', minHeight: '0' }
  }
  const prop = props.direction === 'horizontal' ? 'width' : 'height'
  return { [prop]: `${size}px`, flexShrink: '0' }
}
</script>

<template>
  <div
    ref="containerRef"
    class="split-panel"
    :class="[direction, { dragging }]"
  >
    <template v-for="(_, index) in initialSizes" :key="index">
      <div class="panel" :style="getPanelStyle(index)">
        <slot :name="`panel-${index}`" />
      </div>
      <div
        v-if="index < initialSizes.length - 1"
        class="splitter"
        :class="direction"
        @mousedown="startDrag(index, $event)"
      />
    </template>
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
  transition: background 0.15s;
}

.splitter.horizontal {
  width: 3px;
  cursor: col-resize;
}

.splitter.vertical {
  height: 3px;
  cursor: row-resize;
}

.splitter:hover {
  background: #409eff;
}

@media (prefers-color-scheme: dark) {
  .splitter {
    background: #3c3c3c;
  }

  .splitter:hover {
    background: #409eff;
  }
}
</style>
