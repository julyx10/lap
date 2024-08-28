<template>
  <div v-if="visible" class="fixed inset-0 flex items-center justify-center bg-gray-800 bg-opacity-50 z-50">
    <div class="bg-gray-200 rounded-lg shadow-lg w-96 p-6">
      <h2 class="text-xl font-semibold mb-4">{{ title }}</h2>
      <p class="text-gray-700 mb-6">{{ message }}</p>
      <div class="flex justify-end space-x-2">
        <button v-if="showCancel" @click="handleCancel" class="px-4 py-2 bg-gray-300 text-gray-700 rounded">
          {{ cancelText }}
        </button>
        <button @click="handleConfirm" class="px-4 py-2 bg-blue-600 text-white rounded">
          {{ confirmText }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, defineEmits } from 'vue';

const props = defineProps({
  title: { type: String, default: 'Message' },
  message: { type: String, required: true },
  confirmText: { type: String, default: 'OK' },
  cancelText: { type: String, default: 'Cancel' },
  showCancel: { type: Boolean, default: true },
  visible: { type: Boolean, default: false }
});

const emit = defineEmits(['confirm', 'cancel', 'close']);

const handleConfirm = () => {
  emit('confirm');
  emit('close');
};

const handleCancel = () => {
  emit('cancel');
  emit('close');
};
</script>

<style scoped>
/* Add your styles here if needed */
</style>
