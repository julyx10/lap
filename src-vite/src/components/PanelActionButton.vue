<template>
  <button
    type="button"
    class="inline-flex h-7 min-h-0 items-center gap-1 rounded-box border-0 bg-transparent px-2 text-xs font-medium shadow-none transition-colors disabled:pointer-events-none"
    :class="buttonClass"
    :disabled="disabled"
    @click="emit('click', $event)"
  >
    <component
      v-if="icon"
      :is="icon"
      class="h-3.5 w-3.5 shrink-0"
    />
    <span class="whitespace-nowrap">
      <slot />
    </span>
  </button>
</template>

<script setup lang="ts">
import { computed, type Component } from 'vue';

const props = withDefaults(defineProps<{
  icon?: Component;
  disabled?: boolean;
  danger?: boolean;
  primary?: boolean;
  selected?: boolean;
}>(), {
  disabled: false,
  danger: false,
  primary: false,
  selected: false,
});

const emit = defineEmits<{
  click: [event: MouseEvent];
}>();

const buttonClass = computed(() => {
  if (props.disabled) return 'text-base-content/30';
  if (props.danger) return 'text-error/70 hover:text-error hover:bg-error/10 cursor-pointer';
  if (props.primary) return 'text-base-content/70 bg-primary/70 hover:text-base-content hover:bg-primary cursor-pointer';
  if (props.selected) return 'text-base-content/70 bg-base-100/30 hover:text-base-content cursor-pointer';
  return 'text-base-content/70 hover:bg-base-100 hover:text-base-content cursor-pointer';
});
</script>
