<template>

  <dialog id="imageEditorDialog" class="modal">
    <div class="p-4 flex flex-col gap-2 text-base-content/70 bg-base-100 border border-base-content/30 rounded-box">
      
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
          <!-- image container -->
          <div ref="containerRef" class="relative w-[570px] h-[430px] bg-base-200 cursor-pointer rounded-lg overflow-hidden">
            <img ref="imageRef" :src="imageSrc" :style="imageStyle" draggable="false" @load="onImageLoad" />

            <!-- crop box -->
            <div v-if="cropStatus==1" class="crop-box-active" :style="cropBoxStyle" @mousedown="startDrag('move', $event)">
              <div v-if="isDragging" class="crop-dimensions-display">
                {{ crop.width }} x {{ crop.height }}
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
            <div v-if="cropStatus==2" class="crop-box-done" :style="cropBoxStyle"></div>
          </div>

          <!-- crop controls -->
          <div class="pt-2 flex justify-between">
            <!-- crop shape controls -->
            <div 
              :class="['flex border rounded-box transition-all ease-out duration-200', 
                cropStatus==0 ? 'border-transparent' : '',
                cropStatus==1 ? 'border-primary' : '',
                cropStatus==2 ? 'border-base-content/30' : '',
              ]"
            >
              <TButton
                :icon="cropStatus==0 ? IconCrop : (cropStatus==1 ? IconClose : IconRestore)"
                :tooltip="cropStatus==0 ? $t('msgbox.image_editor.crop') : (cropStatus==1 ? $t('msgbox.image_editor.cancel_crop') : $t('msgbox.image_editor.restore'))"
                @click="toggleCrop" 
              />
              <div v-if="cropStatus==1" class="flex items-center gap-2 pr-2">
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
                  :tooltip="$t('msgbox.image_editor.done_crop')"
                  @click="doCrop"
                />
              </div>
            </div>

            <!-- rotate and flip controls -->
            <div class="flex gap-2">
              <TButton
                :icon="IconRotateLeft"
                :disabled="cropStatus==1"
                :tooltip="$t('msgbox.image_editor.rotate_left')"
                @click="clickRotate(-90)" 
              />
              <TButton
                :icon="IconRotateRight"
                :disabled="cropStatus==1"
                :tooltip="$t('msgbox.image_editor.rotate_right')"
                @click="clickRotate(90)" 
              />
              <TButton
                :icon="IconFlipHorizontal"
                :disabled="cropStatus==1"
                :tooltip="$t('msgbox.image_editor.flip_horizontal')"
                @click="clickFlipX" 
              />
              <TButton
                :icon="IconFlipVertical"
                :disabled="cropStatus==1"
                :tooltip="$t('msgbox.image_editor.flip_vertical')"
                @click="clickFlipY" 
              />
            </div>
          </div>
        </div>

        <!-- edit controls -->
        <div class="w-48 flex flex-col gap-4">

          <!-- Resize -->
          <div>
            <h3>{{ $t('msgbox.image_editor.resize') }}</h3>
            <table class="w-full text-sm border-separate border-spacing-2">
              <tbody>
                <tr>
                  <td class="w-1/2">{{ $t('msgbox.image_editor.width') }}</td>
                  <td>
                    <input type="number" min="100" max="100000" :placeholder="$t('msgbox.image_editor.width')"  class="input input-bordered w-full"
                      v-model="resizeWidth" :disabled="cropStatus==1"
                    />
                  </td>
                </tr>
                <tr>
                  <td>{{ $t('msgbox.image_editor.height') }}</td>
                  <td>
                    <input type="number" min="100" max="100000" :placeholder="$t('msgbox.image_editor.height')"  class="input input-bordered w-full"
                      v-model="resizeHeight" :disabled="cropStatus==1"
                    />
                  </td>
                </tr>
                <tr>
                  <td>{{ $t('msgbox.image_editor.percentage') }}</td>
                  <td class="flex items-center">
                    <input type="number" min="1" max="100" :placeholder="$t('msgbox.image_editor.percentage')"  class="input input-bordered w-full"
                      v-model="resizePercentage" :disabled="cropStatus==1"
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
            <input type="text" :placeholder="$t('msgbox.image_editor.save_as_placeholder')" v-model="saveName" class="input input-bordered w-full px-2" :disabled="cropStatus==1" />

            <table class="w-full text-sm border-separate border-spacing-2">
              <tbody>
                <tr>
                  <td>{{ $t('msgbox.image_editor.format') }}</td>
                  <td>
                    <select v-model="config.imageEditor.format" class="select select-bordered w-full" :disabled="cropStatus==1">
                      <option v-for="option in fileFormatOptions" :value="option.value" :key="option.value">{{ option.label }}</option>
                    </select>
                  </td>
                </tr>
                <tr>
                  <td>{{ $t('msgbox.image_editor.quality') }}</td>
                  <td>
                    <select v-model="config.imageEditor.quality" class="select select-bordered w-full" :disabled="cropStatus==1">
                      <option v-for="option in fileQualityOptions" :value="option.value" :key="option.value">{{ option.label }}</option>
                    </select>
                  </td>
                </tr>
              </tbody>
            </table>

            <!-- debug -->
            <div class="text-xs text-base-content/30 flex flex-col gap-1 mt-2">
              <span>containerBounds: {{ containerBounds?.left.toFixed(0) }}, {{ containerBounds?.top.toFixed(0) }}, {{ containerBounds?.width.toFixed(0) }}, {{ containerBounds?.height.toFixed(0) }}</span>
              <span>cropBox:{{ cropBox.left.toFixed(0) }}, {{ cropBox.top.toFixed(0) }}, {{ cropBox.width.toFixed(0) }}, {{ cropBox.height.toFixed(0) }}</span> 
              <span>cropBoxOriginal:{{ cropBoxOriginal.left.toFixed(0) }}, {{ cropBoxOriginal.top.toFixed(0) }}, {{ cropBoxOriginal.width.toFixed(0) }}, {{ cropBoxOriginal.height.toFixed(0) }}</span> 
              <span>imageRect: {{ imageRect?.left.toFixed(0) }}, {{ imageRect?.top.toFixed(0) }}, {{ imageRect?.width.toFixed(0) }}, {{ imageRect?.height.toFixed(0) }}</span>
              <span>scale: {{ scale.toFixed(2) }}</span>
              <span>position: {{ position.left.toFixed(0) }}, {{ position.top.toFixed(0) }}</span>
              <span>crop: {{ crop.x.toFixed(0) }}, {{ crop.y.toFixed(0) }}, {{ crop.width.toFixed(0) }}, {{ crop.height.toFixed(0) }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- dialog buttons -->
      <div class="flex justify-end gap-4">
        <button
          class="px-4 py-1 rounded-lg hover:bg-base-content/30 cursor-pointer" 
          @click="clickCancel"
        >{{ $t('msgbox.image_editor.cancel') }}</button>
        <button 
          class="px-4 py-1 rounded-lg hover:bg-base-content/30 cursor-pointer" 
          @click="clickCopyImage"
        >{{ $t('msgbox.image_editor.copy_image') }}</button>
        <button 
          :class="[
            'px-4 py-1',
            cropStatus==1 ? 'text-base-content/30 cursor-default' : 'hover:bg-primary cursor-pointer rounded-lg',
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
  IconCopy,
  IconRotateLeft, 
  IconRotateRight, 
  IconFlipVertical, 
  IconFlipHorizontal,
  IconOk,
  IconRestore,
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

// container
const containerRef = ref<HTMLElement | null>(null);
const containerRect = ref<DOMRect | null>(null);
const containerPadding = 10;
const containerBounds = ref({ top: 0, left: 0, width: 0, height: 0 });  // container bounds without padding

// image
const imageSrc = ref('');
const imageRef = ref<HTMLImageElement | null>(null);
const imageRect = ref<DOMRect | null>(null);
const imageWidth = ref(0);
const imageHeight = ref(0);

// image transform
const enableTransition = ref(false);    
const position = ref({ left: 0, top: 0 });
const scale = ref(1);

const imageStyle = computed(() => {
  return {
    display: 'block',
    minWidth: `${imageWidth.value}px`,
    minHeight: `${imageHeight.value}px`,
    position: 'absolute',
    transform: `
      translate(${position.value.left}px, ${position.value.top}px) 
      rotate(${crop.value.rotate}deg) 
      scaleX(${isFlippedX.value ? -1 : 1}) 
      scaleY(${isFlippedY.value ? -1 : 1}) 
      scale(${scale.value})
    `,
    transition: enableTransition.value ? 'transform 0.3s ease' : 'none',
    // willChange: 'transform, box-shadow, opacity',
  };
});

// crop shape
const cropStatus = ref(0);    // 0: idle, 1: cropping, 2: cropped
const isPortrait = ref(false);

// cropped image
const crop = ref({
  x: 0,
  y: 0,
  width: 0,
  height: 0,
  rotate: 0,
});

// crop box
const cropBox = ref({ top: 0, left: 0, width: 0, height: 0 });
const cropBoxOriginal = ref({ top: 0, left: 0, width: 0, height: 0 });
const isDragging = ref(false);
const dragHandle = ref('');
const dragStartX = ref(0);
const dragStartY = ref(0);

const cropBoxStyle = computed(() => ({
  top: `${cropBox.value.top}px`,
  left: `${cropBox.value.left}px`,
  width: `${cropBox.value.width}px`,
  height: `${cropBox.value.height}px`,
}));

// crop shape
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

// flip image
const isFlippedX = ref(false);
const isFlippedY = ref(false);

// resize
const resizeWidth = ref(0);
const resizeHeight = ref(0);
const resizePercentage = ref(100);

// save as
const saveName = ref('');

const fileFormatOptions = computed(() => {
  return getSelectOptions(localeMsg.value.msgbox.image_editor.format_options);
});
const fileQualityOptions = computed(() => {
  return getSelectOptions(localeMsg.value.msgbox.image_editor.quality_options);
});

onMounted(async () => {
  const dialog = document.getElementById('imageEditorDialog') as HTMLDialogElement | null;
  dialog?.showModal();
  
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('ImageEditor');

  clickRestore(); // init container, image, crop, etc.
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.popInputHandler();
});

const onImageLoad = () => {
  nextTick(() => {
    imageRect.value = imageRef.value?.getBoundingClientRect() || null;
    enableTransition.value = true;
  });
};

watch(cropBox, (newData) => {
  if (imageRect.value && imageWidth.value && imageHeight.value) {
    crop.value.x      = Math.round(imageWidth.value * (newData.left - (imageRect.value.left - containerRect.value.left)) / imageRect.value.width);
    crop.value.y      = Math.round(imageHeight.value * (newData.top - (imageRect.value.top - containerRect.value.top)) / imageRect.value.height);
    crop.value.width  = Math.round(imageWidth.value * newData.width / imageRect.value.width);
    crop.value.height = Math.round(imageHeight.value * newData.height / imageRect.value.height);
  }
}, { deep: true });

function handleKeyDown(event: KeyboardEvent) {
  if (!uiStore.isInputActive('ImageEditor')) return;

  const { key } = event;

  switch (key) {
    case 'Enter':
      if(cropStatus.value==1) {
        doCrop();
      } else {
        clickSave();
      }
      break;
    case 'Escape':
      if(cropStatus.value==1) {
        cropStatus.value = 0;
      } else {
        clickCancel();
      }
      break;
      default:
        break;
    }
    event.preventDefault();
    event.stopPropagation();
}

const toggleCrop = () => {
  switch (cropStatus.value) {
    case 0:
      cropStatus.value = 1;
      updateCropBox();
      break;
    case 1:
      cropStatus.value = 0;
      break;
    case 2:
      cropStatus.value = 1;
      zoomFit(imageWidth.value, imageHeight.value); // update scale and position
      cropBox.value = { ...cropBoxOriginal.value }; // restore the original crop box data
      break;
    default:
      break;
  }
};

const togglePortraitAndLandscape = () => {
  isPortrait.value = !isPortrait.value;
  updateCropBox();
};

const onChangeCropShape = () => {
  updateCropBox();
};

const doCrop = () => {
  if (!containerBounds.value || !imageRect.value) return;

  cropStatus.value = 2;

  // update resize width and height
  resizeWidth.value = crop.value.width;
  resizeHeight.value = crop.value.height;
  resizePercentage.value = 100;

  // 1. calculate the new scale to fit the container
  scale.value = Math.min(
    containerBounds.value.width / crop.value.width,
    containerBounds.value.height / crop.value.height
  );

  // 2. calculate the new position to center the cropped image
  let newPositionLeft = position.value.left - (crop.value.x + crop.value.width / 2 - imageWidth.value / 2) * scale.value;
  let newPositionTop = position.value.top - (crop.value.y + crop.value.height / 2 - imageHeight.value / 2) * scale.value;
  position.value = { left: newPositionLeft, top: newPositionTop };
  
  // 3. save the original crop box data
  cropBoxOriginal.value = { ...cropBox.value };

  // 4. resize cropBox to fit the container
  const scaledCropWidth = crop.value.width * scale.value;
  const scaledCropHeight = crop.value.height * scale.value;
  cropBox.value = {
    left: (containerBounds.value.width - scaledCropWidth + containerPadding) / 2,
    top: (containerBounds.value.height - scaledCropHeight + containerPadding) / 2,
    width: scaledCropWidth,  
    height: scaledCropHeight,
  };
};

const clickCopyImage = () => {
  // copyImageToClipboard(props.fileInfo.file_path);
};

const clickRotate = (degree: number) => {
  crop.value.rotate += degree;
  
  // update scale to fit the container
  if(crop.value.rotate % 180 !== 0) {
    scale.value = Math.min(containerBounds.value.width / imageHeight.value, containerBounds.value.height / imageWidth.value);
  } else {
    scale.value = Math.min(containerBounds.value.width / imageWidth.value, containerBounds.value.height / imageHeight.value);
  }
};

const clickFlipX = () => {
  if(crop.value.rotate % 180 === 0) {
    isFlippedX.value = !isFlippedX.value;
  } else {
    isFlippedY.value = !isFlippedY.value;
  }
};

const clickFlipY = () => {
  if(crop.value.rotate % 180 === 0) {
    isFlippedY.value = !isFlippedY.value;
  } else {
    isFlippedX.value = !isFlippedX.value;
  }
};

// update scale and position, so the image will be centered in the container
const zoomFit = (imgWidth: number, imgHeight: number) => {
  scale.value = Math.min(containerBounds.value.width / imgWidth, containerBounds.value.height / imgHeight);
  position.value = { 
    left: (containerBounds.value.width - imgWidth + containerPadding) / 2, 
    top: (containerBounds.value.height - imgHeight + containerPadding ) / 2
  };
};

// update crop box by crop shape
const updateCropBox = () => {
  if (!imageRect.value || !containerRect.value) return;

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

    cropBox.value = {
      left: imageLeft + (imageRect.value.width - newWidth) / 2,
      top: imageTop + (imageRect.value.height - newHeight) / 2,
      width: newWidth,
      height: newHeight,
    };
  } else {
    cropBox.value = {
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

  const initialCropBoxData = { ...cropBox.value };

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

      cropBox.value.left = newLeft;
      cropBox.value.top = newTop;

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
        cropBox.value = proposedBox;
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

const clickRestore = () => {
  if (!imageRef.value || !containerRef.value) return;

  // container 
  containerRect.value = containerRef.value.getBoundingClientRect();
  containerBounds.value = {
    left:   containerRect.value.left + containerPadding,
    top:    containerRect.value.top + containerPadding,
    width:  containerRect.value.width - containerPadding,
    height: containerRect.value.height - containerPadding,
  };

  // image
  imageSrc.value = convertFileSrc(props.fileInfo.file_path);
  imageWidth.value = props.fileInfo.width;
  imageHeight.value = props.fileInfo.height;

  // init scale and position
  zoomFit(imageWidth.value, imageHeight.value);

  // crop init
  cropStatus.value = 0;
  isPortrait.value = imageWidth.value < imageHeight.value;
  crop.value = {
    x: 0,
    y: 0,
    width: imageWidth.value,
    height: imageHeight.value,
    rotate: 0,
  };

  // flip init
  isFlippedX.value = false;
  isFlippedY.value = false;

  // resize init
  resizeWidth.value = imageWidth.value;
  resizeHeight.value = imageHeight.value;
  resizePercentage.value = 100;

  // save as name init
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
  if (cropStatus.value == 1) {
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
.crop-box-active {
  position: absolute;
  border: 1px solid #fff;
  box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.8);
  box-sizing: border-box;
}
.crop-box-done {
  position: absolute;
  box-shadow: 0 0 0 9999px var(--color-base-200);
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
