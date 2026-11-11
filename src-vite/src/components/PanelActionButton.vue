<template>
  <button
    type="button"
    class="inline-flex min-h-0 items-center gap-1 rounded-box transition-colors disabled:pointer-events-none"
    :class="[sizeClass, buttonClass]"
    :disabled="disabled"
    @click="emit('click', $event)"
  >
    <component
      v-if="icon"
      :is="icon"
      :class="iconClass"
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
  size?: 'default' | 'small';
}>(), {
  disabled: false,
  danger: false,
  primary: false,
  selected: false,
  size: 'default',
});

const emit = defineEmits<{
  click: [event: MouseEvent];
}>();

const sizeClass = computed(() => props.size === 'small'
  ? 'h-6 px-1.5 text-[10px]'
  : 'h-8 px-2 text-xs');

const iconClass = computed(() => props.size === 'small' ? 'h-3 w-3 shrink-0' : 'h-3.5 w-3.5 shrink-0');

const buttonClass = computed(() => {
  if (props.disabled) return 'text-base-content/30';
  if (props.danger) return 'text-error/70 hover:text-error hover:bg-error/10 cursor-pointer';
  if (props.primary) return 'text-primary-content/70 bg-primary/70 hover:text-primary-content hover:bg-primary cursor-pointer';
  if (props.selected) return 'text-base-content/70 bg-base-100/30 hover:text-base-content cursor-pointer';
  return 'text-base-content/70 hover:bg-base-100 hover:text-base-content cursor-pointer';
});
</script>
