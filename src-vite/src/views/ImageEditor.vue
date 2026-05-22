<template>

  <div class="w-screen h-screen flex flex-col overflow-hidden bg-base-300 text-base-content/70">
    <!-- Title Bar -->
    <div
      class="h-10 shrink-0 flex items-center justify-between px-4 select-none"
      :class="isMac ? 'pl-20' : ''"
      data-tauri-drag-region
    >
      <div class="text-sm font-medium text-base-content/70 truncate">
        {{ $t('msgbox.image_editor.title') }} - {{ shortenFilename(fileInfo?.name || '', 32) }}
      </div>
    </div>

    <!-- Main Content -->
    <div v-if="fileInfo" class="flex-1 flex gap-3 p-3 min-h-0 select-none">
      <!-- Left: Image Preview -->
      <div class="flex-1 min-w-0 flex flex-col items-center justify-center gap-2">
        <div
          ref="containerRef"
          class="relative w-full flex-1 rounded-box overflow-hidden border border-base-content/5 bg-base-300/30 shadow-sm cursor-default"
          @pointerdown="handlePreviewPointerDown"
          @pointerup="handlePreviewPointerUp"
          @pointerleave="handlePreviewPointerUp"
          @pointercancel="handlePreviewPointerUp"
        >
            <transition name="fade">
              <div v-if="isProcessing" class="absolute inset-0 z-50 flex items-center justify-center bg-base-100/55 backdrop-blur-sm">
                <span class="loading loading-dots text-primary"></span>
              </div>
            </transition>

            <template v-if="imageSrc">
              <figure
                v-if="showDiffPreview && canShowDiffPreview"
                class="diff absolute inset-0 z-20 h-full w-full"
                tabindex="0"
              >
                <div class="diff-item-1 relative h-full w-full">
                  <img
                    :src="imageSrc"
                    :style="originalImageStyle"
                    class="block"
                    draggable="false"
                  />
                </div>
                <div class="diff-item-2 relative h-full w-full">
                  <img
                    :src="imageSrc"
                    :style="adjustedImageStyle"
                    class="block"
                    draggable="false"
                  />
                </div>
                <div class="diff-resizer"></div>
                
                <div 
                  class="pointer-events-none absolute z-30 rounded-box bg-base-100/80 px-2 py-1 text-[10px] font-semibold uppercase tracking-wide text-base-content/60"
                  :style="cropApplied ? { left: `${cropBox.left + 12}px`, top: `${cropBox.top + 12}px` } : { left: '12px', top: '12px' }"
                >
                  {{ $t('msgbox.image_editor.original') }}
                </div>
                <div 
                  class="pointer-events-none absolute z-30 rounded-box bg-base-100/80 px-2 py-1 text-[10px] font-semibold uppercase tracking-wide text-base-content/60"
                  :style="cropApplied ? { left: `${cropBox.left + cropBox.width - (locale === 'zh' ? 52 : 64)}px`, top: `${cropBox.top + 12}px` } : { right: '12px', top: '12px' }"
                >
                  {{ $t('msgbox.image_editor.adjusted') }}
                </div>
              </figure>

              <img
                v-show="imageReady && !(showDiffPreview && canShowDiffPreview)"
                ref="imageRef"
                :src="imageSrc"
                :style="imageStyle"
                class="block"
                draggable="false"
                @load="onImageLoad"
              />
            </template>

            <div v-if="cropStatus === 1 || cropApplied"
              :class="[
                cropStatus === 1 ? 'crop-box-active' : 'crop-box-done',
                isResizing ? 'no-transition' : '',
                cropStatus === 1
                  ? (
                    cropBoxFixed
                      ? (isDragging ? 'cursor-grabbing no-transition' : 'cursor-grab')
                      : (isDragging ? 'cursor-move no-transition' : 'cursor-move')
                  )
                  : ''
              ]"
              :style="[
                cropBoxStyle,
                activeEditorTab === 'adjust' ? { pointerEvents: 'none', zIndex: 30 } : { zIndex: 40 }
              ]"
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
        class="w-[320px] flex flex-col gap-3 overflow-y-auto"
        :class="isProcessing ? 'pointer-events-none opacity-60' : ''"
      >
        <div class="sticky top-0 z-10 bg-base-300 border-b border-base-content/5 pb-1">
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
                :selected="true"
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
                :selected="true"
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

          <div class="grid grid-cols-[1fr_auto_1fr] items-end gap-1">
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

            <div class="pb-0.5">
              <TButton
                buttonSize="small"
                :icon="keepAspectRatio ? IconLink : IconLinkOff"
                :disabled="cropStatus === 1"
                :tooltip="$t('msgbox.image_editor.keep_aspect_ratio')"
                @click="keepAspectRatio = !keepAspectRatio"
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
              <path :d="histogramPath" fill="url(#histGradientEdit)" />
              <path :d="histogramPathR" fill="rgba(239,68,68,0.35)" style="mix-blend-mode: screen" />
              <path :d="histogramPathG" fill="rgba(34,197,94,0.35)" style="mix-blend-mode: screen" />
              <path :d="histogramPathB" fill="rgba(59,130,246,0.35)" style="mix-blend-mode: screen" />
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
            <div class="flex items-center gap-1">
              <TButton
                buttonSize="small"
                :icon="IconSplitOn"
                :selected="showDiffPreview && canShowDiffPreview"
                :disabled="!hasAdjustmentChanges"
                :tooltip="$t('msgbox.image_editor.compare_view')"
                @click="toggleDiffPreview"
              />
              <TButton
                buttonSize="small"
                :icon="IconRestore"
                :disabled="!hasAdjustmentChanges"
                :tooltip="$t('msgbox.image_editor.reset')"
                @click.stop="resetAdjustments"
              />
            </div>
          </div>

          <div class="grid grid-cols-4 gap-1">
            <div
              v-for="option in presetOptions"
              :key="option.value"
              class="group min-w-0 cursor-pointer"
              @click="selectedPreset = option.value"
            >
              <div
                :class="[
                  'aspect-4/3 rounded-box border-2 transition-all duration-200 flex items-center justify-center overflow-hidden relative',
                  selectedPreset === option.value ? 'border-primary ring-2 ring-primary/20' : 'border-base-content/5 hover:border-base-content/20',
                ]"
              >
                <div class="w-full h-full bg-base-300 flex items-center justify-center overflow-hidden rounded-[inherit]">
                  <img
                    v-if="fileInfo.thumbnail"
                    :src="fileInfo.thumbnail"
                    class="w-full h-full rounded-box object-cover pointer-events-none"
                    :style="{ filter: presetThumbnailFilter(option.value) }"
                  />
                  <IconPalette v-else class="w-4 h-4 text-base-content/10" />
                </div>
              </div>
              <div
                class="mt-1 text-[9px] text-center truncate font-medium transition-colors uppercase tracking-tight"
                :class="selectedPreset === option.value ? 'text-primary' : 'text-base-content/50 group-hover:text-base-content'"
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
      </div>
    </div>

    <!-- Bottom Bar -->
    <div v-if="fileInfo" class="h-12 shrink-0 flex items-center justify-end px-4 gap-2 border-t border-base-content/5">
      <button
        class="px-4 py-1 rounded-box hover:bg-base-100 hover:text-base-content cursor-pointer text-sm mr-4"
        @click="clickCancel"
      >{{ $t('msgbox.image_editor.cancel') }}</button>

      <template v-if="effectiveSaveAsNew">
        <select v-model="combinedFormatKey" class="select select-bordered select-xs" :disabled="cropStatus===1">
          <option v-for="option in combinedFormatOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
        </select>
      </template>

      <div class="join">
        <button
          class="btn btn-sm btn-primary join-item"
          :disabled="cropStatus === 1 || isProcessing"
          @click="clickSave"
        >{{ effectiveSaveAsNew ? $t('msgbox.image_editor.save_as_new') : $t('msgbox.image_editor.overwrite') }}</button>
        <div class="dropdown dropdown-top dropdown-end">
          <button
            tabindex="0"
            class="btn btn-sm btn-primary join-item border-l border-primary-content/20 px-1.5"
            :disabled="!canOverwriteOriginal || cropStatus === 1"
          >
            <IconArrowDown class="w-3 h-3" />
          </button>
          <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box shadow-lg mb-1 p-1 text-sm w-32">
            <li>
              <a :class="config.imageEditor.saveAs === 0 ? 'active' : ''"
                 @click="config.imageEditor.saveAs = 0; closeSaveDropdown()">
                {{ $t('msgbox.image_editor.overwrite') }}
              </a>
            </li>
            <li>
              <a :class="config.imageEditor.saveAs === 1 ? 'active' : ''"
                 @click="config.imageEditor.saveAs = 1; closeSaveDropdown()">
                {{ $t('msgbox.image_editor.save_as_new') }}
              </a>
            </li>
          </ul>
        </div>
      </div>
    </div>
  </div>

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
import { useRouter } from 'vue-router';
import { useUIStore } from '@/stores/uiStore';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { isMac, setTheme, SCALE_VALUES, getFolderPath, getFileExtension, shortenFilename, getFullPath, combineFileName, getSelectOptions, getAssetSrc, getPreviewUrl, getThumbUrl, shouldUseBackendPreview } from '@/common/utils';
import { editImage, checkFileExists, getFileInfo } from '@/common/api';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { emit as tauriEmit, listen } from '@tauri-apps/api/event';

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
  IconLink,
  IconLinkOff,
  IconArrowDown,
  IconPalette,
  IconSplitOn,
} from '@/common/icons';

const router = useRouter();
const fileInfo = ref<any>(null);
const initialImageSrc = ref('');

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const uiStore = useUIStore();

function sendToParent(payload: Record<string, any>) {
  tauriEmit('message-from-image-editor', payload);
}

async function loadFileInfo(fileId: number) {
  try {
    imageReady.value = false;
    const file = await getFileInfo(fileId);
    if (file) {
      file.thumbnail = getThumbUrl(file.id);
      fileInfo.value = file;
      newFileName.value = file.name?.substring(0, file.name.lastIndexOf('.')) || file.name || '';
      const src = getPreviewUrl(file);
      initialImageSrc.value = typeof src === 'string' ? src : '';
    }
  } catch {
    getCurrentWindow().close();
  }
}

const isProcessing = ref(false);
const imageReady = ref(false);
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
const isRawFile = computed(() => Number(fileInfo.value?.file_type || 0) === 3);
const normalizeRotate = (value: number) => {
  const normalized = Number(value || 0) % 360;
  return normalized < 0 ? normalized + 360 : normalized;
};
const initialDisplayRotate = computed(() => normalizeRotate(Number(fileInfo.value?.rotate || 0)));
const isPortraitForRotation = (width: number, height: number, rotation: number) => {
  const normalized = normalizeRotate(rotation);
  return normalized % 180 !== 0 ? width > height : height > width;
};
const usesBackendPreview = computed(() =>
  shouldUseBackendPreview(
    fileInfo.value?.name || fileInfo.value?.file_path || '',
    Number(fileInfo.value?.file_type || 0)
  )
);

const enableTransition = ref(false);
const position = ref({ left: 0, top: 0 });
const isFlippedX = ref(false);
const isFlippedY = ref(false);
const scale = ref(1);
const rotate = ref(0);
const showDiffPreview = ref(false);
const showOriginalWhilePressed = ref(false);
const brightness = ref(0);
const contrast = ref(0);
const saturation = ref(100);
const hue = ref(0);
const blur = ref(0);
const selectedFilter = ref('');
const selectedPreset = ref('natural');
const autoPresetValues = ref<AdjustmentValues | null>(null);
const HISTOGRAM_BIN_COUNT = 256;
const HISTOGRAM_HEIGHT = 58;
const histogramData = new Float32Array(HISTOGRAM_BIN_COUNT);
const histogramDataR = new Float32Array(HISTOGRAM_BIN_COUNT);
const histogramDataG = new Float32Array(HISTOGRAM_BIN_COUNT);
const histogramDataB = new Float32Array(HISTOGRAM_BIN_COUNT);
const displayedHistData = new Float32Array(HISTOGRAM_BIN_COUNT);
const displayedHistDataR = new Float32Array(HISTOGRAM_BIN_COUNT);
const displayedHistDataG = new Float32Array(HISTOGRAM_BIN_COUNT);
const displayedHistDataB = new Float32Array(HISTOGRAM_BIN_COUNT);
const smoothedHistData = new Float32Array(HISTOGRAM_BIN_COUNT);
const smoothedHistDataR = new Float32Array(HISTOGRAM_BIN_COUNT);
const smoothedHistDataG = new Float32Array(HISTOGRAM_BIN_COUNT);
const smoothedHistDataB = new Float32Array(HISTOGRAM_BIN_COUNT);
const histogramVersion = ref(0);
let histogramAnimRaf: number | null = null;
let isApplyingPreset = false;
let skipNextCustomPresetLoad = false;
let histogramLoadId = 0;
let autoPresetRequestId = 0;

let containerResizeObserver: ResizeObserver | null = null;
let unlistenUpdateFile: (() => void) | null = null;
const isResizing = ref(false);
type AdjustmentValues = {
  brightness: number;
  contrast: number;
  saturation: number;
  hue: number;
  blur: number;
  filter: string;
};
function buildAdjustmentFilter(values: AdjustmentValues) {
  return `
    brightness(${100 + values.brightness}%)
    contrast(${100 + values.contrast}%)
    blur(${values.blur}px)
    hue-rotate(${values.hue}deg)
    saturate(${values.saturation}%)
    ${values.filter === 'grayscale' ? 'grayscale(100%)' : ''}
    ${values.filter === 'sepia' ? 'sepia(100%)' : ''}
    ${values.filter === 'invert' ? 'invert(100%)' : ''}
  `;
}

const imageStyle = computed((): CSSProperties => ({
  display: 'block',
  width: `${imageWidth.value}px`,
  height: `${imageHeight.value}px`,
  maxWidth: 'none',
  maxHeight: 'none',
  position: 'absolute',
  filter: showOriginalWhilePressed.value ? 'none' : adjustmentFilter.value,
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
const originalImageStyle = computed((): CSSProperties => ({
  ...imageStyle.value,
  filter: 'none',
}));
const adjustedImageStyle = computed((): CSSProperties => ({
  ...imageStyle.value,
  filter: adjustmentFilter.value,
}));
const canShowDiffPreview = computed(() => activeEditorTab.value === 'adjust' && hasAdjustmentChanges.value);
const adjustmentFilter = computed(() => {
  return buildAdjustmentFilter({
    brightness: brightness.value,
    contrast: contrast.value,
    saturation: saturation.value,
    hue: hue.value,
    blur: blur.value,
    filter: selectedFilter.value,
  });
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

  const buildResizeResult = (width: number, height: number) => ({
    width,
    height,
    hasResize: width !== baseWidth || height !== baseHeight,
  });

  if (keepAspectRatio.value) {
    if (widthInput && heightInput) {
      return buildResizeResult(widthInput, heightInput);
    }
    if (widthInput) {
      return buildResizeResult(widthInput, Math.max(1, Math.round(widthInput / ratio)));
    }
    if (heightInput) {
      return buildResizeResult(Math.max(1, Math.round(heightInput * ratio)), heightInput);
    }
  }

  return buildResizeResult(widthInput || baseWidth, heightInput || baseHeight);
});
const hasEditImageChanges = computed(() =>
  normalizeRotate(rotate.value) !== initialDisplayRotate.value ||
  isFlippedX.value ||
  isFlippedY.value
);
const presets: Record<string, AdjustmentValues> = {
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

function sameAdjustmentValues(a: AdjustmentValues, b: AdjustmentValues) {
  return a.brightness === b.brightness
    && a.contrast === b.contrast
    && a.saturation === b.saturation
    && a.hue === b.hue
    && a.blur === b.blur
    && a.filter === b.filter;
}

function resolvePresetKey(values: AdjustmentValues) {
  if (autoPresetValues.value && sameAdjustmentValues(autoPresetValues.value, values)) {
    return 'auto';
  }

  for (const [key, preset] of Object.entries(presets)) {
    if (sameAdjustmentValues(preset, values)) {
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

function applyAdjustmentValues(values: AdjustmentValues) {
  brightness.value = values.brightness;
  contrast.value = values.contrast;
  saturation.value = values.saturation;
  hue.value = values.hue;
  blur.value = values.blur;
  selectedFilter.value = values.filter;
}

const presetOptions = computed(() => [
  { value: 'auto', label: localeMsg.value.msgbox.image_editor.presets.auto },
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
  { value: 'custom', label: localeMsg.value.msgbox.image_editor.presets.custom },
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

const newFileName = ref('');

const fileFormatOptions = computed(() => getSelectOptions(localeMsg.value.msgbox.image_editor.format_options));
const fileQualityOptions = computed(() => getSelectOptions(localeMsg.value.msgbox.image_editor.quality_options));
const outputFormatValues = ['jpg', 'png', 'webp'] as const;

function getSelectedOutputFormat() {
  return outputFormatValues[config.imageEditor.format] || outputFormatValues[0];
}

const combinedFormatKey = computed({
  get: () => {
    if (config.imageEditor.format !== 0) return String(config.imageEditor.format);
    return `0-${config.imageEditor.quality}`;
  },
  set: (key: string) => {
    if (key.includes('-')) {
      const [f, q] = key.split('-').map(Number);
      config.imageEditor.format = f;
      config.imageEditor.quality = q;
    } else {
      config.imageEditor.format = Number(key);
      config.imageEditor.quality = 0;
    }
  },
});

const combinedFormatOptions = computed(() => {
  const fmt = fileFormatOptions.value;
  const qual = fileQualityOptions.value;
  const items: { value: string; label: string }[] = [];
  // JPEG with quality levels
  items.push({ value: '0-0', label: `${fmt[0].label} (${qual[0].label})` });
  items.push({ value: '0-1', label: `${fmt[0].label} (${qual[1].label})` });
  items.push({ value: '0-2', label: `${fmt[0].label} (${qual[2].label})` });
  // PNG, WebP — no quality variants
  for (let i = 1; i < fmt.length; i++) {
    items.push({ value: String(i), label: fmt[i].label });
  }
  return items;
});

const canOverwriteOriginal = computed(() => {
  const ext = getFileExtension(fileInfo.value?.name || fileInfo.value?.file_path || '').toLowerCase();
  return ['jpg', 'jpeg', 'png', 'webp'].includes(ext);
});
const effectiveSaveAsNew = computed(() => config.imageEditor.saveAs === 1 || !canOverwriteOriginal.value);

const showOverwriteConfirm = ref(false);

const handleOverwriteConfirm = () => {
  showOverwriteConfirm.value = false;

  if (!canOverwriteOriginal.value) {
    return;
  }

  const originalPath = fileInfo.value.file_path;
  const ext = getFileExtension(fileInfo.value.name).toLowerCase();
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
    autoPresetRequestId++;
    if (skipNextCustomPresetLoad) {
      skipNextCustomPresetLoad = false;
      return;
    }

    const custom = getConfiguredCustomPreset();
    isApplyingPreset = true;
    applyAdjustmentValues(custom);
    nextTick(() => {
      isApplyingPreset = false;
    });
    return;
  }

  if (selectedPreset.value === 'auto') {
    applyAutoPreset();
    return;
  }

  const p = presets[selectedPreset.value];
  if (!p) return;
  autoPresetRequestId++;
  isApplyingPreset = true;
  applyAdjustmentValues(p);
  nextTick(() => {
    isApplyingPreset = false;
  });
});

watch(activeEditorTab, (tab) => {
  if (tab !== 'adjust') {
    showDiffPreview.value = false;
    showOriginalWhilePressed.value = false;
    return;
  }
});


watch([brightness, contrast, saturation, hue, blur, selectedFilter], () => {
  scheduleHistogramRecompute();

  if (isApplyingPreset) return;

  const currentValues = getCurrentAdjustmentValues();
  const resolvedPreset = resolvePresetKey(currentValues);

  if (resolvedPreset === 'natural') {
    showDiffPreview.value = false;
  }

  if (resolvedPreset === 'custom') {
    persistCustomPreset(currentValues);
  }

  if (selectedPreset.value !== 'custom') {
    const p = presets[selectedPreset.value];
    if (!p) {
      skipNextCustomPresetLoad = true;
      selectedPreset.value = 'custom';
      return;
    }
    if (
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
    if (!fileInfo.value?.file_path) return;
    uiStore.setActiveAdjustments(fileInfo.value.file_path, {
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

watch(() => config.settings.language, (newLanguage) => {
  locale.value = newLanguage;
});

watch(() => config.settings.appearance, (newAppearance) => {
  setTheme(newAppearance, newAppearance === 0 ? config.settings.lightTheme : config.settings.darkTheme);
});

watch(() => config.settings.lightTheme, (newLightTheme) => {
  setTheme(config.settings.appearance, newLightTheme);
});

watch(() => config.settings.darkTheme, (newDarkTheme) => {
  setTheme(config.settings.appearance, newDarkTheme);
});

watch(() => Number(config.settings.scale || 1), (newScale) => {
  const normalizedScale = SCALE_VALUES.find((item) => item === newScale) ?? 1;
  document.documentElement.style.fontSize = `${normalizedScale * 16}px`;
});

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('EditImage');
  activeEditorTab.value = config.imageEditor.tab === 'adjust' ? 'adjust' : 'edit';

  const query = router.currentRoute.value.query;
  const fileId = Number(query.fileId || 0);
  if (fileId > 0) {
    await loadFileInfo(fileId);
  }

  if (!fileInfo.value) {
    getCurrentWindow().close();
    return;
  }

  isProcessing.value = true;
  initEditImage();
  updateRealHistogram();

  unlistenUpdateFile = await listen('update-file', async (event: any) => {
    const newFileId = Number(event?.payload?.fileId || 0);
    if (newFileId > 0 && newFileId !== Number(fileInfo.value?.id || 0)) {
      await loadFileInfo(newFileId);
      if (fileInfo.value) {
        initEditImage();
        updateRealHistogram();
      }
    }
  });

  containerResizeObserver = new ResizeObserver(() => {
    isResizing.value = true;
    enableTransition.value = false;
    cropBoxFixed.value = false;
    containerRect.value = containerRef.value?.getBoundingClientRect() || null;
    if (containerRect.value) {
      containerBounds.value = {
        left: containerRect.value.left + containerPadding,
        top: containerRect.value.top + containerPadding,
        width: containerRect.value.width - containerPadding * 2,
        height: containerRect.value.height - containerPadding * 2,
      };
      autoFitVisualArea();
    }
    if (cropStatus.value === 1 || cropApplied.value) {
      requestAnimationFrame(() => {
        imageRectOriginal.value = imageRef.value?.getBoundingClientRect() || null;
        updateCropBoxFromCrop();
        enableTransition.value = true;
        isResizing.value = false;
      });
    } else {
      enableTransition.value = true;
      isResizing.value = false;
    }
  });
  if (containerRef.value) {
    containerResizeObserver.observe(containerRef.value);
  }
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('EditImage');
  if (histogramAnimRaf !== null) {
    cancelAnimationFrame(histogramAnimRaf);
    histogramAnimRaf = null;
  }
  if (recomputeHistogramTimer !== null) {
    clearTimeout(recomputeHistogramTimer);
    recomputeHistogramTimer = null;
  }
  releaseHistogramSourceImage();
  if (containerResizeObserver) {
    containerResizeObserver.disconnect();
    containerResizeObserver = null;
  }
  if (unlistenUpdateFile) {
    unlistenUpdateFile();
    unlistenUpdateFile = null;
  }
});

const onImageLoad = async () => {
  await nextTick();

  if (imageRef.value && imageRef.value.naturalWidth > 0 && imageRef.value.naturalHeight > 0) {
    imageWidth.value = imageRef.value.naturalWidth;
    imageHeight.value = imageRef.value.naturalHeight;
    isPortrait.value = isPortraitForRotation(imageWidth.value, imageHeight.value, rotate.value);
  }

  autoFitVisualArea();
  updateRealHistogram();

  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      enableTransition.value = true;
      imageReady.value = true;
      isProcessing.value = false;
    });
  });
};

const initEditImageLoadingId = ref(0);

const initEditImage = async () => {
  initEditImageLoadingId.value++;
  const loadingId = initEditImageLoadingId.value;

  if (usesBackendPreview.value) {
    if (initialImageSrc.value) {
      imageSrc.value = initialImageSrc.value;
    }

    void (async () => {
      try {
        if (loadingId !== initEditImageLoadingId.value) return;
        const previewSrc = getPreviewUrl(fileInfo.value.id, fileInfo.value.file_path);
        if (previewSrc) {
          imageSrc.value = previewSrc;
        }
      } catch {
        if (loadingId !== initEditImageLoadingId.value) return;
      }
    })();
  } else {
    imageSrc.value = getAssetSrc(fileInfo.value.file_path);
  }

  imageWidth.value = fileInfo.value.width;
  imageHeight.value = fileInfo.value.height;
  isPortrait.value = isPortraitForRotation(imageWidth.value, imageHeight.value, initialDisplayRotate.value);
  if (isRawFile.value || !canOverwriteOriginal.value) {
    config.imageEditor.saveAs = 1;
  } else if (config.imageEditor.saveAs !== 0) {
    config.imageEditor.saveAs = 0;
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

  if (uiStore.activeAdjustments.filePath === fileInfo.value.file_path) {
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

};

function clearHistogram() {
  releaseHistogramSourceImage();
  autoPresetValues.value = null;
  if (histogramAnimRaf !== null) {
    cancelAnimationFrame(histogramAnimRaf);
    histogramAnimRaf = null;
  }
  histogramData.fill(0);
  histogramDataR.fill(0);
  histogramDataG.fill(0);
  histogramDataB.fill(0);
  displayedHistData.fill(0);
  displayedHistDataR.fill(0);
  displayedHistDataG.fill(0);
  displayedHistDataB.fill(0);
  smoothedHistData.fill(0);
  smoothedHistDataR.fill(0);
  smoothedHistDataG.fill(0);
  smoothedHistDataB.fill(0);
  histogramVersion.value++;
}

function resolveHistogramSource() {
  return imageSrc.value || fileInfo.value?.thumbnail || '';
}

function writeNormalizedHistogram(counts: Float32Array, output: Float32Array) {
  let maxVal = 0;
  for (let i = 0; i < HISTOGRAM_BIN_COUNT; i++) {
    if (counts[i] > maxVal) maxVal = counts[i];
  }

  if (maxVal <= 0) {
    output.fill(0);
    return;
  }

  for (let i = 0; i < HISTOGRAM_BIN_COUNT; i++) {
    output[i] = (counts[i] / maxVal) * HISTOGRAM_HEIGHT;
  }
}

function updateHistogramTargets(data: Uint8ClampedArray) {
  const hist = new Float32Array(HISTOGRAM_BIN_COUNT);
  const histR = new Float32Array(HISTOGRAM_BIN_COUNT);
  const histG = new Float32Array(HISTOGRAM_BIN_COUNT);
  const histB = new Float32Array(HISTOGRAM_BIN_COUNT);

  for (let i = 0; i < data.length; i += 4) {
    const alpha = data[i + 3] / 255;
    if (alpha <= 0) continue;

    const r = data[i];
    const g = data[i + 1];
    const b = data[i + 2];
    const gray = Math.round(0.2126 * r + 0.7152 * g + 0.0722 * b);
    hist[gray] += alpha;
    histR[r] += alpha;
    histG[g] += alpha;
    histB[b] += alpha;
  }

  writeNormalizedHistogram(hist, histogramData);
  writeNormalizedHistogram(histR, histogramDataR);
  writeNormalizedHistogram(histG, histogramDataG);
  writeNormalizedHistogram(histB, histogramDataB);
}

function clampColor(value: number) {
  return Math.max(0, Math.min(255, value));
}

function clampNumber(value: number, min: number, max: number) {
  return Math.max(min, Math.min(max, value));
}

function applyPerPixel(data: Uint8ClampedArray, transform: (r: number, g: number, b: number) => [number, number, number]) {
  for (let i = 0; i < data.length; i += 4) {
    const [r, g, b] = transform(data[i], data[i + 1], data[i + 2]);
    data[i] = clampColor(r);
    data[i + 1] = clampColor(g);
    data[i + 2] = clampColor(b);
  }
}

function applyBoxBlur(data: Uint8ClampedArray, width: number, height: number, radius: number) {
  if (radius <= 0) return;

  const temp = new Uint8ClampedArray(data.length);
  const output = new Uint8ClampedArray(data.length);
  const windowSize = radius * 2 + 1;

  for (let y = 0; y < height; y++) {
    let r = 0, g = 0, b = 0, a = 0;
    for (let x = -radius; x <= radius; x++) {
      const clampedX = Math.max(0, Math.min(width - 1, x));
      const idx = (y * width + clampedX) * 4;
      r += data[idx];
      g += data[idx + 1];
      b += data[idx + 2];
      a += data[idx + 3];
    }

    for (let x = 0; x < width; x++) {
      const idx = (y * width + x) * 4;
      temp[idx] = r / windowSize;
      temp[idx + 1] = g / windowSize;
      temp[idx + 2] = b / windowSize;
      temp[idx + 3] = a / windowSize;

      const removeX = Math.max(0, Math.min(width - 1, x - radius));
      const addX = Math.max(0, Math.min(width - 1, x + radius + 1));
      const removeIdx = (y * width + removeX) * 4;
      const addIdx = (y * width + addX) * 4;
      r += data[addIdx] - data[removeIdx];
      g += data[addIdx + 1] - data[removeIdx + 1];
      b += data[addIdx + 2] - data[removeIdx + 2];
      a += data[addIdx + 3] - data[removeIdx + 3];
    }
  }

  for (let x = 0; x < width; x++) {
    let r = 0, g = 0, b = 0, a = 0;
    for (let y = -radius; y <= radius; y++) {
      const clampedY = Math.max(0, Math.min(height - 1, y));
      const idx = (clampedY * width + x) * 4;
      r += temp[idx];
      g += temp[idx + 1];
      b += temp[idx + 2];
      a += temp[idx + 3];
    }

    for (let y = 0; y < height; y++) {
      const idx = (y * width + x) * 4;
      output[idx] = r / windowSize;
      output[idx + 1] = g / windowSize;
      output[idx + 2] = b / windowSize;
      output[idx + 3] = a / windowSize;

      const removeY = Math.max(0, Math.min(height - 1, y - radius));
      const addY = Math.max(0, Math.min(height - 1, y + radius + 1));
      const removeIdx = (removeY * width + x) * 4;
      const addIdx = (addY * width + x) * 4;
      r += temp[addIdx] - temp[removeIdx];
      g += temp[addIdx + 1] - temp[removeIdx + 1];
      b += temp[addIdx + 2] - temp[removeIdx + 2];
      a += temp[addIdx + 3] - temp[removeIdx + 3];
    }
  }

  data.set(output);
}

function applyHistogramAdjustments(data: Uint8ClampedArray, width: number, height: number) {
  // Keep this order aligned with buildAdjustmentFilter() so the histogram
  // represents the previewed pixels instead of a separate editor pipeline.
  const br = (100 + brightness.value) / 100;
  const ct = (100 + contrast.value) / 100;

  applyPerPixel(data, (r, g, b) => [r * br, g * br, b * br]);
  applyPerPixel(data, (r, g, b) => [
    (r - 128) * ct + 128,
    (g - 128) * ct + 128,
    (b - 128) * ct + 128,
  ]);

  applyBoxBlur(data, width, height, Math.round(blur.value));

  const hueRad = hue.value * Math.PI / 180;
  const cos = Math.cos(hueRad);
  const sin = Math.sin(hueRad);
  const sat = saturation.value / 100;

  applyPerPixel(data, (r, g, b) => [
    (0.213 + 0.787 * cos - 0.213 * sin) * r + (0.715 - 0.715 * cos - 0.715 * sin) * g + (0.072 - 0.072 * cos + 0.928 * sin) * b,
    (0.213 - 0.213 * cos + 0.143 * sin) * r + (0.715 + 0.285 * cos + 0.140 * sin) * g + (0.072 - 0.072 * cos - 0.283 * sin) * b,
    (0.213 - 0.213 * cos - 0.787 * sin) * r + (0.715 - 0.715 * cos + 0.715 * sin) * g + (0.072 + 0.928 * cos + 0.072 * sin) * b,
  ]);

  applyPerPixel(data, (r, g, b) => [
    (0.213 + 0.787 * sat) * r + (0.715 - 0.715 * sat) * g + (0.072 - 0.072 * sat) * b,
    (0.213 - 0.213 * sat) * r + (0.715 + 0.285 * sat) * g + (0.072 - 0.072 * sat) * b,
    (0.213 - 0.213 * sat) * r + (0.715 - 0.715 * sat) * g + (0.072 + 0.928 * sat) * b,
  ]);

  if (selectedFilter.value === 'grayscale') {
    applyPerPixel(data, (r, g, b) => {
      const gray = 0.2126 * r + 0.7152 * g + 0.0722 * b;
      return [gray, gray, gray];
    });
  } else if (selectedFilter.value === 'sepia') {
    applyPerPixel(data, (r, g, b) => [
      0.393 * r + 0.769 * g + 0.189 * b,
      0.349 * r + 0.686 * g + 0.168 * b,
      0.272 * r + 0.534 * g + 0.131 * b,
    ]);
  } else if (selectedFilter.value === 'invert') {
    applyPerPixel(data, (r, g, b) => [255 - r, 255 - g, 255 - b]);
  }
}

function buildHistogramData(img: HTMLImageElement, applyAdjustments = true) {
  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d', { willReadFrequently: true });
  if (!ctx) {
    clearHistogram();
    return;
  }

  const size = 512;
  canvas.width = size;
  canvas.height = size;
  ctx.drawImage(img, 0, 0, size, size);

  try {
    const imageData = ctx.getImageData(0, 0, size, size);
    if (applyAdjustments) {
      applyHistogramAdjustments(imageData.data, size, size);
    }
    const data = imageData.data;
    updateHistogramTargets(data);
  } catch {
    clearHistogram();
  }
  startHistogramAnimation();
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

function percentileFromCounts(counts: Float32Array, total: number, percentile: number) {
  if (total <= 0) return 0;
  const target = total * percentile;
  let cumulative = 0;
  for (let i = 0; i < counts.length; i++) {
    cumulative += counts[i];
    if (cumulative >= target) return i;
  }
  return counts.length - 1;
}

function computeAutoPresetValues(img: HTMLImageElement): AdjustmentValues {
  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d', { willReadFrequently: true });
  if (!ctx) return presets.natural;

  const size = 256;
  canvas.width = size;
  canvas.height = size;
  ctx.drawImage(img, 0, 0, size, size);

  try {
    const data = ctx.getImageData(0, 0, size, size).data;
    const lumaCounts = new Float32Array(HISTOGRAM_BIN_COUNT);
    let total = 0;
    let lumaSum = 0;
    let saturationSum = 0;

    for (let i = 0; i < data.length; i += 4) {
      const alpha = data[i + 3] / 255;
      if (alpha <= 0) continue;

      const r = data[i];
      const g = data[i + 1];
      const b = data[i + 2];
      const luma = Math.round(0.2126 * r + 0.7152 * g + 0.0722 * b);
      const maxChannel = Math.max(r, g, b);
      const minChannel = Math.min(r, g, b);
      const chroma = maxChannel > 0 ? (maxChannel - minChannel) / maxChannel : 0;

      lumaCounts[luma] += alpha;
      total += alpha;
      lumaSum += luma * alpha;
      saturationSum += chroma * alpha;
    }

    if (total <= 0) return presets.natural;

    const low = percentileFromCounts(lumaCounts, total, 0.01);
    const high = percentileFromCounts(lumaCounts, total, 0.99);
    const range = Math.max(1, high - low);
    const mean = Math.max(1, lumaSum / total);
    const avgSaturation = saturationSum / total;

    const isUnderexposed = mean < 92 && high < 210;
    const isOverexposed = mean > 168 && low > 35;
    const isFlat = range < 150;
    const isHealthyExposure = !isUnderexposed && !isOverexposed && mean >= 108 && mean <= 148;

    let contrastFactor = 1;
    if (isFlat) {
      contrastFactor = clampNumber(170 / range, 1, 1.28);
    }

    let targetMean = mean;
    if (isUnderexposed) {
      targetMean = 126;
    } else if (isOverexposed) {
      targetMean = 136;
    } else if (isFlat && !isHealthyExposure) {
      targetMean = 132;
    }

    const brightnessFactor = targetMean === mean
      ? 1
      : clampNumber((((targetMean - 128) / contrastFactor) + 128) / mean, 0.82, 1.22);

    let saturationBoost = 0;
    if (avgSaturation >= 0.04 && avgSaturation < 0.16) {
      saturationBoost = 10;
    } else if (avgSaturation >= 0.16 && avgSaturation < 0.24) {
      saturationBoost = 5;
    } else if (avgSaturation > 0.62) {
      saturationBoost = -4;
    }

    return {
      brightness: Math.round((brightnessFactor - 1) * 100),
      contrast: Math.round((contrastFactor - 1) * 100),
      saturation: clampNumber(Math.round(100 + saturationBoost), 85, 120),
      hue: 0,
      blur: 0,
      filter: '',
    };
  } catch {
    return presets.natural;
  }
}

async function getAutoPresetValues() {
  if (autoPresetValues.value) return autoPresetValues.value;
  if (histogramSourceImage) {
    autoPresetValues.value = computeAutoPresetValues(histogramSourceImage);
    return autoPresetValues.value;
  }

  const source = resolveHistogramSource();
  if (!source) return presets.natural;

  try {
    const loaded = await loadHistogramImage(source);
    try {
      autoPresetValues.value = computeAutoPresetValues(loaded.img);
      return autoPresetValues.value;
    } finally {
      URL.revokeObjectURL(loaded.objectUrl);
    }
  } catch {
    autoPresetValues.value = presets.natural;
    return autoPresetValues.value;
  }
}

async function applyAutoPreset() {
  const requestId = ++autoPresetRequestId;
  try {
    const values = await getAutoPresetValues();
    if (requestId !== autoPresetRequestId || selectedPreset.value !== 'auto') return;
    isApplyingPreset = true;
    applyAdjustmentValues(values);
  } finally {
    if (isApplyingPreset && requestId === autoPresetRequestId) {
      nextTick(() => {
        isApplyingPreset = false;
      });
    }
  }
}

let histogramSourceImage: HTMLImageElement | null = null;
let histogramSourceObjectUrl = '';
let recomputeHistogramTimer: ReturnType<typeof setTimeout> | null = null;

function releaseHistogramSourceImage() {
  histogramSourceImage = null;
  if (histogramSourceObjectUrl) {
    URL.revokeObjectURL(histogramSourceObjectUrl);
    histogramSourceObjectUrl = '';
  }
}

function recomputeHistogramWithFilter() {
  if (!histogramSourceImage) return;
  buildHistogramData(histogramSourceImage);
}

function scheduleHistogramRecompute() {
  if (recomputeHistogramTimer !== null) return;
  recomputeHistogramTimer = setTimeout(() => {
    recomputeHistogramTimer = null;
    recomputeHistogramWithFilter();
  }, 32);
}

const updateRealHistogram = async () => {
  const histogramSource = resolveHistogramSource();
  if (!histogramSource) {
    clearHistogram();
    return;
  }

  const loadId = ++histogramLoadId;

  try {
    const loaded = await loadHistogramImage(histogramSource);
    if (loadId !== histogramLoadId) {
      URL.revokeObjectURL(loaded.objectUrl);
      return;
    }

    releaseHistogramSourceImage();
    histogramSourceObjectUrl = loaded.objectUrl;
    histogramSourceImage = loaded.img;
    autoPresetValues.value = computeAutoPresetValues(loaded.img);
    buildHistogramData(loaded.img);
  } catch {
    if (loadId !== histogramLoadId) return;
    clearHistogram();
  }
};


function gaussianSmooth(source: Float32Array, target: Float32Array, radius = 5, sigma = 2.6) {
  for (let i = 0; i < HISTOGRAM_BIN_COUNT; i++) {
    let sum = 0, weight = 0;
    for (let j = -radius; j <= radius; j++) {
      const idx = i + j;
      if (idx < 0 || idx >= HISTOGRAM_BIN_COUNT) continue;
      const w = Math.exp(-(j * j) / (2 * sigma * sigma));
      sum += source[idx] * w;
      weight += w;
    }
    target[i] = sum / weight;
  }
}

function sampleSmoothedHistogram(smoothed: Float32Array, center: number, radius = 2) {
  let sum = 0;
  let count = 0;
  for (let i = Math.max(0, center - radius); i <= Math.min(HISTOGRAM_BIN_COUNT - 1, center + radius); i++) {
    sum += smoothed[i];
    count++;
  }
  return count > 0 ? sum / count : 0;
}

function buildPathFromSmoothed(smoothed: Float32Array) {
  const width = 256;
  const height = 64;
  const sampledPoints: { x: number; y: number }[] = [];

  for (let i = 0; i < HISTOGRAM_BIN_COUNT; i += 4) {
    const val = Math.max(0, sampleSmoothedHistogram(smoothed, i));
    const x = i;
    const y = height - val;
    sampledPoints.push({ x, y });
  }
  const lastVal = Math.max(0, sampleSmoothedHistogram(smoothed, HISTOGRAM_BIN_COUNT - 1));
  sampledPoints.push({ x: width, y: height - lastVal });

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
}

const histogramPath = computed(() => {
  histogramVersion.value;
  return buildPathFromSmoothed(smoothedHistData);
});
const histogramPathR = computed(() => {
  histogramVersion.value;
  return buildPathFromSmoothed(smoothedHistDataR);
});
const histogramPathG = computed(() => {
  histogramVersion.value;
  return buildPathFromSmoothed(smoothedHistDataG);
});
const histogramPathB = computed(() => {
  histogramVersion.value;
  return buildPathFromSmoothed(smoothedHistDataB);
});

function lerpHistogram(target: Float32Array, display: Float32Array): boolean {
  let changed = false;
  for (let i = 0; i < HISTOGRAM_BIN_COUNT; i++) {
    const diff = target[i] - display[i];
    if (Math.abs(diff) < 0.5) {
      display[i] = target[i];
    } else {
      display[i] += diff * 0.3;
      changed = true;
    }
  }
  return changed;
}

function startHistogramAnimation() {
  if (histogramAnimRaf !== null) {
    cancelAnimationFrame(histogramAnimRaf);
    histogramAnimRaf = null;
  }
  const step = () => {
    const changedL = lerpHistogram(histogramData, displayedHistData);
    const changedR = lerpHistogram(histogramDataR, displayedHistDataR);
    const changedG = lerpHistogram(histogramDataG, displayedHistDataG);
    const changedB = lerpHistogram(histogramDataB, displayedHistDataB);
    gaussianSmooth(displayedHistData, smoothedHistData);
    gaussianSmooth(displayedHistDataR, smoothedHistDataR);
    gaussianSmooth(displayedHistDataG, smoothedHistDataG);
    gaussianSmooth(displayedHistDataB, smoothedHistDataB);
    histogramVersion.value++;
    if (changedL || changedR || changedG || changedB) {
      histogramAnimRaf = requestAnimationFrame(step);
    } else {
      histogramAnimRaf = null;
    }
  };
  histogramAnimRaf = requestAnimationFrame(step);
}

function presetThumbnailFilter(presetKey: string) {
  const p = presetKey === 'custom'
    ? getConfiguredCustomPreset()
    : presetKey === 'auto'
      ? (autoPresetValues.value || presets.natural)
      : presets[presetKey];
  if (!p) return '';
  return buildAdjustmentFilter(
    presetKey === 'custom'
      ? { ...p, brightness: brightness.value, contrast: contrast.value, saturation: saturation.value, hue: hue.value, blur: blur.value, filter: selectedFilter.value }
      : p
  );
}

const resetAdjustments = () => {
  const p = presets.natural;
  brightness.value = p.brightness;
  contrast.value = p.contrast;
  saturation.value = p.saturation;
  hue.value = p.hue;
  blur.value = p.blur;
  selectedFilter.value = p.filter;
  selectedPreset.value = 'natural';
  showDiffPreview.value = false;
  showOriginalWhilePressed.value = false;
};

function setActiveEditorTab(tab: 'edit' | 'adjust') {
  if (cropStatus.value === 1) return;
  activeEditorTab.value = tab;
  config.imageEditor.tab = tab;
}

function handlePreviewPointerDown() {
  if (activeEditorTab.value !== 'adjust' || showDiffPreview.value || !hasAdjustmentChanges.value) return;
  showOriginalWhilePressed.value = true;
}

function handlePreviewPointerUp() {
  showOriginalWhilePressed.value = false;
}

function toggleDiffPreview() {
  if (!hasAdjustmentChanges.value) return;
  showOriginalWhilePressed.value = false;
  showDiffPreview.value = !showDiffPreview.value;
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
  autoFitVisualArea();
};

const clickRestoreAll = () => {
  if (cropStatus.value === 1 || cropApplied.value) return;

  rotate.value = initialDisplayRotate.value;
  isFlippedX.value = false;
  isFlippedY.value = false;
  isPortrait.value = isPortraitForRotation(imageWidth.value, imageHeight.value, initialDisplayRotate.value);
  autoFitVisualArea();
};

const clickCancelCrop = () => {
  cropStatus.value = 0;
  cropApplied.value = false;
  crop.value = { left: 0, top: 0, width: 0, height: 0 };
  cropBox.value = { left: 0, top: 0, width: 0, height: 0 };
  autoFitVisualArea();
};

const clickRestoreCrop = () => {
  cropStatus.value = 1;
  cropBoxFixed.value = false;
  autoFitVisualArea();
};

const autoFitVisualArea = () => {
  if (cropApplied.value) {
    fitCropBoxToContainer();
  } else {
    fitImageToContainer();
  }
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
      if (activeEditorTab.value === 'adjust' && !isProcessing.value && cropStatus.value !== 1) {
        movePresetSelection(-1);
        event.preventDefault();
        event.stopPropagation();
      }
      break;
    case 'ArrowRight':
      if (activeEditorTab.value === 'adjust' && !isProcessing.value && cropStatus.value !== 1) {
        movePresetSelection(1);
        event.preventDefault();
        event.stopPropagation();
      }
      break;
    case 'Enter':
      if (isProcessing.value) break;
      if (cropStatus.value === 1) {
        clickDoCrop();
      } else {
        clickSave();
      }
      event.preventDefault();
      event.stopPropagation();
      break;
    case 'Escape':
      if (showDiffPreview.value) {
        showDiffPreview.value = false;
      } else if (cropStatus.value === 1) {
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
  if (uiStore.activeAdjustments.filePath === fileInfo.value?.file_path) {
    uiStore.clearActiveAdjustments();
  }
  getCurrentWindow().close();
};

function closeSaveDropdown() {
  (document.activeElement as HTMLElement)?.blur();
}

function movePresetSelection(direction: number) {
  const currentIndex = presetOptions.value.findIndex(option => option.value === selectedPreset.value);
  if (currentIndex === -1) return;
  const nextIndex = Math.max(0, Math.min(presetOptions.value.length - 1, currentIndex + direction));
  selectedPreset.value = presetOptions.value[nextIndex].value;
}

const setEditParams = (overrides: { fileName?: string; destFilePath?: string; outputFormat?: string } = {}) => {
  let name = overrides.fileName || newFileName.value;
  let outputFormat = overrides.outputFormat || getSelectedOutputFormat();

  let destFilePath = overrides.destFilePath;
  if (!destFilePath) {
    destFilePath = getFullPath(getFolderPath(fileInfo.value.file_path), combineFileName(name, outputFormat));
  }

  return {
    sourceFilePath: fileInfo.value.file_path,
    destFilePath,
    outputFormat,
    quality: [90, 80, 60][config.imageEditor.quality] || 80,
    orientation: fileInfo.value.e_orientation || 1,
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
      width: resizeOutput.value.hasResize ? resizeOutput.value.width : null,
      height: resizeOutput.value.hasResize ? resizeOutput.value.height : null,
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
  const savedFilePath = overrides.destFilePath || fileInfo.value.file_path;
  const saveAsNew = savedFilePath !== fileInfo.value.file_path;
  try {
    success = await editImage(setEditParams(overrides));
  } finally {
    isProcessing.value = false;
    if (success) {
      uiStore.updateFileVersion(fileInfo.value.file_path);
      if (savedFilePath !== fileInfo.value.file_path) {
        uiStore.updateFileVersion(savedFilePath);
      }
      if (uiStore.activeAdjustments.filePath === fileInfo.value.file_path) {
        uiStore.clearActiveAdjustments();
      }
      sendToParent({ type: 'success', saveAsNew, filePath: savedFilePath });
      getCurrentWindow().close();
    } else {
      sendToParent({ type: 'failed' });
    }
  }
};

const clickSave = async () => {
  if (cropStatus.value === 1 || isProcessing.value) return;

  if (effectiveSaveAsNew.value) {
    isProcessing.value = true;
    try {
      const folderPath = getFolderPath(fileInfo.value.file_path);
      const ext = getSelectedOutputFormat();
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
      sendToParent({ type: 'failed' });
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
