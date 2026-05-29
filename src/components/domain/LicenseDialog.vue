<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import VDialog from '../base/VDialog.vue'
import VInput from '../base/VInput.vue'
import VButton from '../base/VButton.vue'
import { useToast } from '../../composables/useToast'
import { useI18n } from '../../i18n'

const { t } = useI18n()
const toast = useToast()
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
    toast.success(lic.registerSuccess)
  } catch (e) {
    toast.error(String(e))
  } finally {
    loading.value = false
  }
}

function copyMachineId() {
  navigator.clipboard.writeText(machineId.value)
  toast.success((t.value.common as Record<string, string>).copySuccess)
}

defineExpose({ show })
</script>

<template>
  <VDialog :visible="visible" :title="lic.title" @update:visible="visible = $event">
    <div v-if="isRegistered" class="text-center py-4">
      <p class="text-5xl text-success-500 mb-2">&#10003;</p>
      <p>{{ lic.registered }}</p>
      <p class="text-xs text-text-muted mt-2">{{ lic.machineId }}: {{ machineId }}</p>
    </div>
    <div v-else class="flex flex-col gap-4">
      <div>
        <label class="block text-sm text-text-primary mb-1">{{ lic.machineId }}</label>
        <div class="flex gap-2">
          <div class="flex-1">
            <VInput :model-value="machineId" disabled />
          </div>
          <VButton size="sm" variant="secondary" @click="copyMachineId">
            {{ t.newProject.browse }}
          </VButton>
        </div>
        <p class="text-xs text-text-muted mt-1">{{ lic.machineIdHint }}</p>
      </div>
      <div>
        <label class="block text-sm text-text-primary mb-1">{{ lic.verificationCode }}</label>
        <VInput
          v-model="verificationCode"
          :placeholder="lic.verificationCodePlaceholder"
          @keyup.enter="handleRegister"
        />
      </div>
    </div>
    <template #footer>
      <VButton variant="secondary" @click="visible = false">{{ t.newProject.cancel }}</VButton>
      <VButton v-if="!isRegistered" :loading="loading" @click="handleRegister">
        {{ lic.registerBtn }}
      </VButton>
    </template>
  </VDialog>
</template>
