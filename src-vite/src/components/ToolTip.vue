<template>

  <transition name="fade">
    <div v-if="isVisible" class="m-10 fixed inset-0 flex items-center justify-center pointer-events-none z-50">
      <div :class="[
        'px-4 py-2 text-base-content/70 rounded-box',
        isError ? 'bg-base-100/70' : 'bg-base-100/70'
      ]">
        {{ message }}
      </div>
    </div>
  </transition>

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

<style scoped>
/* fade transition */
.fade-enter-active {
  transition: opacity 0.2s ease; /* Adjust duration and easing as needed */
}
.fade-leave-active {
  transition: opacity 0.2s ease; /* Adjust duration and easing as needed */
}
.fade-enter-from, .fade-leave-to {
  opacity: 0; /* Initial and final opacity for fading in and out */
}
.fade-enter-to, .fade-leave-from {
  opacity: 1; /* Final opacity when fully visible */
}
</style>