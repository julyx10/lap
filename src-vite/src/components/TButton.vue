<template>
  <div>
    <div 
      ref="triggerRef"
      class="relative inline-block"
      @mouseenter="showTooltip"
      @mouseleave="hideTooltip"
    >
      <div
        class="btn btn-ghost btn-square rounded-box border-0 focus:outline-none shadow-none flex flex-col text-base-content/70 hover:text-base-content"
        :class="[
          buttonClasses,
          {
            'btn-xs hover:bg-base-100/30': buttonSize === 'small',
            'btn-sm hover:bg-base-100/30': buttonSize === 'medium',
            'btn-lg hover:bg-base-100/30': buttonSize === 'large',
            'bg-base-100/30': selected,
            'pointer-events-none cursor-default text-base-content/30 hover:text-base-content/30': disabled,
          }
        ]"
        role="button"
        :aria-disabled="disabled"
        :tabindex="disabled ? -1 : 0"
        @click="handleClick"
        @keydown.enter.prevent="handleClick"
        @keydown.space.prevent="handleClick"
      >
        <component v-if="icon"
          :is="icon"
          :class="[
            iconClasses,
            {
              'w-4 h-4': buttonSize === 'small',
              'w-5 h-5': buttonSize === 'medium',
              'w-6 h-6': buttonSize === 'large',
              'text-primary': selected && !disabled,
              'text-base-content/30': disabled,
            }
          ]"
          :style="iconStyle"
        />
        <span 
          v-if="text.length > 0"
          class='text-xs/2 whitespace-nowrap'
          :class="[
            textClasses,
            {
              'text-primary': selected && !disabled,
              'text-base-content/30': disabled,
            }
          ]"
        >
          {{ text }}
        </span>
      </div>
    </div>
    <Teleport to="body">
      <transition name="fade">
        <div
          v-if="isHovered && hasTooltip"
          ref="tooltipRef"
          class="fixed z-1000 px-2 py-1 text-xs leading-tight max-w-xs whitespace-normal text-center rounded-box bg-base-100 shadow-lg pointer-events-none"
          :style="tooltipStyle"
        >
          {{ tooltipText }}
        </div>
      </transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onBeforeUnmount, nextTick, type CSSProperties } from 'vue';
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
  tooltipPlacement: {
    type: String,
    default: 'bottom'
  }
});

const emit = defineEmits(['click']);
const TOOLTIP_GAP = 4;
const TOOLTIP_PADDING = 8;
const TOOLTIP_AUTO_HIDE_MS = 3000;

const tooltipText = computed(() => {
  if (!props.shortcut) return props.tooltip;
  return props.tooltip ? `${props.tooltip} (${props.shortcut})` : props.shortcut;
});
const hasTooltip = computed(() => Boolean(tooltipText.value && config.settings.showToolTip));

const isHovered = ref(false);
const triggerRef = ref<HTMLElement | null>(null);
const tooltipRef = ref<HTMLElement | null>(null);
const tooltipStyle = ref<CSSProperties>({
  left: '0px',
  top: '0px',
});
let tooltipTimer: ReturnType<typeof setTimeout> | null = null;
let removeTooltipPositionListeners: (() => void) | null = null;

const clearTooltipTimer = () => {
  if (tooltipTimer) {
    clearTimeout(tooltipTimer);
    tooltipTimer = null;
  }
};

const removePositionListeners = () => {
  if (removeTooltipPositionListeners) {
    removeTooltipPositionListeners();
    removeTooltipPositionListeners = null;
  }
};

const updateTooltipPosition = async () => {
  await nextTick();

  if (!triggerRef.value || !tooltipRef.value) return;

  const triggerRect = triggerRef.value.getBoundingClientRect();
  const tooltipRect = tooltipRef.value.getBoundingClientRect();

  let left = triggerRect.left + (triggerRect.width - tooltipRect.width) / 2;
  let top = triggerRect.bottom + TOOLTIP_GAP;

  if (props.tooltipPlacement === 'right') {
    left = triggerRect.right + TOOLTIP_GAP;
    top = triggerRect.top + (triggerRect.height - tooltipRect.height) / 2;
    if (left + tooltipRect.width > window.innerWidth - TOOLTIP_PADDING) {
      left = triggerRect.left - tooltipRect.width - TOOLTIP_GAP;
    }
  } else if (top + tooltipRect.height > window.innerHeight - TOOLTIP_PADDING) {
    top = triggerRect.top - tooltipRect.height - TOOLTIP_GAP;
  }

  left = Math.max(TOOLTIP_PADDING, Math.min(left, window.innerWidth - tooltipRect.width - TOOLTIP_PADDING));
  top = Math.max(TOOLTIP_PADDING, Math.min(top, window.innerHeight - tooltipRect.height - TOOLTIP_PADDING));

  tooltipStyle.value = {
    left: `${Math.round(left)}px`,
    top: `${Math.round(top)}px`,
  };
};

const attachPositionListeners = () => {
  if (removeTooltipPositionListeners) return;

  const handler = () => {
    void updateTooltipPosition();
  };

  window.addEventListener('resize', handler);
  window.addEventListener('scroll', handler, true);
  removeTooltipPositionListeners = () => {
    window.removeEventListener('resize', handler);
    window.removeEventListener('scroll', handler, true);
  };
};

const showTooltip = () => {
  if (!hasTooltip.value) return;

  clearTooltipTimer();
  isHovered.value = true;
  attachPositionListeners();
  void updateTooltipPosition();

  tooltipTimer = setTimeout(() => {
    isHovered.value = false;
    removePositionListeners();
    tooltipTimer = null;
  }, TOOLTIP_AUTO_HIDE_MS);
};

const hideTooltip = () => {
  clearTooltipTimer();
  isHovered.value = false;
  removePositionListeners();
};

const handleClick = (event: Event) => {
  if (props.disabled) return;
  hideTooltip();
  emit('click', event);
};

onBeforeUnmount(() => {
  clearTooltipTimer();
  removePositionListeners();
});
</script>
