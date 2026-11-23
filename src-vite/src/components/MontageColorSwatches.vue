<template>
  <div class="flex gap-2">
    <button
      v-for="color in swatches"
      :key="color"
      class="size-5.5 rounded-md border border-base-content/15 cursor-pointer"
      :class="{ 'outline-2 outline-offset-2 outline-primary': modelValue === color }"
      :style="{ backgroundColor: color }"
      @click="emit('update:modelValue', color)"
    ></button>
    <label
      class="relative size-5.5 rounded-md border border-base-content/15 cursor-pointer overflow-hidden"
      style="background: conic-gradient(red, yellow, lime, cyan, blue, magenta, red)"
    >
      <input :value="modelValue" type="color" class="absolute inset-0 opacity-0 cursor-pointer"
        @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)" @change="rememberCustom"
        @keydown.esc.stop="($event.target as HTMLInputElement).blur()" />
    </label>
  </div>
</template>
<script setup lang="ts">
import { computed } from 'vue';
const props = defineProps<{ modelValue: string; recent: string; presets: readonly string[] }>();
const emit = defineEmits(['update:modelValue', 'update:recent']);
const swatches = computed(() => {
  const saved = String(props.recent).toLowerCase();
  const recent = /^#[0-9a-f]{6}$/.test(saved) && !props.presets.includes(saved) ? saved : '#7c2d12';
  return [...props.presets, props.presets.includes(props.modelValue) ? recent : props.modelValue];
});
function rememberCustom(event: Event) {
  const color = (event.target as HTMLInputElement).value.toLowerCase();
  emit('update:modelValue', color);
  if (!props.presets.includes(color)) emit('update:recent', color);
}
</script>
