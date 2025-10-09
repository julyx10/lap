<template>

  <dialog id="imageEditorDialog" class="modal">
    <div class="w-4/5 p-4 text-base-content/70 bg-base-100 border border-base-content/30 rounded-box">
      
      <!-- title bar -->
      <div class="text-lg mb-4 flex items-center justify-between">
        {{ $t('msgbox.image_editor.title') }}
        <TButton
          :icon="IconClose"
          :buttonSize="'small'"
          @click="clickCancel"
        />
      </div>

      <div class="mt-2 flex flex-row gap-4">
        <!-- Image Cropper -->
        <div class="w-full h-full">
          <div class="img-container bg-base-300 h-[60vh]">
            <img ref="image" :src="imageSrc" @load="initializeCropper" alt="Image to edit" class="block max-w-full"/>
          </div>
        </div>

        <!-- Controls -->
        <div class="w-full md:w-64 flex-shrink-0 flex flex-col gap-4">
          <div>
            <h3 class="font-semibold mb-2">{{ $t('msgbox.image_editor.transform') }}</h3>
            <div class="">
              <TButton
                :icon="IconRotateLeft"
                :tooltip="$t('msgbox.image_editor.rotate_left')"
                @click="clickRotate(-90)" 
              />
              <TButton
                :icon="IconRotateRight"
                :tooltip="$t('msgbox.image_editor.rotate_right')"
                @click="clickRotate(90)" 
              />
              <TButton
                :icon="IconFlipVertical"
                :tooltip="$t('msgbox.image_editor.flip_vertical')"
                @click="clickFlipY" 
              />
              <TButton
                :icon="IconFlipHorizontal"
                :tooltip="$t('msgbox.image_editor.flip_horizontal')"
                @click="clickFlipX" 
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
          @click="clickCancel"
        >{{ $t('msgbox.image_editor.cancel') }}</button>
        
        <button 
          class="px-4 py-1 rounded-lg hover:bg-base-content/30 cursor-pointer" 
          @click="clickSave"
        >{{ $t('msgbox.image_editor.ok') }}</button>
      </div>
    </div>
  </dialog>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import Cropper from 'cropperjs';
import { useUIStore } from '@/stores/uiStore';
import { convertFileSrc } from '@tauri-apps/api/core';
import { editImage } from '@/common/api';
import { listen } from '@tauri-apps/api/event';
import TButton from '@/components/TButton.vue';

import { 
  IconClose, 
  IconRotateLeft, 
  IconRotateRight, 
  IconFlipHorizontal, 
  IconFlipVertical 
} from '@/common/icons';

const props = defineProps({
  filePath: {
    type: String,
    required: true,
  },
});

const emit = defineEmits(['ok', 'cancel']);

const uiStore = useUIStore();

const image = ref(null);
const imageSrc = ref('');
const cropper = ref(null);
const isFlippedX = ref(false);
const isFlippedY = ref(false);
const resizeWidth = ref(null);
const resizeHeight = ref(null);
const outputFormat = ref('jpeg');

let cropperInstance: Cropper | null = null;
let unlistenKeydown: () => void;

const initializeCropper = () => {
  if (image.value && !cropperInstance) { // prevent re-initialization
    cropperInstance = new Cropper(image.value, {
      viewMode: 2,
      dragMode: 'move',
      autoCropArea: 0.8,
      restore: false,
      guides: false,
      center: false,
      highlight: false,
      cropBoxMovable: true,
      cropBoxResizable: true,
      toggleDragModeOnDblclick: false,
    } as any);
    cropper.value = cropperInstance;
  }
};

// const destroyCropper = () => {
//   if (cropperInstance) {
//     cropperInstance.destroy(); // In v2, this might not exist, but let's keep it for now as the error was on .rotate
//     cropperInstance = null;
//     cropper.value = null;
//   }
// };

onMounted(async () => {
  const dialog = document.getElementById('imageEditorDialog') as HTMLDialogElement | null;
  dialog?.showModal();

  unlistenKeydown = await listen('global-keydown', handleKeyDown);
  uiStore.pushInputHandler('ImageEditor');

  imageSrc.value = convertFileSrc(props.filePath);
});

onUnmounted(() => {
  if (unlistenKeydown) {
    unlistenKeydown();
  }
  uiStore.popInputHandler();
  // destroyCropper();
});

function handleKeyDown(event: KeyboardEvent) {
  if (!uiStore.isInputActive('ImageEditor')) return;

  const { key } = event.payload;

  switch (key) {
    case 'Enter':
      clickSave();
      break;
    case 'Escape':
      clickCancel();
      break;
    default:
      break;
  }
}

const clickRotate = (degree: number) => {
  if (cropper.value) {
    cropper.value.rotate(degree);
  }
};

const clickFlipX = () => {
  if (cropper.value) {
    const currentScaleX = cropper.value.getData().scaleX || 1;
    cropper.value.scaleX(-currentScaleX);
    isFlippedX.value = !isFlippedX.value;
  }
};

const clickFlipY = () => {
  if (cropper.value) {
    const currentScaleY = cropper.value.getData().scaleY || 1;
    cropper.value.scaleY(-currentScaleY);
    isFlippedY.value = !isFlippedY.value;
  }
};

const clickSave = async () => {
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

  try {
    await editImage(editParams);
    emit('ok');
  } catch (error) {
    console.error('Failed to edit image:', error);
    // TODO: show an error message to the user
  }
};

const clickCancel = () => {
  emit('cancel');
};

</script>

<style>
/* .img-container > img {
  visibility: hidden;
} */
</style>
