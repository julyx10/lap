<template>
  <button
    class="relative inline-flex h-4 w-8 items-center rounded-full transition-colors focus:outline-none"
    :class="modelValue ? 't-color-bg-highlight' : 't-color-bg-light'"
    role="switch"
    :aria-checked="modelValue.toString()"
    @click="emit('update:modelValue', !modelValue)"
  >
    <span
      class="inline-block h-4 w-4 transform rounded-full bg-gray-200 transition"
      :class="modelValue ? 'translate-x-4' : 'translate-x-0'"
    />
  </button>
</template>

<script setup lang="ts">
const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  (event: 'update:modelValue', value: boolean): void
}>()
</script>
