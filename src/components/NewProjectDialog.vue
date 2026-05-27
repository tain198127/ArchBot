<script setup lang="ts">
import { reactive, ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import { useI18n } from '../i18n'

const { t } = useI18n()

const visible = ref(false)
const loading = ref(false)
const form = reactive({
  name: '',
  location: ''
})

const emit = defineEmits<{
  created: [path: string, name: string]
}>()

function show() {
  form.name = ''
  form.location = ''
  visible.value = true
}

async function selectDirectory() {
  const selected = await open({ directory: true })
  if (selected) {
    form.location = selected as string
  }
}

async function handleCreate() {
  if (!form.name.trim()) {
    ElMessage.warning(t.value.newProject.nameRequired)
    return
  }
  if (!form.location) {
    ElMessage.warning(t.value.newProject.locationRequired)
    return
  }

  loading.value = true
  try {
    const path = await invoke<string>('create_project', {
      dir: form.location,
      name: form.name.trim()
    })
    ElMessage.success(t.value.newProject.success)
    visible.value = false
    emit('created', path, form.name.trim())
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    loading.value = false
  }
}

defineExpose({ show })
</script>

<template>
  <el-dialog
    v-model="visible"
    :title="t.newProject.title"
    width="480px"
    :close-on-click-modal="false"
  >
    <el-form label-width="90px" label-position="left">
      <el-form-item :label="t.newProject.name">
        <el-input
          v-model="form.name"
          :placeholder="t.newProject.namePlaceholder"
          @keyup.enter="handleCreate"
        />
      </el-form-item>
      <el-form-item :label="t.newProject.location">
        <div style="display: flex; gap: 8px; width: 100%">
          <el-input
            v-model="form.location"
            :placeholder="t.newProject.locationPlaceholder"
            readonly
            style="flex: 1"
          />
          <el-button @click="selectDirectory">{{ t.newProject.browse }}</el-button>
        </div>
      </el-form-item>
    </el-form>
    <p class="file-ext-hint">{{ t.newProject.fileExtHint }}</p>
    <template #footer>
      <el-button @click="visible = false">{{ t.newProject.cancel }}</el-button>
      <el-button type="primary" :loading="loading" @click="handleCreate">
        {{ t.newProject.create }}
      </el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.file-ext-hint {
  font-size: 12px;
  color: var(--text-muted, #999);
  margin-top: 4px;
  padding-left: 90px;
}
</style>
