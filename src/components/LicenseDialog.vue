<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import { useI18n } from '../i18n'

const { t } = useI18n()
const lic = t.value.license as Record<string, string>

const visible = ref(false)
const loading = ref(false)
const machineId = ref('')
const verificationCode = ref('')
const isRegistered = ref(false)

function show() {
  verificationCode.value = ''
  loadStatus()
  visible.value = true
}

async function loadStatus() {
  try {
    const status = await invoke<{ registered: boolean; machine_id: string }>('get_license_status')
    machineId.value = status.machine_id
    isRegistered.value = status.registered
  } catch {
    machineId.value = '--'
  }
}

async function handleRegister() {
  if (!verificationCode.value.trim()) return
  loading.value = true
  try {
    await invoke<boolean>('register_software', { verificationCode: verificationCode.value.trim() })
    isRegistered.value = true
    ElMessage.success(lic.registerSuccess)
  } catch (e) {
    ElMessage.error(String(e))
  } finally {
    loading.value = false
  }
}

function copyMachineId() {
  navigator.clipboard.writeText(machineId.value)
  ElMessage.success('Copied')
}

defineExpose({ show })
</script>

<template>
  <el-dialog v-model="visible" :title="lic.title" width="440px" :close-on-click-modal="false">
    <div v-if="isRegistered" class="license-registered">
      <p class="license-status-icon">✓</p>
      <p>{{ lic.registered }}</p>
      <p class="license-machine">{{ lic.machineId }}: {{ machineId }}</p>
    </div>
    <div v-else class="license-form">
      <div class="license-field">
        <label>{{ lic.machineId }}</label>
        <div class="license-machine-row">
          <el-input :model-value="machineId" readonly />
          <el-button size="small" @click="copyMachineId">{{ t.newProject.browse }}</el-button>
        </div>
        <p class="license-hint">{{ lic.machineIdHint }}</p>
      </div>
      <div class="license-field">
        <label>{{ lic.verificationCode }}</label>
        <el-input
          v-model="verificationCode"
          :placeholder="lic.verificationCodePlaceholder"
          @keyup.enter="handleRegister"
        />
      </div>
    </div>
    <template #footer>
      <el-button @click="visible = false">{{ t.newProject.cancel }}</el-button>
      <el-button v-if="!isRegistered" type="primary" :loading="loading" @click="handleRegister">
        {{ lic.registerBtn }}
      </el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.license-registered {
  text-align: center;
  padding: 16px;
}
.license-status-icon {
  font-size: 48px;
  color: #67c23a;
  margin: 0 0 8px;
}
.license-machine {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 8px;
}
.license-field {
  margin-bottom: 16px;
}
.license-field label {
  display: block;
  font-size: 13px;
  margin-bottom: 4px;
  color: var(--text-primary);
}
.license-machine-row {
  display: flex;
  gap: 8px;
}
.license-hint {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 4px;
}
</style>
