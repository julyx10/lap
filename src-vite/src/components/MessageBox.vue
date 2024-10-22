<template>

  <div v-if="visible" class="fixed inset-0 flex items-center justify-center bg-gray-900 bg-opacity-60 z-50 backdrop-blur-0">
    <div class="w-96 p-4 t-color-bg-light rounded-lg shadow-lg">
      <div class="mb-2 text-lg">
        {{ title }}
      </div>
      <div class="mb-6">
        {{ message }}
      </div>
      <div class="flex justify-end space-x-4">
        <button v-if="showCancel" class="px-4 py-1 rounded-full t-color-bg-light t-color-bg-hover t-icon-hover" @click="clickCancel">
          {{ cancelText }}
        </button>
        <button class="px-4 rounded-full text-red-500 hover:bg-red-800 t-icon-hover" @click="clickConfirm">
          {{ confirmText }}
        </button>
      </div>
    </div>
  </div>

</template>

<script setup lang="ts">

const props = defineProps({
  title: { 
    type: String, 
    default: 'Message' 
  },
  message: { 
    type: String, 
    required: true 
  },
  confirmText: { 
    type: String, 
    default: 'OK' 
  },
  cancelText: { 
    type: String, 
    default: 'Cancel' 
  },
  showCancel: { 
    type: Boolean, 
    default: true 
  },
  visible: { 
    type: Boolean, 
    default: false 
  }
});

const emit = defineEmits(['confirm', 'cancel', 'close']);

const clickConfirm = () => {
  emit('confirm');
  emit('close');
};

const clickCancel = () => {
  emit('cancel');
  emit('close');
};
</script>

<style scoped>
/* Add your styles here if needed */
</style>
