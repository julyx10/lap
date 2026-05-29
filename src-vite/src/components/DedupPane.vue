<template>
  <div class="w-full h-full rounded-box bg-base-200 flex flex-col overflow-hidden">
    <div class="flex items-center w-full shrink-0 px-2 mb-2">
      <div class="flex-1 pl-1">
        <span class="text-[11px] font-bold uppercase tracking-[0.22em] text-base-content/35">{{ $t('info_panel.dedup.title') }}</span>
      </div>
      <div class="mt-2 flex items-center gap-1">
        <TButton
          :icon="IconRefresh"
          :tooltip="$t('toolbar.tooltip.refresh')"
          :buttonSize="'small'"
          :disabled="isDedupLoading"
          @click.stop="triggerBackendDedup(true)"
        />
        <TButton
          :icon="IconClose"
          :tooltip="$t('msgbox.close')"
          :buttonSize="'small'"
          @click.stop="$emit('close')"
        />
      </div>
    </div>

    <div class="mb-2 px-2 flex-1 overflow-y-auto overflow-x-hidden flex flex-col">
      <div v-if="isDedupLoading" class="border-t border-base-content/5 p-4 flex-1 flex items-center justify-center">
        <div class="text-center text-base-content/40 space-y-3 max-w-[260px]">
          <span class="loading loading-spinner text-primary w-8 h-8 mx-auto"></span>
          <p class="text-xs font-medium">{{ $t('info_panel.dedup.scanning') }}</p>
        </div>
      </div>

      <div v-else-if="duplicateGroups.length === 0" class="border-t border-base-content/5 p-4 flex-1 flex items-center justify-center">
        <div class="text-center text-base-content/40 space-y-3 max-w-[260px]">
          <IconSimilar class="w-8 h-8 mx-auto text-base-content/30" />
          <p class="text-xs font-medium">{{ $t('info_panel.dedup.empty_title') }}</p>
          <p class="text-xs text-base-content/40">{{ $t('info_panel.dedup.empty_desc') }}</p>
        </div>
      </div>

      <template v-else>
        <div class="border-t border-base-content/5 px-1 py-3 space-y-3">
          <div class="flex items-center gap-2 text-base-content/70">
            <span class="font-bold uppercase text-xs tracking-wide">{{ $t('info_panel.dedup.groups_title') }}</span>
          </div>
          <div class="text-xs font-semibold text-base-content/60">
            <span>
              {{ $t('info_panel.dedup.duplicate_files_summary', { count: totalDuplicateFileCount.toLocaleString(), size: formatFileSize(totalReclaimableBytes) }) }}
            </span>
          </div>
          <div v-if="showGroupLimitHint" class="pt-0.5 text-xs font-medium leading-relaxed text-warning">
            {{ $t('info_panel.dedup.group_limit_hint', { count: DEDUP_GROUP_LIMIT }) }}
          </div>
          <div class="space-y-1.5 max-h-44 overflow-y-auto overflow-x-hidden pr-1">
            <button
              v-for="(group, idx) in duplicateGroups"
              :key="group.id"
              class="w-full flex items-center gap-2 text-left rounded-box p-2.5 border transition-colors cursor-pointer"
              :class="selectedGroupId === group.id
                ? 'border-primary/50 bg-primary/8'
                : 'border-base-content/5 bg-base-100/30 hover:border-base-content/10 hover:bg-base-100/50'"
              @click="selectedGroupId = group.id"
            >
              <div class="w-8 h-8 rounded-box overflow-hidden shrink-0">
                <img v-if="group.keepItem?.file?.thumbnail" :src="group.keepItem.file.thumbnail" class="w-full h-full object-cover" />
                <div v-else class="w-full h-full skeleton"></div>
              </div>
              <span class="text-xs font-semibold text-base-content/70 truncate">{{ $t('info_panel.dedup.group_label', { index: idx + 1 }) }}</span>
              <span class="text-[11px] text-base-content/50 shrink-0">{{ group.file_count }} {{ $t('info_panel.dedup.items') }}</span>
              <div class="ml-auto text-right shrink-0">
                <div class="text-[11px] text-base-content/55">{{ formatFileSize(group.reclaimableBytes) }}</div>
                <div v-if="group.keepItem?.file?.width && group.keepItem?.file?.height" class="text-[11px] text-base-content/40">
                  {{ group.keepItem.file.width }} x {{ group.keepItem.file.height }}
                </div>
              </div>
            </button>
          </div>
        </div>

        <div v-if="activeGroup" class="border-t border-base-content/5 px-1 py-4 space-y-3">
          <div class="flex items-center gap-2 text-base-content/70">
            <span class="font-bold uppercase text-xs tracking-wide">{{ $t('info_panel.dedup.group_label', { index: activeGroupIndex + 1 }) }}</span>
          </div>

          <div class="flex flex-wrap gap-1">
            <button class="btn btn-xs btn-ghost text-base-content/70 hover:text-base-content" @click="emit('compare-group', String(activeGroup.id), activeGroup.keepItem?.file_id || 0)">
              <IconSplitOn class="w-3.5 h-3.5" />
              {{ $t('info_panel.dedup.compare_group') }}
            </button>
            <button class="btn btn-xs btn-ghost text-base-content/70 hover:text-base-content" @click="selectGroupDuplicates(activeGroup.id, activeGroup.keepItem?.file_id || 0)">
              <component :is="isAllGroupDuplicatesSelected(activeGroup.id) ? IconCheckNone : IconCheckAll" class="w-3.5 h-3.5" />
              {{ isAllGroupDuplicatesSelected(activeGroup.id) ? $t('menu.select.none') : $t('info_panel.dedup.select_group_duplicates') }}
            </button>
            <button
              class="btn btn-xs btn-ghost"
              :class="selectedDeleteCount === 0 ? 'text-base-content/30' : 'text-error'"
              :disabled="selectedDeleteCount === 0"
              @click="trashSelectedDuplicates(activeGroup.id, selectedDeleteBytes)"
            >
              <IconTrash class="w-3.5 h-3.5" />
              {{ $t('menu.file.move_to_trash') }}{{ selectedDeleteCount > 0 ? `(${formatFileSize(selectedDeleteBytes)})` : '' }}
            </button>
          </div>
          <div class="space-y-2.5">
            <button
              v-if="activeGroup.keepItem?.file"
              :key="`keep-${activeGroup.keepItem.file_id}`"
              class="w-full rounded-box p-2.5 border text-left transition-colors cursor-pointer"
              :class="[
                selectedFileId === activeGroup.keepItem.file_id
                  ? 'border-primary/50 bg-primary/8'
                  : 'border-base-content/5 bg-base-100/30 hover:border-base-content/10 hover:bg-base-100/50'
              ]"
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
                  <div class="text-xs font-semibold text-base-content/75 truncate">{{ activeGroup.keepItem.file.name }}</div>
                  <div
                    class="text-[11px] text-base-content/50 truncate"
                    :title="formatDedupFolderPath(activeGroup.keepItem.file)"
                  >
                    {{ formatDedupFolderPath(activeGroup.keepItem.file) }}
                  </div>
                  <div v-if="activeGroup.keepItem.file?.created_at" class="text-[11px] text-base-content/45">
                    {{ formatTimestamp(activeGroup.keepItem.file.created_at, $t('format.date_time')) }}
                  </div>
                </div>
              </div>
            </button>

            <button
              v-for="item in activeGroup.duplicateItems"
              :key="item.file_id"
              class="w-full rounded-box p-2.5 border text-left transition-colors cursor-pointer"
              :class="[
                (selectedFileId === item.file_id && !isDupSelected(activeGroup.id, item.file_id))
                  ? 'border-primary/50 bg-primary/8'
                  : 'border-base-content/5 bg-base-100/30 hover:border-base-content/10 hover:bg-base-100/50',
                (isDupSelected(activeGroup.id, item.file_id) && selectedFileId === item.file_id)
                  ? 'border-error/30 bg-error/5 hover:border-error/30 hover:bg-error/10'
                  : (isDupSelected(activeGroup.id, item.file_id)
                    ? 'border-error/30 bg-error/5 hover:border-error/30 hover:bg-error/10'
                    : ''),
              ]"
              @click="handleDuplicateSelection(item.file_id)"
              @dblclick="handleDuplicateSelection(item.file_id, true)"
            >
              <div class="flex items-center gap-2" 
                @click="toggleDupSelected(activeGroup.id, item.file_id)"
                @dblclick.stop
              >
                <label class="flex items-center cursor-pointer shrink-0" @click.stop>
                  <input
                    type="checkbox"
                    class="checkbox checkbox-xs checkbox-primary"
                    :checked="isDupSelected(activeGroup.id, item.file_id)"
                    @change="toggleDupSelected(activeGroup.id, item.file_id)"
                  />
                </label>
                <div class="w-10 h-10 rounded-box overflow-hidden shrink-0">
                  <img v-if="item.file?.thumbnail" :src="item.file.thumbnail" class="w-full h-full object-cover" />
                  <div v-else class="w-full h-full skeleton"></div>
                </div>
                <div class="min-w-0 flex-1">
                  <div class="text-xs font-semibold text-base-content/75 truncate">{{ item.file?.name }}</div>
                  <div
                    class="text-[11px] text-base-content/50 truncate"
                    :title="formatDedupFolderPath(item.file)"
                  >
                    {{ formatDedupFolderPath(item.file) }}
                  </div>
                  <div v-if="item.file?.created_at" class="text-[11px] text-base-content/45">
                    {{ formatTimestamp(item.file.created_at, $t('format.date_time')) }}
                  </div>
                </div>
                <button class="btn btn-xs btn-ghost shrink-0" @click.stop="setKeep(activeGroup.id, item.file_id)">
                  {{ $t('info_panel.dedup.set_keep') }}
                </button>
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
import { IconCheckAll, IconCheckNone, IconClose, IconLock, IconSimilar, IconSplitOn, IconTrash, IconRefresh } from '@/common/icons';
import { dedupStartScan, dedupGetScanStatus, dedupGetOverview, listenDedupScanProgress, dedupListGroups, dedupSetKeep, getAlbum, getFileThumb } from '@/common/api';
import { config } from '@/common/config';

const dedupPaneGlobalState = ((globalThis as any).__lapDedupPaneState ||= {
  lastScanKey: '',
});
const DEDUP_GROUP_LIMIT = 200;
const thumbnailPlaceholder = new URL('@/assets/images/image-file.png', import.meta.url).href;

const props = defineProps({
  selectedFileId: {
    type: Number,
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
  refreshKey: {
    type: Number,
    default: 0,
  },
});

const emit = defineEmits<{
  close: [];
  'select-file': [fileId: number];
  'preview-file': [fileId: number];
  'compare-group': [groupId: string, keepFileId: number];
  'trash-selected-duplicates': [groupId: string, fileIds: number[], reclaimableBytes: number];
}>();

const selectedDupIdsByGroup = ref<Map<number, Set<number>>>(new Map());
const isDedupLoading = ref(false);
const unlistenDedupProgress = ref<null | (() => void)>(null);
const queuedScanKey = ref('');
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

const activeGroupIndex = computed(() => {
  if (!activeGroup.value) return -1;
  return duplicateGroups.value.findIndex(group => group.id === activeGroup.value.id);
});

const showGroupLimitHint = computed(() => totalGroupCount.value > DEDUP_GROUP_LIMIT);
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
  getDupSelectedSet(groupId).delete(fileId);
  await fetchGroups(groupId);
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

async function hydrateGroupThumbnails(groups: any[]) {
  const tasks: Promise<void>[] = [];
  for (const group of groups || []) {
    const items = Array.isArray(group?.items) ? group.items : [];
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
    const groups = await dedupListGroups(1, DEDUP_GROUP_LIMIT, 'size_desc', 'all');
    const normalized = Array.isArray(groups) ? groups : [];
    await hydrateAlbumRootPaths(normalized);
    await hydrateGroupThumbnails(normalized);
    rawGroups.value = normalized;
    await refreshOverview();

    const available = new Set(rawGroups.value.map((group: any) => Number(group.id)));
    for (const key of Array.from(selectedDupIdsByGroup.value.keys())) {
      if (!available.has(key)) {
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

    if (preferredGroupId && rawGroups.value.some((group: any) => group.id === preferredGroupId)) {
      selectedGroupId.value = preferredGroupId;
      return;
    }

    if (selectedGroupId.value && rawGroups.value.some((group: any) => group.id === selectedGroupId.value)) {
      return;
    }

    selectedGroupId.value = rawGroups.value.length > 0 ? Number(rawGroups.value[0].id) : null;
  } catch (error) {
    console.error('fetchGroups error:', error);
    rawGroups.value = [];
    selectedGroupId.value = null;
  }
}

function stopDedupStatusPolling() {
  if (dedupStatusPollTimer.value) {
    clearInterval(dedupStatusPollTimer.value);
    dedupStatusPollTimer.value = null;
  }
}

async function handleDedupScanSettled() {
  stopDedupStatusPolling();
  await fetchGroups();
  // Only clear the loading flag after results are ready, so the
  // template never shows "no duplicates" before the scan finishes.
  if (!queuedScanKey.value || queuedScanKey.value === dedupPaneGlobalState.lastScanKey) {
    isDedupLoading.value = false;
  }
  if (queuedScanKey.value && queuedScanKey.value !== dedupPaneGlobalState.lastScanKey) {
    queuedScanKey.value = '';
    await triggerBackendDedup(true);
  }
}

function ensureDedupStatusPolling() {
  if (dedupStatusPollTimer.value) return;

  dedupStatusPollTimer.value = setInterval(async () => {
    if (isPollingDedupStatus.value) return;

    isPollingDedupStatus.value = true;
    try {
      const status = await dedupGetScanStatus();
      totalGroupCount.value = Math.max(Number(status?.groups || 0), rawGroups.value.length);
      if (status?.state && status.state !== 'running') {
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

  if (!force && dedupPaneGlobalState.lastScanKey === props.dedupScanKey) {
    isDedupLoading.value = true;
    const status = await dedupGetScanStatus();
    totalGroupCount.value = Math.max(Number(status?.groups || 0), rawGroups.value.length);
    if (status?.state === 'running') {
      queuedScanKey.value = props.dedupScanKey;
      ensureDedupStatusPolling();
      return;
    }
    await fetchGroups();
    isDedupLoading.value = false;
    return;
  }

  isDedupLoading.value = true;

  try {
    const status = await dedupGetScanStatus();
    totalGroupCount.value = Math.max(Number(status?.groups || 0), rawGroups.value.length);
    if (status?.state === 'running') {
      queuedScanKey.value = props.dedupScanKey;
      ensureDedupStatusPolling();
      return;
    }

    dedupPaneGlobalState.lastScanKey = props.dedupScanKey;
    await dedupStartScan(props.dedupQueryParams || null);

    const latest = await dedupGetScanStatus();
    totalGroupCount.value = Math.max(Number(latest?.groups || 0), rawGroups.value.length);
    if (latest?.state === 'running') {
      ensureDedupStatusPolling();
    } else {
      await handleDedupScanSettled();
    }
  } catch (error) {
    console.error('triggerBackendDedup error:', error);
    stopDedupStatusPolling();
    isDedupLoading.value = false;
  }
}

watch(
  () => props.dedupScanKey,
  (newKey) => {
    if (!newKey) {
      stopDedupStatusPolling();
      isDedupLoading.value = false;
      rawGroups.value = [];
      selectedGroupId.value = null;
      totalGroupCount.value = 0;
      totalDuplicateFileCount.value = 0;
      totalReclaimableBytes.value = 0;
    }
  }
);

watch(
  () => props.refreshKey,
  (newKey, oldKey) => {
    if (oldKey === undefined) return;
    triggerBackendDedup();
  }
);

watch(selectedGroupId, (groupId, prevGroupId) => {
  if (!groupId || groupId === prevGroupId) return;
  const group = duplicateGroups.value.find((item: any) => item.id === groupId);
  const keepId = group?.keepItem?.file_id;
  if (keepId) {
    emit('select-file', keepId);
  }
});

onMounted(async () => {
  if (!props.dedupScanKey) return;

  isDedupLoading.value = true;
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
</script>
