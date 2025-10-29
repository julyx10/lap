<template>

  <dialog id="imageEditorDialog" class="modal">
    <div class="relative p-4 flex flex-col gap-2 text-base-content/70 bg-base-100 border border-base-content/30 rounded-box">
      
      <!-- Loading overlay -->
      <transition name="fade">
        <div v-if="isProcessing" class="absolute inset-0 bg-base-100/50 flex items-center justify-center z-50 rounded-box">
          <span class="loading loading-dots text-primary"></span>
        </div>
      </transition>
      
      <!-- title bar -->
      <div class="flex items-center justify-between text-wrap break-all">
        {{ $t('msgbox.image_editor.title') }} - {{ props.fileInfo.name }}
        <TButton
          :icon="IconClose"
          :buttonSize="'small'"
          :disabled="cropStatus===1"
          @click="clickCancel"
        />
      </div>

      <!-- content -->
      <div class="flex-grow flex gap-4">
        <div class="flex-1">
          <!-- image container -->
          <div ref="containerRef" class="relative w-[570px] h-[430px] bg-base-200 cursor-default rounded-lg overflow-hidden select-none">
            <img ref="imageRef" :src="imageSrc" :style="imageStyle" draggable="false" @load="onImageLoad" />
            <!-- crop box -->
            <div v-if="cropStatus > 0" 
              :class="{
                'crop-box-active': cropStatus === 1,
                'crop-box-done': cropStatus === 2,
                'cursor-grabbing no-transition': cropStatus === 1 && isDragging,
                'cursor-grab': cropStatus === 1 && !isDragging
              }"
              :style="cropBoxStyle" 
              @mousedown="cropStatus===1 ? startDrag('move', $event) : null"
              @dblclick="clickDoCrop"
            >
              <template v-if="cropStatus===1 && isDragging">
                <div class="crop-dimensions-display">
                  {{ crop.width }} x {{ crop.height }}
                </div>
                <div class="grid-lines">
                  <div class="grid-line-h grid-line-h-1"></div>
                  <div class="grid-line-h grid-line-h-2"></div>
                  <div class="grid-line-v grid-line-v-1"></div>
                  <div class="grid-line-v grid-line-v-2"></div>
                </div>
              </template>
              <template v-else-if="cropStatus===1 && !cropBoxFixed">
                <div class="drag-handle top-left" @mousedown.stop="startDrag('top-left', $event)"></div>
                <div class="drag-handle top" @mousedown.stop="startDrag('top', $event)"></div>
                <div class="drag-handle top-right" @mousedown.stop="startDrag('top-right', $event)"></div>
                <div class="drag-handle left" @mousedown.stop="startDrag('left', $event)"></div>
                <div class="drag-handle right" @mousedown.stop="startDrag('right', $event)"></div>
                <div class="drag-handle bottom-left" @mousedown.stop="startDrag('bottom-left', $event)"></div>
                <div class="drag-handle bottom" @mousedown.stop="startDrag('bottom', $event)"></div>
                <div class="drag-handle bottom-right" @mousedown.stop="startDrag('bottom-right', $event)"></div>
              </template>
            </div>
          </div>

          <!-- crop controls -->
          <div class="pt-2 flex justify-between">
            <!-- rotate and flip controls -->
            <div class="flex gap-2">
              <TButton
                :icon="IconRotateLeft"
                :disabled="cropStatus > 0"
                :tooltip="$t('msgbox.image_editor.rotate_left')"
                @click="clickRotate(-90)" 
              />
              <TButton
                :icon="IconRotateRight"
                :disabled="cropStatus > 0"
                :tooltip="$t('msgbox.image_editor.rotate_right')"
                @click="clickRotate(90)" 
              />
              <TButton
                :icon="IconFlipHorizontal"
                :disabled="cropStatus > 0"
                :tooltip="$t('msgbox.image_editor.flip_horizontal')"
                @click="clickFlipX" 
              />
              <TButton
                :icon="IconFlipVertical"
                :disabled="cropStatus > 0"
                :tooltip="$t('msgbox.image_editor.flip_vertical')"
                @click="clickFlipY" 
              />
            </div>

            <!-- crop shape controls -->
            <div 
              :class="['flex border rounded-box', 
                cropStatus==1 ? 'border-primary' : 'border-transparent',
              ]"
            >
              <TButton v-if="cropStatus===0 || cropStatus===2"
                :icon="IconCrop"
                :selected="cropStatus===2"
                :tooltip="$t('msgbox.image_editor.crop')"
                @click="cropStatus===0 ? clickStartCrop() : clickRestoreCrop()" 
              />

              <div v-if="cropStatus==1" class="flex items-center">
                <TButton
                  :icon="IconClose"
                  :tooltip="$t('msgbox.image_editor.cancel_crop')"
                  @click="clickCancelCrop" 
                />
                <TButton
                  :icon="IconCropLandscape"
                  :iconStyle="{ transform: `rotate(${isPortrait ? 90 : 0}deg)`, transition: 'transform 0.3s ease-in-out' }" 
                  :tooltip="isPortrait ? $t('msgbox.image_editor.crop_shape_portrait') : $t('msgbox.image_editor.crop_shape_landscape')"
                  @click="togglePortraitAndLandscape" 
                />
                <select v-model="config.imageEditor.cropShape" class="select select-bordered" @change="onChangeCropShape">
                  <option v-for="option in cropShapeOptions" :value="option.value" :key="option.value">{{ option.label }}</option>
                </select>
                <TButton
                  :icon="cropBoxFixed ? IconCropMin : IconCropMax"
                  :tooltip="cropBoxFixed ? $t('msgbox.image_editor.fix_crop_box') : $t('msgbox.image_editor.resize_crop_box')"
                  @click="toggleCropBoxFixed"
                />
                <TButton
                  :icon="IconOk"
                  :tooltip="$t('msgbox.image_editor.done_crop')"
                  @click="clickDoCrop"
                />
              </div>  
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
                      v-model="resizedWidth" :disabled="cropStatus==1"
                    />
                  </td>
                </tr>
                <tr>
                  <td>{{ $t('msgbox.image_editor.height') }}</td>
                  <td>
                    <input type="number" min="100" max="100000" :placeholder="$t('msgbox.image_editor.height')"  class="input input-bordered w-full"
                      v-model="resizedHeight" :disabled="cropStatus==1"
                    />
                  </td>
                </tr>
                <tr>
                  <td>{{ $t('msgbox.image_editor.percentage') }}</td>
                  <td class="flex items-center">
                    <input type="number" min="1" max="100" :placeholder="$t('msgbox.image_editor.percentage')"  class="input input-bordered w-full"
                      v-model="resizedPercentage" :disabled="cropStatus==1"
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
            <input type="text" :placeholder="$t('msgbox.image_editor.save_as_placeholder')" v-model="newFileName" class="input input-bordered w-full px-2" :disabled="cropStatus==1" />

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
            <div class="text-[10px] text-base-content/30 flex flex-col gap-1 mt-2">
              <span>containerRect: {{ containerRect?.left.toFixed(0) }}, {{ containerRect?.top.toFixed(0) }}, {{ containerRect?.width.toFixed(0) }}, {{ containerRect?.height.toFixed(0) }}</span>
              <span>containerBounds: {{ containerBounds?.left.toFixed(0) }}, {{ containerBounds?.top.toFixed(0) }}, {{ containerBounds?.width.toFixed(0) }}, {{ containerBounds?.height.toFixed(0) }}</span>
              <span>imageRect: {{ imageRect?.left.toFixed(0) }}, {{ imageRect?.top.toFixed(0) }}, {{ imageRect?.width.toFixed(0) }}, {{ imageRect?.height.toFixed(0) }}</span>
              <span>cropBox:{{ cropBox.left.toFixed(0) }}, {{ cropBox.top.toFixed(0) }}, {{ cropBox.width.toFixed(0) }}, {{ cropBox.height.toFixed(0) }}</span> 
              <span>scale: {{ scale.toFixed(2) }}</span>
              <span>position: {{ position.left.toFixed(0) }}, {{ position.top.toFixed(0) }}</span>
              <span>crop: {{ crop.left.toFixed(0) }}, {{ crop.top.toFixed(0) }}, {{ crop.width.toFixed(0) }}, {{ crop.height.toFixed(0) }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- dialog buttons -->
      <div class="flex justify-end gap-4">
        <button
          :class="[
            'px-4 py-1 rounded-lg',
            cropStatus===1 ? 'text-base-content/30 cursor-default' : 'hover:bg-base-content/30 cursor-pointer',
          ]" 
          @click="clickCancel"
        >{{ $t('msgbox.image_editor.cancel') }}</button>
        <button 
          :class="[
            'px-4 py-1 rounded-lg',
            cropStatus===1 ? 'text-base-content/30 cursor-default' : 'hover:bg-base-content/30 cursor-pointer',
          ]" 
          @click="clickCopyImage"
        >{{ $t('msgbox.image_editor.copy_image') }}</button>
        <button 
          :class="[
            'px-4 py-1 rounded-lg',
            cropStatus===1 ? 'text-base-content/30 cursor-default' : 'hover:bg-primary cursor-pointer',
          ]" 
          @click="clickSave"
        >{{ $t('msgbox.image_editor.ok') }}</button>
      </div>
    </div>

    <ToolTip ref="toolTipRef" />

  </dialog>

</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { useUIStore } from '@/stores/uiStore';
import { useI18n } from 'vue-i18n';
import { config, getFolderPath, extractFileName, getFullPath, combineFileName, getSelectOptions, getFileExtension, getAssetSrc } from '@/common/utils';
import { editImage, copyEditedImage } from '@/common/api';

import TButton from '@/components/TButton.vue';
import ToolTip from '@/components/ToolTip.vue';

import { 
  IconCrop,
  IconCropLandscape,
  IconCropMax,
  IconCropMin,
  IconRotateLeft, 
  IconRotateRight, 
  IconFlipVertical, 
  IconFlipHorizontal,
  IconClose, 
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

const toolTipRef = ref(null);
const isProcessing = ref(false);  // show processing status

// container
const containerRef = ref<HTMLElement | null>(null);
const containerRect = ref<DOMRect | null>(null);
const containerBounds = ref({ top: 0, left: 0, width: 0, height: 0 });
const containerPadding = 5; // container padding

// image
const imageRef = ref<HTMLImageElement | null>(null);
const imageRect = ref<DOMRect | null>(null);
const imageRectOriginal = ref<DOMRect | null>(null);
const imageSrc = ref('');
const imageWidth = ref(0);     // original image width
const imageHeight = ref(0);    // original image height

// image transform
const enableTransition = ref(false);    
const position = ref({ left: 0, top: 0 });
const isFlippedX = ref(false);
const isFlippedY = ref(false);
const scale = ref(1);
const rotate = ref(0);  // 0, 90, 180, 270, -90, -180, -270

const imageStyle = computed(() => ({
  display: 'block',
  minWidth: `${imageWidth.value}px`,
  minHeight: `${imageHeight.value}px`,
  position: 'absolute',
  transform: `
    translate(${position.value.left}px, ${position.value.top}px) 
    rotate(${rotate.value}deg) 
    scaleX(${isFlippedX.value ? -1 : 1}) 
    scaleY(${isFlippedY.value ? -1 : 1}) 
    scale(${scale.value})
  `,
  transition: enableTransition.value ? 'transform 0.3s ease' : 'none',
}));

// crop shape
const cropStatus = ref(0);    // 0: idle, 1: cropping, 2: cropped
const isPortrait = ref(false);
const cropBoxFixed = ref(false);  // false: can resize the crop box; true: fix the crop box size, drag to move image

// crop box
const cropBox = ref({ left: 0, top: 0, width: 0, height: 0 });  // crop box on the image
const crop = ref({ left: 0, top: 0, width: 0, height: 0 });  // cropped image area on the image

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

// resized image after having cropped
const resizedWidth = ref(0);
const resizedHeight = ref(0);
const resizedPercentage = ref(100);

// save as
const newFileName = ref('');

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

  initImageEditor(); // init container, image, crop, etc.
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.popInputHandler();
});

const onImageLoad = () => {
  nextTick(() => {
    enableTransition.value = true;
  });
};

const initImageEditor = () => {
  // container 
  containerRect.value = containerRef.value?.getBoundingClientRect() || null;
  if (!containerRect.value) return;

  containerBounds.value = {
    left:   containerRect.value.left + containerPadding,
    top:    containerRect.value.top + containerPadding,
    width:  containerRect.value.width - containerPadding * 2,
    height: containerRect.value.height - containerPadding * 2,
  };

  // image
  imageSrc.value = getAssetSrc(props.fileInfo.file_path);
  imageWidth.value = props.fileInfo.width;
  imageHeight.value = props.fileInfo.height;

  // image transform init
  enableTransition.value = false;
  
  // flip init
  isFlippedX.value = false;
  isFlippedY.value = false;
  
  // rotate
  rotate.value = 0;
  
  // crop init
  cropStatus.value = 0;
  isPortrait.value = imageWidth.value < imageHeight.value;
  cropBoxFixed.value = false;     // false: can resize the crop box; true: fix the crop box size, drag to move image
  
  // cropped image
  cropBox.value = { left: 0, top: 0, width: 0, height: 0 };
  fitImageToContainer();

  // resize init
  resizedWidth.value = imageWidth.value;
  resizedHeight.value = imageHeight.value;
  resizedPercentage.value = 100;

  // save as name init
  newFileName.value = extractFileName(props.fileInfo.name).name;
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

const clickStartCrop = () => {
  cropStatus.value = 1;   // cropping
  cropBoxFixed.value = false;
  initCropBox();
};

const clickCancelCrop = () => {
  cropStatus.value = 0;   // idle
  cropBox.value = { left: 0, top: 0, width: 0, height: 0 };

  fitImageToContainer();  // restore the original scale and position

  // update resize width and height
  const rotatedWidth = rotate.value % 180 !== 0 ? imageHeight.value : imageWidth.value;
  const rotatedHeight = rotate.value % 180 !== 0 ? imageWidth.value : imageHeight.value;
  resizedWidth.value = rotatedWidth;
  resizedHeight.value = rotatedHeight;
  resizedPercentage.value = 100;
};

const clickRestoreCrop = () => {
  cropStatus.value = 1;   // cropping
  
  if(!cropBoxFixed.value) {
    fitImageToContainer();
  }
};

// do crop
const clickDoCrop = () => {
  cropStatus.value = 2;   // cropped

  if(!cropBoxFixed.value) {
    fitCropBoxToContainer();
  }

  // update resized width and height
  resizedWidth.value = crop.value.width;
  resizedHeight.value = crop.value.height;
  resizedPercentage.value = 100;
};

const togglePortraitAndLandscape = () => {
  isPortrait.value = !isPortrait.value;
  initCropBox();
};

const toggleCropBoxFixed = () => {
  cropBoxFixed.value = !cropBoxFixed.value;
  cropBoxFixed.value ? fitCropBoxToContainer() : fitImageToContainer();
};

const onChangeCropShape = () => {
  initCropBox();
};

// initialize crop box shape
const initCropBox = () => {
  imageRect.value = imageRef.value?.getBoundingClientRect() || null;
  if (!imageRect.value) return;

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
  } else {    // custom shape
    cropBox.value = {
      left: imageRect.value.left - containerRect.value.left,
      top: imageRect.value.top - containerRect.value.top,
      width: imageRect.value.width,
      height: imageRect.value.height,
    };
  }
  updateCropFromCropBox(); // cropbox -> crop
}

// update crop from crop box
const updateCropFromCropBox = () => {
  if (cropBox.value.width === 0 || cropBox.value.height === 0) { // no crop box
    crop.value = { left: 0, top: 0, width: 0, height: 0 };
    return;
  };
  
  imageRect.value = imageRef.value?.getBoundingClientRect() || null;
  if (!imageRect.value) return;

  const imgWidth = rotate.value % 180 === 0 ? imageWidth.value : imageHeight.value;
  const imgHeight = rotate.value % 180 === 0 ? imageHeight.value : imageWidth.value;

  const scaleX = imgWidth / imageRect.value.width;
  const scaleY = imgHeight / imageRect.value.height;
  crop.value = {
    left: Math.round(scaleX * (cropBox.value.left + containerRect.value.left - imageRect.value.left)),
    top:  Math.round(scaleY * (cropBox.value.top + containerRect.value.top - imageRect.value.top)),
    width: Math.round(scaleX * cropBox.value.width),
    height: Math.round(scaleY * cropBox.value.height)
  };

  // consider the image rotation
  // switch (rotate.value % 360) {
  //   case 0:
  //     crop.value = rotatedCrop;
  //     break;
  //   case 90:
  //   case -270:
  //     crop.value = {
  //       left: rotatedCrop.top,
  //       top:  imageHeight.value - rotatedCrop.left - rotatedCrop.width,
  //       width: rotatedCrop.height,
  //       height: rotatedCrop.width
  //     }
  //     break;
  //   case 180:
  //   case -180:
  //     crop.value = {
  //       left: imageWidth.value - rotatedCrop.left - rotatedCrop.width,
  //       top:  imageHeight.value - rotatedCrop.top - rotatedCrop.height,
  //       width: rotatedCrop.width,
  //       height: rotatedCrop.height
  //     }
  //     break;
  //   case 270:
  //   case -90:
  //     crop.value = {
  //       left: imageWidth.value - rotatedCrop.top - rotatedCrop.height,
  //       top:  rotatedCrop.left,
  //       width: rotatedCrop.height,
  //       height: rotatedCrop.width
  //     }
  //     break;
  //   default:
  //     break;
  // }
}

// update crop box from crop
const updateCropBoxFromCrop = () => {
  if (crop.value.width === 0 || crop.value.height === 0) { // no crop
    cropBox.value = { left: 0, top: 0, width: 0, height: 0 };
    return;
  }

  imageRect.value = imageRectOriginal.value;

  const imgWidth = rotate.value % 180 === 0 ? imageWidth.value : imageHeight.value;
  const imgHeight = rotate.value % 180 === 0 ? imageHeight.value : imageWidth.value;

  const scaleX = imgWidth / imageRect.value.width;
  const scaleY = imgHeight / imageRect.value.height;

  if (scaleX === 0 || scaleY === 0) return;

  cropBox.value = {
    left: (crop.value.left / scaleX) - containerRect.value.left + imageRect.value.left,
    top: (crop.value.top / scaleY) - containerRect.value.top + imageRect.value.top,
    width: crop.value.width / scaleX,
    height: crop.value.height / scaleY,
  };
}

// update scale to fit the container bounds
const scaleFit = (imgWidth: number, imgHeight: number) => {
  scale.value = Math.min(containerBounds.value.width / imgWidth, containerBounds.value.height / imgHeight);
};

// fit image to container
const fitImageToContainer = () => {
  if (!containerRect.value) return;

  position.value = { 
    left: (containerRect.value.width - imageWidth.value) / 2, 
    top: (containerRect.value.height - imageHeight.value) / 2,
  };

  rotate.value % 180 !== 0 ? 
    scaleFit(imageHeight.value, imageWidth.value) :
    scaleFit(imageWidth.value, imageHeight.value);

  updateCropBoxFromCrop(); // crop -> cropbox
}

// fit crop box to container
const fitCropBoxToContainer = () => {
  if (!containerBounds.value || !containerRect.value) return;
  
  imageRectOriginal.value = imageRect.value;

  // save old scale
  const oldScale = scale.value;

  // calculate the new scale to fit the container
  scale.value = Math.min(
    (containerBounds.value.width / cropBox.value.width) * oldScale, 
    (containerBounds.value.height / cropBox.value.height) * oldScale
  );

  // calculate the new position to center the cropped image
  position.value = { 
    left: position.value.left + ( containerRect.value.width / 2 - (cropBox.value.left + cropBox.value.width / 2) ) * scale.value / oldScale, 
    top: position.value.top + ( containerRect.value.height / 2 - (cropBox.value.top + cropBox.value.height / 2) ) * scale.value / oldScale,
  };

  // resize cropBox to fit the container
  const newCropBoxWidth = cropBox.value.width * scale.value / oldScale;
  const newCropBoxHeight = cropBox.value.height * scale.value / oldScale;
  cropBox.value = {
    left: (containerRect.value.width - newCropBoxWidth) / 2,
    top:  (containerRect.value.height - newCropBoxHeight) / 2,
    width:  newCropBoxWidth,  
    height: newCropBoxHeight,
  };

  // update crop from crop box after the transition ends
  imageRef.value?.addEventListener('transitionend', updateCropFromCropBox, { once: true });
};

const clickRotate = (degree: number) => {
  rotate.value += degree;  // 90, 180, 270, -90, -180, -270
  isPortrait.value = !isPortrait.value; // update isPortrait
  
  const rotatedWidth = rotate.value % 180 !== 0 ? imageHeight.value : imageWidth.value;
  const rotatedHeight = rotate.value % 180 !== 0 ? imageWidth.value : imageHeight.value;
  resizedWidth.value = rotatedWidth;
  resizedHeight.value = rotatedHeight;
  resizedPercentage.value = 100;
  
  scaleFit(rotatedWidth, rotatedHeight);
};

const clickFlipX = () => {
  rotate.value % 180 !== 0 ?
    isFlippedY.value = !isFlippedY.value :
    isFlippedX.value = !isFlippedX.value;
};

const clickFlipY = () => {
  rotate.value % 180 !== 0 ?
    isFlippedX.value = !isFlippedX.value :
    isFlippedY.value = !isFlippedY.value;
};

const startDrag = (handle: string, event: MouseEvent) => {
  isDragging.value = true;
  dragHandle.value = handle;
  dragStartX.value = event.clientX;
  dragStartY.value = event.clientY;

  if (cropBoxFixed.value && dragHandle.value === 'move') {
    enableTransition.value = false;
  }

  const initialCropBoxData = { ...cropBox.value };
  const initialImagePosition = { ...position.value };
  const initialImageRect = imageRef.value?.getBoundingClientRect() || null;

  const doDrag = (e: MouseEvent) => {
    if (!isDragging.value || !initialImageRect || !containerRect.value) return;

    const dx = e.clientX - dragStartX.value;
    const dy = e.clientY - dragStartY.value;

    if (cropBoxFixed.value && dragHandle.value === 'move') {
      // Move image instead of crop box
      const initialImageLeft = initialImageRect.left - containerRect.value.left;
      const initialImageRight = initialImageLeft + initialImageRect.width;
      const max_dx = cropBox.value.left - initialImageLeft;
      const min_dx = (cropBox.value.left + cropBox.value.width) - initialImageRight;
      const clamped_dx = Math.max(min_dx, Math.min(dx, max_dx));

      const initialImageTop = initialImageRect.top - containerRect.value.top;
      const initialImageBottom = initialImageTop + initialImageRect.height;
      const max_dy = cropBox.value.top - initialImageTop;
      const min_dy = (cropBox.value.top + cropBox.value.height) - initialImageBottom;
      const clamped_dy = Math.max(min_dy, Math.min(dy, max_dy));

      position.value.left = initialImagePosition.left + clamped_dx;
      position.value.top = initialImagePosition.top + clamped_dy;

    } else if (dragHandle.value === 'move') {
      const imageLeft = imageRect.value.left - containerRect.value.left;
      const imageTop = imageRect.value.top - containerRect.value.top;
      const imageRight = imageLeft + imageRect.value.width;
      const imageBottom = imageTop + imageRect.value.height;

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
      const imageLeft = imageRect.value.left - containerRect.value.left;
      const imageTop = imageRect.value.top - containerRect.value.top;
      const imageRight = imageLeft + imageRect.value.width;
      const imageBottom = imageTop + imageRect.value.height;
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
    updateCropFromCropBox(); // cropbox -> crop
  };

  const stopDrag = () => {
    if (cropBoxFixed.value && dragHandle.value === 'move') {
      enableTransition.value = true;
    }
    isDragging.value = false;
    window.removeEventListener('mousemove', doDrag);
    window.removeEventListener('mouseup', stopDrag);
  };

  window.addEventListener('mousemove', doDrag);
  window.addEventListener('mouseup', stopDrag);
};

function handleKeyDown(event: KeyboardEvent) {
  if (!uiStore.isInputActive('ImageEditor')) return;

  const { key } = event;

  switch (key) {
    case 'Enter':
      if(cropStatus.value==1) {
        clickDoCrop();
      } else {
        clickSave();
      }
      event.preventDefault();
      event.stopPropagation();
      break;
    case 'Escape':
      if(cropStatus.value==1) {
        cropStatus.value = 0;
      } else {
        clickCancel();
      }
      event.preventDefault();
      event.stopPropagation();
      break;
      default:
        break;
    }
}

const clickCancel = () => {
  if (cropStatus.value === 1) return;
  emit('cancel');
};

const setEditParams = () => {
  const fileName = combineFileName(newFileName.value, fileFormatOptions.value[config.imageEditor.format].label.toLowerCase());
  const destFilePath = getFullPath(getFolderPath(props.fileInfo.file_path), fileName);
  const orientation = props.fileInfo.e_orientation || 1;

  return {
    sourceFilePath: props.fileInfo.file_path,
    destFilePath: destFilePath,
    outputFormat: fileFormatOptions.value[config.imageEditor.format].label.toLowerCase(),
    orientation: orientation,
    flipHorizontal: isFlippedX.value,
    flipVertical: isFlippedY.value,
    rotate: rotate.value,
    crop: {
      x: crop.value.left,
      y: crop.value.top,
      width: crop.value.width,
      height: crop.value.height,
    },
    resize: {
      width: resizedWidth.value,
      height: resizedHeight.value,
    },
  };
};

const clickCopyImage = async () => {
  if (cropStatus.value === 1 || isProcessing.value) return;

  isProcessing.value = true;

  let success = false;
  try {
    const editParams = setEditParams();
    console.log(editParams);
    success = await copyEditedImage(editParams);
  } finally {
    isProcessing.value = false;
    if (success) {
      toolTipRef.value.showTip(localeMsg.value.tooltip.copy_image.success);
    } else {
      toolTipRef.value.showTip(localeMsg.value.tooltip.copy_image.failed, true);
    }
  }
};

const clickSave = async () => {
  if (cropStatus.value === 1 || isProcessing.value) return;

  isProcessing.value = true;

  let success = false;
  try {
    const editParams = setEditParams();
    console.log(editParams);
    success = await editImage(editParams);
  } finally {
    isProcessing.value = false;
    if (success) {
      toolTipRef.value.showTip(localeMsg.value.tooltip.save_image.success);
      uiStore.updateFileVersion(props.fileInfo.file_path);
      emit('ok');
    } else {
      toolTipRef.value.showTip(localeMsg.value.tooltip.save_image.failed, true);
    }
  }
};

</script>

<style scoped>
.crop-box-active {
  position: absolute;
  border: 1px solid #fff;
  /* box-shadow: 0 0 0 9999px rgba(0,0,0,0.8);  */
  box-shadow: 0 0 0 9999px color-mix(in srgb, var(--color-base-200) 80%, transparent);
  box-sizing: border-box;
  will-change: transform;
  transition: all 0.3s ease;
}
.crop-box-done {
  position: absolute;
  /* border: 1px solid #000; */
  box-shadow: 0 0 0 9999px var(--color-base-200);
  box-sizing: border-box;
  will-change: transform;
}
.no-transition {
  transition: none !important;
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
