<template>
  <div class="w-24 flex flex-col items-start justify-center">
    <label v-if="label.length > 0" :for="id" class="mb-2">
      {{ label }}: <span>{{ sliderValue }}</span>
    </label>
    <input
      :id="id"
      type="range"
      :min="min"
      :max="max"
      :step="step"
      v-model="sliderValue"
      @input="updateValue"
      class="w-full h-1 rounded-full t-color-border appearance-none cursor-pointer accent-gray-400"
    />
  </div>
</template>

<script setup lang="js">
import { ref, watch, defineEmits } from 'vue';

const props = defineProps({
  modelValue: {
    type: Number,
    required: true,
  },
  min: {
    type: Number,
    default: 0,
  },
  max: {
    type: Number,
    default: 100,
  },
  step: {
    type: Number,
    default: 1,
  },
  label: {
    type: String,
    default: 'Slider',
  },
  id: {
    type: String,
    default: 'slider-input',
  },
});

const emit = defineEmits(['update:modelValue']);

// Define slider value
const sliderValue = ref(props.modelValue);

// Watch for changes in the modelValue from the parent component
watch(() => props.modelValue, (newValue) => { 
  sliderValue.value = newValue; 
});

// Update slider value
const updateValue = () => {
  emit('update:modelValue', Number(sliderValue.value));
};
</script>

<style scoped>
/* input[type="range"]::-webkit-slider-thumb {
  appearance: none;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background-color: #3b82f6;
  cursor: pointer;
  transition: background-color 0.2s;
}

input[type="range"]::-moz-range-thumb {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background-color: #3b82f6;
  cursor: pointer;
} */
</style>
