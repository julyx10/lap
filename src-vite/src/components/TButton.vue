<template>
  <div>
    <div 
      class="relative inline-block"
      @mouseenter="showTooltip"
      @mouseleave="hideTooltip"
    >
      <button
        class="btn btn-ghost btn-square rounded-box border-0 focus:outline-none shadow-none flex flex-col"
        :class="[
          buttonClasses,
          {
            'btn-xs hover:bg-base-100/30': buttonSize === 'small',
            'btn-sm hover:bg-base-100/30': buttonSize === 'medium',
            'btn-lg hover:bg-base-100/30': buttonSize === 'large',
            'btn-disabled': disabled,
          }
        ]"
        :disabled="disabled"
        @click="$emit('click', $event)"
      >
        <component v-if="icon"
          :is="icon"
          :class="[
            iconClasses,
            {
              'w-4 h-4': buttonSize === 'small',
              'w-5 h-5': buttonSize === 'medium',
              'w-6 h-6': buttonSize === 'large',
              'text-primary': selected,
            }
          ]"
          :style="iconStyle"
        />
        <span 
          v-if="config.settings.showButtonText && text.length > 0"
          class='text-xs/2 whitespace-nowrap'
          :class="[
            textClasses,
            {
              'text-primary': selected,
            }
          ]"
        >
          {{ text }}
        </span>
      </button>
      <transition name="fade">
        <div 
          v-if="isHovered && tooltipText && config.settings.showToolTip"
          class="absolute z-1000 left-1/2 -translate-x-1/2 top-full mt-1 px-2 py-1 text-xs whitespace-nowrap rounded-box bg-neutral text-neutral-content shadow-lg pointer-events-none"
        >
          {{ tooltipText }}
        </div>
      </transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onBeforeUnmount } from 'vue';
import { config } from '@/common/config';
import type { Component } from 'vue';

const props = defineProps({
  buttonSize: {
    type: String,
    default: 'medium'         // 'small', 'medium'(default), 'large'
  },
  buttonClasses: {
    type: String,
    default: ''
  },
  icon: {
    type: Object as () => Component,
    required: false,
  },
  iconClasses: {
    type: [String, Array],
    default: ''
  },
  iconStyle: {
    type: Object,
    default: () => ({})
  },
  text: {
    type: String,
    default: ''
  },
  textClasses: {
    type: String,
    default: ''
  },
  disabled: {
    type: Boolean,
    default: false
  },
  selected: {
    type: Boolean,
    default: false
  },
  tooltip: {
    type: String,
    default: ''
  },
  shortcut: {
    type: String,
    default: ''
  },
  tooltipClasses: {
    type: String,
    default: 'tooltip-bottom'
  }
});

const emit = defineEmits(['click']);
const tooltipText = computed(() => {
  if (!props.shortcut) return props.tooltip;
  return props.tooltip ? `${props.tooltip} (${props.shortcut})` : props.shortcut;
});

const isHovered = ref(false);
let tooltipTimer: ReturnType<typeof setTimeout> | null = null;

const clearTooltipTimer = () => {
  if (tooltipTimer) {
    clearTimeout(tooltipTimer);
    tooltipTimer = null;
  }
};

const showTooltip = () => {
  if (!tooltipText.value || !config.settings.showToolTip) return;

  clearTooltipTimer();
  isHovered.value = true;

  tooltipTimer = setTimeout(() => {
    isHovered.value = false;
    tooltipTimer = null;
  }, 3000);
};

const hideTooltip = () => {
  clearTooltipTimer();
  isHovered.value = false;
};

onBeforeUnmount(() => {
  clearTooltipTimer();
});
</script>
