import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const isRegistered = ref(false)
const machineId = ref('')

export function useLicense() {
  async function initLicense() {
    try {
      const status = await invoke<{ registered: boolean; machine_id: string }>('get_license_status')
      isRegistered.value = status.registered
      machineId.value = status.machine_id
    } catch {
      isRegistered.value = false
    }
  }

  return {
    isRegistered,
    machineId,
    initLicense
  }
}
