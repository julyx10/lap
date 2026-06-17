<template>
  <div class="w-full h-full rounded-box bg-base-200 flex flex-col overflow-hidden">
    <div class="flex items-center w-full shrink-0 px-2 mb-2">
      <div class="flex-1 pl-1">
        <span class="text-sm font-semibold text-base-content/70">
          {{ $t('info_panel.dedup.title') }}
        </span>
      </div>
      <div class="mt-2 flex items-center gap-1">
        <TButton
          :icon="IconClose"
          :tooltip="$t('msgbox.close')"
          :buttonSize="'small'"
          @click.stop="$emit('close')"
        />
      </div>
    </div>

    <div class="mb-2 px-2 flex-1 overflow-y-auto overflow-x-hidden flex flex-col">
      <div v-if="isDedupLoading" class="p-4 flex-1 flex items-center justify-center">
        <div class="text-center text-base-content/30 space-y-3 max-w-[260px]">
          <span class="loading loading-spinner text-primary w-8 h-8 mx-auto"></span>
          <p class="text-xs font-medium">{{ $t('info_panel.dedup.scanning') }}</p>
        </div>
      </div>

      <div v-else-if="dedupScanError" class="p-4 flex-1 flex items-center justify-center">
        <div class="text-center text-base-content/30 space-y-3 max-w-[260px]">
          <p class="text-xs font-medium">{{ $t('info_panel.dedup.error_title') }}</p>
          <p class="text-xs text-base-content/30">{{ $t('info_panel.dedup.error_desc') }}</p>
          <PanelActionButton
            class="mx-auto"
            :icon="IconRefresh"
            primary
            @click="triggerBackendDedup(true)"
          >
            {{ $t('info_panel.dedup.rescan') }}
          </PanelActionButton>
        </div>
      </div>

      <div v-else-if="duplicateGroups.length === 0" class="p-4 flex-1 flex items-center justify-center">
        <div class="text-center text-base-content/30 space-y-3 max-w-[260px]">
          <IconSimilar class="w-8 h-8 mx-auto text-base-content/30" />
          <p class="text-xs font-medium">{{ $t('info_panel.dedup.empty_title') }}</p>
          <p class="text-xs text-base-content/30">{{ $t('info_panel.dedup.empty_desc') }}</p>
        </div>
      </div>

      <template v-else>

        <div class="border-t border-base-content/5 px-1 py-3 space-y-3">
          <div class="flex items-center gap-2">
            <span class="text-[10px] uppercase tracking-widest font-bold text-base-content/30">
              {{ $t('info_panel.dedup.groups_title') }}
            </span>
            <span class="ml-auto min-w-0 truncate text-right text-[11px] font-semibold text-base-content/70">
              {{ $t('info_panel.dedup.duplicate_files_summary', {
                count: totalDuplicateFileCount.toLocaleString(),
                size: formatFileSize(totalReclaimableBytes),
              }) }}
            </span>
          </div>
          <div class="mx-2 grid grid-cols-[repeat(auto-fill,minmax(5rem,1fr))] gap-1.5">
            <button
              v-for="group in visibleDuplicateGroups"
              :key="group.id"
              class="group/thumb relative h-20 min-w-0 overflow-hidden rounded-box border bg-base-100/50 transition-colors cursor-pointer"
              :class="selectedGroupId === group.id
                ? 'border-primary/70 ring-1 ring-primary/30'
                : 'border-base-content/5 hover:border-base-content/20'"
              @click="selectedGroupId = group.id"
            >
              <img
                v-if="group.keepItem?.file?.thumbnail"
                :src="group.keepItem.file.thumbnail"
                class="h-full w-full object-cover"
                loading="lazy"
              />
              <div v-else class="h-full w-full skeleton"></div>
              <div class="absolute left-1 top-1 rounded bg-base-300/85 px-1.5 py-0.5 text-[10px] font-semibold text-base-content/70 backdrop-blur-sm">
                {{ group.file_count }}
              </div>
              <div
                class="absolute inset-x-0 bottom-0 bg-linear-to-t from-black/80 to-transparent px-1.5 pb-1 pt-4 text-left text-[10px] leading-tight text-white/90 opacity-0 transition-opacity group-hover/thumb:opacity-100"
                :class="{ 'opacity-100': selectedGroupId === group.id }"
              >
                <div>{{ formatFileSize(group.file_size) }}</div>
                <div v-if="group.keepItem?.file?.width && group.keepItem?.file?.height" class="text-white/70">
                  {{ group.keepItem.file.width }} x {{ group.keepItem.file.height }}
                </div>
              </div>
            </button>
            <div
              v-if="hiddenDuplicateGroupCount > 0"
              class="flex h-20 min-w-0 items-center justify-center rounded-box border border-dashed border-base-content/20 bg-base-100/50 text-xs font-semibold text-base-content/70"
            >
              +{{ hiddenDuplicateGroupCount }}
            </div>
          </div>
        </div>

        <div v-if="activeGroup" class="border-t border-base-content/5 px-1 py-4 space-y-3">
          <div class="flex items-center gap-2">
            <span class="text-[10px] uppercase tracking-widest font-bold text-base-content/30">
              {{ $t('info_panel.dedup.actions_title') }}
            </span>
            <span class="ml-auto min-w-0 truncate text-right text-[11px] font-semibold text-base-content/70">
              <template v-if="selectedDeleteCount > 0">
                {{ $t('toolbar.filter.select_count', { count: selectedDeleteCount.toLocaleString() }) }}
                ({{ formatFileSize(selectedDeleteBytes) }})
              </template>
              <template v-else>{{ $t('info_panel.select_hint') }}</template>
            </span>
          </div>

          <div class="flex flex-wrap gap-1">
            <PanelActionButton
              :icon="isAllGroupDuplicatesSelected(activeGroup.id) ? IconCheckNone : IconCheckAll"
              @click="selectGroupDuplicates(activeGroup.id, activeGroup.keepItem?.file_id || 0)"
            >
              {{ isAllGroupDuplicatesSelected(activeGroup.id) ? $t('menu.select.none') : $t('menu.select.all') }}
            </PanelActionButton>
            <PanelActionButton
              :icon="IconTrash"
              :disabled="selectedDeleteCount === 0"
              danger
              @click="trashSelectedDuplicates(activeGroup.id, selectedDeleteBytes)"
            >
              {{ $t('info_panel.dedup.move_selected_to_trash') }}
            </PanelActionButton>
          </div>
          <div class="space-y-2.5">
            <button
              v-if="activeGroup.keepItem?.file"
              :key="`keep-${activeGroup.keepItem.file_id}`"
              class="w-full rounded-box p-2.5 border text-left transition-colors cursor-pointer"
              :class="getDedupItemClass(activeGroup.keepItem.file_id)"
              @click="emit('select-file', activeGroup.keepItem.file_id)"
              @dblclick="emit('preview-file', activeGroup.keepItem.file_id)"
            >
              <div class="flex items-center gap-2">
                <div class="shrink-0 text-primary/80" :title="$t('info_panel.dedup.keep_label')">
                  <IconLock class="w-4 h-4" />
                </div>
                <div class="w-10 h-10 rounded-box overflow-hidden shrink-0">
                  <img v-if="activeGroup.keepItem.file.thumbnail" :src="activeGroup.keepItem.file.thumbnail" class="w-full h-full object-cover" />
                  <div v-else class="w-full h-full skeleton"></div>
                </div>
                <div class="min-w-0 flex-1">
                  <div class="text-xs font-semibold text-base-content/70 truncate">{{ activeGroup.keepItem.file.name }}</div>
                  <div
                    class="text-[11px] text-base-content/30 truncate"
                    :title="formatDedupFolderPath(activeGroup.keepItem.file)"
                  >
                    {{ formatDedupFolderPath(activeGroup.keepItem.file) }}
                  </div>
                  <div v-if="activeGroup.keepItem.file?.modified_at" class="text-[11px] text-base-content/30">
                    {{ $t('file_info.modified_at') }}: {{ formatTimestamp(activeGroup.keepItem.file.modified_at, $t('format.date_time')) }}
                  </div>
                </div>
              </div>
            </button>

            <button
              v-for="item in activeGroup.duplicateItems"
              :key="item.file_id"
              class="w-full rounded-box p-2.5 border text-left transition-colors cursor-pointer"
              :class="getDedupItemClass(
                item.file_id,
                isDupSelected(activeGroup.id, item.file_id),
              )"
              @click="handleDuplicateSelection(item.file_id)"
              @dblclick="handleDuplicateSelection(item.file_id, true)"
            >
              <div class="flex items-center gap-2" 
                @dblclick.stop
              >
                <label class="flex items-center cursor-pointer shrink-0" @click.stop>
                  <input
                    type="checkbox"
                    class="checkbox checkbox-xs"
                    :class="isDupSelected(activeGroup.id, item.file_id)
                      ? 'checkbox-error'
                      : 'hover:checkbox-error'"
                    :checked="isDupSelected(activeGroup.id, item.file_id)"
                    @change="toggleDupSelected(activeGroup.id, item.file_id)"
                  />
                </label>
                <div class="w-10 h-10 rounded-box overflow-hidden shrink-0">
                  <img v-if="item.file?.thumbnail" :src="item.file.thumbnail" class="w-full h-full object-cover" />
                  <div v-else class="w-full h-full skeleton"></div>
                </div>
                <div class="min-w-0 flex-1">
                  <div class="text-xs font-semibold text-base-content/70 truncate">{{ item.file?.name }}</div>
                  <div
                    class="text-[11px] text-base-content/30 truncate"
                    :title="formatDedupFolderPath(item.file)"
                  >
                    {{ formatDedupFolderPath(item.file) }}
                  </div>
                  <div v-if="item.file?.modified_at" class="text-[11px] text-base-content/30">
                    {{ $t('file_info.modified_at') }}: {{ formatTimestamp(item.file.modified_at, $t('format.date_time')) }}
                  </div>
                </div>
                <PanelActionButton class="shrink-0" primary @click.stop="setKeep(activeGroup.id, item.file_id)">
                  {{ $t('info_panel.dedup.set_keep') }}
                </PanelActionButton>
              </div>
            </button>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted, nextTick } from 'vue';
import { formatFileSize, getFolderName, getFolderPath, formatFolderBreadcrumb, getThumbnailDataUrl, isMac, formatTimestamp } from '@/common/utils';
import TButton from '@/components/TButton.vue';
import PanelActionButton from '@/components/PanelActionButton.vue';
import { IconCheckAll, IconCheckNone, IconClose, IconLock, IconRefresh, IconSimilar, IconTrash } from '@/common/icons';
import { dedupStartScan, dedupCancelScan, dedupGetScanStatus, dedupGetOverview, listenDedupScanProgress, dedupListGroups, dedupSetKeep, getAlbum, getFileThumb } from '@/common/api';
import { config } from '@/common/config';

const dedupPaneGlobalState = ((globalThis as any).__lapDedupPaneState ||= {
  lastScanKey: '',
});
const DEDUP_THUMBNAIL_LIMIT = 19;
const thumbnailPlaceholder = new URL('@/assets/images/image-file.png', import.meta.url).href;

const props = defineProps({
  selectedFileId: {
    type: [Number, String],
    default: -1,
  },
  dedupScanKey: {
    type: String,
    default: '',
  },
  dedupQueryParams: {
    type: Object as () => Record<string, any> | null,
    default: null,
  },
});

const emit = defineEmits<{
  close: [];
  'select-file': [fileId: number];
  'preview-file': [fileId: number];
  'trash-selected-duplicates': [groupId: string, fileIds: number[], reclaimableBytes: number];
}>();

const selectedDupIdsByGroup = ref<Map<number, Set<number>>>(new Map());
const isDedupLoading = ref(false);
const dedupScanError = ref(false);
const unlistenDedupProgress = ref<null | (() => void)>(null);
const queuedDedupScan = ref(false);
const scanGeneration = ref(0);
const isStartingDedupScan = ref(false);
const rawGroups = ref<any[]>([]);
const selectedGroupId = ref<number | null>(null);
const totalGroupCount = ref(0);
const totalDuplicateFileCount = ref(0);
const totalReclaimableBytes = ref(0);
const albumRootPaths = ref<Map<number, string>>(new Map());
const dedupStatusPollTimer = ref<ReturnType<typeof setInterval> | null>(null);
const isPollingDedupStatus = ref(false);

const duplicateGroups = computed(() =>
  rawGroups.value.map((group: any) => {
    const keepItem = (group.items || []).find((i: any) => i.is_keep === 1) || null;
    const duplicateItems = (group.items || []).filter((i: any) => i.is_keep === 0);
    return {
      ...group,
      keepItem,
      duplicateItems,
      reclaimableBytes: Math.max(0, Number(group.total_size || 0) - Number(group.file_size || 0)),
    };
  })
);

const activeGroup = computed(() => {
  if (selectedGroupId.value === null) return null;
  return duplicateGroups.value.find(group => group.id === selectedGroupId.value) || null;
});
const visibleDuplicateGroups = computed(() => duplicateGroups.value.slice(0, DEDUP_THUMBNAIL_LIMIT));
const hiddenDuplicateGroupCount = computed(() => Math.max(0, duplicateGroups.value.length - DEDUP_THUMBNAIL_LIMIT));

const selectedDeleteCount = computed(() => {
  if (!activeGroup.value) return 0;
  return activeGroup.value.duplicateItems.filter((item: any) => isDupSelected(activeGroup.value.id, item.file_id)).length;
});

const selectedDeleteBytes = computed(() => {
  if (!activeGroup.value) return 0;
  return activeGroup.value.duplicateItems.reduce((sum: number, item: any) => {
    return isDupSelected(activeGroup.value.id, item.file_id) ? sum + Number(item.file?.size || 0) : sum;
  }, 0);
});

function getDupSelectedSet(groupId: number): Set<number> {
  const existing = selectedDupIdsByGroup.value.get(groupId);
  if (existing) return existing;
  const set = new Set<number>();
  selectedDupIdsByGroup.value.set(groupId, set);
  return set;
}

function isDupSelected(groupId: number, fileId: number) {
  return getDupSelectedSet(groupId).has(fileId);
}

function getDedupItemClass(fileId: number, isDuplicateSelected = false) {
  const isActive = Number(props.selectedFileId) === Number(fileId);
  if (isDuplicateSelected) {
    return isActive
      ? 'border-error/70 bg-error/10'
      : 'border-error/30 hover:border-error/30 hover:bg-error/10';
  }
  return isActive
    ? 'border-primary/70 bg-primary/10'
    : 'border-base-content/10 hover:border-primary/30 hover:bg-primary/10';
}

function toggleDupSelected(groupId: number, fileId: number) {
  const set = getDupSelectedSet(groupId);
  if (set.has(fileId)) set.delete(fileId);
  else set.add(fileId);
}

function handleDuplicateSelection(fileId: number, preview = false) {
  emit('select-file', fileId);
  if (preview) {
    emit('preview-file', fileId);
  }
}

async function setKeep(groupId: number, fileId: number) {
  await dedupSetKeep(groupId, fileId);
  const groupIndex = rawGroups.value.findIndex((group: any) => Number(group.id) === groupId);
  if (groupIndex < 0) return;

  const group = rawGroups.value[groupIndex];
  const items = (group.items || []).map((item: any) => ({
    ...item,
    is_keep: Number(item.file_id) === fileId ? 1 : 0,
  }));
  rawGroups.value[groupIndex] = { ...group, items };

  const selectedIds = getDupSelectedSet(groupId);
  selectedIds.delete(fileId);
  emit('select-file', fileId);
}

function selectGroupDuplicates(groupId: number, keepFileId: number) {
  const group = duplicateGroups.value.find(g => g.id === groupId);
  if (!group) return;

  const set = getDupSelectedSet(groupId);
  const duplicateIds = group.duplicateItems.map((item: any) => item.file_id);
  const allSelected = duplicateIds.length > 0 && duplicateIds.every((id: number) => set.has(id));

  if (allSelected) {
    set.clear();
    return;
  }

  set.clear();
  for (const id of duplicateIds) {
    if (id !== keepFileId) set.add(id);
  }
}

function isAllGroupDuplicatesSelected(groupId: number) {
  const group = duplicateGroups.value.find(g => g.id === groupId);
  if (!group || group.duplicateItems.length === 0) return false;
  const set = getDupSelectedSet(groupId);
  return group.duplicateItems.every((item: any) => set.has(item.file_id));
}

function trashSelectedDuplicates(groupId: number, reclaimableBytes: number) {
  const ids = Array.from(getDupSelectedSet(groupId).values());
  if (ids.length === 0) return;
  emit('trash-selected-duplicates', String(groupId), ids, reclaimableBytes);
}

function applyDeletedFiles(groupId: number, deletedFileIds: number[]) {
  const groupIndex = rawGroups.value.findIndex((group: any) => Number(group.id) === groupId);
  if (groupIndex < 0 || deletedFileIds.length === 0) return;

  const group = rawGroups.value[groupIndex];
  const oldItems = Array.isArray(group.items) ? group.items : [];
  const deletedIds = new Set(deletedFileIds);
  const remainingItems = oldItems.filter((item: any) => !deletedIds.has(Number(item.file_id)));
  if (remainingItems.length === oldItems.length) return;

  const fileSize = Number(group.file_size || 0);
  const oldFileCount = oldItems.length;
  const newFileCount = remainingItems.length > 1 ? remainingItems.length : 0;
  const oldDuplicateCount = Math.max(0, oldFileCount - 1);
  const newDuplicateCount = Math.max(0, newFileCount - 1);
  const oldReclaimableBytes = Math.max(0, (oldFileCount - 1) * fileSize);
  const newReclaimableBytes = Math.max(0, (newFileCount - 1) * fileSize);

  totalDuplicateFileCount.value = Math.max(
    0,
    totalDuplicateFileCount.value + newDuplicateCount - oldDuplicateCount
  );
  totalReclaimableBytes.value = Math.max(
    0,
    totalReclaimableBytes.value + newReclaimableBytes - oldReclaimableBytes
  );

  const selectedIds = selectedDupIdsByGroup.value.get(groupId);
  for (const fileId of deletedIds) {
    selectedIds?.delete(fileId);
  }

  if (remainingItems.length <= 1) {
    rawGroups.value.splice(groupIndex, 1);
    selectedDupIdsByGroup.value.delete(groupId);
    totalGroupCount.value = Math.max(0, totalGroupCount.value - 1);

    if (selectedGroupId.value === groupId) {
      const nextGroup = rawGroups.value[groupIndex] || rawGroups.value[groupIndex - 1];
      selectedGroupId.value = nextGroup ? Number(nextGroup.id) : null;
    }
    return;
  }

  rawGroups.value[groupIndex] = {
    ...group,
    items: remainingItems,
    file_count: remainingItems.length,
    total_size: remainingItems.length * fileSize,
  };
}

function formatDedupFolderPath(file: any): string {
  const folderPath = getFolderPath(file?.file_path);
  if (!folderPath) return '';

  const albumId = Number(file?.album_id || 0);
  const albumRoot = albumId ? albumRootPaths.value.get(albumId) || '' : '';
  const albumLabel = file?.album_name || (albumRoot ? getFolderName(albumRoot) : '');
  return formatFolderBreadcrumb(folderPath, albumRoot, albumLabel);
}

async function hydrateAlbumRootPaths(groups: any[]) {
  const albumIds = new Set<number>();
  for (const group of groups || []) {
    for (const item of Array.isArray(group?.items) ? group.items : []) {
      const albumId = Number(item?.file?.album_id || 0);
      if (albumId > 0 && !albumRootPaths.value.has(albumId)) {
        albumIds.add(albumId);
      }
    }
  }

  if (albumIds.size === 0) return;

  const results = await Promise.all(
    Array.from(albumIds).map(async (albumId) => ({
      albumId,
      album: await getAlbum(albumId),
    }))
  );

  for (const { albumId, album } of results) {
    if (album?.path) {
      albumRootPaths.value.set(albumId, album.path);
    }
  }
}

async function hydrateGroupThumbnails(groups: any[], activeGroupId: number | null) {
  const tasks: Promise<void>[] = [];
  const visibleGroupIds = new Set(
    (groups || []).slice(0, DEDUP_THUMBNAIL_LIMIT).map((group: any) => Number(group.id))
  );

  for (const group of groups || []) {
    const groupId = Number(group?.id);
    const allItems = Array.isArray(group?.items) ? group.items : [];
    const items = groupId === activeGroupId
      ? allItems
      : visibleGroupIds.has(groupId)
        ? [allItems.find((item: any) => item.is_keep === 1) || allItems[0]].filter(Boolean)
        : [];

    for (const item of items) {
      const file = item?.file;
      if (!file) continue;
      if (file.thumbnail) continue;
      if (!file.file_path) {
        file.thumbnail = thumbnailPlaceholder;
        continue;
      }
      tasks.push((async () => {
        const thumb = await getFileThumb(
          file.id,
          file.file_path,
          file.file_type || 1,
          file.e_orientation || 0,
          config.settings.thumbnailSize,
          false
        );
        file.thumbnail = getThumbnailDataUrl(thumb, thumbnailPlaceholder, false, config.settings.thumbnailSize, file.file_path);
      })());
    }
  }
  await Promise.all(tasks);
}

async function refreshOverview() {
  try {
    const overview = await dedupGetOverview();
    if (!overview) return;
    totalGroupCount.value = Number(overview.total_groups || 0);
    totalDuplicateFileCount.value = Number(overview.total_files || 0);
    totalReclaimableBytes.value = Number(overview.total_reclaimable_bytes || 0);
  } catch (error) {
    console.error('refreshOverview error:', error);
  }
}

async function fetchGroups(preferredGroupId: number | null = null) {
  try {
    const groups = await dedupListGroups(1, 0, 'count_desc', 'all');
    const normalized = Array.isArray(groups) ? groups : [];
    const availableGroupIds = new Set(normalized.map((group: any) => Number(group.id)));
    const nextSelectedGroupId =
      preferredGroupId && availableGroupIds.has(preferredGroupId)
        ? preferredGroupId
        : selectedGroupId.value && availableGroupIds.has(selectedGroupId.value)
          ? selectedGroupId.value
          : normalized.length > 0
            ? Number(normalized[0].id)
            : null;

    await hydrateAlbumRootPaths(normalized);
    await hydrateGroupThumbnails(normalized, nextSelectedGroupId);
    rawGroups.value = normalized;
    await refreshOverview();

    for (const key of Array.from(selectedDupIdsByGroup.value.keys())) {
      if (!availableGroupIds.has(key)) {
        selectedDupIdsByGroup.value.delete(key);
      }
    }

    // Default-select all duplicate (non-keep) items for newly loaded groups
    for (const group of rawGroups.value) {
      const groupId = Number(group.id);
      if (!selectedDupIdsByGroup.value.has(groupId)) {
        const set = new Set<number>();
        for (const item of (group.items || [])) {
          if (item.is_keep !== 1) {
            set.add(Number(item.file_id));
          }
        }
        if (set.size > 0) {
          selectedDupIdsByGroup.value.set(groupId, set);
        }
      }
    }

    selectedGroupId.value = nextSelectedGroupId;
    dedupScanError.value = false;
  } catch (error) {
    console.error('fetchGroups error:', error);
    showDedupScanError();
  }
}

function stopDedupStatusPolling() {
  if (dedupStatusPollTimer.value) {
    clearInterval(dedupStatusPollTimer.value);
    dedupStatusPollTimer.value = null;
  }
}

function showDedupScanError() {
  stopDedupStatusPolling();
  dedupPaneGlobalState.lastScanKey = '';
  rawGroups.value = [];
  selectedGroupId.value = null;
  totalGroupCount.value = 0;
  totalDuplicateFileCount.value = 0;
  totalReclaimableBytes.value = 0;
  dedupScanError.value = true;
  isDedupLoading.value = false;
}

async function handleDedupScanSettled(allowWhileStarting = false) {
  if (isStartingDedupScan.value && !allowWhileStarting) return;

  const gen = scanGeneration.value;
  const status = await dedupGetScanStatus();
  if (!status) {
    ensureDedupStatusPolling();
    return;
  }
  if (status?.state === 'running' || status?.isScanning) {
    ensureDedupStatusPolling();
    return;
  }
  if (gen !== scanGeneration.value) return;

  stopDedupStatusPolling();
  if (queuedDedupScan.value) {
    queuedDedupScan.value = false;
    await triggerBackendDedup(true);
    return;
  }
  if (status.state === 'error') {
    showDedupScanError();
    return;
  }
  await fetchGroups();
  if (gen !== scanGeneration.value) return;
  // Only clear the loading flag after results are ready, so the
  // template never shows "no duplicates" before the scan finishes.
  isDedupLoading.value = false;
}

function ensureDedupStatusPolling() {
  if (dedupStatusPollTimer.value) return;

  dedupStatusPollTimer.value = setInterval(async () => {
    if (isPollingDedupStatus.value) return;

    isPollingDedupStatus.value = true;
    try {
      const status = await dedupGetScanStatus();
      totalGroupCount.value = Math.max(Number(status?.groups || 0), rawGroups.value.length);
      if (status?.state && status.state !== 'running' && !status?.isScanning) {
        await handleDedupScanSettled();
      }
    } catch (error) {
      console.error('ensureDedupStatusPolling error:', error);
    } finally {
      isPollingDedupStatus.value = false;
    }
  }, 1000);
}

async function triggerBackendDedup(force = false) {
  if (!props.dedupScanKey) {
    stopDedupStatusPolling();
    isDedupLoading.value = false;
    return;
  }

  scanGeneration.value++;
  isStartingDedupScan.value = true;
  isDedupLoading.value = true;
  dedupScanError.value = false;

  try {
    const status = await dedupGetScanStatus();
    totalGroupCount.value = Math.max(Number(status?.groups || 0), rawGroups.value.length);

    if (status?.state === 'running' || status?.isScanning) {
      queuedDedupScan.value = true;
      await dedupCancelScan();
      ensureDedupStatusPolling();
      return;
    } else if (!force && dedupPaneGlobalState.lastScanKey === props.dedupScanKey) {
      await fetchGroups();
      isDedupLoading.value = false;
      return;
    }

    await dedupStartScan(props.dedupQueryParams || null);
    dedupPaneGlobalState.lastScanKey = props.dedupScanKey;

    const latest = await dedupGetScanStatus();
    totalGroupCount.value = Math.max(Number(latest?.groups || 0), rawGroups.value.length);
    if (latest?.state === 'running') {
      ensureDedupStatusPolling();
    } else {
      await handleDedupScanSettled(true);
    }
  } catch (error) {
    if (String(error).includes('already running')) {
      queuedDedupScan.value = true;
      await dedupCancelScan();
      ensureDedupStatusPolling();
      return;
    }
    console.error('triggerBackendDedup error:', error);
    showDedupScanError();
  } finally {
    isStartingDedupScan.value = false;
  }
}

watch(
  () => props.dedupScanKey,
  (newKey) => {
    if (!newKey) {
      scanGeneration.value++;
      stopDedupStatusPolling();
      isDedupLoading.value = true;
      rawGroups.value = [];
      selectedGroupId.value = null;
      queuedDedupScan.value = false;
      dedupScanError.value = false;
      totalGroupCount.value = 0;
      totalDuplicateFileCount.value = 0;
      totalReclaimableBytes.value = 0;
      return;
    }
    triggerBackendDedup();
  }
);

watch(selectedGroupId, async (groupId, prevGroupId) => {
  if (!groupId || groupId === prevGroupId) return;
  await hydrateGroupThumbnails(rawGroups.value, groupId);
  const group = duplicateGroups.value.find((item: any) => item.id === groupId);
  const keepId = group?.keepItem?.file_id;
  if (keepId) {
    emit('select-file', keepId);
  }
});

onMounted(async () => {
  isDedupLoading.value = true;
  if (!props.dedupScanKey) return;

  await nextTick();

  unlistenDedupProgress.value = await listenDedupScanProgress(async (event: any) => {
    const state = event?.payload?.state;
    totalGroupCount.value = Math.max(Number(event?.payload?.groups || 0), totalGroupCount.value);
    if (state === 'running') {
      ensureDedupStatusPolling();
      return;
    }
    if (state === 'finished' || state === 'idle' || state === 'error') {
      await handleDedupScanSettled();
    }
  });

  triggerBackendDedup();
});

onUnmounted(() => {
  stopDedupStatusPolling();
  if (unlistenDedupProgress.value) {
    unlistenDedupProgress.value();
    unlistenDedupProgress.value = null;
  }
});

defineExpose({
  applyDeletedFiles,
});
</script>
