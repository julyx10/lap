<template>
  <div class="flex flex-col items-start justify-center" :style="{width: slider_width + 'px'}">
    <label v-if="label.length > 0" :for="id" class="mb-2">
      {{ label }}: <span>{{ sliderValue }}</span>
    </label>
    <input
      type="range"
      class="range range-xs [--range-fill:0]"
      v-model="sliderValue"
      :id="id"
      :min="min"
      :max="max"
      :step="step"
      @input="updateValue"
    />
  </div>
</template>

<script setup lang="js">
import { ref, watch } from 'vue';

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
    default: '',
  },
  id: {
    type: String,
    default: 'slider-input',
  },
  slider_width: {
    type: Number,
    default: 160,
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
