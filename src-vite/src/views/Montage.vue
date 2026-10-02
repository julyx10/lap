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
      <!-- Left: Preview and hint -->
      <div class="flex-1 min-w-0 flex flex-col gap-2">
        <div
          ref="containerRef"
          class="relative flex-1 min-h-0 rounded-box overflow-hidden border border-base-content/5 bg-base-300/30 shadow-sm flex items-center justify-center"
        >
          <transition name="fade">
            <div v-if="isProcessing" class="absolute inset-0 z-50 flex items-center justify-center bg-base-100/55 backdrop-blur-sm">
              <span class="loading loading-dots text-primary"></span>
            </div>
          </transition>

          <div v-if="pageBox.width > 0" class="relative" :style="{ width: `${pageBox.width}px`, height: `${pageBox.height}px` }">
            <div
              ref="pageRef"
              class="absolute inset-0 overflow-hidden shadow-lg"
              :style="{ backgroundColor: background }"
              @pointerdown.self="activeId = undefined"
            >
              <div
                v-for="item in movedLayout"
                :key="item.fileId"
                class="absolute"
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
                <div class="relative w-full h-full overflow-hidden">
                  <img
                    :src="fileById.get(item.fileId)?.thumbnail"
                    class="absolute left-1/2 top-1/2 max-w-none object-cover pointer-events-none"
                    :style="photoStyle(item)"
                  />
                </div>
              </div>
            </div>

            <!-- dial of the active photo (Picasa style): drag the handle around to rotate, away from the center to scale -->
            <div
              v-if="dial"
              class="absolute pointer-events-none"
              :style="{ left: `${dial.x}px`, top: `${dial.y}px`, zIndex: layout.length + 1 }"
            >
              <svg class="absolute left-0 top-0 overflow-visible" width="1" height="1">
                <circle :r="DIAL_RADIUS" fill="none" stroke="rgb(0 0 0 / 0.4)" :stroke-width="DIAL_WIDTH" />
                <line
                  v-for="tick in 24"
                  :key="tick"
                  :x1="DIAL_RADIUS - DIAL_WIDTH / 2 + 3" :x2="DIAL_RADIUS - DIAL_WIDTH / 2 + (tick % 2 ? 6 : 9)"
                  stroke="rgb(255 255 255 / 0.7)"
                  :transform="`rotate(${tick * 15})`"
                />
                <line x1="-6" x2="6" stroke="rgb(255 255 255 / 0.7)" />
                <line y1="-6" y2="6" stroke="rgb(255 255 255 / 0.7)" />
                <g :transform="`rotate(${dial.angle})`">
                  <line :x1="DIAL_RADIUS + DIAL_WIDTH / 2" :x2="DIAL_HANDLE" stroke="white" stroke-width="2" />
                  <circle :cx="DIAL_HANDLE" r="7" fill="white" stroke="var(--color-primary)" stroke-width="4" />
                </g>
              </svg>
              <div
                class="absolute size-6 -translate-1/2 rounded-full pointer-events-auto cursor-grab"
                :style="dial.handle"
                @pointerdown.stop="startDrag($event, dial.item.fileId, 'transform')"
              ></div>
              <div class="absolute -translate-1/2 flex flex-col items-center gap-1 text-[11px] font-medium text-white whitespace-nowrap">
                <span class="px-1.5 rounded bg-black/40">{{ $t('msgbox.montage.angle', { value: dial.angleLabel }) }}</span>
                <span class="px-1.5 rounded bg-black/40 mt-4">{{ $t('msgbox.montage.scale', { value: dial.scaleLabel }) }}</span>
              </div>
            </div>
          </div>
        </div>
        <div class="h-4 shrink-0 text-center text-[11px] text-base-content/50 truncate">{{ $t(`msgbox.montage.hint_${mode}`) }}</div>
      </div>

      <!-- Right: Settings -->
      <div ref="panelRef" class="w-80 shrink-0 flex flex-col gap-3 overflow-y-auto text-xs">
        <section class="rounded-box p-3 space-y-2 border border-base-content/5 shadow-sm bg-base-300/30">
          <div class="font-semibold text-base-content/80">{{ $t('msgbox.montage.layout') }}</div>
          <div class="flex items-center gap-1">
            <TButton
              v-for="option in modeOptions"
              :key="option.value"
              buttonSize="small"
              :icon="option.icon"
              :selected="mode === option.value"
              :tooltip="option.label"
              @click="mode = option.value"
            />
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
            <div class="join">
              <button class="btn btn-xs join-item" :class="{ 'btn-active': landscape }" @click="landscape = true">{{ $t('msgbox.montage.landscape') }}</button>
              <button class="btn btn-xs join-item" :class="{ 'btn-active': !landscape }" @click="landscape = false">{{ $t('msgbox.montage.portrait') }}</button>
            </div>
          </div>
          <div class="text-[11px] text-base-content/45">{{ $t('msgbox.montage.export', { size: exportLabel }) }}</div>
        </section>

        <section class="rounded-box p-3 space-y-3 border border-base-content/5 shadow-sm bg-base-300/30">
          <div class="font-semibold text-base-content/80">{{ $t('msgbox.montage.style') }}</div>
          <div class="flex items-center justify-between gap-2">
            <span>{{ $t('msgbox.montage.background') }}</span>
            <div class="flex gap-2">
              <button
                v-for="color in BACKGROUND_PRESETS"
                :key="color"
                class="size-5.5 rounded-md border border-base-content/15 cursor-pointer"
                :class="{ 'outline-2 outline-offset-2 outline-primary': background === color }"
                :style="{ backgroundColor: color }"
                @click="background = color"
              ></button>
              <label
                class="relative size-5.5 rounded-md border border-base-content/15 cursor-pointer overflow-hidden"
                :class="{ 'outline-2 outline-offset-2 outline-primary': !BACKGROUND_PRESETS.includes(background) }"
                style="background: conic-gradient(red, yellow, lime, cyan, blue, magenta, red)"
              >
                <input v-model="background" type="color" class="absolute inset-0 opacity-0 cursor-pointer" />
              </label>
            </div>
          </div>

          <div v-if="mode === 'pile'" class="flex items-center justify-between gap-2">
            <span>{{ $t('msgbox.montage.random_rotation') }} <span class="text-base-content/45">± {{ maxRotation }}°</span></span>
            <SliderInput id="montage-rotation" v-model="maxRotation" :min="0" :max="30" :step="1" :slider_width="140" />
          </div>
          <div v-else class="flex items-center justify-between gap-2">
            <span>{{ $t('msgbox.montage.spacing') }} <span class="text-base-content/45">{{ toExportPx(spacing) }} px</span></span>
            <SliderInput id="montage-spacing" v-model="spacing" :min="0" :max="5" :step="0.25" :slider_width="140" />
          </div>

          <label class="flex items-center justify-between gap-2 cursor-pointer">
            <span>{{ $t('msgbox.montage.border') }}</span>
            <input v-model="borderOn" type="checkbox" class="toggle toggle-xs toggle-primary" />
          </label>
          <div class="flex items-center justify-between gap-2" :class="{ 'opacity-50': !borderOn }">
            <span>{{ $t('msgbox.montage.border_width') }} <span class="text-base-content/45">{{ toExportPx(border) }} px</span></span>
            <SliderInput id="montage-border" v-model="border" :min="0.25" :max="4" :step="0.25" :slider_width="140" :disabled="!borderOn" />
          </div>

          <label class="flex items-center justify-between gap-2 cursor-pointer">
            <span>{{ $t('msgbox.montage.shadow') }}</span>
            <input v-model="shadow" type="checkbox" class="toggle toggle-xs toggle-primary" />
          </label>
        </section>

        <section class="rounded-box p-3 space-y-2 border border-base-content/5 shadow-sm bg-base-300/30">
          <div class="flex items-center justify-between">
            <span class="font-semibold text-base-content/80">{{ $t('msgbox.montage.photos', { count: files.length }) }}</span>
            <button class="btn btn-ghost btn-xs" :disabled="files.length >= MAX_PHOTOS" @click="requestAdd">+ {{ $t('msgbox.montage.add') }}</button>
          </div>
          <div class="grid grid-cols-6 gap-1">
            <div
              v-for="file in files"
              :key="file.id"
              class="aspect-square rounded overflow-hidden cursor-grab"
              :class="{
                'opacity-35': Number(file.id) === dragSource,
                'outline-2 outline-dashed outline-primary': Number(file.id) === dropTarget,
              }"
              :data-thumb-id="file.id"
              @pointerdown="startDrag($event, Number(file.id), 'reorder')"
              @contextmenu.prevent="removePhoto(Number(file.id))"
            >
              <img
                :src="file.thumbnail"
                class="w-full h-full object-cover pointer-events-none"
                :style="{ transform: `rotate(${Number(file.rotate || 0)}deg)` }"
              />
            </div>
          </div>
          <div class="text-[11px] text-base-content/45">{{ $t('msgbox.montage.photos_hint') }}</div>
        </section>
      </div>
    </div>

    <!-- Bottom Bar -->
    <div class="h-14 shrink-0 flex items-center justify-end px-4 gap-2">
      <div v-if="errorMessage" class="flex-1 min-w-0 text-xs text-error truncate" :title="errorMessage">{{ errorMessage }}</div>
      <button
        class="px-4 py-1 rounded-box hover:bg-base-100 hover:text-base-content cursor-pointer text-sm mr-4"
        :disabled="isProcessing"
        @click="closeWindow"
      >{{ $t('msgbox.image_editor.cancel') }}</button>
      <select v-model="combinedFormatKey" class="select select-bordered select-xs">
        <option v-for="option in combinedFormatOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
      </select>
      <button
        class="btn btn-sm btn-primary px-4"
        :disabled="isProcessing || files.length < 2"
        @click="clickSave"
      >{{ $t('msgbox.image_editor.save_as_new') }}</button>
    </div>

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
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { isWin, isLinux, setTheme, getFolderPath, getFullPath, combineFileName, getSelectOptions, getThumbUrl, isValidFileName } from '@/common/utils';
import { getFileInfo, checkFileExists, renderMontage } from '@/common/api';
import { computeMontageLayout, shuffled, type MontageItem, type MontageMode } from '@/common/montageLayout';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { LogicalSize } from '@tauri-apps/api/dpi';
import { emit as tauriEmit, listen } from '@tauri-apps/api/event';
import { open as openDialog } from '@tauri-apps/plugin-dialog';

import TitleBar from '@/components/TitleBar.vue';
import TButton from '@/components/TButton.vue';
import SliderInput from '@/components/SliderInput.vue';
import ModalDialog from '@/components/ModalDialog.vue';

import { IconTile, IconJustified, IconStack, IconSortingShuffle, IconFolder } from '@/common/icons';

// export size in pixels, landscape (300 dpi for print formats)
const PAGE_SIZES = {
  a4: [3508, 2480],
  photo: [1772, 1181],
  screen: [3840, 2160],
  square: [3000, 3000],
} as const;
type PageKey = keyof typeof PAGE_SIZES;
const PRINT_PAGES: PageKey[] = ['a4', 'photo'];
const BACKGROUND_PRESETS = ['#f5f0e6', '#ffffff', '#111827', '#7c2d12'];
const MAX_PHOTOS = 50; // same limit as the main window

const router = useRouter();
const { t, locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const appWindow = getCurrentWebviewWindow();
const showDesktopTitleBar = isWin || isLinux;

const files = ref<any[]>([]); // montage order
const mode = ref<MontageMode>('grid');
const pageKey = ref<PageKey>('a4');
const landscape = ref(true);
const background = ref('#ffffff');
const spacing = ref(1);     // % of the page width
const borderOn = ref(true);
const border = ref(1.5);    // % of the page width
const maxRotation = ref(12); // pile, degrees
const shadow = ref(true);
const seed = ref(1);        // pile placement
const isProcessing = ref(false);
const errorMessage = ref('');

const titleText = computed(() => `${t('msgbox.montage.title')} - ${t('toolbar.filter.select_count', { count: files.value.length })}`);

const modeOptions = computed(() => [
  { value: 'grid' as const, icon: IconTile, label: t('msgbox.montage.grid') },
  { value: 'mosaic' as const, icon: IconJustified, label: t('msgbox.montage.mosaic') },
  { value: 'pile' as const, icon: IconStack, label: t('msgbox.montage.pile') },
]);

const pageOptions = computed(() => [
  { value: 'a4', label: 'A4' },
  { value: 'photo', label: '10 × 15 cm' },
  { value: 'screen', label: t('msgbox.montage.page_screen') },
  { value: 'square', label: t('msgbox.montage.page_square') },
]);

const pageSize = computed(() => {
  const [long, short] = PAGE_SIZES[pageKey.value];
  return landscape.value ? [long, short] : [short, long];
});

const exportLabel = computed(() =>
  `${pageSize.value[0]} × ${pageSize.value[1]} px${PRINT_PAGES.includes(pageKey.value) ? ' · 300 dpi' : ''}`);

// a % of the page width, in pixels of the exported image
function toExportPx(percent: number) {
  return Math.round(percent / 100 * pageSize.value[0]);
}

const fileById = computed(() => new Map(files.value.map(file => [Number(file.id), file])));

// upright aspect ratio: stored dimensions already follow the EXIF orientation
function photoRatio(file: any) {
  const width = Number(file.width) || 0;
  const height = Number(file.height) || 0;
  if (!width || !height) return 1;
  return Number(file.rotate || 0) % 180 !== 0 ? height / width : width / height;
}

const baseLayout = computed(() => computeMontageLayout(
  files.value.map(file => ({ fileId: Number(file.id), ratio: photoRatio(file) })),
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
watch([mode, seed, pageKey, landscape, () => files.value.map(file => file.id).join(',')], () => {
  adjusts.value = new Map();
  raised.value = [];
  activeId.value = undefined;
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

// dial geometry, in px around the photo's center
const DIAL_RADIUS = 56;
const DIAL_WIDTH = 22;
const DIAL_HANDLE = 92;

const dial = computed(() => {
  if (mode.value !== 'pile' || activeId.value === undefined) return null;
  const item = movedLayout.value.find(other => other.fileId === activeId.value);
  if (!item) return null;
  const radians = item.rotation * Math.PI / 180;
  return {
    item,
    x: (item.x + item.w / 2) * pageBox.value.width,
    y: (item.y + item.h / 2) * pageBox.value.height,
    angle: item.rotation,
    handle: { left: `${Math.cos(radians) * DIAL_HANDLE}px`, top: `${Math.sin(radians) * DIAL_HANDLE}px` },
    angleLabel: Math.round((((item.rotation % 360) + 540) % 360) - 180),
    scaleLabel: Math.round((adjusts.value.get(item.fileId)?.scale ?? 1) * 100),
  };
});

// Pointer drags: move / transform a pile photo, swap two photos of the grid or mosaic,
// reorder the photo strip. Swap and reorder find their target under the pointer.
type DragKind = 'move' | 'transform' | 'swap' | 'reorder';
const DROP_SELECTORS: Partial<Record<DragKind, string>> = { swap: '[data-frame-id]', reorder: '[data-thumb-id]' };
const pageRef = ref<HTMLElement | null>(null);
const dragSource = ref<number>(); // swap / reorder: dragged photo
const dropTarget = ref<number>(); // swap / reorder: photo under the pointer
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
}

function onDrag(event: PointerEvent) {
  if (!drag) return;
  const selector = DROP_SELECTORS[drag.kind];
  if (selector) {
    const target = document.elementFromPoint(event.clientX, event.clientY)?.closest<HTMLElement>(selector);
    const id = Number(target?.dataset.frameId ?? target?.dataset.thumbId);
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
    if (drag.kind === 'swap') {
      [next[from], next[to]] = [next[to], next[from]];
    } else {
      next.splice(to, 0, ...next.splice(from, 1));
    }
    files.value = next;
  }
  drag = null;
  dragSource.value = undefined;
  dropTarget.value = undefined;
  window.removeEventListener('pointermove', onDrag);
  window.removeEventListener('pointerup', endDrag);
}

function shuffle() {
  files.value = shuffled(files.value);
  seed.value = Math.floor(Math.random() * 0xffffffff) + 1;
}

function removePhoto(fileId: number) {
  if (files.value.length > 2) files.value = files.value.filter(file => Number(file.id) !== fileId);
}

async function loadFiles(ids: number[]) {
  const infos = await Promise.all(ids.map(id => getFileInfo(id)));
  return infos
    .filter((file: any) => file && file.file_type !== 2)
    .map((file: any) => ({ ...file, thumbnail: getThumbUrl(file.id) }));
}

// "+ Add": the main window answers with the photos selected in Lap (montage-add-files)
function requestAdd() {
  errorMessage.value = '';
  void tauriEmit('montage-add-request');
}

async function addFiles(fileIds: number[]) {
  const ids = fileIds.filter(id => !fileById.value.has(id));
  if (ids.length === 0) {
    errorMessage.value = t('msgbox.montage.add_none');
    return;
  }
  const room = MAX_PHOTOS - files.value.length;
  if (ids.length > room) errorMessage.value = t('msgbox.montage.too_many_photos', { count: MAX_PHOTOS });
  files.value = [...files.value, ...await loadFiles(ids.slice(0, room))];
  await fitWindowToPanel();
}

// grow the window until the settings panel needs no scrollbar, never beyond the screen
const panelRef = ref<HTMLElement | null>(null);
async function fitWindowToPanel() {
  await nextTick();
  const panel = panelRef.value;
  const overflow = panel ? panel.scrollHeight - panel.clientHeight : 0;
  if (overflow <= 0) return;
  const height = Math.min(window.innerHeight + overflow, window.screen.availHeight);
  if (height <= window.innerHeight) return;
  await appWindow.setSize(new LogicalSize(window.innerWidth, height));
  await appWindow.center();
}

// fit the page into the preview area
const containerRef = ref<HTMLElement | null>(null);
const containerSize = ref({ width: 0, height: 0 });
let resizeObserver: ResizeObserver | null = null;
const pageBox = computed(() => {
  const padding = 24;
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
    backgroundColor: item.border > 0 ? borderColor.value : 'transparent',
    transform: item.rotation ? `rotate(${item.rotation}deg)` : undefined,
    boxShadow: shadow.value ? shadowStyle(item.rotation) : undefined,
    zIndex: layout.value.findIndex(other => other.fileId === item.fileId),
  };
}

// cover the frame with the thumbnail, applying Lap's own rotation
function photoStyle(item: MontageItem) {
  const rotate = Number(fileById.value.get(item.fileId)?.rotate || 0);
  const innerW = item.w * pageBox.value.width - 2 * item.border * pageBox.value.width;
  const innerH = item.h * pageBox.value.height - 2 * item.border * pageBox.value.width;
  const swap = rotate % 180 !== 0;
  return {
    width: `${swap ? innerH : innerW}px`,
    height: `${swap ? innerW : innerH}px`,
    transform: `translate(-50%, -50%) rotate(${rotate}deg)`,
  };
}

// output format and quality follow the image editor settings
const outputFormatValues = ['jpg', 'png', 'webp'] as const;
const combinedFormatKey = computed({
  get: () => config.imageEditor.format !== 0 ? String(config.imageEditor.format) : `0-${config.imageEditor.quality}`,
  set: (key: string) => {
    const [format, quality] = key.split('-').map(Number);
    config.imageEditor.format = format;
    config.imageEditor.quality = quality || 0;
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

const outputFormat = computed(() => outputFormatValues[config.imageEditor.format] || outputFormatValues[0]);

// save dialog, prefilled with the first photo's folder and a free montage_N name
const showSaveDialog = ref(false);
const saveFolder = ref('');
const saveName = ref('');
const saveError = ref('');
const saveNameRef = ref<HTMLInputElement | null>(null);

async function clickSave() {
  if (isProcessing.value || files.value.length < 2) return;
  errorMessage.value = '';
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
    await renderMontage({
      width: pageSize.value[0],
      height: pageSize.value[1],
      background: background.value,
      borderColor: borderColor.value,
      shadow: shadow.value,
      items: layout.value.map(item => {
        const file = fileById.value.get(item.fileId);
        return { ...item, path: file.file_path, orientation: Number(file.e_orientation || 1), rotate: Number(file.rotate || 0) };
      }),
      destFilePath,
      outputFormat: outputFormat.value,
      quality: [90, 80, 60][config.imageEditor.quality] || 80,
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

function handleKeyDown(event: KeyboardEvent) {
  if (event.key !== 'Escape') return;
  if (showSaveDialog.value) closeSaveDialog();
  else void closeWindow();
}

watch(() => config.settings.language, (newLanguage) => {
  locale.value = newLanguage;
});
watch(() => config.settings.appearance, (newAppearance) => {
  setTheme(newAppearance, newAppearance === 0 ? config.settings.lightTheme : config.settings.darkTheme);
});

let unlistenAddFiles: (() => void) | null = null;

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown);
  resizeObserver = new ResizeObserver(([entry]) => {
    containerSize.value = { width: entry.contentRect.width, height: entry.contentRect.height };
  });
  if (containerRef.value) resizeObserver.observe(containerRef.value);

  unlistenAddFiles = await listen('montage-add-files', (event: any) => addFiles(event.payload.fileIds || []));

  const ids = String(router.currentRoute.value.query.fileIds || '').split(',').map(Number).filter(id => id > 0);
  files.value = await loadFiles(ids);
  await fitWindowToPanel();
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  endDrag();
  resizeObserver?.disconnect();
  unlistenAddFiles?.();
});

</script>
