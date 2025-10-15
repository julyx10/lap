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
        <div class="flex-1 overflow-hidden">
          <!-- image container -->
          <div ref="imageContainer" class="relative w-full h-[400px] flex justify-center items-center bg-base-200 cursor-pointer rounded-box overflow-hidden">
            <img ref="image" :src="imageSrc" :style="imageStyle" draggable="false" />

            <!-- crop box -->
            <div v-if="isCropping" ref="cropBox" class="crop-box" :style="cropBoxStyle" @mousedown="startDrag('move', $event)">
              <div v-if="isDragging" class="crop-dimensions-display">
                {{ croppedWidth }} x {{ croppedHeight }}
              </div>
              <div v-if="isDragging" class="grid-lines">
                <div class="grid-line-h grid-line-h-1"></div>
                <div class="grid-line-h grid-line-h-2"></div>
                <div class="grid-line-v grid-line-v-1"></div>
                <div class="grid-line-v grid-line-v-2"></div>
              </div>
              <div class="drag-handle top-left" @mousedown.stop="startDrag('top-left', $event)"></div>
              <div class="drag-handle top" @mousedown.stop="startDrag('top', $event)"></div>
              <div class="drag-handle top-right" @mousedown.stop="startDrag('top-right', $event)"></div>
              <div class="drag-handle left" @mousedown.stop="startDrag('left', $event)"></div>
              <div class="drag-handle right" @mousedown.stop="startDrag('right', $event)"></div>
              <div class="drag-handle bottom-left" @mousedown.stop="startDrag('bottom-left', $event)"></div>
              <div class="drag-handle bottom" @mousedown.stop="startDrag('bottom', $event)"></div>
              <div class="drag-handle bottom-right" @mousedown.stop="startDrag('bottom-right', $event)"></div>
            </div>
          </div>

          <!-- crop controls -->
          <div class="pt-2 flex justify-between">
            <!-- crop shape controls -->
            <div :class="['flex border rounded-box', isCropping ? 'border-primary' : 'border-base-content/30']">
              <TButton
                :icon="!isCropping ? IconCrop : IconClose"
                :tooltip="!isCropping ? $t('msgbox.image_editor.crop') : $t('msgbox.image_editor.cancel_crop')"
                @click="toggleCrop" 
              />
              <div v-if="isCropping" class="flex items-center gap-2 pr-2">
                <TButton
                  :icon="IconCropLandscape"
                  :iconStyle="{ transform: `rotate(${isPortrait ? 90 : 0}deg)`, transition: 'transform 0.3s ease-in-out' }" 
                  :tooltip="isPortrait ? $t('msgbox.image_editor.crop_shape_portrait') : $t('msgbox.image_editor.crop_shape_landscape')"
                  @click="togglePortraitAndLandscape" 
                />
                <select v-model="config.imageEditor.cropShape" class="select select-bordered w-full" @change="onChangeCropShape">
                  <option v-for="option in cropShapeOptions" :value="option.value" :key="option.value">{{ option.label }}</option>
                </select>
                <TButton
                  :icon="IconOk"
                  :tooltip="$t('msgbox.image_editor.crop')"
                  @click="doCrop"
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

      <!-- debug -->
      <!-- <div class="text-xs text-base-content/30">
        cropBoxData: {{ cropBoxData }}
      </div> -->

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
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/core';
import { useUIStore } from '@/stores/uiStore';
import { useI18n } from 'vue-i18n';
import { config, extractFileName, getSelectOptions, getFileExtension } from '@/common/utils';
import { editImage } from '@/common/api';

import TButton from '@/components/TButton.vue';

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

const imageContainer = ref<HTMLElement | null>(null);
const enableTransition = ref(false);
const imageSrc = ref('');

// crop shape
const isCropping = ref(false);
const isPortrait = ref(false);

const cropShapeOptions = computed(() => {
  if(isPortrait.value) {
    return [
      { value: '0', label: localeMsg.value.msgbox.image_editor.crop_shape_custom },
      { value: '1', label: '1:1'},   // 1.0
      { value: '2', label: '3:4'},   // 0.75
      { value: '3', label: '2:3'},   // 0.67
      { value: '4', label: '10:16'}, // 0.63
      { value: '5', label: '9:16'},  // 0.56
      { value: '6', label: '1:2'},   // 0.5
    ];
  } else {
    return [
      { value: '0', label: localeMsg.value.msgbox.image_editor.crop_shape_custom },
      { value: '1', label: '1:1'},    // 1.0
      { value: '2', label: '4:3'},    // 1.33
      { value: '3', label: '3:2'},    // 1.5
      { value: '4', label: '16:10'},  // 1.6
      { value: '5', label: '16:9'},   // 1.78
      { value: '6', label: '2:1'},    // 2.0
    ];
  }
});

// rotate and flip
const rotation = ref(0);
const isFlippedX = ref(false);
const isFlippedY = ref(false);

// cropped image
const croppedWidth = ref(0);
const croppedHeight = ref(0);

// resized image
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

const imageStyle = computed(() => {
  if (!imageContainer.value || !props.fileInfo.width || !props.fileInfo.height) {
    return {};
  }

  const containerWidth = imageContainer.value.clientWidth - 8;
  const containerHeight = imageContainer.value.clientHeight - 8;
  const imageWidth = props.fileInfo.width;
  const imageHeight = props.fileInfo.height;

  const isRotated = rotation.value % 180 !== 0;
  const rotatedWidth = isRotated ? imageHeight : imageWidth;
  const rotatedHeight = isRotated ? imageWidth : imageHeight;

  const scale = Math.min(containerWidth / rotatedWidth, containerHeight / rotatedHeight);

  return {
    display: 'block',
    minWidth: `${imageWidth}px`,
    minHeight: `${imageHeight}px`,
    transform: `rotate(${rotation.value}deg) scaleX(${isFlippedX.value ? -1 : 1}) scaleY(${isFlippedY.value ? -1 : 1}) scale(${scale})`,
    transition: enableTransition.value ? 'transform 0.3s ease' : 'none',
  };
});

const image = ref<HTMLImageElement | null>(null);
const imageRect = ref<DOMRect | null>(null);
const containerRect = ref<DOMRect | null>(null);

const cropBox = ref<HTMLElement | null>(null);
const cropBoxData = ref({ top: 0, left: 0, width: 0, height: 0 });
const isDragging = ref(false);
const dragHandle = ref('');
const dragStartX = ref(0);
const dragStartY = ref(0);

const cropBoxStyle = computed(() => ({
  top: `${cropBoxData.value.top}px`,
  left: `${cropBoxData.value.left}px`,
  width: `${cropBoxData.value.width}px`,
  height: `${cropBoxData.value.height}px`,
}));

onMounted(async () => {
  const dialog = document.getElementById('imageEditorDialog') as HTMLDialogElement | null;
  dialog?.showModal();
  
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('ImageEditor');

  imageSrc.value = convertFileSrc(props.fileInfo.file_path);
  clickReset(); // init

  nextTick(() => {
    enableTransition.value = true;
  });
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.popInputHandler();
});

watch(cropBoxData, (newData) => {
  if (imageRect.value && props.fileInfo.width && props.fileInfo.height) {
    croppedWidth.value = Math.round(props.fileInfo.width * newData.width / imageRect.value.width);
    croppedHeight.value = Math.round(props.fileInfo.height * newData.height / imageRect.value.height);
  }
}, { deep: true });

function handleKeyDown(event: KeyboardEvent) {
  if (!uiStore.isInputActive('ImageEditor')) return;

  const { key } = event;

  switch (key) {
    case 'Enter':
      clickSave();
      event.preventDefault();
      event.stopPropagation();
      break;
    case 'Escape':
      if(isCropping.value) {
        isCropping.value = false;
        event.preventDefault();
        event.stopPropagation();
        break;
      }
      clickCancel();
      event.preventDefault();
      event.stopPropagation();
      break;
    default:
      break;
  }
}

const toggleCrop = () => {
  isCropping.value = !isCropping.value;
  if (isCropping.value) {
    if (image.value && imageContainer.value) {
        imageRect.value = image.value.getBoundingClientRect();
        containerRect.value = imageContainer.value.getBoundingClientRect();
        updateCropBoxData();
    }
  }
};

const togglePortraitAndLandscape = () => {
  isPortrait.value = !isPortrait.value;
  updateCropBoxData();
};

const onChangeCropShape = () => {
  updateCropBoxData();
};

const doCrop = () => {
  isCropping.value = false;

  resizeWidth.value = croppedWidth.value;
  resizeHeight.value = croppedHeight.value;

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

// update crop box data by crop shape
const updateCropBoxData = () => {
  const selectedShape = cropShapeOptions.value.find(option => option.value === config.imageEditor.cropShape && option.value !== '0');
  if (selectedShape && selectedShape.label) {
    const parts = selectedShape.label.split(':');
    const aspectRatio = parseInt(parts[0]) / parseInt(parts[1]);

    let newWidth, newHeight;
    if (imageRect.value.width / imageRect.value.height > aspectRatio) {
      newHeight = imageRect.value.height;
      newWidth = newHeight * aspectRatio;
    } else {
      newWidth = imageRect.value.width;
      newHeight = newWidth / aspectRatio;
    }

    const imageLeft = imageRect.value.left - containerRect.value.left;
    const imageTop = imageRect.value.top - containerRect.value.top;

    cropBoxData.value = {
      left: imageLeft + (imageRect.value.width - newWidth) / 2,
      top: imageTop + (imageRect.value.height - newHeight) / 2,
      width: newWidth,
      height: newHeight,
    };
  } else {
    cropBoxData.value = {
      left: imageRect.value.left - containerRect.value.left,
      top: imageRect.value.top - containerRect.value.top,
      width: imageRect.value.width,
      height: imageRect.value.height,
    };
  }
}

const startDrag = (handle: string, event: MouseEvent) => {
  isDragging.value = true;
  dragHandle.value = handle;
  dragStartX.value = event.clientX;
  dragStartY.value = event.clientY;

  const initialCropBoxData = { ...cropBoxData.value };

  const doDrag = (e: MouseEvent) => {
    if (!isDragging.value) return;

    const dx = e.clientX - dragStartX.value;
    const dy = e.clientY - dragStartY.value;

    const imageLeft = imageRect.value.left - containerRect.value.left;
    const imageTop = imageRect.value.top - containerRect.value.top;
    const imageRight = imageLeft + imageRect.value.width;
    const imageBottom = imageTop + imageRect.value.height;

    // Use clamping for move, and strict validation for resize
    if (dragHandle.value === 'move') {
      let newLeft = initialCropBoxData.left + dx;
      let newTop = initialCropBoxData.top + dy;

      // Clamp position to boundaries
      if (newLeft < imageLeft) newLeft = imageLeft;
      if (newTop < imageTop) newTop = imageTop;
      if (newLeft + initialCropBoxData.width > imageRight) newLeft = imageRight - initialCropBoxData.width;
      if (newTop + initialCropBoxData.height > imageBottom) newTop = imageBottom - initialCropBoxData.height;

      cropBoxData.value.left = newLeft;
      cropBoxData.value.top = newTop;

    } else { // Resize logic
      let proposedBox = { ...initialCropBoxData };

      if (dragHandle.value.includes('right')) proposedBox.width += dx;
      if (dragHandle.value.includes('left')) {
        proposedBox.width -= dx;
        proposedBox.left += dx;
      }
      if (dragHandle.value.includes('bottom')) proposedBox.height += dy;
      if (dragHandle.value.includes('top')) {
        proposedBox.height -= dy;
        proposedBox.top += dy;
      }

      const shape = config.imageEditor.cropShape;
      if (shape !== '0') {
        const selectedShape = cropShapeOptions.value.find(o => o.value === shape);
        if (selectedShape && selectedShape.label) {
          const parts = selectedShape.label.split(':');
          const aspectRatio = parseInt(parts[0]) / parseInt(parts[1]);

          if (dragHandle.value.includes('left') || dragHandle.value.includes('right')) {
            proposedBox.height = proposedBox.width / aspectRatio;
          } else {
            proposedBox.width = proposedBox.height * aspectRatio;
          }
          if (dragHandle.value.includes('top')) {
            proposedBox.top = initialCropBoxData.top + (initialCropBoxData.height - proposedBox.height);
          }
          if (dragHandle.value.includes('left')) {
            proposedBox.left = initialCropBoxData.left + (initialCropBoxData.width - proposedBox.width);
          }
        }
      }

      // Strict validation for resize
      if (
        proposedBox.width >= 10 &&
        proposedBox.height >= 10 &&
        proposedBox.left >= imageLeft &&
        proposedBox.top >= imageTop &&
        proposedBox.left + proposedBox.width <= imageRight + 0.1 && // Add tolerance for float precision
        proposedBox.top + proposedBox.height <= imageBottom + 0.1 // Add tolerance for float precision
      ) {
        cropBoxData.value = proposedBox;
      }
    }
  };

  const stopDrag = () => {
    isDragging.value = false;
    window.removeEventListener('mousemove', doDrag);
    window.removeEventListener('mouseup', stopDrag);
  };

  window.addEventListener('mousemove', doDrag);
  window.addEventListener('mouseup', stopDrag);
};

const clickReset = () => {
  // crop init
  isCropping.value = false;
  isPortrait.value = props.fileInfo.width < props.fileInfo.height;
  croppedWidth.value = 0;
  croppedHeight.value = 0;

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
.crop-box {
  position: absolute;
  border: 1px solid #fff;
  box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.8);
  box-sizing: border-box;
}

.drag-handle {
  position: absolute;
  width: 10px;
  height: 10px;
  background-color: #fff;
  border: 1px solid #000;
  box-sizing: border-box;
}

.top-left {
  top: -5px;
  left: -5px;
  cursor: nwse-resize;
}

.top {
  top: -5px;
  left: 50%;
  transform: translateX(-50%);
  cursor: ns-resize;
}

.top-right {
  top: -5px;
  right: -5px;
  cursor: nesw-resize;
}

.left {
  top: 50%;
  left: -5px;
  transform: translateY(-50%);
  cursor: ew-resize;
}

.right {
  top: 50%;
  right: -5px;
  transform: translateY(-50%);
  cursor: ew-resize;
}

.bottom-left {
  bottom: -5px;
  left: -5px;
  cursor: nesw-resize;
}

.bottom {
  bottom: -5px;
  left: 50%;
  transform: translateX(-50%);
  cursor: ns-resize;
}

.bottom-right {
  bottom: -5px;
  right: -5px;
  cursor: nwse-resize;
}

.grid-line-h,
.grid-line-v {
  position: absolute;
  background-color: rgba(255, 255, 255, 0.2);
}

.grid-line-h {
  width: 100%;
  height: 1px;
  left: 0;
}

.grid-line-v {
  width: 1px;
  height: 100%;
  top: 0;
}

.grid-line-h-1 {
  top: 33.33%;
}

.grid-line-h-2 {
  top: 66.66%;
}

.grid-line-v-1 {
  left: 33.33%;
}

.grid-line-v-2 {
  left: 66.66%;
}

.crop-dimensions-display {
  position: absolute;
  bottom: 10px;
  left: 50%;
  transform: translateX(-50%);
  background-color: rgba(0, 0, 0, 0.5);
  color: #aaa;
  padding: 2px 8px;
  border-radius: 14px;
  font-size: 12px;
  white-space: nowrap;
}
</style>
