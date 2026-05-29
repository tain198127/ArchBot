<script setup lang="ts">
import { useId, provide } from 'vue'
import { RADIO_GROUP_NAME } from './formKeys'

interface Props {
  modelValue?: string
  disabled?: boolean
}

defineProps<Props>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const groupName = useId()
provide(RADIO_GROUP_NAME, groupName)

function update(val: string) {
  emit('update:modelValue', val)
}
</script>

<template>
  <div class="flex flex-wrap gap-2" role="radiogroup">
    <slot :update="update" :value="modelValue" :name="groupName" />
  </div>
</template>
