<template>

  <dialog id="imageEditorDialog" class="modal">
    <div class="w-[800px] p-4 flex flex-col gap-4 text-base-content/70 bg-base-100 border border-base-content/30 rounded-box">
      
      <!-- title bar -->
      <div class="flex items-center justify-between text-wrap break-all">
        {{ $t('msgbox.image_editor.title') }} - {{ props.fileInfo.name }}
        <TButton
          :icon="IconClose"
          :buttonSize="'small'"
          @click="clickCancel"
        />
      </div>

      <!-- content -->
      <div class="flex-grow flex gap-4">
        <div class="flex-1">
          <!-- image -->
          <div class="w-full bg-base-200 rounded-box">
            <img :src="imageSrc" class="p-2 w-full h-[400px] object-contain" :style="imageStyle" draggable="false" />
          </div>

          <!-- crop controls -->
          <div class="pt-2 flex justify-between">
            <!-- crop shape controls -->
            <div :class="['flex border rounded-box', isCropping ? 'border-primary' : 'border-base-content/30']">
              <TButton
                :icon="!isCropping ? IconCrop : IconClose"
                :tooltip="!isCropping ? $t('msgbox.image_editor.crop') : $t('msgbox.image_editor.cancel_crop')"
                @click="!isCropping ? clickCrop() : isCropping = false" 
              />
              <div v-if="isCropping" class="flex items-center gap-2 pr-2">
                <TButton
                  :icon="IconCropLandscape"
                  :iconStyle="{ transform: `rotate(${isPortrait ? 90 : 0}deg)`, transition: 'transform 0.3s ease-in-out' }" 
                  :tooltip="isPortrait ? $t('msgbox.image_editor.crop_shape_portrait') : $t('msgbox.image_editor.crop_shape_landscape')"
                  @click="toggleCropShape" 
                />
                <select v-model="config.imageEditor.cropShape" class="select select-bordered w-full">
                  <option v-for="option in cropShapeOptions" :value="option.value" :key="option.value">{{ option.label }}</option>
                </select>
                <TButton
                  :icon="IconOk"
                  :tooltip="$t('msgbox.image_editor.crop')"
                  @click="isCropping = false"
                />
              </div>
            </div>

            <!-- rotate and flip controls -->
            <div class="flex gap-2">
              <TButton
                :icon="IconRotateLeft"
                :disabled="isCropping"
                :tooltip="$t('msgbox.image_editor.rotate_left')"
                @click="clickRotate(-90)" 
              />
              <TButton
                :icon="IconRotateRight"
                :disabled="isCropping"
                :tooltip="$t('msgbox.image_editor.rotate_right')"
                @click="clickRotate(90)" 
              />
              <TButton
                :icon="IconFlipHorizontal"
                :disabled="isCropping"
                :tooltip="$t('msgbox.image_editor.flip_horizontal')"
                @click="clickFlipX" 
              />
              <TButton
                :icon="IconFlipVertical"
                :disabled="isCropping"
                :tooltip="$t('msgbox.image_editor.flip_vertical')"
                @click="clickFlipY" 
              />
            </div>
          </div>
        </div>

        <!-- edit controls -->
        <div class="w-48 flex flex-col gap-4">
          <!-- Original dimension -->
          <!-- <div>
            <h3 class="mb-2 ">{{ $t('msgbox.image_editor.original_dimension') }}</h3>
            <span class="p-2 text-sm">{{ fileInfo.width }}x{{ fileInfo.height }} ({{ formatFileSize(fileInfo.size) }})</span>
          </div> -->

          <!-- Resize -->
          <div>
            <h3>{{ $t('msgbox.image_editor.resize') }}</h3>
            <table class="w-full text-sm border-separate border-spacing-2">
              <tbody>
                <tr>
                  <td class="w-1/2">{{ $t('msgbox.image_editor.width') }}</td>
                  <td>
                    <input type="number" :placeholder="$t('msgbox.image_editor.width')"  class="input input-bordered w-full"
                      v-model="resizeWidth" :disabled="isCropping"
                    />
                  </td>
                </tr>
                <tr>
                  <td>{{ $t('msgbox.image_editor.height') }}</td>
                  <td>
                    <input type="number" :placeholder="$t('msgbox.image_editor.height')"  class="input input-bordered w-full"
                      v-model="resizeHeight" :disabled="isCropping"
                    />
                  </td>
                </tr>
                <tr>
                  <td>{{ $t('msgbox.image_editor.percentage') }}</td>
                  <td class="flex items-center">
                    <input type="number" :placeholder="$t('msgbox.image_editor.percentage')"  class="input input-bordered w-full"
                      v-model="resizePercentage" :disabled="isCropping"
                    />
                    <span class="pl-1 text-xs text-base-content/30">%</span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- Format -->
          <div>
            <h3 class="mb-2">{{ $t('msgbox.image_editor.save_as') }}</h3>
            <input type="text" :placeholder="$t('msgbox.image_editor.save_as_placeholder')" v-model="saveName" class="input input-bordered w-full px-2" :disabled="isCropping" />

            <table class="w-full text-sm border-separate border-spacing-2">
              <tbody>
                <tr>
                  <td>{{ $t('msgbox.image_editor.format') }}</td>
                  <td>
                    <select v-model="config.imageEditor.format" class="select select-bordered w-full" :disabled="isCropping">
                      <option v-for="option in fileFormatOptions" :value="option.value" :key="option.value">{{ option.label }}</option>
                    </select>
                  </td>
                </tr>
                <tr>
                  <td>{{ $t('msgbox.image_editor.quality') }}</td>
                  <td>
                    <select v-model="config.imageEditor.quality" class="select select-bordered w-full" :disabled="isCropping">
                      <option v-for="option in fileQualityOptions" :value="option.value" :key="option.value">{{ option.label }}</option>
                    </select>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>

      <!-- dialog buttons -->
      <div class="flex justify-end gap-4">
        <button
          class="px-4 py-1 rounded-lg hover:bg-base-content/30 cursor-pointer" 
          @click="clickReset"
        >{{ $t('msgbox.image_editor.reset') }}</button>
        <button
          class="px-4 py-1 rounded-lg hover:bg-base-content/30 cursor-pointer" 
          @click="clickCancel"
        >{{ $t('msgbox.image_editor.cancel') }}</button>
        <button 
          :class="[
            'px-4 py-1',
            isCropping ? 'text-base-content/30 cursor-default' : 'hover:bg-primary cursor-pointer rounded-lg',
          ]" 
          @click="clickSave"
        >{{ $t('msgbox.image_editor.ok') }}</button>
      </div>
    </div>
  </dialog>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/core';
import { useUIStore } from '@/stores/uiStore';
import { useI18n } from 'vue-i18n';
import { config, extractFileName, getSelectOptions, getFileExtension } from '@/common/utils';
import { editImage } from '@/common/api';

import TButton from '@/components/TButton.vue';
// import Image from '@/components/Image.vue';

import { 
  IconClose, 
  IconCrop,
  IconCropLandscape,
  IconRotateLeft, 
  IconRotateRight, 
  IconFlipVertical, 
  IconFlipHorizontal,
  IconOk,
} from '@/common/icons';

const props = defineProps({
  fileInfo: {
    type: Object,
    required: true,
  },
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[config.language]);

const uiStore = useUIStore();
const emit = defineEmits(['ok', 'cancel']);

const imageSrc = ref('');

// crop
const cropData = ref({
  x: 0,
  y: 0,
  width: 0,
  height: 0,
  rotate: 0,
});

// crop shape
const isCropping = ref(false);
const isPortrait = ref(false);

const cropShapeOptions = computed(() => {
  if(isPortrait.value) {
    return [
      { value: '0', label: localeMsg.value.msgbox.image_editor.crop_shape_custom },
      { value: '1', label: '1:1'},
      { value: '2', label: '1:2'},
      { value: '3', label: '2:3'},
      { value: '4', label: '3:4'},
      { value: '5', label: '9:16'},
    ];
  } else {
    return [
      { value: '0', label: localeMsg.value.msgbox.image_editor.crop_shape_custom },
      { value: '1', label: '1:1'},
      { value: '2', label: '2:1'},
      { value: '3', label: '3:2'},
      { value: '4', label: '4:3'},
      { value: '5', label: '16:9'},
    ];
  }
});

// rotate and flip
const rotation = ref(0);
const isFlippedX = ref(false);
const isFlippedY = ref(false);

// resize
const resizeWidth = ref(null);
const resizeHeight = ref(null);
const resizePercentage = ref(100);

// save as
const saveName = ref('');

const fileFormatOptions = computed(() => {
  return getSelectOptions(localeMsg.value.msgbox.image_editor.format_options);
});
const fileQualityOptions = computed(() => {
  return getSelectOptions(localeMsg.value.msgbox.image_editor.quality_options);
});

const imageStyle = computed(() => ({
  transform: `rotate(${rotation.value}deg) scaleX(${isFlippedX.value ? -1 : 1}) scaleY(${isFlippedY.value ? -1 : 1})`,
  transition: 'transform 0.3s ease',
}));

onMounted(async () => {
  const dialog = document.getElementById('imageEditorDialog') as HTMLDialogElement | null;
  dialog?.showModal();
  
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('ImageEditor');

  imageSrc.value = convertFileSrc(props.fileInfo.file_path);
  clickReset(); // init
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.popInputHandler();
});

function handleKeyDown(event: KeyboardEvent) {
  if (!uiStore.isInputActive('ImageEditor')) return;

  const { key } = event;

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

const clickCrop = () => {
  isCropping.value = !isCropping.value;
  console.log('clickCrop', isCropping.value);
};

const toggleCropShape = () => {
  isPortrait.value = !isPortrait.value;
};

const clickRotate = (degree: number) => {
  rotation.value += degree;
};

const clickFlipX = () => {
  isFlippedX.value = !isFlippedX.value;
};

const clickFlipY = () => {
  isFlippedY.value = !isFlippedY.value;
};

const clickReset = () => {
  // crop init
  isCropping.value = false;
  cropData.value = {
    x: 0,
    y: 0,
    width: props.fileInfo.width,
    height: props.fileInfo.height,
    rotate: 0,
  };
  isPortrait.value = props.fileInfo.width < props.fileInfo.height;

  // rotate and flip init
  rotation.value = 0;
  isFlippedX.value = false;
  isFlippedY.value = false;

  // resize init
  resizeWidth.value = props.fileInfo.width;
  resizeHeight.value = props.fileInfo.height;
  resizePercentage.value = 100;

  // save as init
  saveName.value = extractFileName(props.fileInfo.name).name;
  const fileExt = getFileExtension(props.fileInfo.name).toLowerCase();
  switch (fileExt) {
    case 'jpg':
    case 'jpeg':
      config.imageEditor.format = 0;
      break;
    case 'png':
      config.imageEditor.format = 1;
      break;
    case 'webp':
      config.imageEditor.format = 2;
      break;
    default:
      config.imageEditor.format = 0;
      break;
  }
};

const clickCancel = () => {
  emit('cancel');
};

const clickSave = async () => {
  if (isCropping.value) {
    return;
  }

  // const editParams = {
  //   filePath: props.filePath,
  //   crop: {
  //     x: cropData.x,
  //     y: cropData.y,
  //     width: cropData.width,
  //     height: cropData.height,
  //     rotate: cropData.rotate,
  //   },
  //   resize: {
  //     width: resizeWidth.value,
  //     height: resizeHeight.value,
  //   },
  //   flipHorizontal: isFlippedX.value,
  //   flipVertical: isFlippedY.value,
  //   outputFormat: outputFormat.value,
  // };

  // await editImage(editParams);
  emit('ok');
};
</script>

<style scoped>
</style>
