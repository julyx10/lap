<template>

  <ModalDialog :title="`${$t('msgbox.image_editor.title')} - ${shortenFilename(props.fileInfo.name, 32)}`" :width="984" @cancel="clickCancel">
    <!-- content -->
    <div class="h-[516px] flex gap-4 select-none">
      <div class="flex-none h-full aspect-4/3 flex flex-col min-w-0">
        <!-- image container -->
        <div ref="containerRef" class="relative flex-1 outline outline-base-content/5 cursor-default rounded-box overflow-hidden select-none">

          <!-- Loading overlay -->
          <transition name="fade">
            <div v-if="isProcessing" class="absolute inset-0 bg-base-100/50 flex items-center justify-center z-50 rounded-box">
              <span class="loading loading-dots text-primary"></span>
            </div>
          </transition>
    
          <!-- image -->
          <img ref="imageRef" :src="imageSrc" :style="imageStyle" draggable="false" @load="onImageLoad" />

          <!-- crop box -->
          <div v-if="cropStatus > 0" 
            :class="[
              cropStatus === 1 ? 'crop-box-active' : cropStatus === 2 ? 'crop-box-done' : '',
              cropStatus === 1
                ? cropBoxFixed
                  ? (isDragging ? 'cursor-grabbing no-transition' : 'cursor-grab')
                  : (isDragging ? 'cursor-move no-transition' : 'cursor-move')
                : ''
            ]"
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
            <template v-if="cropStatus===1 && !cropBoxFixed">
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
      </div>

      <!-- Right: Sidebar -->
      <div class="w-64 flex flex-col gap-2 overflow-y-auto">

        <!-- Transform Section -->
        <div class="rounded-box p-3 space-y-3 bg-base-300/30 border border-base-content/5">
          <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content"
            @click="showTransform = !showTransform">
            <IconCrop class="w-4 h-4" />
            <span class="font-bold text-xs uppercase tracking-wide mr-auto">{{ $t('msgbox.image_editor.transform') }}</span>
            <component :is="showTransform ? IconArrowUp : IconArrowDown" class="w-3.5 h-3.5 opacity-50" />
          </div>
          <div v-if="showTransform">
            <!-- Default: all tools in one row -->
            <div v-if="cropStatus !== 1" class="flex items-center gap-1">
              <TButton
                :icon="IconCrop"
                :selected="cropStatus===2"
                :tooltip="$t('msgbox.image_editor.crop')"
                @click="cropStatus===0 ? clickStartCrop() : clickRestoreCrop()" 
              />
              <IconSeparator class="t-icon-size-sm text-base-content/30" />
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

            <!-- Crop active: sub-controls replace the row -->
            <div v-else class="flex items-center gap-1 p-1 rounded-box border border-primary">
              <TButton
                :icon="IconClose"
                :tooltip="$t('msgbox.image_editor.cancel_crop')"
                @click="clickCancelCrop" 
              />
              <TButton
                :icon="IconCropLandscape"
                :disabled="cropBoxFixed"
                :iconStyle="{ transform: `rotate(${isPortrait ? 90 : 0}deg)`, transition: 'transform 0.3s ease-in-out' }" 
                :tooltip="isPortrait ? $t('msgbox.image_editor.crop_shape_portrait') : $t('msgbox.image_editor.crop_shape_landscape')"
                @click="togglePortraitAndLandscape" 
              />
              <select v-model="config.imageEditor.cropShape" class="select select-bordered select-xs flex-1 min-w-0" :disabled="cropBoxFixed" @change="onChangeCropShape">
                <option v-for="option in cropShapeOptions" :value="option.value" :key="option.value">{{ option.label }}</option>
              </select>
              <TButton
                :icon="cropBoxFixed ? IconZoomOut : IconZoomIn"
                :tooltip="cropBoxFixed ? $t('msgbox.image_editor.zoom_out') : $t('msgbox.image_editor.zoom_in')"
                @click="toggleCropBoxFixed"
              />
              <TButton
                :icon="IconOk"
                :tooltip="$t('msgbox.image_editor.confirm_crop')"
                @click="clickDoCrop"
              />
            </div>
          </div>
        </div>

        <!-- Resize Section -->
        <div class="rounded-box p-3 space-y-3 bg-base-300/30 border border-base-content/5">
          <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content"
            @click="showResize = !showResize">
            <IconResize class="w-4 h-4" />
            <span class="font-bold text-xs uppercase tracking-wide mr-auto">{{ $t('msgbox.image_editor.resize') }}</span>
            <component :is="showResize ? IconArrowUp : IconArrowDown" class="w-3.5 h-3.5 opacity-50" />
          </div>
          <div v-if="showResize" class="space-y-2">
            <div class="grid grid-cols-2 gap-2">
              <div class="form-control w-full">
                <label class="label py-1">
                  <span class="label-text text-xs font-medium opacity-70">{{ $t('msgbox.image_editor.width') }}</span>
                </label>
                <input type="number" :placeholder="$t('msgbox.image_editor.width')" class="input input-bordered input-sm w-full"
                  v-model.number="resizedWidth" :disabled="cropStatus==1"
                  @keypress="onNumberKeyPress"
                  @blur="handleResizeInput('width')"
                />
              </div>
              <div class="form-control w-full">
                <label class="label py-1">
                  <span class="label-text text-xs font-medium opacity-70">{{ $t('msgbox.image_editor.height') }}</span>
                </label>
                <input type="number" :placeholder="$t('msgbox.image_editor.height')" class="input input-bordered input-sm w-full"
                  v-model.number="resizedHeight" :disabled="cropStatus==1"
                  @keypress="onNumberKeyPress"
                  @blur="handleResizeInput('height')"
                />
              </div>
            </div>
            <div class="form-control w-full">
              <label class="label py-1">
                <span class="label-text text-xs font-medium opacity-70">{{ $t('msgbox.image_editor.percentage') }}</span>
              </label>
              <div class="flex items-center gap-2">
                 <input type="number" :placeholder="$t('msgbox.image_editor.percentage')" class="input input-bordered input-sm w-full"
                  v-model.number="resizedPercentage" :disabled="cropStatus==1"
                  @keypress="onNumberKeyPress"
                  @blur="handleResizeInput('percentage')"
                />
                <span class="text-xs opacity-50">%</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Options Section -->
        <div class="rounded-box p-3 space-y-3 bg-base-300/30 border border-base-content/5">
          <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content"
            @click="showOptions = !showOptions">
            <IconSave class="w-4 h-4" />
            <span class="font-bold text-xs uppercase tracking-wide mr-auto">{{ $t('msgbox.image_editor.options') }}</span>
            <component :is="showOptions ? IconArrowUp : IconArrowDown" class="w-3.5 h-3.5 opacity-50" />
          </div>
          <div v-if="showOptions" class="space-y-2">
            <div class="form-control w-full">
              <label class="label py-1">
                <span class="label-text text-xs font-medium opacity-70">{{ $t('msgbox.image_editor.save_as') }}</span>
              </label>
              <select v-model="config.imageEditor.saveAs" class="select select-bordered select-sm w-full" :disabled="cropStatus==1">
                <option v-for="option in fileSaveAsOptions" :value="option.value" :key="option.value">{{ option.label }}</option>
              </select>
            </div>
            
            <div v-if="config.imageEditor.saveAs !== 0" class="grid grid-cols-2 gap-2">
              <div class="form-control w-full">
                <label class="label py-1">
                  <span class="label-text text-xs font-medium opacity-70">{{ $t('msgbox.image_editor.format') }}</span>
                </label>
                <select v-model="config.imageEditor.format" class="select select-bordered select-sm w-full" :disabled="cropStatus==1">
                  <option v-for="option in fileFormatOptions" :value="option.value" :key="option.value">{{ option.label }}</option>
                </select>
              </div>
              
              <div v-if="config.imageEditor.format == 0" class="form-control w-full">
                <label class="label py-1">
                  <span class="label-text text-xs font-medium opacity-70">{{ $t('msgbox.image_editor.quality') }}</span>
                </label>
                <select v-model="config.imageEditor.quality" class="select select-bordered select-sm w-full" :disabled="cropStatus==1">
                  <option v-for="option in fileQualityOptions" :value="option.value" :key="option.value">{{ option.label }}</option>
                </select>
              </div>
            </div>
          </div>
        </div>

      </div>
    </div>

    <!-- dialog buttons -->
    <div class="flex justify-end gap-4">
      <button
        class="px-4 py-1 rounded-box hover:bg-base-100 hover:text-base-content cursor-pointer"
        @click="clickCancel"
      >{{ $t('msgbox.image_editor.cancel') }}</button>
      <button 
        :class="[
          'px-4 py-1 rounded-box',
          cropStatus===1 ? 'text-base-content/30 cursor-default' : 'hover:bg-primary hover:text-base-100 cursor-pointer',
        ]" 
        @click="clickCopyImage"
      >{{ $t('msgbox.image_editor.copy_image') }}</button>
      <button 
        :class="[
          'px-4 py-1 rounded-box',
          cropStatus===1 ? 'text-base-content/30 cursor-default' : 'hover:bg-primary hover:text-base-100 cursor-pointer',
        ]" 
        @click="clickSave"
      >{{ config.imageEditor.saveAs === 1 ? $t('msgbox.image_editor.save_as_new') : $t('msgbox.image_editor.overwrite') }}</button>
    </div>
  </ModalDialog>

  <ToolTip ref="toolTipRef" />

  <MessageBox v-if="showOverwriteConfirm"
    :title="$t('msgbox.image_editor.overwrite')"
    :message="$t('msgbox.image_editor.overwrite_confirm')"
    :warningOk="true"
    :OkText="$t('msgbox.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="handleOverwriteConfirm"
    @cancel="handleOverwriteCancel"
  />

</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch, type CSSProperties } from 'vue';
import { useUIStore } from '@/stores/uiStore';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { getFolderPath, shortenFilename, getFullPath, combineFileName, getSelectOptions, getFileExtension, getAssetSrc } from '@/common/utils';
import { editImage, copyEditedImage, checkFileExists } from '@/common/api';

import ToolTip from '@/components/ToolTip.vue';
import ModalDialog from '@/components/ModalDialog.vue';
import MessageBox from '@/components/MessageBox.vue';
import TButton from '@/components/TButton.vue';

import { 
  IconCrop,
  IconCropLandscape,
  IconZoomIn,
  IconZoomOut,
  IconRotateLeft, 
  IconRotateRight, 
  IconFlipVertical, 
  IconFlipHorizontal,
  IconClose, 
  IconOk,
  IconArrowUp,
  IconArrowDown,
  IconResize,
  IconSave,
  IconSeparator,
} from '@/common/icons';

const props = defineProps({
  fileInfo: {
    type: Object,
    required: true,
  },
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const uiStore = useUIStore();
const emit = defineEmits(['success', 'failed', 'cancel']);

const toolTipRef = ref<InstanceType<typeof ToolTip> | null>(null);
const isProcessing = ref(false);  // show processing status

// sidebar section toggles
const showTransform = ref(true);
const showResize = ref(true);
const showOptions = ref(true);

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

const imageStyle = computed((): CSSProperties => ({
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
  filter: `
    brightness(${100 + brightness.value}%)
    contrast(${100 + contrast.value}%)
    blur(${blur.value}px)
    hue-rotate(${hue.value}deg)
    saturate(${saturation.value}%)
    ${selectedFilter.value === 'grayscale' ? 'grayscale(100%)' : ''}
    ${selectedFilter.value === 'sepia' ? 'sepia(100%)' : ''}
    ${selectedFilter.value === 'invert' ? 'invert(100%)' : ''}
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

// adjustments
const selectedFilter = ref(''); 
const brightness = ref(0); 
const contrast = ref(0);   
const blur = ref(0);       
const hue = ref(0);        
const saturation = ref(100); 

// save as
const newFileName = ref('');

const fileSaveAsOptions = computed(() => {
  return getSelectOptions(localeMsg.value.msgbox.image_editor.save_as_options);
});
const fileFormatOptions = computed(() => {
  return getSelectOptions(localeMsg.value.msgbox.image_editor.format_options);
});
const fileQualityOptions = computed(() => {
  return getSelectOptions(localeMsg.value.msgbox.image_editor.quality_options);
});

// Overwrite Confirmation Logic
const showOverwriteConfirm = ref(false);

const handleOverwriteConfirm = () => {
  showOverwriteConfirm.value = false;
  
  // Force use of original filename/path regardless of inputs
  const originalPath = props.fileInfo.file_path;
  const ext = getFileExtension(props.fileInfo.name).toLowerCase();
  const outputFormat = (ext === 'jpg' || ext === 'jpeg') ? 'jpg' : ext;

  const overrides = {
    destFilePath: originalPath,
    outputFormat: outputFormat
  };
  
  executeSave(overrides);
};

const handleOverwriteCancel = () => {
  showOverwriteConfirm.value = false;
  isProcessing.value = false;
};

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('ImageEditor');

  isProcessing.value = true;
  initImageEditor(); // init container, image, crop, etc.
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('ImageEditor');
});

const onImageLoad = async () => {
  await nextTick() // wait for Vue DOM update
  
  if (imageRef.value && (imageWidth.value === 0 || imageHeight.value === 0)) {
    imageWidth.value = imageRef.value.naturalWidth;
    imageHeight.value = imageRef.value.naturalHeight;
  }
  
  fitImageToContainer();
  
  requestAnimationFrame(() => {
    enableTransition.value = true;
    isProcessing.value = false;
  });
};

const initImageEditor = () => {
  // image
  imageSrc.value = getAssetSrc(props.fileInfo.file_path);
  imageWidth.value = props.fileInfo.width;
  imageHeight.value = props.fileInfo.height;
  isPortrait.value = imageHeight.value > imageWidth.value;

  // container 
  containerRect.value = containerRef.value?.getBoundingClientRect() || null;
  if (!containerRect.value) return;

  containerBounds.value = {
    left:   containerRect.value.left + containerPadding,
    top:    containerRect.value.top + containerPadding,
    width:  containerRect.value.width - containerPadding * 2,
    height: containerRect.value.height - containerPadding * 2,
  };

  // image transform init
  enableTransition.value = false;
  
  // Inherit adjustments from uiStore
  if (uiStore.activeAdjustments.filePath === props.fileInfo.file_path) {
    const adj = uiStore.activeAdjustments;
    brightness.value = adj.brightness || 0;
    contrast.value = adj.contrast || 0;
    saturation.value = adj.saturation ?? 100;
    hue.value = adj.hue || 0;
    blur.value = adj.blur || 0;
    selectedFilter.value = adj.filter || '';
    rotate.value = adj.rotate || 0;
    isFlippedX.value = !!adj.flipX;
    isFlippedY.value = !!adj.flipY;
  } else {
     // Default state
     brightness.value = 0;
     contrast.value = 0;
     saturation.value = 100;
     hue.value = 0;
     blur.value = 0;
     selectedFilter.value = '';
     rotate.value = 0;
     isFlippedX.value = false;
     isFlippedY.value = false;
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
  containerRect.value = containerRef.value?.getBoundingClientRect() || null;
  imageRect.value = imageRef.value?.getBoundingClientRect() || null;
  if (!imageRect.value || !containerRect.value) return;

  const selectedShape = cropShapeOptions.value.find(option => option.value === String(config.imageEditor.cropShape) && option.value !== '0');
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
  
  containerRect.value = containerRef.value?.getBoundingClientRect() || null;
  imageRect.value = imageRef.value?.getBoundingClientRect() || null;
  if (!imageRect.value || !containerRect.value) return;

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
}

// update crop box from crop
const updateCropBoxFromCrop = () => {
  if (crop.value.width === 0 || crop.value.height === 0) { // no crop
    cropBox.value = { left: 0, top: 0, width: 0, height: 0 };
    return;
  }

  imageRect.value = imageRectOriginal.value;
  if (!imageRect.value || !containerRect.value) return;

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
  containerRect.value = containerRef.value?.getBoundingClientRect() || null;
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
      if (!imageRect.value) return;
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
      if (!imageRect.value) return;
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

      const shape = String(config.imageEditor.cropShape);
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

// prevent non-number key press
const onNumberKeyPress = (e: KeyboardEvent) => {
  if (!/[0-9]/.test(e.key)) {
    e.preventDefault();
  }
}

const handleResizeInput = (sourceType: 'width' | 'height' | 'percentage') => {
  // 1. Define source dimensions
  const sourceWidth = cropStatus.value === 2 ? crop.value.width : (rotate.value % 180 !== 0 ? imageHeight.value : imageWidth.value);
  const sourceHeight = cropStatus.value === 2 ? crop.value.height : (rotate.value % 180 !== 0 ? imageWidth.value : imageHeight.value);
  if (sourceWidth <= 0 || sourceHeight <= 0) return;

  let percentage = resizedPercentage.value;

  // 2. Based on input source, calculate the percentage
  if (sourceType === 'width') {
    const clampedWidth = Math.max(10, Math.min(resizedWidth.value, sourceWidth));
    percentage = (clampedWidth / sourceWidth) * 100;
  } else if (sourceType === 'height') {
    const clampedHeight = Math.max(10, Math.min(resizedHeight.value, sourceHeight));
    percentage = (clampedHeight / sourceHeight) * 100;
  }
  
  // 3. Define min/max percentage and clamp the calculated percentage
  const minWidthPercentage = (10 / sourceWidth) * 100;
  const minHeightPercentage = (10 / sourceHeight) * 100;
  const minPercentage = Math.max(minWidthPercentage, minHeightPercentage);
  
  const finalPercentage = Math.round(Math.max(minPercentage, Math.min(percentage, 100)));

  // 4. Update all three reactive variables from the final, clamped percentage
  resizedPercentage.value = finalPercentage;
  resizedWidth.value = Math.round(sourceWidth * (finalPercentage / 100));
  resizedHeight.value = Math.round(sourceHeight * (finalPercentage / 100));
};

const clickCancel = () => {
  emit('cancel');
};

const setEditParams = (overrides: { fileName?: string; destFilePath?: string; outputFormat?: string } = {}) => {
  let name = overrides.fileName || newFileName.value;

  let outputFormat = overrides.outputFormat || fileFormatOptions.value[config.imageEditor.format].label.toLowerCase();
  
  let destFilePath = overrides.destFilePath;
  if (!destFilePath) {
    destFilePath = getFullPath(getFolderPath(props.fileInfo.file_path), combineFileName(name, outputFormat));
  }

  const orientation = props.fileInfo.e_orientation || 1;

  return {
    sourceFilePath: props.fileInfo.file_path,
    destFilePath: destFilePath,
    outputFormat: outputFormat,
    quality: [90, 80, 60][config.imageEditor.quality] || 80,
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
    // adjustments
    filter: selectedFilter.value || null,
    brightness: brightness.value !== 0 ? brightness.value : null,
    contrast: contrast.value !== 0 ? contrast.value : null,
    blur: blur.value > 0 ? blur.value : null,
    hue_rotate: hue.value !== 0 ? hue.value : null,
    saturation: saturation.value !== 100 ? saturation.value / 100.0 : null,
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
      toolTipRef.value?.showTip(localeMsg.value.tooltip.copy_image.success);
    } else {
      toolTipRef.value?.showTip(localeMsg.value.tooltip.copy_image.failed, true);
    }
  }
};

const executeSave = async (overrides: { fileName?: string; destFilePath?: string; outputFormat?: string } = {}) => {
  isProcessing.value = true;
  let success = false;
  try {
    const editParams = setEditParams(overrides);
    console.log(editParams);
    success = await editImage(editParams);
  } finally {
    isProcessing.value = false;
    if (success) {
      uiStore.updateFileVersion(props.fileInfo.file_path);
      emit('success');
    } else {
      emit('failed');
    }
  }
};

const clickSave = async () => {
  if (cropStatus.value === 1 || isProcessing.value) return;

  if (config.imageEditor.saveAs === 1) { // 1: Save as new file
    isProcessing.value = true;
    try {
      // calculate unique filename
      const folderPath = getFolderPath(props.fileInfo.file_path);
      const ext = fileFormatOptions.value[config.imageEditor.format].label.toLowerCase();
      
      let baseName = newFileName.value;
      
      let counter = 1;
      let candidateName = `${baseName}_${counter}`;
      let candidatePath = getFullPath(folderPath, combineFileName(candidateName, ext));
      
      while (await checkFileExists(candidatePath)) {
        counter++;
        candidateName = `${baseName}_${counter}`;
        candidatePath = getFullPath(folderPath, combineFileName(candidateName, ext));
      }
      // set overrides
      const overrides = {
        fileName: candidateName,
        destFilePath: candidatePath // optimization: we already calculated path
      };
      
      await executeSave(overrides);
    } catch(err) {
      isProcessing.value = false;
      emit('failed');
    }
  } else if (config.imageEditor.saveAs === 0) { // 0: Overwrite
    // Confirm overwrite
    showOverwriteConfirm.value = true;
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
