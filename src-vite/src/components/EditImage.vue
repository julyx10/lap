<template>

  <ModalDialog :title="`${$t('msgbox.image_editor.title')} - ${shortenFilename(props.fileInfo.name, 32)}`" :width="1040" @cancel="clickCancel">
    <div class="h-[560px] flex gap-4 select-none">
      <div class="flex-1 min-w-0 h-full flex items-start">
        <div ref="containerRef" class="relative w-full aspect-4/3 max-h-full rounded-box overflow-hidden border border-base-content/5 bg-base-300/30 shadow-sm cursor-default">
          <transition name="fade">
            <div v-if="isProcessing" class="absolute inset-0 z-50 flex items-center justify-center bg-base-100/55 backdrop-blur-sm">
              <span class="loading loading-dots text-primary"></span>
            </div>
          </transition>

          <img ref="imageRef" :src="imageSrc" :style="imageStyle" draggable="false" @load="onImageLoad" />

          <div v-if="cropStatus === 1 || cropApplied"
            :class="[
              cropStatus === 1 ? 'crop-box-active' : 'crop-box-done',
              cropStatus === 1
                ? (
                  cropBoxFixed
                    ? (isDragging ? 'cursor-grabbing no-transition' : 'cursor-grab')
                    : (isDragging ? 'cursor-move no-transition' : 'cursor-move')
                )
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

      <div class="w-[268px] flex flex-col gap-3 overflow-y-auto">

        <section class="rounded-box p-3 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
          <div class="flex items-center justify-between gap-2">
            <div class="text-[11px] font-bold uppercase tracking-[0.22em] text-base-content/35">{{ $t('msgbox.image_editor.transform') }}</div>
            <TButton
              buttonSize="small"
              :icon="IconRestore"
              :disabled="cropStatus === 1 || !hasEditImageChanges || cropApplied"
              :tooltip="$t('msgbox.image_editor.reset')"
              @click="clickRestoreAll"
            />
          </div>

          <div class="flex gap-3">
            <TButton
              :icon="IconRotateLeft"
              :disabled="cropStatus === 1 || cropApplied"
              :tooltip="$t('msgbox.image_editor.rotate_left')"
              @click="clickRotate(-90)"
            />
            <TButton
              :icon="IconRotateRight"
              :disabled="cropStatus === 1 || cropApplied"
              :tooltip="$t('msgbox.image_editor.rotate_right')"
              @click="clickRotate(90)"
            />
            <TButton
              :icon="IconFlipHorizontal"
              :disabled="cropStatus === 1 || cropApplied"
              :tooltip="$t('msgbox.image_editor.flip_horizontal')"
              @click="clickFlipX"
            />
            <TButton
              :icon="IconFlipVertical"
              :disabled="cropStatus === 1 || cropApplied"
              :tooltip="$t('msgbox.image_editor.flip_vertical')"
              @click="clickFlipY"
            />
          </div>
        </section>

        <section class="rounded-box p-3 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
          <div class="flex items-center justify-between gap-2">
            <div class="text-[11px] font-bold uppercase tracking-[0.22em] text-base-content/35">{{ $t('msgbox.image_editor.crop') }}</div>
            <TButton
              buttonSize="small"
              :icon="IconRestore"
              :disabled="cropStatus === 0 && !cropApplied"
              :tooltip="$t('msgbox.image_editor.reset')"
              @click="clearCrop"
            />
          </div>

          <div v-if="cropStatus === 0" class="flex items-center gap-2">
            <TButton
              :icon="IconCrop"
              :selected="cropApplied"
              :tooltip="cropApplied ? $t('msgbox.image_editor.restore') : $t('msgbox.image_editor.crop')"
              @click="toggleCropMode"
            />
            <div class="text-xs leading-5 text-base-content/45">
              {{ cropApplied ? $t('msgbox.image_editor.crop_applied_hint') : $t('msgbox.image_editor.crop_hint') }}
            </div>
          </div>

          <div v-else class="space-y-3">
            <div class="flex items-center gap-1">
              <TButton
                buttonSize="small"
                :icon="IconClose"
                :tooltip="$t('msgbox.image_editor.cancel_crop')"
                @click="clickCancelCrop"
              />
              
              <select v-model="config.imageEditor.cropShape" class="select select-bordered select-sm flex-1 min-w-0" :disabled="cropBoxFixed" @change="onChangeCropShape">
                <option v-for="option in cropShapeOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>

              <TButton
                buttonSize="small"
                :icon="IconCropLandscape"
                :disabled="cropBoxFixed"
                :tooltip="isPortrait ? $t('msgbox.image_editor.crop_shape_portrait') : $t('msgbox.image_editor.crop_shape_landscape')"
                :iconStyle="{ transform: `rotate(${isPortrait ? 90 : 0}deg)` }"
                @click="togglePortraitAndLandscape"
              />
              
              <TButton
                buttonSize="small"
                :icon="cropBoxFixed ? IconZoomOut : IconZoomIn"
                :tooltip="cropBoxFixed ? $t('msgbox.image_editor.zoom') : $t('msgbox.image_editor.zoom')"
                @click="toggleCropBoxFixed"
              />

              <TButton
                buttonSize="small"
                :icon="IconOk"
                :tooltip="$t('msgbox.image_editor.confirm_crop')"
                @click="clickDoCrop"
              />
            </div>



          </div>
        </section>

        <section class="rounded-box p-3 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
          <div class="flex items-center justify-between gap-2">
            <div class="text-[11px] font-bold uppercase tracking-[0.22em] text-base-content/35">{{ $t('msgbox.image_editor.resize') }}</div>
            <TButton
              buttonSize="small"
              :icon="IconRestore"
              :disabled="cropStatus === 1 || !hasResizeChanges"
              :tooltip="$t('msgbox.image_editor.reset')"
              @click="resetResize"
            />
          </div>

          <div class="grid grid-cols-2 gap-2">
            <div class="form-control w-full">
              <label class="label py-1">
                <span class="label-text text-xs font-medium opacity-70">{{ $t('msgbox.image_editor.width') }}</span>
              </label>
              <input
                v-model="resizeWidthInput"
                type="number"
                min="1"
                :max="maxResizeWidth"
                step="1"
                inputmode="numeric"
                class="input input-bordered input-sm w-full"
                :disabled="cropStatus === 1"
                @input="handleResizeWidthInput"
              />
            </div>

            <div class="form-control w-full">
              <label class="label py-1">
                <span class="label-text text-xs font-medium opacity-70">{{ $t('msgbox.image_editor.height') }}</span>
              </label>
              <input
                v-model="resizeHeightInput"
                type="number"
                min="1"
                :max="maxResizeHeight"
                step="1"
                inputmode="numeric"
                class="input input-bordered input-sm w-full"
                :disabled="cropStatus === 1"
                @input="handleResizeHeightInput"
              />
            </div>
          </div>

          <div class="flex items-center justify-between rounded-box bg-base-100/35 px-3 py-2">
            <span class="text-xs text-base-content/65">{{ $t('msgbox.image_editor.keep_aspect_ratio') }}</span>
            <input type="checkbox" class="toggle toggle-primary toggle-sm shrink-0" v-model="keepAspectRatio" :disabled="cropStatus === 1" />
          </div>
        </section>

        <section class="rounded-box p-3 space-y-3 bg-base-300/30 border border-base-content/5 shadow-sm">
          <div class="text-[11px] font-bold uppercase tracking-[0.22em] text-base-content/35">{{ $t('msgbox.image_editor.save_file') }}</div>

          <div class="space-y-3">
            <div class="form-control w-full">
              <select v-model="config.imageEditor.saveAs" class="select select-bordered select-sm w-full" :disabled="cropStatus===1">
                <option v-for="option in fileSaveAsOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
            </div>

            <div v-if="config.imageEditor.saveAs !== 0" class="grid grid-cols-2 gap-2">
              <div class="form-control w-full">
                <label class="label py-1">
                  <span class="label-text text-xs font-medium opacity-70">{{ $t('msgbox.image_editor.format') }}</span>
                </label>
                <select v-model="config.imageEditor.format" class="select select-bordered select-sm w-full" :disabled="cropStatus===1">
                  <option v-for="option in fileFormatOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
                </select>
              </div>

              <div v-if="config.imageEditor.format == 0" class="form-control w-full">
                <label class="label py-1">
                  <span class="label-text text-xs font-medium opacity-70">{{ $t('msgbox.image_editor.quality') }}</span>
                </label>
                <select v-model="config.imageEditor.quality" class="select select-bordered select-sm w-full" :disabled="cropStatus===1">
                  <option v-for="option in fileQualityOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
                </select>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>

    <div class="mt-1 flex justify-end space-x-4">
      <button
        class="px-4 py-1 rounded-box hover:bg-base-100 hover:text-base-content cursor-pointer"
        @click="clickCancel"
      >{{ $t('msgbox.image_editor.cancel') }}</button>
      <button
        :class="[
          'px-4 py-1 rounded-box',
          cropStatus === 1 || isProcessing
            ? 'text-base-content/30 cursor-default'
            : 'hover:bg-primary hover:text-base-100 cursor-pointer'
        ]"
        @click="clickSave"
      >{{ config.imageEditor.saveAs === 1 ? $t('msgbox.image_editor.save_as_new') : $t('msgbox.image_editor.overwrite') }}</button>
    </div>
  </ModalDialog>

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
import { editImage, checkFileExists } from '@/common/api';

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
  IconRestore,
} from '@/common/icons';

const props = defineProps({
  fileInfo: {
    type: Object,
    required: true,
  },
});

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const uiStore = useUIStore();
const emit = defineEmits(['success', 'failed', 'cancel']);

const isProcessing = ref(false);

const containerRef = ref<HTMLElement | null>(null);
const containerRect = ref<DOMRect | null>(null);
const containerBounds = ref({ top: 0, left: 0, width: 0, height: 0 });
const containerPadding = 5;

const imageRef = ref<HTMLImageElement | null>(null);
const imageRect = ref<DOMRect | null>(null);
const imageRectOriginal = ref<DOMRect | null>(null);
const imageSrc = ref('');
const imageWidth = ref(0);
const imageHeight = ref(0);

const enableTransition = ref(false);
const position = ref({ left: 0, top: 0 });
const isFlippedX = ref(false);
const isFlippedY = ref(false);
const scale = ref(1);
const rotate = ref(0);

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
  transition: enableTransition.value ? 'transform 0.3s ease' : 'none',
}));

const cropStatus = ref(0);
const cropApplied = ref(false);
const isPortrait = ref(false);
const cropBoxFixed = ref(false);

const cropBox = ref({ left: 0, top: 0, width: 0, height: 0 });
const crop = ref({ left: 0, top: 0, width: 0, height: 0 });

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

const baseOutputWidth = computed(() => {
  if ((cropStatus.value === 1 || cropApplied.value) && crop.value.width > 0) {
    return crop.value.width;
  }
  return rotate.value % 180 !== 0 ? imageHeight.value : imageWidth.value;
});
const baseOutputHeight = computed(() => {
  if ((cropStatus.value === 1 || cropApplied.value) && crop.value.height > 0) {
    return crop.value.height;
  }
  return rotate.value % 180 !== 0 ? imageWidth.value : imageHeight.value;
});
const resizeWidthInput = ref('');
const resizeHeightInput = ref('');
const keepAspectRatio = ref(true);
const resizeAspectRatio = computed(() => {
  if (!baseOutputWidth.value || !baseOutputHeight.value) return 1;
  return baseOutputWidth.value / baseOutputHeight.value;
});
const parsedResizeWidth = computed(() => {
  const width = Number.parseInt(resizeWidthInput.value, 10);
  return Number.isFinite(width) && width > 0 ? width : null;
});
const parsedResizeHeight = computed(() => {
  const height = Number.parseInt(resizeHeightInput.value, 10);
  return Number.isFinite(height) && height > 0 ? height : null;
});
const maxResizeWidth = computed(() => Math.max(1, baseOutputWidth.value));
const maxResizeHeight = computed(() => Math.max(1, baseOutputHeight.value));
const hasResizeChanges = computed(() => {
  return (
    resizeWidthInput.value !== String(baseOutputWidth.value) ||
    resizeHeightInput.value !== String(baseOutputHeight.value) ||
    !keepAspectRatio.value
  );
});
const resizeOutput = computed(() => {
  const widthInput = parsedResizeWidth.value;
  const heightInput = parsedResizeHeight.value;
  const baseWidth = baseOutputWidth.value;
  const baseHeight = baseOutputHeight.value;
  const ratio = resizeAspectRatio.value || 1;

  if (!widthInput && !heightInput) {
    return { width: baseWidth, height: baseHeight, hasResize: false };
  }

  if (keepAspectRatio.value) {
    if (widthInput && heightInput) {
      return { width: widthInput, height: heightInput, hasResize: true };
    }
    if (widthInput) {
      return { width: widthInput, height: Math.max(1, Math.round(widthInput / ratio)), hasResize: true };
    }
    if (heightInput) {
      return { width: Math.max(1, Math.round(heightInput * ratio)), height: heightInput, hasResize: true };
    }
  }

  return {
    width: widthInput || baseWidth,
    height: heightInput || baseHeight,
    hasResize: !!widthInput || !!heightInput,
  };
});
const hasEditImageChanges = computed(() =>
  rotate.value % 360 !== 0 ||
  isFlippedX.value ||
  isFlippedY.value
);

const cropShapeOptions = computed(() => {
  if (isPortrait.value) {
    return [
      { value: '0', label: localeMsg.value.msgbox.image_editor.crop_shape_custom },
      { value: '1', label: '1:1' },
      { value: '2', label: '3:4' },
      { value: '3', label: '2:3' },
      { value: '4', label: '10:16' },
      { value: '5', label: '9:16' },
      { value: '6', label: '1:2' },
    ];
  }

  return [
    { value: '0', label: localeMsg.value.msgbox.image_editor.crop_shape_custom },
    { value: '1', label: '1:1' },
    { value: '2', label: '4:3' },
    { value: '3', label: '3:2' },
    { value: '4', label: '16:10' },
    { value: '5', label: '16:9' },
    { value: '6', label: '2:1' },
  ];
});

const newFileName = ref(props.fileInfo.name.substring(0, props.fileInfo.name.lastIndexOf('.')) || props.fileInfo.name);

const fileSaveAsOptions = computed(() => getSelectOptions(localeMsg.value.msgbox.image_editor.save_as_options));
const fileFormatOptions = computed(() => getSelectOptions(localeMsg.value.msgbox.image_editor.format_options));
const fileQualityOptions = computed(() => getSelectOptions(localeMsg.value.msgbox.image_editor.quality_options));

const showOverwriteConfirm = ref(false);

const handleOverwriteConfirm = () => {
  showOverwriteConfirm.value = false;

  const originalPath = props.fileInfo.file_path;
  const ext = getFileExtension(props.fileInfo.name).toLowerCase();
  const outputFormat = (ext === 'jpg' || ext === 'jpeg') ? 'jpg' : ext;

  executeSave({
    destFilePath: originalPath,
    outputFormat,
  });
};

const handleOverwriteCancel = () => {
  showOverwriteConfirm.value = false;
  isProcessing.value = false;
};

const handleResizeWidthInput = () => {
  const width = parsedResizeWidth.value;
  if (!width) return;

  const clampedWidth = Math.min(maxResizeWidth.value, Math.max(1, width));
  if (clampedWidth !== width) {
    resizeWidthInput.value = String(clampedWidth);
  }

  if (!keepAspectRatio.value) return;
  resizeHeightInput.value = String(Math.min(maxResizeHeight.value, Math.max(1, Math.round(clampedWidth / resizeAspectRatio.value))));
};

const handleResizeHeightInput = () => {
  const height = parsedResizeHeight.value;
  if (!height) return;

  const clampedHeight = Math.min(maxResizeHeight.value, Math.max(1, height));
  if (clampedHeight !== height) {
    resizeHeightInput.value = String(clampedHeight);
  }

  if (!keepAspectRatio.value) return;
  resizeWidthInput.value = String(Math.min(maxResizeWidth.value, Math.max(1, Math.round(clampedHeight * resizeAspectRatio.value))));
};

const resetResize = () => {
  resizeWidthInput.value = String(baseOutputWidth.value);
  resizeHeightInput.value = String(baseOutputHeight.value);
  keepAspectRatio.value = true;
};

watch(
  () => [baseOutputWidth.value, baseOutputHeight.value],
  ([width, height]) => {
    resizeWidthInput.value = String(width);
    resizeHeightInput.value = String(height);
  },
  { immediate: true }
);

watch(
  () => keepAspectRatio.value,
  (enabled) => {
    if (!enabled || !parsedResizeWidth.value) return;
    resizeHeightInput.value = String(Math.max(1, Math.round(parsedResizeWidth.value / resizeAspectRatio.value)));
  }
);

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('EditImage');

  isProcessing.value = true;
  initEditImage();
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('EditImage');
});

const onImageLoad = async () => {
  await nextTick();

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

const initEditImage = () => {
  imageSrc.value = getAssetSrc(props.fileInfo.file_path);
  imageWidth.value = props.fileInfo.width;
  imageHeight.value = props.fileInfo.height;
  isPortrait.value = imageHeight.value > imageWidth.value;

  containerRect.value = containerRef.value?.getBoundingClientRect() || null;
  if (!containerRect.value) return;

  containerBounds.value = {
    left: containerRect.value.left + containerPadding,
    top: containerRect.value.top + containerPadding,
    width: containerRect.value.width - containerPadding * 2,
    height: containerRect.value.height - containerPadding * 2,
  };

  enableTransition.value = false;

  if (uiStore.activeAdjustments.filePath === props.fileInfo.file_path) {
    const adj = uiStore.activeAdjustments;
    rotate.value = adj.rotate || 0;
    isFlippedX.value = !!adj.flipX;
    isFlippedY.value = !!adj.flipY;
  } else {
    rotate.value = 0;
    isFlippedX.value = false;
    isFlippedY.value = false;
  }
};

const clickStartCrop = () => {
  cropStatus.value = 1;
  cropApplied.value = false;
  cropBoxFixed.value = false;
  initCropBox();
};

const toggleCropMode = () => {
  if (cropStatus.value === 1) {
    clearCrop();
    return;
  }

  if (cropApplied.value) {
    clickRestoreCrop();
    return;
  }

  if (cropStatus.value === 0) {
    clickStartCrop();
    return;
  }
};

const clearCrop = () => {
  cropStatus.value = 0;
  cropApplied.value = false;
  cropBoxFixed.value = false;
  crop.value = { left: 0, top: 0, width: 0, height: 0 };
  cropBox.value = { left: 0, top: 0, width: 0, height: 0 };
  fitImageToContainer();
};

const clickRestoreAll = () => {
  if (cropStatus.value === 1 || cropApplied.value) return;

  rotate.value = 0;
  isFlippedX.value = false;
  isFlippedY.value = false;
  isPortrait.value = imageHeight.value > imageWidth.value;
  fitImageToContainer();
};

const clickCancelCrop = () => {
  cropStatus.value = 0;
  cropApplied.value = false;
  crop.value = { left: 0, top: 0, width: 0, height: 0 };
  cropBox.value = { left: 0, top: 0, width: 0, height: 0 };
  fitImageToContainer();
};

const clickRestoreCrop = () => {
  cropStatus.value = 1;
  cropBoxFixed.value = false;
  fitImageToContainer();
};

const clickDoCrop = () => {
  cropApplied.value = true;
  cropBoxFixed.value = false;
  fitCropBoxToContainer();

  cropStatus.value = 0;
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

const initCropBox = () => {
  containerRect.value = containerRef.value?.getBoundingClientRect() || null;
  imageRect.value = imageRef.value?.getBoundingClientRect() || null;
  if (!imageRect.value || !containerRect.value) return;

  const selectedShape = cropShapeOptions.value.find(option => option.value === String(config.imageEditor.cropShape) && option.value !== '0');
  if (selectedShape && selectedShape.label) {
    const parts = selectedShape.label.split(':');
    const aspectRatio = parseInt(parts[0]) / parseInt(parts[1]);

    let newWidth;
    let newHeight;
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

  updateCropFromCropBox();
};

const updateCropFromCropBox = () => {
  if (cropBox.value.width === 0 || cropBox.value.height === 0) {
    crop.value = { left: 0, top: 0, width: 0, height: 0 };
    return;
  }

  containerRect.value = containerRef.value?.getBoundingClientRect() || null;
  imageRect.value = imageRef.value?.getBoundingClientRect() || null;
  if (!imageRect.value || !containerRect.value) return;

  const imgWidth = rotate.value % 180 === 0 ? imageWidth.value : imageHeight.value;
  const imgHeight = rotate.value % 180 === 0 ? imageHeight.value : imageWidth.value;

  const scaleX = imgWidth / imageRect.value.width;
  const scaleY = imgHeight / imageRect.value.height;

  crop.value = {
    left: Math.round(scaleX * (cropBox.value.left + containerRect.value.left - imageRect.value.left)),
    top: Math.round(scaleY * (cropBox.value.top + containerRect.value.top - imageRect.value.top)),
    width: Math.round(scaleX * cropBox.value.width),
    height: Math.round(scaleY * cropBox.value.height),
  };
};

const updateCropBoxFromCrop = () => {
  if (crop.value.width === 0 || crop.value.height === 0) {
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
};

const scaleFit = (imgWidth: number, imgHeight: number) => {
  scale.value = Math.min(containerBounds.value.width / imgWidth, containerBounds.value.height / imgHeight);
};

const fitImageToContainer = () => {
  containerRect.value = containerRef.value?.getBoundingClientRect() || null;
  if (!containerRect.value) return;

  position.value = {
    left: (containerRect.value.width - imageWidth.value) / 2,
    top: (containerRect.value.height - imageHeight.value) / 2,
  };

  rotate.value % 180 !== 0
    ? scaleFit(imageHeight.value, imageWidth.value)
    : scaleFit(imageWidth.value, imageHeight.value);

  updateCropBoxFromCrop();
};

const fitCropBoxToContainer = () => {
  if (!containerBounds.value || !containerRect.value) return;

  imageRectOriginal.value = imageRect.value;
  const oldScale = scale.value;

  scale.value = Math.min(
    (containerBounds.value.width / cropBox.value.width) * oldScale,
    (containerBounds.value.height / cropBox.value.height) * oldScale,
  );

  position.value = {
    left: position.value.left + (containerRect.value.width / 2 - (cropBox.value.left + cropBox.value.width / 2)) * scale.value / oldScale,
    top: position.value.top + (containerRect.value.height / 2 - (cropBox.value.top + cropBox.value.height / 2)) * scale.value / oldScale,
  };

  const newCropBoxWidth = cropBox.value.width * scale.value / oldScale;
  const newCropBoxHeight = cropBox.value.height * scale.value / oldScale;
  cropBox.value = {
    left: (containerRect.value.width - newCropBoxWidth) / 2,
    top: (containerRect.value.height - newCropBoxHeight) / 2,
    width: newCropBoxWidth,
    height: newCropBoxHeight,
  };

  imageRef.value?.addEventListener('transitionend', updateCropFromCropBox, { once: true });
};

const clickRotate = (degree: number) => {
  rotate.value += degree;
  isPortrait.value = !isPortrait.value;
  scaleFit(
    rotate.value % 180 !== 0 ? imageHeight.value : imageWidth.value,
    rotate.value % 180 !== 0 ? imageWidth.value : imageHeight.value,
  );
};

const clickFlipX = () => {
  rotate.value % 180 !== 0
    ? isFlippedY.value = !isFlippedY.value
    : isFlippedX.value = !isFlippedX.value;
};

const clickFlipY = () => {
  rotate.value % 180 !== 0
    ? isFlippedX.value = !isFlippedX.value
    : isFlippedY.value = !isFlippedY.value;
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
      const initialImageLeft = initialImageRect.left - containerRect.value.left;
      const initialImageRight = initialImageLeft + initialImageRect.width;
      const maxDx = cropBox.value.left - initialImageLeft;
      const minDx = (cropBox.value.left + cropBox.value.width) - initialImageRight;
      const clampedDx = Math.max(minDx, Math.min(dx, maxDx));

      const initialImageTop = initialImageRect.top - containerRect.value.top;
      const initialImageBottom = initialImageTop + initialImageRect.height;
      const maxDy = cropBox.value.top - initialImageTop;
      const minDy = (cropBox.value.top + cropBox.value.height) - initialImageBottom;
      const clampedDy = Math.max(minDy, Math.min(dy, maxDy));

      position.value.left = initialImagePosition.left + clampedDx;
      position.value.top = initialImagePosition.top + clampedDy;
    } else if (dragHandle.value === 'move') {
      if (!imageRect.value) return;
      const imageLeft = imageRect.value.left - containerRect.value.left;
      const imageTop = imageRect.value.top - containerRect.value.top;
      const imageRight = imageLeft + imageRect.value.width;
      const imageBottom = imageTop + imageRect.value.height;

      let newLeft = initialCropBoxData.left + dx;
      let newTop = initialCropBoxData.top + dy;

      if (newLeft < imageLeft) newLeft = imageLeft;
      if (newTop < imageTop) newTop = imageTop;
      if (newLeft + initialCropBoxData.width > imageRight) newLeft = imageRight - initialCropBoxData.width;
      if (newTop + initialCropBoxData.height > imageBottom) newTop = imageBottom - initialCropBoxData.height;

      cropBox.value.left = newLeft;
      cropBox.value.top = newTop;
    } else {
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

      if (
        proposedBox.width >= 10 &&
        proposedBox.height >= 10 &&
        proposedBox.left >= imageLeft &&
        proposedBox.top >= imageTop &&
        proposedBox.left + proposedBox.width <= imageRight + 0.1 &&
        proposedBox.top + proposedBox.height <= imageBottom + 0.1
      ) {
        cropBox.value = proposedBox;
      }
    }

    updateCropFromCropBox();
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
  if (!uiStore.isInputActive('EditImage')) return;

  switch (event.key) {
    case 'Enter':
      if (cropStatus.value === 1) {
        clickDoCrop();
      } else {
        clickSave();
      }
      event.preventDefault();
      event.stopPropagation();
      break;
    case 'Escape':
      if (cropStatus.value === 1) {
        clickCancelCrop();
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
  emit('cancel');
};

const setEditParams = (overrides: { fileName?: string; destFilePath?: string; outputFormat?: string } = {}) => {
  let name = overrides.fileName || newFileName.value;
  let outputFormat = overrides.outputFormat || fileFormatOptions.value[config.imageEditor.format].label.toLowerCase();

  let destFilePath = overrides.destFilePath;
  if (!destFilePath) {
    destFilePath = getFullPath(getFolderPath(props.fileInfo.file_path), combineFileName(name, outputFormat));
  }

  return {
    sourceFilePath: props.fileInfo.file_path,
    destFilePath,
    outputFormat,
    quality: [90, 80, 60][config.imageEditor.quality] || 80,
    orientation: props.fileInfo.e_orientation || 1,
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
      width: resizeOutput.value.hasResize && resizeOutput.value.width !== baseOutputWidth.value ? resizeOutput.value.width : null,
      height: resizeOutput.value.hasResize && resizeOutput.value.height !== baseOutputHeight.value ? resizeOutput.value.height : null,
    },
    filter: null,
    brightness: null,
    contrast: null,
    blur: null,
    hue_rotate: null,
    saturation: null,
  };
};

const executeSave = async (overrides: { fileName?: string; destFilePath?: string; outputFormat?: string } = {}) => {
  isProcessing.value = true;
  let success = false;
  try {
    success = await editImage(setEditParams(overrides));
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

  if (config.imageEditor.saveAs === 1) {
    isProcessing.value = true;
    try {
      const folderPath = getFolderPath(props.fileInfo.file_path);
      const ext = fileFormatOptions.value[config.imageEditor.format].label.toLowerCase();
      const baseName = newFileName.value;

      let counter = 1;
      let candidateName = `${baseName}_${counter}`;
      let candidatePath = getFullPath(folderPath, combineFileName(candidateName, ext));

      while (await checkFileExists(candidatePath)) {
        counter++;
        candidateName = `${baseName}_${counter}`;
        candidatePath = getFullPath(folderPath, combineFileName(candidateName, ext));
      }

      await executeSave({
        fileName: candidateName,
        destFilePath: candidatePath,
      });
    } catch {
      isProcessing.value = false;
      emit('failed');
    }
  } else {
    showOverwriteConfirm.value = true;
  }
};

</script>

<style scoped>
.crop-box-active {
  position: absolute;
  border: 1px solid #fff;
  box-shadow: 0 0 0 9999px color-mix(in srgb, var(--color-base-200) 80%, transparent);
  box-sizing: border-box;
  will-change: transform;
  transition: all 0.3s ease;
}

.crop-box-done {
  position: absolute;
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
