<template>

  <div class="w-screen h-screen flex flex-col overflow-hidden bg-base-300 text-base-content/70">
    <!-- Title Bar -->
    <TitleBar
      v-if="showDesktopTitleBar"
      :titlebar="titleText"
      :resizable="false"
      viewName="Montage"
      class="shrink-0 z-50"
    />
    <div
      v-else
      class="h-10 shrink-0 flex items-center justify-center px-20 select-none"
      data-tauri-drag-region
    >
      <div class="min-w-0 max-w-full text-center text-sm font-medium text-base-content/70 truncate" data-tauri-drag-region>
        {{ titleText }}
      </div>
    </div>

    <!-- Main Content -->
    <div class="flex-1 flex gap-3 p-3 min-h-0 select-none">
      <!-- Left: Preview -->
      <div class="flex-1 min-w-0 flex flex-col items-center justify-center gap-2">
        <div
          ref="containerRef"
          class="relative w-full flex-1 min-h-0 rounded-box overflow-hidden border border-base-content/5 bg-base-300/30 shadow-sm flex items-center justify-center"
        >
          <transition name="fade">
            <div v-if="isProcessing" class="absolute inset-0 bg-base-100/50 flex items-center justify-center z-50 rounded-box">
              <span class="loading loading-dots text-primary"></span>
            </div>
          </transition>

          <div v-if="pageBox.width > 0" class="relative isolate" :style="{ width: `${pageBox.width}px`, height: `${pageBox.height}px` }">
            <div
              ref="pageRef"
              class="absolute inset-0 overflow-hidden shadow-lg"
              :style="{ backgroundColor: background }"
              @pointerdown.self="activeId = undefined"
            >
              <div
                v-for="item in movedLayout"
                :key="item.fileId"
                class="group absolute"
                :class="{
                  'cursor-move': mode === 'pile',
                  'cursor-grab': mode !== 'pile',
                  'outline-2 outline-primary': mode === 'pile' && item.fileId === activeId,
                  'opacity-35': mode !== 'pile' && item.fileId === dragSource,
                  'outline-3 outline-dashed -outline-offset-3 outline-primary': item.fileId === dropTarget,
                }"
                :style="frameStyle(item)"
                :data-frame-id="item.fileId"
                @pointerdown="startDrag($event, item.fileId, mode === 'pile' ? 'move' : 'swap')"
              >
                <div class="relative w-full overflow-hidden" :style="{ height: `calc(100% - ${captionHeight(item) * pageBox.width}px)` }">
                  <img
                    :src="fileById.get(item.fileId)?.thumbnail"
                    class="absolute left-1/2 top-1/2 max-w-none object-cover pointer-events-none"
                    :style="photoStyle(item)"
                  />
                </div>
                <img v-if="captions.get(item.fileId)" :src="captions.get(item.fileId)" class="absolute pointer-events-none" :style="{ left: `${item.border * pageBox.width}px`, bottom: `${item.border * pageBox.width}px`, width: `calc(100% - ${2 * item.border * pageBox.width}px)`, height: `${captionHeight(item) * pageBox.width}px` }" />
                <button
                  class="absolute top-1 right-1 size-5 rounded-full bg-black/60 text-white flex items-center justify-center opacity-0 group-hover:opacity-100 focus-visible:opacity-100 cursor-pointer disabled:cursor-default"
                  :disabled="files.length <= 2 || isProcessing"
                  :aria-label="$t('tooltip.album.remove')"
                  @pointerdown.stop
                  @click.stop="removePhoto(item.fileId)"
                ><IconClose class="size-3" /></button>
              </div>
            </div>

            <!-- Drag the handle around to rotate, away from the center to scale. -->
            <div
              v-if="dial"
              class="absolute pointer-events-none"
              :style="{ left: `${dial.x}px`, top: `${dial.y}px`, zIndex: layout.length + 1 }"
            >
              <svg class="absolute left-0 top-0 overflow-visible drop-shadow-sm" width="1" height="1">
                <circle :r="DIAL_RADIUS" fill="none" stroke="rgb(0 0 0 / 0.35)" :stroke-width="DIAL_WIDTH" />
                <line
                  v-for="tick in 24"
                  :key="tick"
                  :x1="DIAL_RADIUS - DIAL_WIDTH / 2 + 4"
                  :x2="DIAL_RADIUS - DIAL_WIDTH / 2 + (tick % 6 === 0 ? 12 : tick % 3 === 0 ? 9 : 6)"
                  stroke="rgb(255 255 255 / 0.75)"
                  :stroke-width="tick % 6 === 0 ? 2 : 1"
                  stroke-linecap="round"
                  :transform="`rotate(${tick * 15})`"
                />
                <g :transform="`rotate(${dial.angle})`" stroke-linecap="round">
                  <line :x1="DIAL_RADIUS + DIAL_WIDTH / 2" :x2="DIAL_HANDLE - 12" stroke="rgb(0 0 0 / 0.35)" stroke-width="4" />
                  <line :x1="DIAL_RADIUS + DIAL_WIDTH / 2" :x2="DIAL_HANDLE - 12" stroke="var(--color-primary)" stroke-width="2" />
                </g>
              </svg>
              <div
                class="group absolute size-9 -translate-1/2 rounded-full pointer-events-auto flex items-center justify-center"
                :class="activeDragKind === 'transform' ? 'cursor-grabbing' : 'cursor-grab'"
                :style="dial.handle"
                @pointerdown.stop="startDrag($event, dial.item.fileId, 'transform')"
              >
                <span class="size-6 rounded-full border border-base-content/15 shadow-md flex items-center justify-center transition-colors group-hover:bg-primary group-hover:text-primary-content"
                  :class="activeDragKind === 'transform' ? 'bg-primary text-primary-content' : 'bg-base-100 text-primary'">
                  <IconRotate class="size-3.5" />
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Right: Settings -->
      <div class="w-80 shrink-0 flex flex-col gap-3 overflow-y-auto text-xs">
        <section class="rounded-box p-3 space-y-2 border border-base-content/5 shadow-sm bg-base-300/30">
          <div class="font-semibold text-base-content/80">{{ $t('msgbox.montage.layout') }}</div>
          <div class="flex items-center gap-1">
            <TButton
              v-for="option in modeOptions"
              :key="option.value"
              buttonSize="small"
              :icon="option.icon"
              :icon-style="{ transform: `rotate(${option.value === 'masonry' ? 90 : 0}deg)` }"
              :selected="mode === option.value"
              :tooltip="option.label"
              @click="mode = option.value"
            />
          </div>
          <div v-if="mode === 'card' || mode === 'grid'" class="flex items-center justify-between gap-2">
            <span>{{ $t('settings.grid.scaling') }}</span>
            <select v-model="scaling" class="select select-bordered select-xs w-40">
              <option v-for="(label, index) in localeMsg.settings.grid.scaling_options" :key="index" :value="index">{{ label }}</option>
            </select>
          </div>
          <div v-if="mode === 'card'" class="flex items-center justify-between gap-2">
            <span>{{ $t('settings.grid.label_primary') }}</span>
            <select v-model="captionField" class="select select-bordered select-xs w-40">
              <option v-for="(label, index) in localeMsg.settings.grid.label_options" :key="index" :value="index">{{ label }}</option>
            </select>
          </div>
          <button class="btn btn-sm w-full" @click="shuffle">
            <IconSortingShuffle class="size-4" />{{ $t('msgbox.montage.shuffle') }}
          </button>
        </section>

        <section class="rounded-box p-3 space-y-2 border border-base-content/5 shadow-sm bg-base-300/30">
          <div class="font-semibold text-base-content/80">{{ $t('msgbox.montage.page') }}</div>
          <div class="flex items-center justify-between gap-2">
            <span>{{ $t('msgbox.montage.page_size') }}</span>
            <select v-model="pageKey" class="select select-bordered select-xs w-40">
              <option v-for="option in pageOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
            </select>
          </div>
          <div v-if="pageKey !== 'square'" class="flex items-center justify-between gap-2">
            <span>{{ $t('msgbox.montage.orientation') }}</span>
            <div class="flex items-center gap-1">
              <TButton buttonSize="small" :icon="IconCropLandscape" :selected="landscape" :tooltip="$t('msgbox.montage.landscape')" @click="landscape = true" />
              <TButton buttonSize="small" :icon="IconCropPortrait" :selected="!landscape" :tooltip="$t('msgbox.montage.portrait')" @click="landscape = false" />
            </div>
          </div>
          <div class="text-[11px] text-base-content/30">{{ $t('msgbox.montage.export', { size: exportLabel }) }}</div>
        </section>

        <section class="rounded-box p-3 space-y-3 border border-base-content/5 shadow-sm bg-base-300/30">
          <div class="font-semibold text-base-content/80">{{ $t('msgbox.montage.style') }}</div>
          <div class="flex items-center justify-between gap-2">
            <span>{{ $t('msgbox.montage.background') }}</span>
            <div class="flex gap-2">
              <button
                v-for="color in backgroundSwatches"
                :key="color"
                class="size-5.5 rounded-md border border-base-content/15 cursor-pointer"
                :class="{ 'outline-2 outline-offset-2 outline-primary': background === color }"
                :style="{ backgroundColor: color }"
                @click="background = color"
              ></button>
              <label
                class="relative size-5.5 rounded-md border border-base-content/15 cursor-pointer overflow-hidden"
                style="background: conic-gradient(red, yellow, lime, cyan, blue, magenta, red)"
              >
                <input :value="background" type="color" class="absolute inset-0 opacity-0 cursor-pointer"
                  @input="background = ($event.target as HTMLInputElement).value" @change="rememberCustomBackground"
                  @keydown.esc.stop="($event.target as HTMLInputElement).blur()" />
              </label>
            </div>
          </div>

          <div v-if="mode === 'pile'" class="space-y-1">
            <div class="flex items-center justify-between gap-2">
              <label for="montage-rotation">{{ $t('msgbox.montage.random_rotation') }}</label>
              <span class="text-base-content/30 tabular-nums">± {{ maxRotation }}°</span>
            </div>
            <SliderInput id="montage-rotation" v-model="maxRotation" :min="0" :max="30" :step="1" class="w-full!" />
          </div>
          <div v-else class="space-y-1">
            <div class="flex items-center justify-between gap-2">
              <label for="montage-spacing">{{ $t('msgbox.montage.spacing') }}</label>
              <span class="text-base-content/30 tabular-nums whitespace-nowrap">{{ toExportPx(spacing) }} px</span>
            </div>
            <SliderInput id="montage-spacing" v-model="spacing" :min="0" :max="5" :step="0.25" class="w-full!" />
          </div>

          <label class="flex items-center justify-between gap-2 cursor-pointer">
            <span>{{ $t('msgbox.montage.border') }}</span>
            <input v-model="borderOn" type="checkbox" class="toggle toggle-xs toggle-primary" />
          </label>
          <div :class="{ 'opacity-50': !borderOn }" class="space-y-1">
            <div class="flex items-center justify-between gap-2">
              <label for="montage-border">{{ $t('msgbox.montage.border_width') }}</label>
              <span class="text-base-content/30 tabular-nums whitespace-nowrap">{{ toExportPx(border) }} px</span>
            </div>
            <SliderInput id="montage-border" v-model="border" :min="0" :max="4" :step="0.25" class="w-full!" :disabled="!borderOn" />
          </div>

          <label class="flex items-center justify-between gap-2 cursor-pointer">
            <span>{{ $t('msgbox.montage.shadow') }}</span>
            <input v-model="shadow" type="checkbox" class="toggle toggle-xs toggle-primary" />
          </label>
        </section>

      </div>
    </div>

    <!-- Bottom Bar -->
    <div class="h-14 shrink-0 flex items-center justify-end px-4 gap-2">
      <button
        class="px-4 py-1 rounded-box hover:bg-base-100 hover:text-base-content cursor-pointer text-sm mr-4"
        :disabled="isProcessing"
        @click="closeWindow"
      >{{ $t('msgbox.image_editor.cancel') }}</button>
      <button class="btn btn-sm" :disabled="isProcessing || files.length < 2" @click="printCollage">
        <IconPrint class="size-4" />{{ $t('menu.file.print') }}
      </button>
      <select v-model="combinedFormatKey" class="select select-bordered select-xs">
        <option v-for="option in combinedFormatOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
      </select>
      <button
        class="btn btn-sm btn-primary px-4"
        :disabled="isProcessing || files.length < 2"
        @click="clickSave"
      >{{ $t('msgbox.image_editor.save_as_new') }}</button>
    </div>

    <PrintImage ref="printImageView" />

    <!-- Save dialog: destination folder and file name -->
    <ModalDialog v-if="showSaveDialog" :title="$t('msgbox.montage.save_title')" :width="480" @cancel="closeSaveDialog">
      <div class="grid grid-cols-[auto_minmax(0,1fr)] gap-x-3 gap-y-3 items-center text-sm">
        <div>{{ $t('msgbox.montage.folder') }}</div>
        <div class="flex items-center gap-1">
          <input v-model="saveFolder" type="text" class="px-2 py-1 flex-1 min-w-0 input" @input="saveError = ''" />
          <TButton :icon="IconFolder" :tooltip="$t('msgbox.montage.browse')" @click="browseFolder" />
        </div>
        <div>{{ $t('msgbox.montage.file_name') }}</div>
        <div class="flex items-center">
          <input
            ref="saveNameRef"
            v-model="saveName"
            type="text"
            maxlength="255"
            class="px-2 py-1 flex-1 min-w-0 input"
            @input="saveError = ''"
            @keydown.enter.prevent="confirmSave"
          />
          <span class="label px-2 text-sm">.{{ outputFormat }}</span>
        </div>
      </div>
      <p class="p-1 min-h-6 text-error text-xs wrap-break-word">{{ saveError }}</p>
      <div class="mt-2 flex justify-end space-x-4">
        <button class="t-button-default" :disabled="isProcessing" @click="closeSaveDialog">{{ $t('msgbox.image_editor.cancel') }}</button>
        <button class="t-button-primary" :disabled="isProcessing" @click="confirmSave">
          <span v-if="isProcessing" class="loading loading-spinner loading-xs mr-2"></span>{{ $t('msgbox.image_editor.save_as_new') }}
        </button>
      </div>
    </ModalDialog>
  </div>

</template>

<script setup lang="ts">
import { MONTAGE_MAX_PHOTOS, MONTAGE_BACKGROUND_PRESETS, MONTAGE_PAGE_SIZES } from '@/common/constants';
import { ref, toRefs, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { isWin, isLinux, getFolderPath, getFullPath, combineFileName, getSelectOptions, getThumbUrl, isValidFileName, formatFileSize, formatDimensionText, formatTimestamp, formatCameraInfo, formatCaptureSettings } from '@/common/utils';
import { getFileInfo, checkFileExists, renderMontage } from '@/common/api';
import { computeMontageLayout, shuffled, type MontageItem } from '@/common/montageLayout';
import { getAspectRatio } from '@/common/layout';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { emit as tauriEmit } from '@tauri-apps/api/event';
import { open as openDialog, message } from '@tauri-apps/plugin-dialog';

import PrintImage from '@/components/PrintImage.vue';
import TitleBar from '@/components/TitleBar.vue';
import TButton from '@/components/TButton.vue';
import SliderInput from '@/components/SliderInput.vue';
import ModalDialog from '@/components/ModalDialog.vue';

import { IconCard, IconTile, IconJustified, IconCollage, IconSortingShuffle, IconFolder, IconClose, IconRotate, IconCropLandscape, IconCropPortrait, IconPrint } from '@/common/icons';

type PageKey = keyof typeof MONTAGE_PAGE_SIZES;
const PRINT_PAGES: PageKey[] = ['a4', 'photo'];

const router = useRouter();
const { t, locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const appWindow = getCurrentWebviewWindow();
const showDesktopTitleBar = isWin || isLinux;

const files = ref<any[]>([]); // montage order
const { mode, captionField, pageKey, landscape, background, spacing,
  borderOn, border, scaling, maxRotation, shadow } = toRefs(config.montage);
watch(() => config.montage, (preferences) => {
  void tauriEmit('montage-config-changed', JSON.parse(JSON.stringify(preferences)));
}, { deep: true });
const backgroundSwatches = computed(() => {
  const saved = String(config.montage.lastCustomBackground).toLowerCase();
  const recent = /^#[0-9a-f]{6}$/.test(saved) && !MONTAGE_BACKGROUND_PRESETS.includes(saved) ? saved : '#7c2d12';
  return [...MONTAGE_BACKGROUND_PRESETS, MONTAGE_BACKGROUND_PRESETS.includes(background.value) ? recent : background.value];
});
function rememberCustomBackground(event: Event) {
  const color = (event.target as HTMLInputElement).value.toLowerCase();
  background.value = color;
  if (!MONTAGE_BACKGROUND_PRESETS.includes(color)) {
    config.montage.lastCustomBackground = color;
  }
}
const photoScaling = computed(() => mode.value === 'card' || mode.value === 'grid' ? scaling.value : 1);
const seed = ref(1);        // pile placement
const isProcessing = ref(false);

const titleText = computed(() => `${t('msgbox.montage.title')} - ${t('toolbar.filter.select_count', { count: files.value.length })}`);

const modeOptions = computed(() => [
  { value: 'card' as const, icon: IconCard, label: localeMsg.value.settings.grid.style_options[0] },
  { value: 'grid' as const, icon: IconTile, label: localeMsg.value.settings.grid.style_options[1] },
  { value: 'mosaic' as const, icon: IconJustified, label: localeMsg.value.settings.grid.style_options[2] },
  { value: 'masonry' as const, icon: IconJustified, label: localeMsg.value.settings.grid.style_options[3] },
  { value: 'pile' as const, icon: IconCollage, label: t('msgbox.montage.pile') },
]);

const pageOptions = computed(() => [
  { value: 'a4', label: 'A4' },
  { value: 'photo', label: '10 × 15 cm' },
  { value: 'screen', label: t('msgbox.montage.page_screen') },
  { value: 'square', label: t('msgbox.montage.page_square') },
]);

const pageSize = computed(() => {
  const [long, short] = MONTAGE_PAGE_SIZES[pageKey.value];
  return landscape.value ? [long, short] : [short, long];
});

const exportLabel = computed(() =>
  `${pageSize.value[0]} × ${pageSize.value[1]} px${PRINT_PAGES.includes(pageKey.value) ? ' · 300 dpi' : ''}`);

// a % of the page width, in pixels of the exported image
function toExportPx(percent: number) {
  return Math.round(percent / 100 * pageSize.value[0]);
}

const fileById = computed(() => new Map(files.value.map(file => [Number(file.id), file])));

const preservedFreeformLayout = ref<MontageItem[] | null>(null);
const baseLayout = computed(() => mode.value === 'pile' && preservedFreeformLayout.value
  ? preservedFreeformLayout.value.filter(item => fileById.value.has(item.fileId))
  : computeMontageLayout(
  files.value.map(file => ({ fileId: Number(file.id), ratio: getAspectRatio(file) })),
  pageSize.value[0] / pageSize.value[1],
  mode.value,
  { spacing: spacing.value / 100, border: borderOn.value ? border.value / 100 : 0, rotation: maxRotation.value },
  seed.value,
));

// pile: photos adjusted by hand (page-relative offset, scale around the center, extra rotation)
// and brought to the front (last = topmost)
type Adjust = { dx: number; dy: number; scale: number; angle: number };
const NO_ADJUST: Adjust = { dx: 0, dy: 0, scale: 1, angle: 0 };
const adjusts = ref(new Map<number, Adjust>());
const raised = ref<number[]>([]);
const activeId = ref<number>(); // photo showing the dial
watch([mode, seed, pageKey, landscape,
  () => mode.value === 'pile' ? '' : files.value.map(file => file.id).join(',')], () => {
  preservedFreeformLayout.value = null;
  adjusts.value = new Map();
  raised.value = [];
  activeId.value = undefined;
});

watch([spacing, borderOn, border, maxRotation], () => {
  preservedFreeformLayout.value = null;
});

const movedLayout = computed(() => baseLayout.value.map(item => {
  const a = adjusts.value.get(item.fileId);
  if (!a) return item;
  const w = item.w * a.scale;
  const h = item.h * a.scale;
  return {
    ...item,
    x: item.x + a.dx + (item.w - w) / 2,
    y: item.y + a.dy + (item.h - h) / 2,
    w,
    h,
    border: item.border * a.scale,
    rotation: item.rotation + a.angle,
  };
}));

// drawing order, shared by the preview (z-index) and the backend render
const layout = computed(() => {
  const rank = (fileId: number) => raised.value.indexOf(fileId);
  return movedLayout.value.slice().sort((a, b) => rank(a.fileId) - rank(b.fileId));
});

function captionHeight(item: MontageItem) {
  return mode.value === 'card' && captionField.value > 0
    ? Math.max(0, Math.min(0.025, (item.h * pageSize.value[1] / pageSize.value[0] - 2 * item.border) * 0.3)) : 0;
}

const captions = computed(() => {
  const result = new Map<number, string>();
  for (const item of layout.value) {
    const height = captionHeight(item);
    if (!height) continue;
    const file = fileById.value.get(item.fileId);
    const fields = ['', file.name, formatFileSize(file.size), formatDimensionText(file.width, file.height),
      formatTimestamp(file.taken_date, 'date_time'), file.geo_name, formatCameraInfo(file.e_make, file.e_model),
      file.e_lens_model, formatCaptureSettings(file.e_focal_length, file.e_exposure_time, file.e_f_number, file.e_iso_speed, file.e_exposure_bias)];
    const canvas = document.createElement('canvas');
    canvas.width = Math.max(1, Math.round((item.w - 2 * item.border) * pageSize.value[0]));
    canvas.height = Math.max(1, Math.round(height * pageSize.value[0]));
    const ctx = canvas.getContext('2d')!;
    ctx.fillStyle = borderColor.value;
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.font = `${canvas.height * 0.55}px sans-serif`;
    ctx.fillStyle = '#333333';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    let text = String(fields[captionField.value] || '');
    const maxWidth = Math.max(0, canvas.width - canvas.height * 0.5);
    if (ctx.measureText(text).width > maxWidth) {
      const chars = Array.from(text);
      while (chars.length && ctx.measureText(chars.join('') + '…').width > maxWidth) chars.pop();
      text = chars.length ? chars.join('') + '…' : '';
    }
    ctx.fillText(text, canvas.width / 2, canvas.height / 2);
    result.set(item.fileId, canvas.toDataURL('image/png'));
  }
  return result;
});

// dial geometry, in px around the photo's center
const DIAL_RADIUS = 56;
const DIAL_WIDTH = 20;
const DIAL_HANDLE = 92;

const dial = computed(() => {
  if (mode.value !== 'pile' || activeId.value === undefined || activeDragKind.value === 'move') return null;
  const item = movedLayout.value.find(other => other.fileId === activeId.value);
  if (!item) return null;
  const radians = item.rotation * Math.PI / 180;
  return {
    item,
    x: (item.x + item.w / 2) * pageBox.value.width,
    y: (item.y + item.h / 2) * pageBox.value.height,
    angle: item.rotation,
    handle: { left: `${Math.cos(radians) * DIAL_HANDLE}px`, top: `${Math.sin(radians) * DIAL_HANDLE}px` },
  };
});

// Move / transform a freeform photo, or swap photos in the preview.
type DragKind = 'move' | 'transform' | 'swap';
const activeDragKind = ref<DragKind>();
const pageRef = ref<HTMLElement | null>(null);
const dragSource = ref<number>(); // swap: dragged photo
const dropTarget = ref<number>(); // swap: photo under the pointer
let drag: { kind: DragKind; fileId: number; startX: number; startY: number; centerX: number; centerY: number; start: Adjust } | null = null;

function startDrag(event: PointerEvent, fileId: number, kind: DragKind) {
  if (event.button !== 0 || isProcessing.value) return;
  let centerX = 0, centerY = 0;
  if (kind === 'move' || kind === 'transform') {
    const item = movedLayout.value.find(other => other.fileId === fileId);
    if (!item || !pageRef.value) return;
    const page = pageRef.value.getBoundingClientRect();
    centerX = page.left + (item.x + item.w / 2) * page.width;
    centerY = page.top + (item.y + item.h / 2) * page.height;
    raised.value = [...raised.value.filter(id => id !== fileId), fileId];
    activeId.value = fileId;
  } else {
    dragSource.value = fileId;
  }
  activeDragKind.value = kind;
  drag = {
    kind,
    fileId,
    startX: event.clientX,
    startY: event.clientY,
    centerX,
    centerY,
    start: { ...(adjusts.value.get(fileId) ?? NO_ADJUST) },
  };
  window.addEventListener('pointermove', onDrag);
  window.addEventListener('pointerup', endDrag);
  window.addEventListener('pointercancel', endDrag);
}

function onDrag(event: PointerEvent) {
  if (!drag) return;
  if (drag.kind === 'swap') {
    const target = document.elementFromPoint(event.clientX, event.clientY)?.closest<HTMLElement>('[data-frame-id]');
    const id = Number(target?.dataset.frameId);
    dropTarget.value = id && id !== drag.fileId ? id : undefined;
    return;
  }
  const { start, centerX, centerY } = drag;
  const next = { ...start };
  if (drag.kind === 'move') {
    next.dx = start.dx + (event.clientX - drag.startX) / pageBox.value.width;
    next.dy = start.dy + (event.clientY - drag.startY) / pageBox.value.height;
  } else {
    // distance to the center scales, angle around it rotates
    const ratio = Math.hypot(event.clientX - centerX, event.clientY - centerY)
      / Math.max(1, Math.hypot(drag.startX - centerX, drag.startY - centerY));
    next.scale = Math.min(4, Math.max(0.2, start.scale * ratio));
    const turn = Math.atan2(event.clientY - centerY, event.clientX - centerX)
      - Math.atan2(drag.startY - centerY, drag.startX - centerX);
    next.angle = start.angle + turn * 180 / Math.PI;
  }
  adjusts.value.set(drag.fileId, next);
}

function endDrag() {
  if (drag && dropTarget.value !== undefined) {
    const ids = files.value.map(file => Number(file.id));
    const from = ids.indexOf(drag.fileId);
    const to = ids.indexOf(dropTarget.value);
    const next = files.value.slice();
    [next[from], next[to]] = [next[to], next[from]];
    files.value = next;
  }
  drag = null;
  activeDragKind.value = undefined;
  dragSource.value = undefined;
  dropTarget.value = undefined;
  window.removeEventListener('pointermove', onDrag);
  window.removeEventListener('pointerup', endDrag);
  window.removeEventListener('pointercancel', endDrag);
}

function shuffle() {
  files.value = shuffled(files.value);
  seed.value = Math.floor(Math.random() * 0xffffffff) + 1;
}

function removePhoto(fileId: number) {
  if (files.value.length <= 2 || isProcessing.value) return;
  if (mode.value === 'pile') preservedFreeformLayout.value = baseLayout.value.filter(item => item.fileId !== fileId);
  files.value = files.value.filter(file => Number(file.id) !== fileId);
  adjusts.value.delete(fileId);
  raised.value = raised.value.filter(id => id !== fileId);
  if (activeId.value === fileId) activeId.value = undefined;
}

async function loadFiles(ids: number[]) {
  const infos = await Promise.all(ids.map(id => getFileInfo(id)));
  return infos
    .filter((file: any) => file && file.file_type !== 2)
    .map((file: any) => ({ ...file, thumbnail: getThumbUrl(file.id) }));
}

// fit the page into the preview area
const containerRef = ref<HTMLElement | null>(null);
const containerSize = ref({ width: 0, height: 0 });
let resizeObserver: ResizeObserver | null = null;
const pageBox = computed(() => {
  const padding = 5;
  const scale = Math.min(
    (containerSize.value.width - padding * 2) / pageSize.value[0],
    (containerSize.value.height - padding * 2) / pageSize.value[1],
  );
  return scale > 0
    ? { width: pageSize.value[0] * scale, height: pageSize.value[1] * scale }
    : { width: 0, height: 0 };
});

// white borders, unless the background is too light for white to show:
// then a shade of the background, close to it but visible
const borderColor = computed(() => {
  const rgb = [1, 3, 5].map(i => parseInt(background.value.slice(i, i + 2), 16));
  const luminance = (0.299 * rgb[0] + 0.587 * rgb[1] + 0.114 * rgb[2]) / 255;
  if (luminance < 0.85) return '#ffffff';
  return '#' + rgb.map(c => Math.round(c * 0.88).toString(16).padStart(2, '0')).join('');
});

// drop shadow, relative to the page width (same values as t_montage.rs);
// the offset is turned back so it points down-right on the page, like the render
const SHADOW_OFFSET = 0.004;
const SHADOW_BLUR = 0.004; // gaussian sigma
function shadowStyle(rotation: number) {
  const offset = SHADOW_OFFSET * pageBox.value.width;
  const radians = -rotation * Math.PI / 180;
  const x = offset * (Math.cos(radians) - Math.sin(radians));
  const y = offset * (Math.sin(radians) + Math.cos(radians));
  return `${x}px ${y}px ${2 * SHADOW_BLUR * pageBox.value.width}px rgb(0 0 0 / 0.45)`;
}

function frameStyle(item: MontageItem) {
  return {
    left: `${item.x * 100}%`,
    top: `${item.y * 100}%`,
    width: `${item.w * 100}%`,
    height: `${item.h * 100}%`,
    padding: `${item.border * pageBox.value.width}px`,
    backgroundColor: item.border > 0 || photoScaling.value === 0 ? borderColor.value : 'transparent',
    transform: item.rotation ? `rotate(${item.rotation}deg)` : undefined,
    boxShadow: shadow.value ? shadowStyle(item.rotation) : undefined,
    zIndex: layout.value.findIndex(other => other.fileId === item.fileId),
  };
}

// cover the frame with the thumbnail, applying Lap's own rotation
function photoStyle(item: MontageItem) {
  const rotate = Number(fileById.value.get(item.fileId)?.rotate || 0);
  const innerW = item.w * pageBox.value.width - 2 * item.border * pageBox.value.width;
  const innerH = item.h * pageBox.value.height - (2 * item.border + captionHeight(item)) * pageBox.value.width;
  const swap = rotate % 180 !== 0;
  return {
    width: `${swap ? innerH : innerW}px`,
    height: `${swap ? innerW : innerH}px`,
    objectFit: ['contain', 'cover', 'fill'][photoScaling.value] as 'contain' | 'cover' | 'fill',
    transform: `translate(-50%, -50%) rotate(${rotate}deg)`,
  };
}

// Collage export preferences are independent of the image editor.
const outputFormatValues = ['jpg', 'png', 'webp'] as const;
const combinedFormatKey = computed({
  get: () => config.montage.format !== 0 ? String(config.montage.format) : `0-${config.montage.quality}`,
  set: (key: string) => {
    const [format, quality] = key.split('-').map(Number);
    config.montage.format = format;
    config.montage.quality = quality || 0;
  },
});
const combinedFormatOptions = computed(() => {
  const formats = getSelectOptions(localeMsg.value.msgbox.image_editor.format_options);
  const qualities = getSelectOptions(localeMsg.value.msgbox.image_editor.quality_options);
  return [
    ...qualities.map((quality: any, i: number) => ({ value: `0-${i}`, label: `${formats[0].label} (${quality.label})` })),
    ...formats.slice(1).map((format: any, i: number) => ({ value: String(i + 1), label: format.label })),
  ];
});

const outputFormat = computed(() => outputFormatValues[config.montage.format] || outputFormatValues[0]);

// save dialog, prefilled with the first photo's folder and a free montage_N name
const showSaveDialog = ref(false);
const saveFolder = ref('');
const saveName = ref('');
const saveError = ref('');
const saveNameRef = ref<HTMLInputElement | null>(null);

async function clickSave() {
  if (isProcessing.value || files.value.length < 2) return;
  saveError.value = '';
  saveFolder.value = getFolderPath(files.value[0].file_path);
  let counter = 1;
  while (await checkFileExists(getFullPath(saveFolder.value, combineFileName(`montage_${counter}`, outputFormat.value)))) {
    counter++;
  }
  saveName.value = `montage_${counter}`;
  showSaveDialog.value = true;
  await nextTick();
  saveNameRef.value?.select();
}

function closeSaveDialog() {
  if (!isProcessing.value) showSaveDialog.value = false;
}

async function browseFolder() {
  const folder = await openDialog({ directory: true, multiple: false, defaultPath: saveFolder.value || undefined });
  if (typeof folder === 'string') {
    saveFolder.value = folder;
    saveError.value = '';
  }
}

function renderParams() {
  return {
    width: pageSize.value[0],
    height: pageSize.value[1],
    background: background.value,
    borderColor: borderColor.value,
    shadow: shadow.value,
    items: layout.value.map(item => {
      const file = fileById.value.get(item.fileId);
      return { ...item, scaling: photoScaling.value, path: file.file_path, orientation: Number(file.e_orientation || 1), rotate: Number(file.rotate || 0), caption: captions.value.get(item.fileId)?.split(',')[1], captionHeight: captionHeight(item) };
    }),
  };
}

const printImageView = ref<InstanceType<typeof PrintImage> | null>(null);
async function printCollage() {
  if (isProcessing.value || files.value.length < 2) return;
  isProcessing.value = true;
  try {
    const params = renderParams();
    const png = await renderMontage({ ...params, forPrint: true, destFilePath: '', outputFormat: 'png' });
    await printImageView.value!.print(`data:image/png;base64,${png}`, params.width > params.height ? 'landscape' : 'portrait');
  } catch (error) {
    await message(`${t('msgbox.montage.save_failed')} ${error}`, { kind: 'error' });
  } finally {
    isProcessing.value = false;
  }
}

async function confirmSave() {
  if (isProcessing.value) return;
  const folderPath = saveFolder.value.trim().replace(/[\\/]+$/, '');
  const name = saveName.value.trim();
  if (!folderPath) {
    saveError.value = t('msgbox.montage.folder_required');
    return;
  }
  if (!name || !isValidFileName(name)) {
    saveError.value = t('msgbox.input.file_name_invalid');
    return;
  }
  const destFilePath = getFullPath(folderPath, combineFileName(name, outputFormat.value));
  if (await checkFileExists(destFilePath)) {
    saveError.value = t('msgbox.file_conflict.title');
    return;
  }

  isProcessing.value = true;
  saveError.value = '';
  try {
    await renderMontage({ ...renderParams(),
      destFilePath,
      outputFormat: outputFormat.value,
      quality: [90, 80, 60][config.montage.quality] || 80,
    });
    await tauriEmit('message-from-montage', { type: 'success', filePath: destFilePath });
  } catch (error) {
    saveError.value = `${t('msgbox.montage.save_failed')} ${error}`;
  } finally {
    isProcessing.value = false;
  }
}

async function closeWindow() {
  if (isProcessing.value) return; // the render reports back through this window
  try {
    await appWindow.close();
  } catch {
    await appWindow.destroy();
  }
}

let closeOnEscapeRelease = false;
function handleKeyDown(event: KeyboardEvent) {
  if (event.key !== 'Escape') return;
  event.preventDefault();
  event.stopPropagation();
  if (event.repeat) return;
  if (showSaveDialog.value) closeSaveDialog();
  else closeOnEscapeRelease = true;
}
function handleKeyUp(event: KeyboardEvent) {
  if (event.key !== 'Escape' || !closeOnEscapeRelease) return;
  closeOnEscapeRelease = false;
  event.preventDefault();
  event.stopPropagation();
  void closeWindow();
}
function handleWindowFocus() {
  const input = document.activeElement;
  if (input instanceof HTMLInputElement && input.type === 'color') input.blur();
}

watch(() => config.settings.language, (newLanguage) => {
  locale.value = newLanguage;
});

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown);
  window.addEventListener('keyup', handleKeyUp);
  window.addEventListener('focus', handleWindowFocus);
  resizeObserver = new ResizeObserver(([entry]) => {
    containerSize.value = { width: entry.contentRect.width, height: entry.contentRect.height };
  });
  if (containerRef.value) resizeObserver.observe(containerRef.value);

  const ids = String(router.currentRoute.value.query.fileIds || '').split(',').map(Number).filter(id => id > 0);
  files.value = await loadFiles(ids.slice(0, MONTAGE_MAX_PHOTOS));
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  window.removeEventListener('keyup', handleKeyUp);
  window.removeEventListener('focus', handleWindowFocus);
  endDrag();
  resizeObserver?.disconnect();
});

</script>
