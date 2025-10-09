<template>
  <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
    <div class="bg-base-200 rounded-lg shadow-xl p-4 w-full max-w-4xl max-h-[90vh] flex flex-col">
      <h2 class="text-xl font-bold mb-4">{{ $t('msgbox.image_editor.title') }}</h2>
      
      <div class="flex-grow flex gap-4 overflow-hidden">
        <!-- Image Cropper -->
        <div class="flex-grow h-full">
          <div class="img-container h-full bg-base-300">
            <img ref="image" :src="imageSrc" alt="Image to edit" class="max-w-full max-h-full"/>
          </div>
        </div>

        <!-- Controls -->
        <div class="w-64 flex-shrink-0 flex flex-col gap-4">
          <div>
            <h3 class="font-semibold mb-2">{{ $t('msgbox.image_editor.transform') }}</h3>
            <div class="">
              <TButton
                :icon="IconRotateLeft"
                :tooltip="$t('msgbox.image_editor.rotate_left')"
                @click="rotate(-90)" 
              />
              <TButton
                :icon="IconRotateRight"
                :tooltip="$t('msgbox.image_editor.rotate_right')"
                @click="rotate(90)" 
              />
              <TButton
                :icon="IconFlipVertical"
                :tooltip="$t('msgbox.image_editor.flip_vertical')"
                @click="flipY" 
              />
              <TButton
                :icon="IconFlipHorizontal"
                :tooltip="$t('msgbox.image_editor.flip_horizontal')"
                @click="flipX" 
              />
            </div>
          </div>
          <div>
            <h3 class="font-semibold mb-2">{{ $t('msgbox.image_editor.resize') }}</h3>
            <div class="flex gap-2">
              <input type="number" :placeholder="$t('msgbox.image_editor.width')" v-model="resizeWidth" class="input input-bordered w-full" />
              <input type="number" :placeholder="$t('msgbox.image_editor.height')" v-model="resizeHeight" class="input input-bordered w-full" />
            </div>
          </div>
          <div>
            <h3 class="font-semibold mb-2">{{ $t('msgbox.image_editor.format') }}</h3>
            <select v-model="outputFormat" class="select select-bordered w-full">
              <option value="jpeg">{{ $t('msgbox.image_editor.jpeg') }}</option>
              <option value="png">{{ $t('msgbox.image_editor.png') }}</option>
              <option value="webp">{{ $t('msgbox.image_editor.webp') }}</option>
            </select>
          </div>
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="mt-6 flex justify-end gap-4">
        <button
          class="px-4 py-1 rounded-lg hover:bg-base-content/30 cursor-pointer" 
          @click="cancel"
        >{{ $t('msgbox.image_editor.cancel') }}</button>
        
        <button 
          class="px-4 py-1 rounded-lg hover:bg-base-content/30 cursor-pointer" 
          @click="save"
        >{{ $t('msgbox.image_editor.ok') }}</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue';
import Cropper from 'cropperjs';

import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import TButton from '@/components/TButton.vue';
import { IconRotateLeft, IconRotateRight, IconFlipHorizontal, IconFlipVertical } from '@/common/icons';

const props = defineProps({
  visible: Boolean,
  filePath: String,
});

const emit = defineEmits(['update:visible', 'image-edited']);

const image = ref(null);
const imageSrc = ref('');
const cropper = ref(null);
const isFlippedX = ref(false);
const isFlippedY = ref(false);
const resizeWidth = ref(null);
const resizeHeight = ref(null);
const outputFormat = ref('jpeg');

let cropperInstance = null;

watch(() => props.visible, (newVal) => {
  if (newVal) {
    imageSrc.value = convertFileSrc(props.filePath);
    setTimeout(initializeCropper, 100); // Wait for image to be in DOM
  } else {
    destroyCropper();
  }
});

const initializeCropper = () => {
  if (image.value) {
    cropperInstance = new Cropper(image.value, {
      viewMode: 1,
      dragMode: 'move',
      autoCropArea: 0.8,
      restore: false,
      guides: false,
      center: false,
      highlight: false,
      cropBoxMovable: true,
      cropBoxResizable: true,
      toggleDragModeOnDblclick: false,
    });
    cropper.value = cropperInstance;
  }
};

const destroyCropper = () => {
  if (cropperInstance) {
    // For Cropper.js v2, there is no 'destroy' method.
    // Cleanup is handled by removing the element from the DOM, which Vue does automatically.
    cropperInstance = null;
    cropper.value = null;
  }
};

const rotate = (degree) => {
  if (cropper.value) {
    cropper.value.rotate(degree);
  }
};

const flipX = () => {
  if (cropper.value) {
    const currentScaleX = cropper.value.getData().scaleX || 1;
    cropper.value.scaleX(-currentScaleX);
    isFlippedX.value = !isFlippedX.value;
  }
};

const flipY = () => {
  if (cropper.value) {
    const currentScaleY = cropper.value.getData().scaleY || 1;
    cropper.value.scaleY(-currentScaleY);
    isFlippedY.value = !isFlippedY.value;
  }
};

const cancel = () => {
  emit('update:visible', false);
};

const save = async () => {
  if (!cropper.value) return;

  const cropData = cropper.value.getData(true); // get rounded data

  const editParams = {
    filePath: props.filePath,
    crop: {
      x: cropData.x,
      y: cropData.y,
      width: cropData.width,
      height: cropData.height,
      rotate: cropData.rotate,
    },
    resize: {
      width: resizeWidth.value,
      height: resizeHeight.value,
    },
    flipHorizontal: isFlippedX.value,
    flipVertical: isFlippedY.value,
    outputFormat: outputFormat.value,
  };

  console.log("Saving with params:", editParams);
  try {
    await invoke('edit_image', { params: editParams });
    emit('image-edited');
    emit('update:visible', false);
  } catch (error) {
    console.error('Failed to edit image:', error);
    // Here you could show an error message to the user
  }
};

onUnmounted(() => {
  destroyCropper();
});
</script>

<style>
.img-container {
  display: flex;
  justify-content: center;
  align-items: center;
}
.cropper-view-box,
.cropper-face {
  border-radius: .5rem;
}
</style>
