<template>

  <ModalDialog :title="`${$t('msgbox.image_editor.title')} - ${shortenFilename(props.fileInfo.name, 32)}`" :width="830" @cancel="clickCancel">
    <!-- content -->
    <div class="flex-grow flex gap-4 select-none">
      <div class="flex-1">
        <!-- image container -->
        <div ref="containerRef" class="relative w-[610px] h-[460px] outline outline-base-content/5 cursor-default rounded-box overflow-hidden select-none">

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

        <!-- crop controls -->
        <div class="py-2 flex justify-between">
   
          <!-- crop shape controls -->
          <div 
            :class="['flex p-1 border rounded-box', 
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
                :disabled="cropBoxFixed"
                :iconStyle="{ transform: `rotate(${isPortrait ? 90 : 0}deg)`, transition: 'transform 0.3s ease-in-out' }" 
                :tooltip="isPortrait ? $t('msgbox.image_editor.crop_shape_portrait') : $t('msgbox.image_editor.crop_shape_landscape')"
                @click="togglePortraitAndLandscape" 
              />
              <select v-model="config.imageEditor.cropShape" class="select select-bordered" :disabled="cropBoxFixed" @change="onChangeCropShape">
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

          <!-- rotate and flip controls -->
          <div class="my-2 flex gap-2">
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

        </div>
      </div>

      <!-- edit controls / adjustments -->
      <div class="w-48 flex flex-col gap-2 overflow-y-auto">

        <!-- Tabs -->
        <div role="tablist" class="tabs tabs-border">
          <a role="tab" :class="['tab', {'tab-active': activeTab === 'adjust'}]" @click="activeTab = 'adjust'">{{ $t('msgbox.image_editor.tab_adjust') }}</a>
          <a role="tab" :class="['tab', {'tab-active': activeTab === 'export'}]" @click="activeTab = 'export'">{{ $t('msgbox.image_editor.tab_export') }}</a>
        </div>

        <!-- Adjust Tab Content -->
        <div v-show="activeTab === 'adjust'" class="flex flex-col gap-4 p-1">
          <!-- filters -->
          <div>
            <h3>{{ $t('msgbox.image_editor.filters') }}</h3>
            <div class="flex flex-col gap-2 mt-2">
              <label class="label cursor-pointer justify-start gap-2 h-8">
                <input type="radio" name="filter" class="radio radio-xs" value="" v-model="selectedFilter" />
                <span class="text-sm">{{ $t('msgbox.image_editor.filter_none') }}</span>
              </label>
              <label class="label cursor-pointer justify-start gap-2 h-8">
                <input type="radio" name="filter" class="radio radio-xs" value="grayscale" v-model="selectedFilter" />
                <span class="text-sm">{{ $t('msgbox.image_editor.filter_grayscale') }}</span>
              </label>
              <label class="label cursor-pointer justify-start gap-2 h-8">
                <input type="radio" name="filter" class="radio radio-xs" value="sepia" v-model="selectedFilter" />
                <span class="text-sm">{{ $t('msgbox.image_editor.filter_sepia') }}</span>
              </label>
              <label class="label cursor-pointer justify-start gap-2 h-8">
                <input type="radio" name="filter" class="radio radio-xs" value="invert" v-model="selectedFilter" />
                <span class="text-sm">{{ $t('msgbox.image_editor.filter_invert') }}</span>
              </label>
            </div>
          </div>

          <!-- adjustments -->
          <div>
            <div class="flex justify-between">
              <span>{{ $t('msgbox.image_editor.adjustments') }}</span>
              <TButton :icon="IconRestore" :buttonSize="'small'" @click="resetAdjustments">{{ $t('msgbox.image_editor.reset') }}</TButton>
            </div>
            <div class="flex flex-col gap-2 mt-2 text-sm text-base-content/70">
              
              <!-- Brightness -->
              <div class="flex flex-col">
                <div class="flex justify-between">
                  <span>{{ $t('msgbox.image_editor.brightness') }}</span>
                  <span>{{ brightness }}</span>
                </div>
                <SliderInput 
                  v-model="brightness" 
                  :min="-100" 
                  :max="100" 
                  :step="1" 
                  label=""
                />
              </div>

              <!-- Contrast -->
              <div class="flex flex-col">
                <div class="flex justify-between">
                  <span>{{ $t('msgbox.image_editor.contrast') }}</span>
                  <span>{{ contrast }}</span>
                </div>
                <SliderInput 
                  v-model="contrast" 
                  :min="-100" 
                  :max="100" 
                  :step="1" 
                  label=""
                />
              </div>

              <!-- Saturation -->
              <div class="flex flex-col">
                <div class="flex justify-between">
                  <span>{{ $t('msgbox.image_editor.saturation') }}</span>
                  <span>{{ saturation }}</span>
                </div>
                <SliderInput 
                  v-model="saturation" 
                  :min="0" 
                  :max="200" 
                  :step="1" 
                  label=""
                />
              </div>

              <!-- Hue -->
              <div class="flex flex-col">
                <div class="flex justify-between">
                  <span>{{ $t('msgbox.image_editor.hue_rotate') }}</span>
                  <span>{{ hue }}</span>
                </div>
                <SliderInput 
                  v-model="hue" 
                  :min="-180" 
                  :max="180" 
                  :step="1" 
                  label=""
                />
              </div>

              <!-- Blur -->
              <!-- <div class="flex flex-col">
                <div class="flex justify-between">
                  <span>{{ $t('msgbox.image_editor.blur') }}</span>
                  <span>{{ blur }}</span>
                </div>
                <SliderInput 
                  v-model="blur" 
                  :min="0" 
                  :max="10" 
                  :step="0.1" 
                  label=""
                />
              </div> -->

            </div>
          </div>
        </div>
       
        <!-- Export Tab Content -->
        <div v-show="activeTab === 'export'" class="flex flex-col gap-4 p-1">
          <!-- Resize -->
          <div>
            <h3>{{ $t('msgbox.image_editor.resize') }}</h3>
            <table class="w-full text-sm border-separate border-spacing-2">
              <tbody>
                <tr>
                  <td class="w-1/2">{{ $t('msgbox.image_editor.width') }}</td>
                  <td>
                    <input type="number" :placeholder="$t('msgbox.image_editor.width')" class="input input-bordered w-full"
                      v-model.number="resizedWidth" :disabled="cropStatus==1"
                      @keypress="onNumberKeyPress"
                      @blur="handleResizeInput('width')"
                    />
                  </td>
                </tr>
                <tr>
                  <td>{{ $t('msgbox.image_editor.height') }}</td>
                  <td>
                    <input type="number" :placeholder="$t('msgbox.image_editor.height')" class="input input-bordered w-full"
                      v-model.number="resizedHeight" :disabled="cropStatus==1"
                      @keypress="onNumberKeyPress"
                      @blur="handleResizeInput('height')"
                    />
                  </td>
                </tr>
                <tr>
                  <td>{{ $t('msgbox.image_editor.percentage') }}</td>
                  <td class="flex items-center">
                    <input type="number" :placeholder="$t('msgbox.image_editor.percentage')" class="input input-bordered w-full"
                      v-model.number="resizedPercentage" :disabled="cropStatus==1"
                      @keypress="onNumberKeyPress"
                      @blur="handleResizeInput('percentage')"
                    />
                    <span class="pl-1 text-xs text-base-content/30">%</span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- options -->
          <div>
            <h3 class="mb-2">{{ $t('msgbox.image_editor.options') }}</h3>
            <!-- <input type="text" :placeholder="$t('msgbox.image_editor.save_as_placeholder')" v-model="newFileName" class="input input-bordered w-full px-2" :disabled="cropStatus==1" /> -->

            <table class="w-full text-sm text-nowrap border-separate border-spacing-2">
              <tbody>
                <tr>
                  <td>{{ $t('msgbox.image_editor.save_as') }}</td>
                  <td>
                    <select v-model="config.imageEditor.saveAs" class="select select-bordered w-full" :disabled="cropStatus==1">
                      <option v-for="option in fileSaveAsOptions" :value="option.value" :key="option.value">{{ option.label }}</option>
                    </select>
                  </td>
                </tr>
                <tr>
                  <td>{{ $t('msgbox.image_editor.format') }}</td>
                  <td>
                    <select v-model="config.imageEditor.format" class="select select-bordered w-full" :disabled="cropStatus==1">
                      <option v-for="option in fileFormatOptions" :value="option.value" :key="option.value">{{ option.label }}</option>
                    </select>
                  </td>
                </tr>
                <tr v-if="config.imageEditor.format == 0">
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
            <!-- <div class="text-[10px] text-base-content/30 flex flex-col gap-1 mt-2">
              <span>containerRect: {{ containerRect?.left.toFixed(0) }}, {{ containerRect?.top.toFixed(0) }}, {{ containerRect?.width.toFixed(0) }}, {{ containerRect?.height.toFixed(0) }}</span>
              <span>containerBounds: {{ containerBounds?.left.toFixed(0) }}, {{ containerBounds?.top.toFixed(0) }}, {{ containerBounds?.width.toFixed(0) }}, {{ containerBounds?.height.toFixed(0) }}</span>
              <span>imageRect: {{ imageRect?.left.toFixed(0) }}, {{ imageRect?.top.toFixed(0) }}, {{ imageRect?.width.toFixed(0) }}, {{ imageRect?.height.toFixed(0) }}</span>
              <span>cropBox:{{ cropBox.left.toFixed(0) }}, {{ cropBox.top.toFixed(0) }}, {{ cropBox.width.toFixed(0) }}, {{ cropBox.height.toFixed(0) }}</span> 
              <span>scale: {{ scale.toFixed(2) }}</span>
              <span>position: {{ position.left.toFixed(0) }}, {{ position.top.toFixed(0) }}</span>
              <span>crop: {{ crop.left.toFixed(0) }}, {{ crop.top.toFixed(0) }}, {{ crop.width.toFixed(0) }}, {{ crop.height.toFixed(0) }}</span>
              <span>resized: {{ resizedWidth.toFixed(0) }} x {{ resizedHeight.toFixed(0) }}</span>
              <span>resizedPercentage: {{ resizedPercentage.toFixed(0) }}%</span>
            </div> -->
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
      >{{ $t('msgbox.image_editor.ok') }}</button>
    </div>
  </ModalDialog>

  <ToolTip ref="toolTipRef" />

</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { useUIStore } from '@/stores/uiStore';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { getFolderPath, extractFileName, shortenFilename, getFullPath, combineFileName, getSelectOptions, getFileExtension, getAssetSrc } from '@/common/utils';
import { editImage, copyEditedImage } from '@/common/api';

import ToolTip from '@/components/ToolTip.vue';
import ModalDialog from '@/components/ModalDialog.vue';
import TButton from '@/components/TButton.vue';
import SliderInput from '@/components/SliderInput.vue';

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
  IconRestore,
} from '@/common/icons';

const props = defineProps({
  fileInfo: {
    type: Object,
    required: true,
  },
});

/// i18n
const { messages } = useI18n();
const localeMsg = computed(() => messages.value[config.settings.language] as any);

const uiStore = useUIStore();
const emit = defineEmits(['success', 'failed', 'cancel']);

const toolTipRef = ref(null);
const isProcessing = ref(false);  // show processing status
const activeTab = ref('adjust'); // 'adjust', 'export'

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
const selectedFilter = ref(''); // '', 'grayscale', 'sepia', 'invert'
const brightness = ref(0); // -100 to 100
const contrast = ref(0);   // -100 to 100
const blur = ref(0);       // 0 to 10
const hue = ref(0);        // -180 to 180
const saturation = ref(100); // 0 to 200 (percent)

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
  requestAnimationFrame(() => {
    enableTransition.value = true;
    isProcessing.value = false;
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

  resetAdjustments();
};

const resetAdjustments = () => {
  selectedFilter.value = '';
  brightness.value = 0;
  contrast.value = 0;
  blur.value = 0;
  hue.value = 0;
  saturation.value = 100;
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

const setEditParams = () => {
  let name = newFileName.value;
  if(config.imageEditor.saveAs === 1) { // 1: Save as new file
    name += "_edited";
  }

  const fileName = combineFileName(name, fileFormatOptions.value[config.imageEditor.format].label.toLowerCase());
  const destFilePath = getFullPath(getFolderPath(props.fileInfo.file_path), fileName);
  const orientation = props.fileInfo.e_orientation || 1;

  return {
    sourceFilePath: props.fileInfo.file_path,
    destFilePath: destFilePath,
    outputFormat: fileFormatOptions.value[config.imageEditor.format].label.toLowerCase(),
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
      uiStore.updateFileVersion(props.fileInfo.file_path);
      emit('success');
    } else {
      emit('failed');
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
