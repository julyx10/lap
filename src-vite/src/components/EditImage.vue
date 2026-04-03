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

          <div
            v-if="activeEditorTab === 'adjust'"
            class="absolute top-2 right-2 z-40"
            @pointerdown.stop="handleComparePointerDown"
            @pointerup.stop="handleComparePointerUp"
            @pointerleave.stop="handleComparePointerUp"
            @pointercancel.stop="handleComparePointerUp"
          >
            <TButton
              buttonSize="small"
              :icon="IconInformation"
              :selected="isComparingOriginal"
              :disabled="!hasAdjustmentChanges"
              :tooltip="$t('msgbox.image_editor.compare')"
            />
          </div>

          <img
            v-if="imageSrc"
            ref="imageRef"
            :src="imageSrc"
            :style="imageStyle"
            class="block"
            draggable="false"
            @load="onImageLoad"
          />

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

      <div
        class="w-[268px] flex flex-col gap-3 overflow-y-auto"
        :class="isProcessing ? 'pointer-events-none opacity-60' : ''"
      >
        <div class="border-b border-base-content/10 pb-2">
          <div role="tablist" class="sidebar-header-tabs">
            <button
              role="tab"
              :class="[
                'sidebar-header-tab',
                activeEditorTab === 'edit' ? 'tab-active' : '',
                cropStatus === 1 || isProcessing ? 'opacity-50 cursor-default' : '',
              ]"
              :disabled="cropStatus === 1 || isProcessing"
              @click="setActiveEditorTab('edit')"
            >{{ $t('msgbox.image_editor.tab_edit') }}</button>
            <button
              role="tab"
              :class="[
                'sidebar-header-tab',
                activeEditorTab === 'adjust' ? 'tab-active' : '',
                cropStatus === 1 || isProcessing ? 'opacity-50 cursor-default' : '',
              ]"
              :disabled="cropStatus === 1 || isProcessing"
              @click="setActiveEditorTab('adjust')"
            >{{ $t('msgbox.image_editor.tab_adjust') }}</button>
          </div>
        </div>

        <template v-if="activeEditorTab === 'edit'">
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
        </template>

        <template v-else>
        <section class="rounded-box p-3 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
          <div class="flex items-center justify-between gap-2">
            <div class="text-[11px] font-bold uppercase tracking-[0.22em] text-base-content/35">{{ $t('msgbox.image_editor.histogram') }}</div>
          </div>

          <div class="relative w-full aspect-4/1 px-0.5">
            <svg viewBox="0 0 256 64" class="w-full h-full text-primary" preserveAspectRatio="none">
              <defs>
                <linearGradient id="histGradientEdit" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stop-color="currentColor" stop-opacity="0.6" />
                  <stop offset="100%" stop-color="currentColor" stop-opacity="0.1" />
                </linearGradient>
              </defs>
              <g class="text-base-content/20">
                <line x1="64" y1="0" x2="64" y2="64" stroke="currentColor" stroke-width="0.5" />
                <line x1="128" y1="0" x2="128" y2="64" stroke="currentColor" stroke-width="0.5" />
                <line x1="192" y1="0" x2="192" y2="64" stroke="currentColor" stroke-width="0.5" />
              </g>
              <path :d="generateHistogramPath()" fill="url(#histGradientEdit)" class="transition-all duration-300" />
            </svg>
          </div>

          <div class="flex justify-between px-0.5 text-[8px] uppercase tracking-tighter font-black text-base-content/25">
            <span>{{ $t('msgbox.image_editor.shadows') }}</span>
            <span>{{ $t('msgbox.image_editor.midtones') }}</span>
            <span>{{ $t('msgbox.image_editor.highlights') }}</span>
          </div>
        </section>

        <section class="rounded-box p-3 space-y-2 border border-base-content/5 shadow-sm bg-base-300/30">
          <div class="flex items-center justify-between gap-2">
            <span class="text-[11px] font-bold uppercase tracking-[0.22em] text-base-content/35">{{ $t('msgbox.image_editor.presets.title') }}</span>
            <TButton
              buttonSize="small"
              :icon="IconRestore"
              :disabled="!hasAdjustmentChanges"
              :tooltip="$t('msgbox.image_editor.reset')"
              @click.stop="resetAdjustments"
            />
          </div>

          <div ref="presetStripRef" class="flex gap-2 overflow-x-auto overflow-y-hidden flex-nowrap">
            <div
              v-for="option in presetOptions"
              :key="option.value"
              :data-preset="option.value"
              class="shrink-0 w-[76px] group cursor-pointer"
              @click="selectedPreset = option.value"
            >
              <div
                :class="[
                  'aspect-4/3 rounded-box border-2 transition-all duration-200 flex items-center justify-center overflow-hidden mb-1 relative',
                  selectedPreset === option.value ? 'border-primary ring-2 ring-primary/20' : 'border-base-content/5 hover:border-base-content/20',
                ]"
              >
                <div class="w-full h-full bg-base-300 flex items-center justify-center relative overflow-hidden rounded-[inherit] isolation-isolate">
                  <img
                    v-if="props.fileInfo.thumbnail"
                    :src="props.fileInfo.thumbnail"
                    class="w-full h-full object-cover pointer-events-none rounded-[inherit] block"
                    :style="{ ...getPresetThumbnailStyle(option.value), transform: 'translateZ(0)' }"
                  />
                  <IconPalette v-else class="w-4 h-4 text-base-content/10" />
                </div>
              </div>
              <div
                :class="[
                  'text-[9px] text-center truncate font-medium transition-colors uppercase tracking-tight',
                  selectedPreset === option.value ? 'text-primary' : 'text-base-content/50 group-hover:text-base-content',
                ]"
              >
                {{ option.label }}
              </div>
            </div>
          </div>
        </section>

        <section class="rounded-box p-3 space-y-2 border border-base-content/5 shadow-sm bg-base-300/30">
          <div class="flex items-center justify-between gap-2">
            <span class="text-[11px] font-bold uppercase tracking-[0.22em] text-base-content/35">{{ $t('msgbox.image_editor.adjustments') }}</span>
            <TButton
              buttonSize="small"
              :icon="IconRestore"
              :disabled="!hasAdjustmentChanges"
              :tooltip="$t('msgbox.image_editor.reset')"
              @click.stop="resetAdjustments"
            />
          </div>

          <div class="space-y-4 overflow-hidden">
            <div class="space-y-3">
              <div v-for="adj in lightSliders" :key="adj.key" class="grid grid-cols-[80px_minmax(0,1fr)] gap-x-4 items-center">
                <div class="font-medium text-base-content/40 tracking-wide text-xs">{{ adj.label }}</div>
                <div class="flex items-center gap-2 pr-2 min-w-0">
                  <SliderInput v-model="adj.model.value" :min="adj.min" :max="adj.max" :step="adj.step" class="flex-1 min-w-0 w-full" />
                  <span class="text-[10px] font-mono text-base-content/60 w-8 text-right shrink-0">{{ adj.valueDisplay }}</span>
                </div>
              </div>
            </div>

            <div class="h-px bg-base-content/5 mx-1"></div>

            <div class="space-y-3">
              <div v-for="adj in colorSliders" :key="adj.key" class="grid grid-cols-[80px_minmax(0,1fr)] gap-x-4 items-center">
                <div class="font-medium text-base-content/40 tracking-wide text-xs">{{ adj.label }}</div>
                <div class="flex items-center gap-2 pr-2 min-w-0">
                  <SliderInput v-model="adj.model.value" :min="adj.min" :max="adj.max" :step="adj.step" class="flex-1 min-w-0 w-full" />
                  <span class="text-[10px] font-mono text-base-content/60 w-8 text-right shrink-0">{{ adj.valueDisplay }}</span>
                </div>
              </div>
            </div>
          </div>
        </section>
        </template>

        <section class="rounded-box p-3 space-y-3 bg-base-300/30 border border-base-content/5 shadow-sm">
          <div class="text-[11px] font-bold uppercase tracking-[0.22em] text-base-content/35">{{ $t('msgbox.image_editor.save_file') }}</div>

          <div class="space-y-3">
            <div class="form-control w-full">
                <select v-model="config.imageEditor.saveAs" class="select select-bordered select-sm w-full" :disabled="cropStatus===1 || !canOverwriteOriginal">
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
      >{{ !canOverwriteOriginal || config.imageEditor.saveAs === 1 ? $t('msgbox.image_editor.save_as_new') : $t('msgbox.image_editor.overwrite') }}</button>
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
import { getFolderPath, getFileExtension, shortenFilename, getFullPath, combineFileName, getSelectOptions, getAssetSrc, getPreviewUrl, shouldUseBackendPreview } from '@/common/utils';
import { editImage, checkFileExists } from '@/common/api';

import ModalDialog from '@/components/ModalDialog.vue';
import MessageBox from '@/components/MessageBox.vue';
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
  IconPalette,
  IconInformation,
} from '@/common/icons';

const props = defineProps({
  fileInfo: {
    type: Object,
    required: true,
  },
  initialImageSrc: {
    type: String,
    default: '',
  },
  initialTab: {
    type: String,
    default: 'edit',
  },
});

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const uiStore = useUIStore();
const emit = defineEmits(['success', 'failed', 'cancel']);

const isProcessing = ref(false);
const activeEditorTab = ref<'edit' | 'adjust'>('edit');

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
const isRawFile = computed(() => Number(props.fileInfo?.file_type || 0) === 3);
const normalizeRotate = (value: number) => {
  const normalized = Number(value || 0) % 360;
  return normalized < 0 ? normalized + 360 : normalized;
};
const initialDisplayRotate = computed(() => normalizeRotate(Number(props.fileInfo?.rotate || 0)));
const isPortraitForRotation = (width: number, height: number, rotation: number) => {
  const normalized = normalizeRotate(rotation);
  return normalized % 180 !== 0 ? width > height : height > width;
};
const usesBackendPreview = computed(() =>
  shouldUseBackendPreview(
    props.fileInfo?.name || props.fileInfo?.file_path || '',
    Number(props.fileInfo?.file_type || 0)
  )
);

const enableTransition = ref(false);
const position = ref({ left: 0, top: 0 });
const isFlippedX = ref(false);
const isFlippedY = ref(false);
const scale = ref(1);
const rotate = ref(0);
const compareHold = ref(false);
const brightness = ref(0);
const contrast = ref(0);
const saturation = ref(100);
const hue = ref(0);
const blur = ref(0);
const displayedHistogramBrightness = ref(0);
const displayedHistogramContrast = ref(0);
const selectedFilter = ref('');
const selectedPreset = ref('natural');
const presetStripRef = ref<HTMLElement | null>(null);
const EMPTY_HISTOGRAM = new Array(256).fill(0);
const histogramData = ref<number[]>([...EMPTY_HISTOGRAM]);
let isApplyingPreset = false;
let histogramToneAnimationFrame: number | null = null;
let skipNextCustomPresetLoad = false;
let histogramLoadId = 0;

const imageStyle = computed((): CSSProperties => ({
  display: 'block',
  width: `${imageWidth.value}px`,
  height: `${imageHeight.value}px`,
  maxWidth: 'none',
  maxHeight: 'none',
  position: 'absolute',
  filter: isComparingOriginal.value ? 'none' : adjustmentFilter.value,
  transform: `
    translate(${position.value.left}px, ${position.value.top}px)
    rotate(${rotate.value}deg)
    scaleX(${isFlippedX.value ? -1 : 1})
    scaleY(${isFlippedY.value ? -1 : 1})
    scale(${scale.value})
  `,
  transition: enableTransition.value ? 'transform 0.3s ease' : 'none',
  backfaceVisibility: 'hidden',
  willChange: 'transform, filter',
}));
const isComparingOriginal = computed(() => compareHold.value);
const adjustmentFilter = computed(() => {
  const filters = [
    `brightness(${100 + brightness.value}%)`,
    `contrast(${100 + contrast.value}%)`,
    `blur(${blur.value}px)`,
    `hue-rotate(${hue.value}deg)`,
    `saturate(${saturation.value}%)`,
  ];

  if (selectedFilter.value === 'grayscale') {
    filters.push('grayscale(100%)');
  } else if (selectedFilter.value === 'sepia') {
    filters.push('sepia(100%)');
  } else if (selectedFilter.value === 'invert') {
    filters.push('invert(100%)');
  }

  return filters.join(' ');
});

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
  normalizeRotate(rotate.value) !== initialDisplayRotate.value ||
  isFlippedX.value ||
  isFlippedY.value
);
const presets: Record<string, any> = {
  natural: { brightness: 0, contrast: 0, saturation: 100, hue: 0, blur: 0, filter: '' },
  vivid: { brightness: 0, contrast: 10, saturation: 120, hue: 0, blur: 0, filter: '' },
  muted: { brightness: 0, contrast: -10, saturation: 80, hue: 0, blur: 0, filter: '' },
  warm: { brightness: 5, contrast: 0, saturation: 100, hue: 5, blur: 0, filter: '' },
  cool: { brightness: 5, contrast: 0, saturation: 100, hue: -5, blur: 0, filter: '' },
  bw: { brightness: 0, contrast: 0, saturation: 0, hue: 0, blur: 0, filter: 'grayscale' },
  vintage: { brightness: 10, contrast: -10, saturation: 60, hue: 0, blur: 0, filter: 'sepia' },
  invert: { brightness: 0, contrast: 0, saturation: 100, hue: 0, blur: 0, filter: 'invert' },
  kodak: { brightness: 10, contrast: 15, saturation: 120, hue: -5, blur: 0, filter: '' },
  toyo: { brightness: 5, contrast: 0, saturation: 110, hue: 5, blur: 0, filter: '' },
  cinematic: { brightness: 0, contrast: 20, saturation: 80, hue: 0, blur: 0, filter: '' },
  dramatic: { brightness: 0, contrast: 30, saturation: 110, hue: 0, blur: 0, filter: '' },
  cyberpunk: { brightness: 10, contrast: 20, saturation: 130, hue: -15, blur: 0, filter: '' },
};

function resolvePresetKey(values: {
  brightness: number;
  contrast: number;
  saturation: number;
  hue: number;
  blur: number;
  filter: string;
}) {
  for (const [key, preset] of Object.entries(presets)) {
    if (
      preset.brightness === values.brightness &&
      preset.contrast === values.contrast &&
      preset.saturation === values.saturation &&
      preset.hue === values.hue &&
      preset.blur === values.blur &&
      preset.filter === values.filter
    ) {
      return key;
    }
  }

  return 'custom';
}

function getCurrentAdjustmentValues() {
  return {
    brightness: brightness.value,
    contrast: contrast.value,
    saturation: saturation.value,
    hue: hue.value,
    blur: blur.value,
    filter: selectedFilter.value,
  };
}

function getConfiguredCustomPreset() {
  return {
    brightness: Number(config.imageEditor.custom?.brightness ?? 0),
    contrast: Number(config.imageEditor.custom?.contrast ?? 0),
    saturation: Number(config.imageEditor.custom?.saturation ?? 100),
    hue: Number(config.imageEditor.custom?.hue ?? 0),
    blur: Number(config.imageEditor.custom?.blur ?? 0),
    filter: String(config.imageEditor.custom?.filter ?? ''),
  };
}

function persistCustomPreset(values = getCurrentAdjustmentValues()) {
  config.imageEditor.custom = {
    brightness: values.brightness,
    contrast: values.contrast,
    saturation: values.saturation,
    hue: values.hue,
    blur: values.blur,
    filter: values.filter,
  };
}

const presetOptions = computed(() => [
  { value: 'custom', label: localeMsg.value.msgbox.image_editor.presets.custom },
  { value: 'natural', label: localeMsg.value.msgbox.image_editor.presets.natural },
  { value: 'vivid', label: localeMsg.value.msgbox.image_editor.presets.vivid },
  { value: 'muted', label: localeMsg.value.msgbox.image_editor.presets.muted },
  { value: 'warm', label: localeMsg.value.msgbox.image_editor.presets.warm },
  { value: 'cool', label: localeMsg.value.msgbox.image_editor.presets.cool },
  { value: 'bw', label: localeMsg.value.msgbox.image_editor.presets.bw },
  { value: 'vintage', label: localeMsg.value.msgbox.image_editor.presets.vintage },
  { value: 'kodak', label: localeMsg.value.msgbox.image_editor.presets.kodak },
  { value: 'toyo', label: localeMsg.value.msgbox.image_editor.presets.toyo },
  { value: 'cinematic', label: localeMsg.value.msgbox.image_editor.presets.cinematic },
  { value: 'dramatic', label: localeMsg.value.msgbox.image_editor.presets.dramatic },
  { value: 'cyberpunk', label: localeMsg.value.msgbox.image_editor.presets.cyberpunk },
  { value: 'invert', label: localeMsg.value.msgbox.image_editor.presets.invert },
]);
const lightSliders = computed(() => [
  {
    key: 'brightness',
    label: localeMsg.value.msgbox.image_editor.brightness,
    model: brightness,
    min: -100,
    max: 100,
    step: 1,
    valueDisplay: `${brightness.value > 0 ? '+' : ''}${brightness.value}`,
  },
  {
    key: 'contrast',
    label: localeMsg.value.msgbox.image_editor.contrast,
    model: contrast,
    min: -100,
    max: 100,
    step: 1,
    valueDisplay: `${contrast.value > 0 ? '+' : ''}${contrast.value}`,
  },
]);
const colorSliders = computed(() => [
  {
    key: 'saturation',
    label: localeMsg.value.msgbox.image_editor.saturation,
    model: saturation,
    min: 0,
    max: 200,
    step: 1,
    valueDisplay: `${saturation.value}%`,
  },
  {
    key: 'hue',
    label: localeMsg.value.msgbox.image_editor.hue_rotate,
    model: hue,
    min: -180,
    max: 180,
    step: 1,
    valueDisplay: `${hue.value > 0 ? '+' : ''}${hue.value}`,
  },
  {
    key: 'blur',
    label: localeMsg.value.msgbox.image_editor.blur,
    model: blur,
    min: 0,
    max: 20,
    step: 1,
    valueDisplay: `${blur.value}`,
  },
]);
const hasAdjustmentChanges = computed(() => {
  const p = presets.natural;
  return (
    brightness.value !== p.brightness ||
    contrast.value !== p.contrast ||
    saturation.value !== p.saturation ||
    hue.value !== p.hue ||
    blur.value !== p.blur ||
    selectedFilter.value !== p.filter
  );
});

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

const fileSaveAsOptions = computed(() => {
  const options = getSelectOptions(localeMsg.value.msgbox.image_editor.save_as_options);
  return canOverwriteOriginal.value ? options : options.filter((option: any) => Number(option.value) === 1);
});
const fileFormatOptions = computed(() => getSelectOptions(localeMsg.value.msgbox.image_editor.format_options));
const fileQualityOptions = computed(() => getSelectOptions(localeMsg.value.msgbox.image_editor.quality_options));

const canOverwriteOriginal = computed(() => {
  const ext = getFileExtension(props.fileInfo?.name || props.fileInfo?.file_path || '').toLowerCase();
  return ['jpg', 'jpeg', 'png', 'webp'].includes(ext);
});

const showOverwriteConfirm = ref(false);

const handleOverwriteConfirm = () => {
  showOverwriteConfirm.value = false;

  if (!canOverwriteOriginal.value) {
    return;
  }

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
    resizeWidthInput.value = width > 0 ? String(width) : '';
    resizeHeightInput.value = height > 0 ? String(height) : '';
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

watch(selectedPreset, () => {
  if (selectedPreset.value === 'custom') {
    if (skipNextCustomPresetLoad) {
      skipNextCustomPresetLoad = false;
      nextTick(() => {
        scrollSelectedPresetIntoView();
      });
      return;
    }

    const custom = getConfiguredCustomPreset();
    isApplyingPreset = true;
    brightness.value = custom.brightness;
    contrast.value = custom.contrast;
    saturation.value = custom.saturation;
    hue.value = custom.hue;
    blur.value = custom.blur;
    selectedFilter.value = custom.filter;
    nextTick(() => {
      isApplyingPreset = false;
      scrollSelectedPresetIntoView();
    });
    return;
  }

  const p = presets[selectedPreset.value];
  if (!p) return;
  isApplyingPreset = true;
  brightness.value = p.brightness;
  contrast.value = p.contrast;
  saturation.value = p.saturation;
  hue.value = p.hue;
  blur.value = p.blur;
  selectedFilter.value = p.filter;
  nextTick(() => {
    isApplyingPreset = false;
    scrollSelectedPresetIntoView();
  });
});

watch(activeEditorTab, (tab) => {
  if (tab !== 'adjust') return;
  nextTick(() => {
    scrollSelectedPresetIntoView();
  });
});

watch([brightness, contrast], () => {
  animateHistogramTone();
});

watch([brightness, contrast, saturation, hue, blur, selectedFilter], () => {
  if (isApplyingPreset) return;

  const currentValues = getCurrentAdjustmentValues();
  const resolvedPreset = resolvePresetKey(currentValues);

  if (resolvedPreset === 'custom') {
    persistCustomPreset(currentValues);
  }

  if (selectedPreset.value !== 'custom') {
    const p = presets[selectedPreset.value];
    if (
      p &&
      (brightness.value !== p.brightness ||
        contrast.value !== p.contrast ||
        saturation.value !== p.saturation ||
        hue.value !== p.hue ||
        blur.value !== p.blur ||
        selectedFilter.value !== p.filter)
    ) {
      skipNextCustomPresetLoad = true;
      selectedPreset.value = 'custom';
    }
  }
});

watch(
  [brightness, contrast, saturation, hue, blur, selectedFilter, () => resizeOutput.value.width, () => resizeOutput.value.height],
  () => {
    uiStore.setActiveAdjustments(props.fileInfo.file_path, {
      brightness: brightness.value,
      contrast: contrast.value,
      saturation: saturation.value,
      hue: hue.value,
      blur: blur.value,
      filter: selectedFilter.value || null,
      resize: resizeOutput.value.hasResize ? {
        width: resizeOutput.value.width,
        height: resizeOutput.value.height,
      } : null,
    });
  },
  { immediate: true }
);

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('EditImage');
  activeEditorTab.value = props.initialTab === 'adjust'
    ? 'adjust'
    : (config.imageEditor.tab === 'adjust' ? 'adjust' : 'edit');

  isProcessing.value = true;
  initEditImage();
  updateRealHistogram();
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('EditImage');
  if (histogramToneAnimationFrame !== null) {
    cancelAnimationFrame(histogramToneAnimationFrame);
    histogramToneAnimationFrame = null;
  }
});

const onImageLoad = async () => {
  await nextTick();

  if (imageRef.value && imageRef.value.naturalWidth > 0 && imageRef.value.naturalHeight > 0) {
    imageWidth.value = imageRef.value.naturalWidth;
    imageHeight.value = imageRef.value.naturalHeight;
    isPortrait.value = isPortraitForRotation(imageWidth.value, imageHeight.value, rotate.value);
  }

  fitImageToContainer();
  updateRealHistogram();

  requestAnimationFrame(() => {
    enableTransition.value = true;
    isProcessing.value = false;
  });
};

const initEditImageLoadingId = ref(0);

const initEditImage = async () => {
  initEditImageLoadingId.value++;
  const loadingId = initEditImageLoadingId.value;

  if (usesBackendPreview.value) {
    if (props.initialImageSrc) {
      imageSrc.value = props.initialImageSrc;
    }

    void (async () => {
      try {
        if (loadingId !== initEditImageLoadingId.value) return;
        const previewSrc = getPreviewUrl(props.fileInfo.id, props.fileInfo.file_path);
        if (previewSrc) {
          imageSrc.value = previewSrc;
        }
      } catch {
        if (loadingId !== initEditImageLoadingId.value) return;
      }
    })();
  } else {
    imageSrc.value = getAssetSrc(props.fileInfo.file_path);
  }

  imageWidth.value = props.fileInfo.width;
  imageHeight.value = props.fileInfo.height;
  isPortrait.value = isPortraitForRotation(imageWidth.value, imageHeight.value, initialDisplayRotate.value);
  if (isRawFile.value) {
    config.imageEditor.saveAs = 1;
  }

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
    rotate.value = initialDisplayRotate.value;
    isFlippedX.value = false;
    isFlippedY.value = false;
    brightness.value = adj.brightness || 0;
    contrast.value = adj.contrast || 0;
    saturation.value = adj.saturation ?? 100;
    hue.value = adj.hue || 0;
    blur.value = adj.blur || 0;
    selectedFilter.value = adj.filter || '';
    const restoredPreset = resolvePresetKey({
      brightness: brightness.value,
      contrast: contrast.value,
      saturation: saturation.value,
      hue: hue.value,
      blur: blur.value,
      filter: selectedFilter.value,
    });
    if (restoredPreset === 'custom') {
      skipNextCustomPresetLoad = true;
    }
    selectedPreset.value = restoredPreset;
  } else {
    rotate.value = initialDisplayRotate.value;
    isFlippedX.value = false;
    isFlippedY.value = false;
    resetAdjustments();
  }

  displayedHistogramBrightness.value = brightness.value;
  displayedHistogramContrast.value = contrast.value;
};

function clearHistogram() {
  histogramData.value = [...EMPTY_HISTOGRAM];
}

function resolveHistogramSource() {
  return props.fileInfo?.thumbnail || imageSrc.value || '';
}

function buildHistogramData(img: HTMLImageElement) {
  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d', { willReadFrequently: true });
  if (!ctx) {
    clearHistogram();
    return;
  }

  const size = 256;
  canvas.width = size;
  canvas.height = size;
  ctx.drawImage(img, 0, 0, size, size);

  try {
    const imageData = ctx.getImageData(0, 0, size, size).data;
    const hist = new Array(256).fill(0);

    for (let i = 0; i < imageData.length; i += 4) {
      const r = imageData[i];
      const g = imageData[i + 1];
      const b = imageData[i + 2];
      const gray = Math.round(0.2126 * r + 0.7152 * g + 0.0722 * b);
      hist[gray]++;
    }

    const maxVal = Math.max(...hist);
    histogramData.value = maxVal > 0 ? hist.map((v) => (v / maxVal) * 58) : [...EMPTY_HISTOGRAM];
  } catch {
    clearHistogram();
  }
}

async function loadHistogramImage(source: string) {
  const response = await fetch(source);
  if (!response.ok) {
    throw new Error(`Failed to fetch histogram source: ${response.status}`);
  }

  const blob = await response.blob();

  return await new Promise<{ img: HTMLImageElement; objectUrl: string }>((resolve, reject) => {
    const objectUrl = URL.createObjectURL(blob);
    const img = new Image();

    img.onload = () => resolve({ img, objectUrl });
    img.onerror = () => {
      URL.revokeObjectURL(objectUrl);
      reject(new Error('Failed to decode histogram source'));
    };
    img.src = objectUrl;
  });
}

const updateRealHistogram = async () => {
  const histogramSource = resolveHistogramSource();
  if (!histogramSource) {
    clearHistogram();
    return;
  }

  const loadId = ++histogramLoadId;
  let objectUrl = '';

  try {
    const loaded = await loadHistogramImage(histogramSource);
    if (loadId !== histogramLoadId) {
      URL.revokeObjectURL(loaded.objectUrl);
      return;
    }

    objectUrl = loaded.objectUrl;
    buildHistogramData(loaded.img);
  } catch {
    if (loadId !== histogramLoadId) return;
    clearHistogram();
  } finally {
    if (objectUrl) {
      URL.revokeObjectURL(objectUrl);
    }
  }
};

const generateHistogramPath = () => {
  if (!histogramData.value) return '';

  const width = 256;
  const height = 64;
  const br = (100 + displayedHistogramBrightness.value) / 100;
  const ct = (100 + displayedHistogramContrast.value) / 100;

  const sampledPoints: { x: number; y: number }[] = [];
  const step = 2;

  for (let i = 0; i <= 256; i += step) {
    let sum = 0;
    let count = 0;
    const windowSize = 2;

    for (let j = Math.max(0, i - windowSize); j < Math.min(256, i + windowSize); j++) {
      sum += histogramData.value[j];
      count++;
    }

    const val = count > 0 ? sum / count : 0;
    const x = (i * br - 128) * ct + 128;
    const y = height - val;

    if (x >= -10 && x <= width + 10) sampledPoints.push({ x, y });
  }

  if (sampledPoints.length < 2) return '';

  let path = `M 0,${height}`;

  for (let i = 0; i < sampledPoints.length; i++) {
    const p = sampledPoints[i];
    if (i === 0) {
      path += ` L ${p.x.toFixed(1)},${p.y.toFixed(1)}`;
    } else {
      const prev = sampledPoints[i - 1];
      const cp1x = prev.x + (p.x - prev.x) / 2;
      const cp1y = prev.y;
      const cp2x = prev.x + (p.x - prev.x) / 2;
      const cp2y = p.y;
      path += ` C ${cp1x.toFixed(1)},${cp1y.toFixed(1)} ${cp2x.toFixed(1)},${cp2y.toFixed(1)} ${p.x.toFixed(1)},${p.y.toFixed(1)}`;
    }
  }

  path += ` L ${width},${height} Z`;
  return path;
};

function animateHistogramTone() {
  if (histogramToneAnimationFrame !== null) {
    cancelAnimationFrame(histogramToneAnimationFrame);
    histogramToneAnimationFrame = null;
  }

  const startBrightness = displayedHistogramBrightness.value;
  const startContrast = displayedHistogramContrast.value;
  const targetBrightness = brightness.value;
  const targetContrast = contrast.value;

  if (startBrightness === targetBrightness && startContrast === targetContrast) {
    return;
  }

  const startTime = performance.now();
  const duration = 180;

  const step = (now: number) => {
    const progress = Math.min(1, (now - startTime) / duration);
    const eased = 1 - Math.pow(1 - progress, 3);

    displayedHistogramBrightness.value = startBrightness + (targetBrightness - startBrightness) * eased;
    displayedHistogramContrast.value = startContrast + (targetContrast - startContrast) * eased;

    if (progress < 1) {
      histogramToneAnimationFrame = requestAnimationFrame(step);
      return;
    }

    displayedHistogramBrightness.value = targetBrightness;
    displayedHistogramContrast.value = targetContrast;
    histogramToneAnimationFrame = null;
  };

  histogramToneAnimationFrame = requestAnimationFrame(step);
}

const getPresetThumbnailStyle = (presetKey: string): CSSProperties => {
  const p = presets[presetKey];
  if (!p) return {};
  return {
    filter: `
      brightness(${100 + p.brightness}%)
      contrast(${100 + p.contrast}%)
      blur(${p.blur}px)
      hue-rotate(${p.hue}deg)
      saturate(${p.saturation}%)
      ${p.filter === 'grayscale' ? 'grayscale(100%)' : ''}
      ${p.filter === 'sepia' ? 'sepia(100%)' : ''}
      ${p.filter === 'invert' ? 'invert(100%)' : ''}
    `,
  };
};

const scrollSelectedPresetIntoView = () => {
  const container = presetStripRef.value;
  const selected = container?.querySelector(`[data-preset="${selectedPreset.value}"]`);
  if (!container || !selected) return;
  selected.scrollIntoView({ behavior: 'smooth', inline: 'center', block: 'nearest' });
};

const resetAdjustments = () => {
  const p = presets.natural;
  brightness.value = p.brightness;
  contrast.value = p.contrast;
  saturation.value = p.saturation;
  hue.value = p.hue;
  blur.value = p.blur;
  selectedFilter.value = p.filter;
  selectedPreset.value = 'natural';
  compareHold.value = false;
};

function setActiveEditorTab(tab: 'edit' | 'adjust') {
  if (cropStatus.value === 1) return;
  activeEditorTab.value = tab;
  config.imageEditor.tab = tab;
}

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

  rotate.value = initialDisplayRotate.value;
  isFlippedX.value = false;
  isFlippedY.value = false;
  isPortrait.value = isPortraitForRotation(imageWidth.value, imageHeight.value, initialDisplayRotate.value);
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
    case 'ArrowLeft':
      if (activeEditorTab.value === 'adjust') {
        movePresetSelection(-1);
        event.preventDefault();
        event.stopPropagation();
      }
      break;
    case 'ArrowRight':
      if (activeEditorTab.value === 'adjust') {
        movePresetSelection(1);
        event.preventDefault();
        event.stopPropagation();
      }
      break;
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
    case ' ':
      if (cropStatus.value === 1) {
        toggleCropBoxFixed();
        event.preventDefault();
        event.stopPropagation();
      }
      break;
    default:
      break;
  }
}

const clickCancel = () => {
  if (uiStore.activeAdjustments.filePath === props.fileInfo.file_path) {
    uiStore.clearActiveAdjustments();
  }
  emit('cancel');
};

function handleComparePointerDown() {
  compareHold.value = true;
}

function handleComparePointerUp() {
  compareHold.value = false;
}

function movePresetSelection(direction: number) {
  const currentIndex = presetOptions.value.findIndex(option => option.value === selectedPreset.value);
  if (currentIndex === -1) return;
  const nextIndex = Math.max(0, Math.min(presetOptions.value.length - 1, currentIndex + direction));
  selectedPreset.value = presetOptions.value[nextIndex].value;
}

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
    filter: selectedFilter.value || null,
    brightness: brightness.value !== 0 ? brightness.value : null,
    contrast: contrast.value !== 0 ? contrast.value : null,
    blur: blur.value > 0 ? blur.value : null,
    hue_rotate: hue.value !== 0 ? hue.value : null,
    saturation: saturation.value !== 100 ? saturation.value / 100.0 : null,
  };
};

const executeSave = async (overrides: { fileName?: string; destFilePath?: string; outputFormat?: string } = {}) => {
  isProcessing.value = true;
  let success = false;
  const savedFilePath = overrides.destFilePath || props.fileInfo.file_path;
  const saveAsNew = savedFilePath !== props.fileInfo.file_path;
  try {
    success = await editImage(setEditParams(overrides));
  } finally {
    isProcessing.value = false;
    if (success) {
      uiStore.updateFileVersion(props.fileInfo.file_path);
      if (uiStore.activeAdjustments.filePath === props.fileInfo.file_path) {
        uiStore.clearActiveAdjustments();
      }
      emit('success', { saveAsNew, filePath: savedFilePath });
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
