<template>

  <transition name="fade">
    <div v-if="isVisible" class="m-10 fixed inset-0 flex items-center justify-center pointer-events-none z-50">
      <div class="px-4 py-2 text-base-content/70 bg-success-content rounded-box">
        {{ message }}
      </div>
    </div>
  </transition>

</template>

<script setup lang="ts">
import { ref } from 'vue';

const isVisible = ref(false);
const message = ref('');

// Function to show the tooltip
const showTip = (msg: string) => {
  message.value = msg; // Set the message
  isVisible.value = true; // Show the tooltip

  // Auto-hide after 1 seconds
  setTimeout(() => {
    isVisible.value = false;
  }, 1000);
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