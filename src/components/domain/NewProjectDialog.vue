<script setup lang="ts">
import { reactive, ref } from 'vue'
import { createProject } from '../../api'
import { openDirectoryDialog } from '../../api/filePicker'
import VDialog from '../base/VDialog.vue'
import VInput from '../base/VInput.vue'
import VFormItem from '../base/VFormItem.vue'
import VButton from '../base/VButton.vue'
import { useToast } from '../../composables/useToast'
import { useI18n } from '../../i18n'

const { t } = useI18n()
const toast = useToast()

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
  try {
    const selected = await openDirectoryDialog()
    if (selected) {
      form.location = selected
    }
  } catch (e) {
    toast.error(String(e))
  }
}

async function handleCreate() {
  if (!form.name.trim()) {
    toast.warning(t.value.newProject.nameRequired)
    return
  }
  if (!form.location) {
    toast.warning(t.value.newProject.locationRequired)
    return
  }

  loading.value = true
  try {
    const path = await createProject(form.location, form.name.trim())
    toast.success(t.value.newProject.success)
    visible.value = false
    emit('created', path, form.name.trim())
  } catch (e) {
    toast.error(String(e))
  } finally {
    loading.value = false
  }
}

defineExpose({ show })
</script>

<template>
  <VDialog :visible="visible" :title="t.newProject.title" @update:visible="visible = $event">
    <VFormItem :label="t.newProject.name">
      <VInput
        v-model="form.name"
        :placeholder="t.newProject.namePlaceholder"
        @keyup.enter="handleCreate"
      />
    </VFormItem>
    <VFormItem :label="t.newProject.location">
      <div class="flex gap-2 w-full">
        <div class="flex-1">
          <VInput v-model="form.location" :placeholder="t.newProject.locationPlaceholder" disabled />
        </div>
        <VButton variant="secondary" @click="selectDirectory">{{ t.newProject.browse }}</VButton>
      </div>
    </VFormItem>
    <p class="text-xs text-text-muted mt-1 pl-0">{{ t.newProject.fileExtHint }}</p>
    <template #footer>
      <VButton variant="secondary" @click="visible = false">{{ t.newProject.cancel }}</VButton>
      <VButton :loading="loading" @click="handleCreate">{{ t.newProject.create }}</VButton>
    </template>
  </VDialog>
</template>
