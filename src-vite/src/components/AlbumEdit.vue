<template>
  <ModalDialog :title="isNewAlbum ? $t('album.edit.title_add') : $t('album.edit.title')" :width="460" position-key="album-edit" @cancel="clickCancel">
    <div role="tablist" class="sidebar-header-tabs shrink-0" :aria-label="$t('album.edit.title')">
      <button
        v-for="tab in ['general', 'exclusions']"
        :key="tab"
        type="button"
        role="tab"
        class="sidebar-header-tab"
        :class="{ 'tab-active': activeTab === tab }"
        :aria-selected="activeTab === tab"
        @click="activeTab = tab"
      >
        {{ $t(`album.edit.tab_${tab}`) }}
      </button>
    </div>
    <section v-show="activeTab === 'general'" class="space-y-2 select-none">

      <!-- General Information -->
      <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
        <div class="flex items-center gap-2 text-base-content/30">
          <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('album.edit.section_general') }}</span>
        </div>
        <div class="w-full grid grid-cols-[84px_1fr] gap-x-4 gap-y-1.5 items-center px-1 text-xs">
          <!-- Folder -->
          <div class="h-6 flex items-center text-[11px] text-base-content/30">{{ $t('album.edit.folder') }}</div>
          <div class="min-h-6 flex flex-col justify-center gap-0.5">
            <div class="flex items-center justify-between gap-x-2">
            <input v-if="selectedFolder !== ''"
              type="text"
              readonly
              :value="selectedFolder"
              class="w-full bg-transparent border-none p-0 text-[12px] text-base-content/70 focus:border-none focus:ring-0 focus:outline-none"
            />
            <IconExclamation v-if="!isNewAlbum && album?.is_accessible === false" class="size-4 shrink-0 text-warning/70" :aria-label="t('offline.original_unavailable')" />
            <button v-if="selectedFolder === ''"
              class="btn btn-primary btn-sm rounded-box"
              @click="clickSelectFolder"
            >
              <IconNewFolder class="w-4 h-4" />
              {{ $t('album.edit.select_folder') }}
            </button>
            <TButton v-if="isNewAlbum && selectedFolder !== ''"
              :icon="IconNewFolder"
              :selected="true"
              @click="clickSelectFolder"
            />
            </div>
          </div>

          <!-- Name -->
          <div class="h-6 flex items-center text-[11px] text-base-content/30">{{ $t('album.edit.name') }}</div>
          <div class="flex min-h-6 items-center">
            <input
              ref="inputNameRef"
              v-model="inputNameValue"
              type="text"
              maxlength="255"
              :disabled="selectedFolder === ''"
              class="w-full input input-xs h-6 px-1.5 text-[12px] font-medium"
            />
          </div>

          <!-- Description -->
          <div class="h-6 flex items-start pt-1 text-[11px] text-base-content/30">{{ $t('album.edit.description') }}</div>
          <div>
            <textarea
              v-if="showDescription"
              ref="descriptionRef"
              v-model="inputDescriptionValue"
              rows="2"
              maxlength="1024"
              :placeholder="$t('album.edit.description_placeholder')"
              :disabled="selectedFolder === ''"
              class="w-full textarea textarea-sm min-h-14 max-h-50 px-1.5 text-[12px] font-medium"
            ></textarea>
            <TButton
              v-else
              :icon="IconEdit"
              :buttonSize="'small'"
              :tooltip="$t('album.edit.description')"
              :disabled="selectedFolder === ''"
              @click="showDescriptionInput"
            />
          </div>

          <template v-if="!isNewAlbum">
            <div class="h-6 flex items-center text-[11px] text-base-content/30">{{ $t('album.edit.created_at') }}</div>
            <div class="h-6 flex items-center text-[12px] text-base-content/70">{{ createdAt }}</div>
            <div class="h-6 flex items-center text-[11px] text-base-content/30">{{ $t('album.edit.modified_at') }}</div>
            <div class="h-6 flex items-center text-[12px] text-base-content/70">{{ modifiedAt }}</div>
          </template>
        </div>
      </div>

      <!-- Scan Status -->
      <div v-if="selectedFolder !== ''" class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
        <div class="flex items-center gap-2 text-base-content/30">
          <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('album.edit.section_scan') }}</span>
        </div>
        <div class="w-full grid grid-cols-[84px_1fr] gap-x-4 gap-y-1.5 items-center px-1 text-xs select-none">
          <div class="h-6 flex items-center text-[11px] text-base-content/30">
            {{ isNewAlbum ? $t('album.edit.files_to_scan') : (isScanning ? $t('album.edit.scanning') : $t('album.edit.scanned_files')) }}
          </div>
          <div class="h-6 flex items-center text-[12px] text-base-content/70" :class="{ 'animate-pulse': scanDisplayCount < 0 }">
            <template v-if="isScanning">
              {{ $t('album.edit.scanning_files', { current: scanDisplayCount.toLocaleString(), total: scanTotalCount.toLocaleString(), size: formatFileSize(scanTotalSize) }) }}
            </template>
            <template v-else-if="isNewAlbum && previewError">
              <span class="text-warning">{{ $t('album.edit.preview_unavailable') }}</span>
            </template>
            <template v-else>
              {{ scanDisplayCount >= 0 ? $t('album.edit.files_count', { count: scanDisplayCount.toLocaleString(), size: formatFileSize(scanDisplaySize) }) : $t('album.edit.files_counting') }}
              <span v-if="isNewAlbum && previewLoading && previewCount !== null" class="loading loading-spinner loading-xs ml-2" :aria-label="$t('album.edit.preview_updating')"></span>
            </template>
          </div>
          <template v-if="!isNewAlbum || isScanning">
          <div v-if="indexedSummaryCount > 0" class="h-6 flex items-center text-[11px] text-base-content/30">{{ $t('album.edit.indexed_files') }}</div>
          <div v-if="indexedSummaryCount > 0" class="h-6 flex items-center text-[12px] text-base-content/70">
            {{ $t('album.edit.files_count', { count: indexedSummaryCount.toLocaleString(), size: formatFileSize(indexedSummarySize) }) }}
          </div>
          <div v-if="displaySkippedCount > 0" class="h-6 flex items-center text-[11px] text-base-content/30">{{ $t('album.edit.skipped_files') }}</div>
          <div v-if="displaySkippedCount > 0" class="h-6 flex items-center text-[12px] text-base-content/70">
            {{ $t('album.edit.files_count', { count: displaySkippedCount.toLocaleString(), size: formatFileSize(displaySkippedSize) }) }}
          </div>
          <div v-if="!isScanning && mergedFileCount > 0" class="min-h-6 flex items-center gap-0.5 text-[11px] leading-4 text-base-content/30">
            <span>{{ $t('album.edit.merged_files') }}</span>
            <InfoTooltip v-if="activeTab === 'general'" :label="$t('album.edit.merged_files')" :text="$t('album.edit.merged_files_hint')" />
          </div>
          <div v-if="!isScanning && mergedFileCount > 0" class="h-6 flex items-center text-[12px] text-base-content/70">
            {{ formatFileCount(mergedFileCount, mergedFileSize) }}
          </div>
          <div v-if="displayFailedCount > 0" class="h-6 flex items-center text-[11px] text-error/70">{{ $t('album.edit.failed_files') }}</div>
          <div v-if="displayFailedCount > 0" class="h-6 flex items-center text-[12px] text-error/70">
            {{ formatFileCount(displayFailedCount, displayFailedSize) }}
          </div>
          <div v-if="!isScanning" class="h-6 flex items-center text-[11px] text-base-content/30">{{ $t('album.edit.last_scan_time') }}</div>
          <div v-if="!isScanning" class="h-6 flex items-center text-[12px] text-base-content/70">{{ lastScanTime }}</div>
          </template>
        </div>
      </div>
    </section>

    <section v-show="activeTab === 'exclusions'" class="space-y-2 text-xs max-h-[60vh] overflow-y-auto pr-1 select-none">
      <fieldset :disabled="busy || !selectedFolder" class="space-y-2">
        <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
          <div class="flex flex-col gap-0.5">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('album.edit.file_types') }}</span>
            </div>
            <div id="album-file-types-hint" class="text-xs text-base-content/30">{{ $t('album.edit.file_types_hint') }}</div>
          </div>
          <div class="flex gap-5">
            <label v-for="type in fileTypeOptions" :key="type.value" class="flex items-center gap-2 cursor-pointer">
              <input v-model="excludedFileTypes" aria-describedby="album-file-types-hint" type="checkbox" :value="type.value" class="checkbox checkbox-primary checkbox-xs" />
              {{ $t(type.label) }}
            </label>
          </div>
          <p v-if="fileTypes === 0" class="text-error">{{ $t('album.edit.file_types_required') }}</p>
        </div>
        <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
          <div class="flex items-center justify-between gap-4 rounded-box hover:bg-base-100/10 transition-colors duration-200">
            <div class="min-w-0 flex flex-col gap-0.5">
              <div class="flex items-center gap-2 text-base-content/30">
                <label for="album-pixel-filter" class="font-bold uppercase text-[10px] tracking-widest">{{ $t('album.edit.pixel_filter') }}</label>
              </div>
              <div id="album-pixel-hint" class="text-xs text-base-content/30">{{ $t('album.edit.pixel_hint') }}</div>
            </div>
            <select id="album-pixel-filter" v-model="smallImageFilter" aria-describedby="album-pixel-hint" class="select select-bordered select-sm w-auto shrink-0">
              <option :value="0">{{ $t('album.edit.filter_off') }}</option>
              <option v-for="size in [160, 320, 640]" :key="size" :value="size">{{ size }} px</option>
            </select>
          </div>
        </div>
        <div class="rounded-box p-2 space-y-1 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('album.edit.excluded_folders') }}</span>
            </div>
          <p class="text-base-content/30">{{ $t('album.edit.excluded_hint') }}</p>
          <div
            v-if="folderOptions.length > 8"
            :class="[
              'h-8 flex items-center rounded-box transition-colors bg-base-100/40',
              isFolderSearchFocused ? 'border-2 border-primary' : 'border border-base-content/10 hover:border-base-content/30',
            ]"
          >
            <IconSearch class="ml-2 w-4 h-4 shrink-0" :class="isFolderSearchFocused ? 'text-primary/70' : 'text-base-content/30'" />
            <input
              v-model="folderSearch"
              type="text"
              :placeholder="$t('album.edit.search_folders')"
              :aria-label="$t('album.edit.search_folders')"
              class="w-full min-w-0 bg-transparent border-none focus:ring-0 px-2 text-sm placeholder-base-content/30 focus:outline-none disabled:opacity-50"
              @focus="isFolderSearchFocused = true"
              @blur="isFolderSearchFocused = false"
              @keydown.esc.stop="folderSearch = ''"
            />
            <button
              v-if="folderSearch"
              type="button"
              class="mr-1 p-1 rounded-box text-base-content/30 hover:text-base-content/70 disabled:opacity-30"
              :aria-label="$t('menu.tag.clear_search')"
              @click="folderSearch = ''"
            >
              <IconClose class="w-4 h-4" />
            </button>
          </div>
          <p v-if="foldersLoading" class="text-base-content/50">{{ $t('album.edit.loading_folders') }}</p>
          <p v-else-if="foldersError" class="text-warning">{{ $t('album.edit.folders_unavailable') }}</p>
          <p v-else-if="!folderOptions.length" class="text-base-content/50">{{ $t('album.edit.no_subfolders') }}</p>
          <div class="mt-2 max-h-40 overflow-y-auto space-y-1">
            <label v-for="folder in visibleFolders" :key="folder" class="flex items-center gap-1 py-1 cursor-pointer">
              <input v-model="excludedFolders" type="checkbox" :value="folder" class="checkbox checkbox-primary checkbox-xs shrink-0" />
              <IconFolder class="ml-1 w-4 h-4 shrink-0" />
              <span class="break-all">{{ folder }}</span>
            </label>
          </div>
        </div>
      </fieldset>
    </section>

    <div v-if="selectedFolder && activeTab === 'exclusions'" class="mt-2 rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm select-none" role="status" aria-live="polite" :aria-busy="previewLoading">
      <div class="flex items-center gap-2 text-base-content/30">
        <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('album.edit.section_scan_preview') }}</span>
      </div>
      <div class="grid grid-cols-[84px_1fr] gap-x-4 gap-y-1.5 items-center px-1 text-xs">
        <div class="min-h-6 flex items-center text-[11px] text-base-content/30">{{ $t('album.edit.files_to_scan') }}</div>
        <div class="min-h-6 flex flex-wrap items-center gap-x-2 gap-y-1 text-[12px] text-base-content/70">
          <span v-if="previewError" class="text-warning">{{ $t('album.edit.preview_unavailable') }}</span>
          <template v-else>
            <span v-if="previewCount !== null">{{ $t('album.edit.files_count', { count: previewCount.toLocaleString(), size: formatFileSize(previewSize) }) }}</span>
            <span v-if="previewLoading" class="flex items-center gap-1 text-[11px] text-base-content/30">
              <span class="loading loading-spinner loading-xs" aria-hidden="true"></span>
              {{ $t(previewCount === null ? 'album.edit.files_counting' : 'album.edit.preview_updating') }}
            </span>
          </template>
        </div>
        <div class="min-h-6 flex items-center text-[11px] text-base-content/30">{{ $t('album.edit.excluded_files') }}</div>
        <div class="min-h-6 flex flex-wrap items-center gap-x-2 gap-y-1 text-[12px] text-base-content/70">
          <span v-if="previewError" class="text-warning">{{ $t('album.edit.preview_unavailable') }}</span>
          <template v-else>
            <span v-if="excludedPreviewCount !== null">{{ $t('album.edit.files_count', { count: excludedPreviewCount.toLocaleString(), size: formatFileSize(excludedPreviewSize) }) }}</span>
            <span v-if="previewLoading" class="flex items-center gap-1 text-[11px] text-base-content/30">
              <span class="loading loading-spinner loading-xs" aria-hidden="true"></span>
              {{ $t(excludedPreviewCount === null ? 'album.edit.files_counting' : 'album.edit.preview_updating') }}
            </span>
          </template>
        </div>
      </div>
    </div>

    <!-- cancel and OK buttons -->
    <div class="mt-4 flex justify-end space-x-4">
      <button 
        class="t-button-default"
        :disabled="busy"
        @mouseup.left.stop.prevent="clickCancel"
        @click="!$event.detail && clickCancel()"
      >
        {{ $t('msgbox.cancel') }}
      </button>
      <button 
        class="t-button-primary"
        :disabled="busy || inputNameValue.trim().length === 0 || selectedFolder.length === 0 || fileTypes === 0"
        @mouseup.left.stop.prevent="clickOk"
        @click="!$event.detail && clickOk()"
      >
        {{ filtersChanged && !isNewAlbum ? $t('album.edit.save_update') : $t('msgbox.ok') }}
      </button>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">

import { ref, watch, onMounted, onUnmounted, computed, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { countFolder, getAlbum, getAllAlbums, listAlbumSubfolders, listenIndexProgress, listenIndexFinished } from '@/common/api';
import { useToast } from '@/common/toast';
import { formatFileSize, formatTimestamp, openFolderDialog, getFolderName, isWithinRootPath } from '@/common/utils';
import { useUIStore } from '@/stores/uiStore';
import { useLibraryStore } from '@/stores/libraryStore';
import { getAlbumScanState } from '@/common/scanStatus';

import ModalDialog from '@/components/ModalDialog.vue';
import TButton from '@/components/TButton.vue';
import InfoTooltip from '@/components/InfoTooltip.vue';
import { IconEdit, IconNewFolder, IconFolder, IconSearch, IconClose, IconExclamation } from '@/common/icons';

const props = defineProps({
  busy: { type: Boolean, default: false },
  albumId: {
    type: Number,
    required: true
  },
  initialFolderPath: {
    type: String, 
    default: '' 
  },
});

const emit = defineEmits(['ok', 'cancel']);
const uiStore = useUIStore();
const libStore = useLibraryStore();
const { t } = useI18n();
const toast = useToast();
const isNewAlbum = computed(() => props.albumId <= 0);
const album = ref<any>(null);
const createdAt = computed(() => formatTimestamp(Number(album.value?.created_at || 0), 'date_time'));
const modifiedAt = computed(() => formatTimestamp(Number(album.value?.modified_at || 0), 'date_time'));
const lastScanTime = computed(() => formatTimestamp(Number(album.value?.last_scan_time || 0) / 1000, 'date_time'));

// select folder
const selectedFolder = ref('');
const activeTab = ref('general');
const excludedFileTypes = ref<number[]>([]);
const smallImageFilter = ref(0);
const excludedFolders = ref<string[]>([]);
const subfolders = ref<string[]>([]);
const folderSearch = ref('');
const isFolderSearchFocused = ref(false);
const foldersLoading = ref(false);
const foldersError = ref(false);
const fileTypeOptions = [
  { value: 1, label: 'album.edit.photos' },
  { value: 4, label: 'album.edit.raw' },
  { value: 2, label: 'album.edit.videos' },
];
// Persist the existing inclusion mask; only the checkbox presentation is inverted.
const fileTypes = computed(() => 7 & ~excludedFileTypes.value.reduce((mask, type) => mask | type, 0));
const filters = computed(() => ({ fileTypes: fileTypes.value, smallImageFilter: smallImageFilter.value, excludedFolders: [...excludedFolders.value].sort() }));
const initialFilters = ref(JSON.stringify(filters.value));
const filtersChanged = computed(() => JSON.stringify(filters.value) !== initialFilters.value);
const folderOptions = computed(() => [...new Set([...subfolders.value, ...excludedFolders.value])].sort((a, b) => a.localeCompare(b)));
const visibleFolders = computed(() => folderOptions.value.filter(folder => folder.toLowerCase().includes(folderSearch.value.toLowerCase())));
let folderRequest = 0;
let countRequest = 0;
let countTimer: ReturnType<typeof setTimeout> | undefined;

const previewCount = ref<number | null>(null);
const previewSize = ref(0);
const excludedPreviewCount = ref<number | null>(null);
const excludedPreviewSize = ref(0);
const previewLoading = ref(false);
const previewError = ref(false);
// Reuse recent results within this dialog; serialize disk probes so rapid edits
// queue only the latest settings instead of starting concurrent directory walks.
const previewCache = new Map<string, { result: number[]; time: number }>();
let countRunning = false;
let disposed = false;
let historyLoaded = false;

async function getPreviewCounts(path: string, settings: typeof filters.value) {
  const key = JSON.stringify([path, settings]);
  const cached = previewCache.get(key);
  if (cached && Date.now() - cached.time < 30000) return cached.result;
  const result = await countFolder(path, settings);
  if (result && !disposed) {
    previewCache.set(key, { result, time: Date.now() });
    if (previewCache.size > 32) previewCache.delete(previewCache.keys().next().value!);
  }
  return result;
}

async function refreshCount() {
  if (countRunning || disposed || !selectedFolder.value) return;
  const request = countRequest;
  const path = selectedFolder.value;
  const settings = JSON.parse(JSON.stringify(filters.value));
  countRunning = true;
  try {
    if (!historyLoaded && !isNewAlbum.value) {
      const savedCounts = await getPreviewCounts(path, JSON.parse(initialFilters.value));
      if (disposed) return;
      if (savedCounts) {
        [, totalImageCount.value, totalImageSize.value, totalVideoCount.value, totalVideoSize.value] = savedCounts;
        historyLoaded = true;
      }
    }
    if (disposed || request !== countRequest) return;
    const result = await getPreviewCounts(path, settings);
    if (disposed) return;
    if (request !== countRequest) return;
    // Subtract the remaining candidates from the unfiltered scope. Overlapping
    // type, pixel and folder rules therefore exclude each file only once.
    const baseline = result ? await getPreviewCounts(path, { fileTypes: 7, smallImageFilter: 0, excludedFolders: [] }) : null;
    if (disposed || request !== countRequest) return;
    previewError.value = !result || !baseline;
    if (result && baseline) {
      excludedPreviewCount.value = Math.max(0, baseline[5] - result[5]);
      excludedPreviewSize.value = Math.max(0, baseline[6] - result[6]);
      previewCount.value = result[5];
      previewSize.value = result[6];
    }
  } catch {
    if (!disposed && request === countRequest) previewError.value = true;
  } finally {
    countRunning = false;
    if (!disposed && request === countRequest) previewLoading.value = false;
    else if (!disposed) {
      clearTimeout(countTimer);
      countTimer = setTimeout(refreshCount, 300);
    }
  }
}

function schedulePreview(reset = false) {
  ++countRequest;
  clearTimeout(countTimer);
  if (reset) {
    previewCount.value = null;
    excludedPreviewCount.value = null;
    excludedPreviewSize.value = 0;
    previewSize.value = 0;
  }
  previewError.value = false;
  previewLoading.value = !!selectedFolder.value;
  countTimer = setTimeout(refreshCount, 300);
}
watch([fileTypes, excludedFolders, smallImageFilter], () => schedulePreview(), { deep: true });


// input 
const inputNameRef = ref<HTMLInputElement | null>(null);
const descriptionRef = ref<HTMLTextAreaElement | null>(null);
const inputNameValue = ref('');
const inputDescriptionValue = ref('');
const showDescription = ref(isNewAlbum.value);

// total file count of the album (from disk probe)
const totalImageCount = ref(-1);
const totalImageSize = ref(-1);
const totalVideoCount = ref(0);
const totalVideoSize = ref(0);

// indexing progress
const indexedCount = ref(0);
const totalCount = ref(0);
const discoveredCount = ref(0);
const scannedSize = ref(0);
const skippedCount = ref(0);
const skippedSize = ref(0);
const failedCount = ref(0);
const failedSize = ref(0);
const scanTotalCount = ref(-1);
const scanTotalSize = ref(0);
const skippedFileCount = computed(() => Number(album.value?.skipped_count || 0));
const skippedFileSize = computed(() => Number(album.value?.skipped_size || 0));
const failedFileCount = computed(() => Number(album.value?.failed_count || 0));
const failedFileSize = computed(() => Number(album.value?.failed_size || 0));
const mergedFileCount = computed(() => Number(album.value?.merged_count || 0));
const mergedFileSize = computed(() => Number(album.value?.merged_size || 0));
const isScanning = computed(() => {
  if (isNewAlbum.value) return false;
  return getAlbumScanState({
    albumId: props.albumId,
    albumQueue: libStore.index.albumQueue as any[],
    pausedAlbumIds: libStore.index.pausedAlbumIds as any[],
    status: Number(libStore.index.status || 0),
  }) === 'scanning';
});

const indexedFileCount = computed(() => totalImageCount.value + totalVideoCount.value);
const indexedFileSize = computed(() => totalImageSize.value + totalVideoSize.value);
const scanDisplayCount = computed(() => isNewAlbum.value
  ? (previewCount.value ?? -1)
  : isScanning.value
    ? discoveredCount.value + skippedCount.value
    : Number(album.value?.total || 0) + mergedFileCount.value + skippedFileCount.value + failedFileCount.value);
const scanDisplaySize = computed(() => isNewAlbum.value
  ? previewSize.value
  : isScanning.value
    ? scannedSize.value
  : indexedFileSize.value + skippedFileSize.value);
const indexedSummaryCount = computed(() => Math.max(0, isScanning.value
  ? discoveredCount.value - failedCount.value
  : Number(album.value?.total || 0)));
const indexedSummarySize = computed(() => Math.max(0, isScanning.value
  ? scannedSize.value - skippedSize.value - failedSize.value
  : indexedFileSize.value - mergedFileSize.value - failedFileSize.value));
const displaySkippedCount = computed(() => isScanning.value ? skippedCount.value : skippedFileCount.value);
const displaySkippedSize = computed(() => isScanning.value ? skippedSize.value : skippedFileSize.value);
const displayFailedCount = computed(() => isScanning.value ? failedCount.value : failedFileCount.value);
const displayFailedSize = computed(() => isScanning.value ? failedSize.value : failedFileSize.value);

const formatFileCount = (count: number, size: number) => size > 0
  ? t('album.edit.files_count', { count: count.toLocaleString(), size: formatFileSize(size) })
  : t('album.edit.files_count_without_size', { count: count.toLocaleString() });

let unlistenIndexProgress: (() => void) | undefined;
let unlistenIndexFinished: (() => void) | undefined;

watch(() => selectedFolder.value, async (newPath) => {
  const request = ++folderRequest;
  subfolders.value = [];
  foldersError.value = false;
  folderSearch.value = '';
  if (!newPath) return;
  if (isNewAlbum.value) {
    inputNameValue.value = getFolderName(newPath);
    inputDescriptionValue.value = '';
    showDescription.value = true;
    excludedFolders.value = [];
  }
  schedulePreview(true);
  foldersLoading.value = true;
  try {
    const folders = await listAlbumSubfolders(newPath);
    if (request === folderRequest) subfolders.value = folders;
  } catch {
    if (request === folderRequest) foldersError.value = true;
  } finally {
    if (request === folderRequest) foldersLoading.value = false;
  }
});

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('AlbumEdit');
  
  // listen for index progress
  unlistenIndexProgress = await listenIndexProgress((event: any) => {
    const { album_id, current, discovered, total, current_size, skipped, skipped_size, failed, failed_size, scan_total, scan_total_size } = event.payload;
    if (Number(album_id) === Number(props.albumId)) {
      indexedCount.value = current;
      totalCount.value = total;
      discoveredCount.value = Number(discovered || 0);
      scannedSize.value = Number(current_size || 0);
      skippedCount.value = Number(skipped || 0);
      skippedSize.value = Number(skipped_size || 0);
      failedCount.value = Number(failed || 0);
      failedSize.value = Number(failed_size || 0);
      scanTotalCount.value = Number(scan_total || 0);
      scanTotalSize.value = Number(scan_total_size || 0);
    }
  });

  if (disposed) { unlistenIndexProgress?.(); return; }

  // Refresh saved scan statistics without overwriting unsaved form fields.
  unlistenIndexFinished = await listenIndexFinished(async (event: any) => {
    if (Number(event.payload.album_id) !== Number(props.albumId)) return;
    const libraryId = libStore._libraryId;
    const updated = await getAlbum(props.albumId);
    if (disposed || libraryId !== libStore._libraryId || !updated) return;
    album.value = updated;
    previewCache.clear();
    historyLoaded = false;
    schedulePreview();
  });
  if (disposed) { unlistenIndexFinished?.(); return; }

  if (isNewAlbum.value) {
    selectedFolder.value = props.initialFolderPath;
  } else {
    album.value = await getAlbum(props.albumId);
    if (disposed || !album.value) return;
    inputNameValue.value = album.value.name || '';
    inputDescriptionValue.value = album.value.description || '';
    showDescription.value = inputDescriptionValue.value.trim().length > 0;
    excludedFileTypes.value = [1, 4, 2].filter(type => (Number(album.value.file_types ?? 7) & type) === 0);
    smallImageFilter.value = Number(album.value.small_image_filter || 0);
    excludedFolders.value = [...(album.value.excluded_folders || [])];
    initialFilters.value = JSON.stringify(filters.value);
    selectedFolder.value = album.value.path || '';
  }

  if (selectedFolder.value) {
    setTimeout(() => {
      inputNameRef.value?.focus();
    }, 50); // 50ms delay
  }
});

onUnmounted(() => {
  disposed = true;
  ++folderRequest;
  ++countRequest;
  clearTimeout(countTimer);
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('AlbumEdit');
  
  if (unlistenIndexProgress) unlistenIndexProgress();
  if (unlistenIndexFinished) unlistenIndexFinished();
});

const clickSelectFolder = async () => {
  const folderPath = await openFolderDialog();
  if (folderPath) {
    selectedFolder.value = folderPath;
    // Auto focus name input after folder selected
    setTimeout(() => {
      inputNameRef.value?.focus();
    }, 100);
  }
};

async function showDescriptionInput() {
  showDescription.value = true;
  await nextTick();
  descriptionRef.value?.focus();
}

function handleKeyDown(event: KeyboardEvent) {
  if (!uiStore.isInputActive('AlbumEdit')) return;

  const { key } = event;
  const activeElement = document.activeElement;

  switch (key) {
    case 'Enter':
      // Let tab buttons and Exclusion controls handle Enter without saving.
      if ((activeElement === inputNameRef.value) || activeElement === document.body) {
        event.preventDefault();
        clickOk();
      }
      break;
    case 'Escape':
      clickCancel();
      break;
    default:
      break;
  }
}

const clickOk = async () => {
  if (props.busy || fileTypes.value === 0) return;
  if (inputNameValue.value.trim().length > 0 && selectedFolder.value.length > 0) {
    // Check if album with this path already exists
    if (isNewAlbum.value) {
      const albums = await getAllAlbums();
      const exists = albums?.some((album: any) => album.path === selectedFolder.value);
      if (exists) {
        toast.warning(t('tooltip.album_exists'));
        return;
      }
      const isNested = albums?.some((album: any) =>
        isWithinRootPath(selectedFolder.value, album.path)
        || isWithinRootPath(album.path, selectedFolder.value)
      );
      if (isNested) {
        toast.warning(t('tooltip.album_nested'));
        return;
      }
    }
    
    emit(
      'ok', 
      selectedFolder.value,
      inputNameValue.value, 
      inputDescriptionValue.value ? inputDescriptionValue.value : '',
      isNewAlbum.value,
      filters.value,
      filtersChanged.value
    );
  }
};

const clickCancel = () => {
  if (props.busy) return;
  emit('cancel');
};

</script>
