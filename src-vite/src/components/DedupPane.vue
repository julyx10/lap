<template>
  <div class="w-full h-full rounded-box bg-base-200 flex flex-col overflow-hidden">
    <div class="flex items-center w-full shrink-0 px-2 mb-2">
      <div class="flex items-center gap-2 text-base-content/70 px-2 mt-2">
        <IconSimilar class="w-5 h-5" />
        <span class="uppercase text-sm tracking-wide">{{ $t('info_panel.dedup.title') }}</span>
      </div>
      <div class="ml-auto mt-2 flex items-center gap-1">
        <button class="btn btn-xs btn-ghost" :disabled="isDedupLoading" @click="triggerBackendDedup(true)">
          <IconRefresh class="w-3.5 h-3.5" />
          {{ $t('toolbar.tooltip.refresh') }}
        </button>
        <TButton
          :icon="IconClose"
          :buttonSize="'small'"
          @click.stop="$emit('close')"
        />
      </div>
    </div>

    <div class="mb-2 px-2 flex-1 overflow-y-auto overflow-x-hidden space-y-2 flex flex-col bg-base-200/50">
      <div v-if="isDedupLoading" class="flex-1 flex items-center justify-center">
        <div class="text-center text-base-content/40 space-y-3 max-w-[240px]">
          <span class="loading loading-spinner text-primary w-8 h-8 mx-auto"></span>
          <p class="text-xs font-medium">{{ $t('info_panel.dedup.scanning') }}</p>
        </div>
      </div>

      <div v-else-if="duplicateGroups.length === 0" class="flex-1 flex items-center justify-center">
        <div class="text-center text-base-content/40 space-y-3 max-w-[240px]">
          <IconFiles class="w-8 h-8 mx-auto text-base-content/30" />
          <p class="text-xs font-medium">{{ $t('info_panel.dedup.empty_title') }}</p>
          <p class="text-xs text-base-content/40">{{ $t('info_panel.dedup.empty_desc') }}</p>
        </div>
      </div>

      <template v-else>
        <div class="rounded-box p-3 space-y-3 bg-base-300/25 border border-base-content/5">
          <div class="flex items-center gap-2 text-base-content/70">
            <span class="font-bold uppercase text-xs tracking-wide">{{ $t('info_panel.dedup.groups_title') }}</span>
            <span class="ml-auto text-xs font-semibold text-base-content/65">
              {{ $t('info_panel.dedup.duplicate_files_summary', { count: duplicateFileCount.toLocaleString(), size: formatFileSize(reclaimableBytes) }) }}
            </span>
          </div>
          <div class="space-y-1 max-h-40 overflow-y-auto overflow-x-hidden pr-1">
            <button
              v-for="(group, idx) in duplicateGroups"
              :key="group.id"
              class="w-full flex items-center gap-2 text-left rounded-box p-2 border transition-colors cursor-pointer"
              :class="selectedGroupId === group.id
                ? 'border-primary/50 bg-primary/8'
                : 'border-base-content/8 bg-base-100/30 hover:border-base-content/18 hover:bg-base-100/50'"
              @click="selectedGroupId = group.id"
            >
              <div class="w-8 h-8 rounded-box overflow-hidden bg-base-content/5 border border-base-content/10 shrink-0">
                <img v-if="group.keepItem?.file?.thumbnail" :src="group.keepItem.file.thumbnail" class="w-full h-full object-cover" />
                <div v-else class="w-full h-full skeleton"></div>
              </div>
              <span class="text-xs font-semibold text-base-content/70">{{ $t('info_panel.dedup.group_label', { index: idx + 1 }) }}</span>
              <span class="text-[11px] text-base-content/50">{{ group.file_count }} {{ $t('info_panel.dedup.items') }}</span>
              <span class="ml-auto text-[11px] text-base-content/55">{{ formatFileSize(group.reclaimableBytes) }}</span>
            </button>
          </div>
        </div>

        <div v-if="activeGroup" class="rounded-box p-3 space-y-3 bg-base-300/25 border border-base-content/5">
          <div class="flex items-center gap-2 text-base-content/70">
            <span class="font-bold uppercase text-xs tracking-wide">{{ $t('info_panel.dedup.group_label', { index: activeGroupIndex + 1 }) }}</span>
          </div>

          <div class="flex flex-wrap gap-1">
            <button class="btn btn-xs btn-ghost text-base-content/60 hover:text-base-content" @click="emit('compare-group', String(activeGroup.id), activeGroup.keepItem?.file_id || 0)">
              <IconSplitOn class="w-3.5 h-3.5" />
              {{ $t('info_panel.dedup.compare_group') }}
            </button>
            <button class="btn btn-xs btn-ghost text-base-content/60 hover:text-base-content" @click="selectGroupDuplicates(activeGroup.id, activeGroup.keepItem?.file_id || 0)">
              <component :is="isAllGroupDuplicatesSelected(activeGroup.id) ? IconCheckNone : IconCheckAll" class="w-3.5 h-3.5" />
              {{ isAllGroupDuplicatesSelected(activeGroup.id) ? $t('menu.select.none') : $t('info_panel.dedup.select_group_duplicates') }}
            </button>
            <button
              class="btn btn-xs btn-ghost"
              :class="selectedDeleteCount === 0 ? 'text-base-content/35 opacity-45' : 'text-error'"
              :disabled="selectedDeleteCount === 0"
              @click="trashSelectedDuplicates(activeGroup.id, selectedDeleteBytes)"
            >
              <IconTrash class="w-3.5 h-3.5" />
              {{ isMac ? $t('menu.file.move_to_trash') : $t('menu.file.delete') }}{{ selectedDeleteCount > 0 ? `(${formatFileSize(selectedDeleteBytes)})` : '' }}
            </button>
          </div>

          <div class="space-y-2">
            <button
              v-if="activeGroup.keepItem?.file"
              :key="`keep-${activeGroup.keepItem.file_id}`"
              class="w-full rounded-box p-2 border text-left transition-colors cursor-pointer"
              :class="[
                selectedFileId === activeGroup.keepItem.file_id
                  ? 'border-primary/50 bg-primary/8'
                  : 'border-base-content/8 bg-base-100/30 hover:border-base-content/18 hover:bg-base-100/50'
              ]"
              @click="emit('select-file', activeGroup.keepItem.file_id)"
              @dblclick="emit('preview-file', activeGroup.keepItem.file_id)"
            >
              <div class="flex items-center gap-2">
                <div class="w-10 h-10 rounded-box overflow-hidden bg-base-content/5 border border-base-content/15 shrink-0">
                  <img v-if="activeGroup.keepItem.file.thumbnail" :src="activeGroup.keepItem.file.thumbnail" class="w-full h-full object-cover" />
                  <div v-else class="w-full h-full skeleton"></div>
                </div>
                <div class="min-w-0 flex-1">
                  <div class="text-[10px] uppercase tracking-widest text-primary/80 font-bold">{{ $t('info_panel.dedup.keep_label') }}</div>
                  <div class="text-xs font-semibold text-base-content/75 truncate">{{ activeGroup.keepItem.file.name }}</div>
                  <div class="text-[11px] text-base-content/50 truncate">{{ getFolderPath(activeGroup.keepItem.file.file_path) }}</div>
                </div>
              </div>
            </button>

            <button
              v-for="item in activeGroup.duplicateItems"
              :key="item.file_id"
              class="w-full rounded-box p-2 border text-left transition-colors cursor-pointer"
              :class="[
                (selectedFileId === item.file_id && !isDupSelected(activeGroup.id, item.file_id))
                  ? 'border-primary/50 bg-primary/8'
                  : 'border-base-content/8 bg-base-100/30 hover:border-base-content/18 hover:bg-base-100/50',
                (isDupSelected(activeGroup.id, item.file_id) && selectedFileId === item.file_id)
                  ? '!border-warning/50 !bg-warning/8'
                  : (isDupSelected(activeGroup.id, item.file_id)
                    ? '!border-warning/18 !bg-warning/4 hover:!border-warning/28 hover:!bg-warning/8'
                    : ''),
              ]"
              @click="onDuplicateItemClick(activeGroup.id, item.file_id)"
              @dblclick="onDuplicateItemDblClick(activeGroup.id, item.file_id)"
            >
              <div class="flex items-center gap-2">
                <div class="w-10 h-10 rounded-box overflow-hidden bg-base-content/5 border border-base-content/10 shrink-0">
                  <img v-if="item.file?.thumbnail" :src="item.file.thumbnail" class="w-full h-full object-cover" />
                  <div v-else class="w-full h-full skeleton"></div>
                </div>
                <div class="min-w-0 flex-1">
                  <div class="text-xs font-semibold text-base-content/75 truncate">{{ item.file?.name }}</div>
                  <div class="text-[11px] text-base-content/50 truncate">{{ getFolderPath(item.file?.file_path) }}</div>
                  <div class="text-[11px] text-base-content/45">
                    {{ formatFileSize(item.file?.size || 0) }}
                    <template v-if="item.file?.width && item.file?.height"> · {{ item.file.width }} x {{ item.file.height }}</template>
                  </div>
                </div>
                <label class="flex items-center cursor-pointer shrink-0" @click.stop>
                  <input
                    type="checkbox"
                    class="checkbox checkbox-xs"
                    :checked="isDupSelected(activeGroup.id, item.file_id)"
                    @change="toggleDupSelected(activeGroup.id, item.file_id)"
                  />
                </label>
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
import { computed, ref, watch, onMounted, onUnmounted } from 'vue';
import { formatFileSize, getFolderPath, isMac } from '@/common/utils';
import TButton from '@/components/TButton.vue';
import { IconCheckAll, IconCheckNone, IconClose, IconFiles, IconSimilar, IconSplitOn, IconTrash, IconRefresh } from '@/common/icons';
import { dedupStartScan, dedupGetScanStatus, listenDedupScanProgress, dedupListGroups, dedupSetKeep, getFileThumb } from '@/common/api';
import { config } from '@/common/config';

const dedupPaneGlobalState = ((globalThis as any).__lapDedupPaneState ||= {
  lastScanKey: '',
});
const thumbnailPlaceholder = new URL('@/assets/images/image-file.png', import.meta.url).href;

const props = defineProps({
  fileList: {
    type: Array as () => any[],
    default: () => [],
  },
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
  dedupScopeFileIds: {
    type: Array as () => number[] | null,
    default: null,
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

const duplicateFileCount = computed(() =>
  duplicateGroups.value.reduce((sum, group) => sum + Number(group.file_count || 0), 0)
);

const reclaimableBytes = computed(() =>
  duplicateGroups.value.reduce((sum, group) => sum + Number(group.reclaimableBytes || 0), 0)
);

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

function onDuplicateItemClick(groupId: number, fileId: number) {
  void groupId;
  emit('select-file', fileId);
}

function onDuplicateItemDblClick(groupId: number, fileId: number) {
  void groupId;
  emit('select-file', fileId);
  emit('preview-file', fileId);
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
        file.thumbnail = thumb?.thumb_data_base64
          ? `data:image/jpeg;base64,${thumb.thumb_data_base64}`
          : thumbnailPlaceholder;
      })());
    }
  }
  await Promise.all(tasks);
}

async function fetchGroups(preferredGroupId: number | null = null) {
  try {
    const groups = await dedupListGroups(1, 200, 'size_desc', 'all');
    const normalized = Array.isArray(groups) ? groups : [];
    await hydrateGroupThumbnails(normalized);
    rawGroups.value = normalized;

    const available = new Set(rawGroups.value.map((group: any) => Number(group.id)));
    for (const key of Array.from(selectedDupIdsByGroup.value.keys())) {
      if (!available.has(key)) {
        selectedDupIdsByGroup.value.delete(key);
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

async function triggerBackendDedup(force = false) {
  if (!props.dedupScanKey) {
    isDedupLoading.value = false;
    return;
  }

  if (!force && dedupPaneGlobalState.lastScanKey === props.dedupScanKey) {
    isDedupLoading.value = false;
    await fetchGroups();
    return;
  }

  isDedupLoading.value = true;

  try {
    const status = await dedupGetScanStatus();
    if (status?.state === 'running') {
      queuedScanKey.value = props.dedupScanKey;
      return;
    }

    await dedupStartScan(props.dedupQueryParams || null, props.dedupScopeFileIds || null);
    dedupPaneGlobalState.lastScanKey = props.dedupScanKey;

    const latest = await dedupGetScanStatus();
    if (latest?.state !== 'running') {
      isDedupLoading.value = false;
      await fetchGroups();
    }
  } catch (error) {
    console.error('triggerBackendDedup error:', error);
    isDedupLoading.value = false;
  }
}

watch(
  () => props.dedupScanKey,
  async (newKey, oldKey) => {
    if (!newKey) {
      isDedupLoading.value = false;
      rawGroups.value = [];
      selectedGroupId.value = null;
      return;
    }
    if (newKey !== oldKey) {
      await triggerBackendDedup();
      return;
    }
    await fetchGroups();
  },
  { immediate: true }
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
  const status = await dedupGetScanStatus();
  if (status?.state === 'running') {
    isDedupLoading.value = true;
  } else if (props.dedupScanKey) {
    await fetchGroups();
  }

  unlistenDedupProgress.value = await listenDedupScanProgress(async (event: any) => {
    const state = event?.payload?.state;
    if (state === 'running') {
      isDedupLoading.value = true;
      return;
    }

    if (state === 'finished' || state === 'idle' || state === 'error') {
      isDedupLoading.value = false;
      await fetchGroups();
      if (queuedScanKey.value && queuedScanKey.value !== dedupPaneGlobalState.lastScanKey) {
        queuedScanKey.value = '';
        await triggerBackendDedup(true);
      }
    }
  });
});

onUnmounted(() => {
  if (unlistenDedupProgress.value) {
    unlistenDedupProgress.value();
    unlistenDedupProgress.value = null;
  }
});
</script>
