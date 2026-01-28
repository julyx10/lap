<template>
  <div>
    <div 
      ref="buttonRef"
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
    </div>

    <!-- Teleported tooltip -->
    <teleport to="body">
      <transition name="fade">
        <div 
          v-if="isHovered && tooltip && config.settings.showToolTip"
          ref="tooltipRef"
          class="fixed z-[1000] px-2 py-1 text-xs whitespace-nowrap rounded-box bg-neutral text-neutral-content shadow-lg pointer-events-none"
          :style="tooltipStyle"
        >
          {{ tooltip }}
        </div>
      </transition>
    </teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue';
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
  tooltipClasses: {
    type: String,
    default: 'tooltip-bottom'
  }
});

const emit = defineEmits(['click']);

const buttonRef = ref<HTMLElement | null>(null);
const tooltipRef = ref<HTMLElement | null>(null);
const isHovered = ref(false);
const tooltipStyle = ref<Record<string, string>>({});

const showTooltip = async () => {
  if (!props.tooltip || !config.settings.showToolTip) return;
  
  isHovered.value = true;
  await nextTick();
  
  if (buttonRef.value && tooltipRef.value) {
    const rect = buttonRef.value.getBoundingClientRect();
    const tooltipRect = tooltipRef.value.getBoundingClientRect();
    const padding = 4;
    
    // Default: position below
    let top = rect.bottom + padding;
    let left = rect.left + (rect.width - tooltipRect.width) / 2;
    
    // Check if tooltip goes off right edge
    if (left + tooltipRect.width > window.innerWidth - padding) {
      left = window.innerWidth - tooltipRect.width - padding;
    }
    // Check if tooltip goes off left edge
    if (left < padding) {
      left = padding;
    }
    // Check if tooltip goes off bottom, place above instead
    if (top + tooltipRect.height > window.innerHeight - padding) {
      top = rect.top - tooltipRect.height - padding;
    }
    
    tooltipStyle.value = {
      top: `${top}px`,
      left: `${left}px`,
    };
  }
};

const hideTooltip = () => {
  isHovered.value = false;
};
</script>
