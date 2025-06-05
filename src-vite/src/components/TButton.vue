<template>
    <div 
      class="tooltip" 
      :class="tooltipClasses"
      :data-tip="config.showToolTip ? tooltip : ''"
    >
      <button
        class="btn btn-ghost btn-square rounded-lg border-0 focus:outline-none shadow-none flex flex-col"
        :class="[
          buttonClasses,
          {
            'btn-xs hover:bg-transparent': buttonSize === 'small',
            'btn-md hover:bg-base-100': buttonSize === 'medium',
            'btn-lg hover:bg-base-100': buttonSize === 'large',
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
          v-if="config.showButtonText && text.length > 0"
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
</template>

<script setup lang="ts">
import { config } from '@/common/utils';
import type { Component } from 'vue';

const props = defineProps({
  buttonSize: {
    type: String,
    default: 'large'         // 'small', 'medium'(default), 'large'
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
    type: String,
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

</script>