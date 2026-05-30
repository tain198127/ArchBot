<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useI18n } from '../../i18n'

const { t } = useI18n()

interface RuntimeStep {
  key: string
  label: string
  size: string
  status: 'pending' | 'extracting' | 'verifying' | 'done' | 'failed'
  progress: number
}

const steps = reactive<RuntimeStep[]>([
  { key: 'claude_code', label: 'Claude Code', size: '25 MB', status: 'pending', progress: 0 },
  { key: 'hermes',     label: 'Hermes',      size: '120 MB', status: 'pending', progress: 0 },
  { key: 'opencode',   label: 'OpenCode',    size: '30 MB', status: 'pending', progress: 0 },
  { key: 'openclaw',   label: 'OpenClaw',    size: '80 MB', status: 'pending', progress: 0 },
])

const overallProgress = ref(0)
const complete = ref(false)
const hasError = ref(false)
const statusText = ref('')

const emit = defineEmits<{
  done: []
}>()

async function runOnboarding() {
  for (let i = 0; i < steps.length; i++) {
    const step = steps[i]
    step.status = 'extracting'
    statusText.value = t.value.onboarding.extracting.replace('{name}', step.label)

    // 模拟解压进度 (实际应调用 Tauri backend 解压 .tar.gz)
    for (let p = 0; p <= 100; p += 10) {
      step.progress = p
      overallProgress.value =
        ((i * 100) + p) / steps.length
      await delay(80)
    }

    step.status = 'verifying'
    statusText.value = t.value.onboarding.verifying.replace('{name}', step.label)
    await delay(200)

    step.progress = 100
    step.status = 'done'
    overallProgress.value = ((i + 1) * 100) / steps.length
  }

  complete.value = true
  statusText.value = t.value.onboarding.ready
  await delay(800)
  emit('done')
}

function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

onMounted(() => {
  runOnboarding()
})
</script>

<template>
  <div class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/60 backdrop-blur-sm">
    <div class="w-[480px] rounded-2xl bg-surface-800 shadow-2xl border border-surface-700 p-8">

      <!-- 标题 -->
      <h1 class="text-lg font-semibold text-text-100 mb-2">
        {{ t.onboarding.title }}
      </h1>
      <p class="text-sm text-text-400 mb-6">
        {{ t.onboarding.subtitle }}
      </p>

      <!-- 进度条 -->
      <div class="mb-6">
        <div class="h-2 rounded-full bg-surface-700 overflow-hidden">
          <div
            class="h-full rounded-full transition-all duration-300"
            :class="hasError ? 'bg-red-500' : 'bg-indigo-500'"
            :style="{ width: overallProgress + '%' }"
          />
        </div>
        <p class="mt-2 text-xs text-text-500 text-center">{{ statusText }}</p>
      </div>

      <!-- Runtime 列表 -->
      <ul class="space-y-2">
        <li
          v-for="step in steps"
          :key="step.key"
          class="flex items-center gap-3 px-3 py-2 rounded-lg"
          :class="
            step.status === 'failed' ? 'bg-red-500/10 text-red-400' :
            step.status === 'done' ? 'bg-emerald-500/10 text-emerald-400' :
            'bg-surface-700/50 text-text-400'
          "
        >
          <!-- 状态图标 -->
          <span class="w-5 h-5 flex-shrink-0 flex items-center justify-center text-sm">
            <span v-if="step.status === 'done'">&#10003;</span>
            <span v-else-if="step.status === 'failed'">&#10007;</span>
            <span v-else-if="step.status === 'extracting' || step.status === 'verifying'" class="animate-spin">&#8635;</span>
            <span v-else>&#9679;</span>
          </span>

          <!-- Runtime 名称 + 大小 -->
          <span class="flex-1 text-sm font-medium">{{ step.label }}</span>
          <span class="text-xs opacity-60">{{ step.size }}</span>

          <!-- 单项进度 -->
          <div class="w-16 h-1.5 rounded-full bg-surface-700 overflow-hidden">
            <div
              class="h-full rounded-full transition-all duration-200"
              :class="step.status === 'failed' ? 'bg-red-500' : 'bg-indigo-500'"
              :style="{ width: step.progress + '%' }"
            />
          </div>
        </li>
      </ul>

      <!-- 完成提示 -->
      <p
        v-if="complete"
        class="mt-6 text-center text-sm text-emerald-400"
      >
        {{ t.onboarding.completeHint }}
      </p>
    </div>
  </div>
</template>
