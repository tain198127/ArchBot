<script setup lang="ts">
import { useId, provide } from 'vue'
import { FORM_ITEM_ID } from './formKeys'

interface Props {
  label: string
  required?: boolean
}

defineProps<Props>()

const id = useId()
provide(FORM_ITEM_ID, id)
</script>

<template>
  <div class="flex flex-col gap-1.5 mb-4">
    <label :for="id" class="text-sm font-medium text-text-primary">
      {{ label }}
      <span v-if="required" class="text-danger-500 ml-0.5">*</span>
    </label>
    <slot />
  </div>
</template>
