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
      <div
        ref="containerRef"
        class="relative flex-1 min-w-0 rounded-box overflow-hidden border border-base-content/5 bg-base-300/30 shadow-sm flex items-center justify-center"
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
              :class="{ 'cursor-move': mode === 'pile', 'outline-2 outline-primary': mode === 'pile' && item.fileId === activeId }"
              :style="frameStyle(item)"
              @pointerdown="startDrag($event, item, 'move')"
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
              @pointerdown.stop="startDrag($event, dial.item, 'transform')"
            ></div>
            <div class="absolute -translate-1/2 flex flex-col items-center gap-1 text-[11px] font-medium text-white whitespace-nowrap">
              <span class="px-1.5 rounded bg-black/40">{{ $t('msgbox.montage.angle', { value: dial.angleLabel }) }}</span>
              <span class="px-1.5 rounded bg-black/40 mt-4">{{ $t('msgbox.montage.scale', { value: dial.scaleLabel }) }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Right: Settings -->
      <div class="w-72 shrink-0 flex flex-col gap-3 overflow-y-auto">
        <section class="rounded-box p-3 space-y-2 border border-base-content/5 shadow-sm bg-base-300/30">
          <span class="text-[11px] font-bold uppercase tracking-[0.22em] text-base-content/30">{{ $t('msgbox.montage.layout') }}</span>
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

        <section class="rounded-box p-3 space-y-3 border border-base-content/5 shadow-sm bg-base-300/30">
          <span class="text-[11px] font-bold uppercase tracking-[0.22em] text-base-content/30">{{ $t('msgbox.montage.page') }}</span>
          <div class="grid grid-cols-[80px_minmax(0,1fr)] gap-x-4 gap-y-3 items-center text-xs">
            <div class="font-medium tracking-wide">{{ $t('msgbox.montage.page_size') }}</div>
            <select v-model="pageKey" class="select select-bordered select-xs">
              <option v-for="option in pageOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
            </select>

            <template v-if="pageKey !== 'square'">
              <div class="font-medium tracking-wide">{{ $t('msgbox.montage.orientation') }}</div>
              <select v-model="landscape" class="select select-bordered select-xs">
                <option :value="true">{{ $t('msgbox.montage.landscape') }}</option>
                <option :value="false">{{ $t('msgbox.montage.portrait') }}</option>
              </select>
            </template>

            <div class="font-medium tracking-wide">{{ $t('msgbox.montage.background') }}</div>
            <input v-model="background" type="color" class="h-6 w-12 cursor-pointer rounded border border-base-content/10 bg-transparent" />

            <div class="col-span-2 flex items-center gap-2 font-medium tracking-wide cursor-pointer">
              <input id="montage-shadow" v-model="shadow" type="checkbox" class="checkbox checkbox-primary checkbox-xs" />
              <label for="montage-shadow" class="cursor-pointer">{{ $t('msgbox.montage.shadow') }}</label>
            </div>

            <template v-if="mode === 'pile'">
              <div class="font-medium tracking-wide">{{ $t('msgbox.montage.border') }}</div>
              <SliderInput v-model="border" :min="0" :max="4" :step="0.5" class="w-full" />
            </template>
            <template v-else>
              <div class="font-medium tracking-wide">{{ $t('msgbox.montage.spacing') }}</div>
              <SliderInput v-model="spacing" :min="0" :max="5" :step="0.5" class="w-full" />
            </template>
          </div>
          <div class="text-[10px] font-mono text-base-content/40">{{ pageSize[0] }} × {{ pageSize[1] }} px</div>
        </section>
      </div>
    </div>

    <!-- Bottom Bar -->
    <div class="h-14 shrink-0 flex items-center justify-end px-4 gap-2">
      <div v-if="errorMessage" class="flex-1 min-w-0 text-xs text-error truncate" :title="errorMessage">{{ errorMessage }}</div>
      <button
        class="px-4 py-1 rounded-box hover:bg-base-100 hover:text-base-content cursor-pointer text-sm mr-4"
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
import { computeMontageLayout, type MontageItem, type MontageMode } from '@/common/montageLayout';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { emit as tauriEmit } from '@tauri-apps/api/event';
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

const router = useRouter();
const { t, locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const appWindow = getCurrentWebviewWindow();
const showDesktopTitleBar = isWin || isLinux;

const files = ref<any[]>([]);
const mode = ref<MontageMode>('grid');
const pageKey = ref<PageKey>('a4');
const landscape = ref(true);
const background = ref('#ffffff');
const spacing = ref(1);   // % of the page width
const border = ref(1.5);  // % of the page width
const seed = ref(0);
const shadow = ref(false);
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
  { spacing: spacing.value / 100, border: border.value / 100 },
  seed.value,
));

// pile: photos adjusted by hand (page-relative offset, scale around the center, extra rotation)
// and brought to the front (last = topmost)
type Adjust = { dx: number; dy: number; scale: number; angle: number };
const NO_ADJUST: Adjust = { dx: 0, dy: 0, scale: 1, angle: 0 };
const adjusts = ref(new Map<number, Adjust>());
const raised = ref<number[]>([]);
const activeId = ref<number>(); // photo showing the dial
watch([mode, seed, pageKey, landscape, () => files.value.length], () => {
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

const pageRef = ref<HTMLElement | null>(null);
type DragKind = 'move' | 'transform';
let drag: { kind: DragKind; fileId: number; startX: number; startY: number; centerX: number; centerY: number; start: Adjust } | null = null;

function startDrag(event: PointerEvent, item: MontageItem, kind: DragKind) {
  if (mode.value !== 'pile' || event.button !== 0 || !pageRef.value) return;
  const page = pageRef.value.getBoundingClientRect();
  drag = {
    kind,
    fileId: item.fileId,
    startX: event.clientX,
    startY: event.clientY,
    centerX: page.left + (item.x + item.w / 2) * page.width,
    centerY: page.top + (item.y + item.h / 2) * page.height,
    start: { ...(adjusts.value.get(item.fileId) ?? NO_ADJUST) },
  };
  raised.value = [...raised.value.filter(id => id !== item.fileId), item.fileId];
  activeId.value = item.fileId;
  window.addEventListener('pointermove', onDrag);
  window.addEventListener('pointerup', endDrag);
}

function onDrag(event: PointerEvent) {
  if (!drag) return;
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
  drag = null;
  window.removeEventListener('pointermove', onDrag);
  window.removeEventListener('pointerup', endDrag);
}

function shuffle() {
  seed.value = Math.floor(Math.random() * 0xffffffff) + 1;
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

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown);
  resizeObserver = new ResizeObserver(([entry]) => {
    containerSize.value = { width: entry.contentRect.width, height: entry.contentRect.height };
  });
  if (containerRef.value) resizeObserver.observe(containerRef.value);

  const ids = String(router.currentRoute.value.query.fileIds || '').split(',').map(Number).filter(id => id > 0);
  const infos = await Promise.all(ids.map(id => getFileInfo(id)));
  files.value = infos
    .filter((file: any) => file && file.file_type !== 2)
    .map((file: any) => ({ ...file, thumbnail: getThumbUrl(file.id) }));
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  endDrag();
  resizeObserver?.disconnect();
});

</script>
