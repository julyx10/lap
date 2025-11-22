<template>

  <teleport to="body">
    <transition name="fade">
      <div v-if="isVisible" class="m-10 fixed inset-0 flex items-center justify-center pointer-events-none z-[1000]">
        <div :class="[
          'px-4 py-2 rounded-box bg-base-100/80 backdrop-blur-sm shadow-lg',
          isError ? 'text-error-content/70' : 'text-base-content/70'
        ]">
          {{ message }}
        </div>
      </div>
    </transition>
  </teleport>

</template>

<script setup lang="ts">
import { ref } from 'vue';

const isVisible = ref(false);
const isError = ref(false);
const message = ref('');

// Function to show the tooltip
const showTip = (msg: string, errorStatus: boolean = false) => {
  message.value = msg; // Set the message
  isError.value = errorStatus; // Set the error status
  isVisible.value = true; // Show the tooltip

  // Auto-hide after 1 seconds or 3 seconds for error
  setTimeout(() => {
    isVisible.value = false;
  }, isError.value ? 3000 : 1000);
};

defineExpose({ 
  showTip
});

</script>
