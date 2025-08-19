<template>
  <div class="px-2 w-full rounded-lg h-0.5">
    <div
      :class="[
        'h-full rounded-lg bg-primary', 
        hasReached ? 'transition-all duration-300 ease-out' : '',
        fadeClass
      ]"
      :style="{ width: progressWidth }"
    ></div>
  </div>
</template>


<script setup>

import { ref, watch, computed } from 'vue';

const props = defineProps({
  percent: {
    type: Number,
    required: true,
  },
});

// Track if the progress has reached 100%
const hasReached = ref(false);

// Watch the percent prop to set or reset the hasReached flag
watch(() => props.percent, (newVal) => {
  if (newVal === 100) {
    hasReached.value = true; // Set flag when percent reaches 100%
  } else if (newVal === 0) {
    hasReached.value = false; // Reset flag when percent returns to 0
  }
});

// Computed property to calculate progress width
const progressWidth = computed(() => {
  // If percent is less than 1, set it to 1
  const adjustedPercent = props.percent < 1 ? 1 : props.percent;
  return `${adjustedPercent}%`;
});

// Conditional class for fading when progress reaches 100%
const fadeClass = computed(() =>
  hasReached.value ? 'transition-opacity opacity-0 duration-1000' : 'opacity-100'
);

</script>
