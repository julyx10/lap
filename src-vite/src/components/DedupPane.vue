<template>
  <div class="w-full h-full rounded-box bg-base-200 flex flex-col overflow-hidden">
    <div class="flex items-center w-full shrink-0 px-2 mb-2">
      <div class="flex items-center gap-2 text-base-content/70 px-2 mt-2">
        <IconSimilar class="w-5 h-5" />
        <span class="uppercase text-sm tracking-wide">{{ $t('info_panel.dedup.title') }}</span>
      </div>
      <div class="ml-auto mt-2 flex items-center gap-1">
        <TButton
          :icon="IconClose"
          :buttonSize="'small'"
          @click.stop="$emit('close')"
        />
      </div>
    </div>

    <div class="mb-2 px-2 flex-1 overflow-y-auto overflow-x-hidden space-y-3 flex flex-col bg-base-200/50">
      <div v-if="duplicateGroups.length === 0" class="flex-1 flex items-center justify-center">
        <div class="text-center text-base-content/40 space-y-3 max-w-[240px]">
          <IconFiles class="w-8 h-8 mx-auto text-base-content/30" />
          <p class="text-xs font-medium">{{ $t('info_panel.dedup.empty_title') }}</p>
          <p class="text-xs text-base-content/40">{{ $t('info_panel.dedup.empty_desc') }}</p>
        </div>
      </div>

      <template v-else>
        <div class="rounded-box p-3 space-y-3 bg-base-300/30 border border-base-content/5 shadow-sm">
          <div class="flex items-center gap-2 text-base-content/70">
            <span class="font-bold uppercase text-xs tracking-wide">{{ $t('info_panel.dedup.groups_title') }}</span>
            <span class="ml-auto text-xs font-semibold text-base-content/65">
              {{ $t('info_panel.dedup.duplicate_files_summary', { count: duplicateFileCount.toLocaleString(), size: formatFileSize(reclaimableBytes) }) }}
            </span>
          </div>
          <div class="space-y-1 max-h-40 overflow-y-auto overflow-x-hidden pr-1">
            <button
              v-for="(group, idx) in duplicateGroups"
              :key="group.key"
              class="w-full flex items-center gap-2 text-left rounded-box p-2 border transition-colors cursor-pointer"
              :class="selectedGroupKey === group.key
                ? 'border-primary bg-primary/10'
                : 'border-base-content/10 bg-base-100/40 hover:border-base-content/20 hover:bg-base-100/60'"
              @click="selectedGroupKey = group.key"
            >
              <div class="w-8 h-8 rounded-box overflow-hidden bg-base-content/5 border border-base-content/10 shrink-0">
                <img v-if="group.keepFile?.thumbnail" :src="group.keepFile.thumbnail" class="w-full h-full object-cover" />
                <div v-else class="w-full h-full skeleton"></div>
              </div>
              <span class="text-xs font-semibold text-base-content/70">{{ $t('info_panel.dedup.group_label', { index: idx + 1 }) }}</span>
              <span class="text-[11px] text-base-content/50">{{ group.files.length }} {{ $t('info_panel.dedup.items') }}</span>
              <span class="ml-auto text-[11px] text-base-content/55">{{ formatFileSize(group.reclaimableBytes) }}</span>
            </button>
          </div>
        </div>

        <div v-if="activeGroup" class="rounded-box p-3 space-y-3 bg-base-300/30 border border-base-content/5 shadow-sm">
          <div class="flex items-center gap-2 text-base-content/70">
            <span class="font-bold uppercase text-xs tracking-wide">{{ $t('info_panel.dedup.group_label', { index: activeGroupIndex + 1 }) }}</span>
          </div>

          <div class="flex flex-wrap gap-2">
            <button class="btn btn-xs btn-ghost" @click="emit('compare-group', activeGroup.key, activeGroup.keepFile?.id || 0)">
              <IconSplitOn class="w-3.5 h-3.5" />
              {{ $t('info_panel.dedup.compare_group') }}
            </button>
            <button class="btn btn-xs btn-ghost" @click="selectGroupDuplicates(activeGroup.key, activeGroup.keepFile?.id || 0)">
              <component :is="isAllGroupDuplicatesSelected(activeGroup.key) ? IconCheckNone : IconCheckAll" class="w-3.5 h-3.5" />
              {{ isAllGroupDuplicatesSelected(activeGroup.key) ? $t('menu.select.none') : $t('info_panel.dedup.select_group_duplicates') }}
            </button>
            <button
              class="btn btn-xs btn-ghost text-error"
              :disabled="selectedDeleteCount === 0"
              @click="trashSelectedDuplicates(activeGroup.key, activeGroup.reclaimableBytes)"
            >
              <IconTrash class="w-3.5 h-3.5" />
              {{ isMac ? $t('menu.file.move_to_trash') : $t('menu.file.delete') }}{{ selectedDeleteCount > 0 ? `(${formatFileSize(selectedDeleteBytes)})` : '' }}
            </button>
          </div>

          <div class="space-y-2">
            <button
              v-if="activeGroup.keepFile"
              :key="`keep-${activeGroup.keepFile.id}`"
              class="w-full rounded-box p-2 border text-left border-success/30 bg-success/10 cursor-pointer"
              @click="emit('select-file', activeGroup.keepFile.id)"
              @dblclick="emit('preview-file', activeGroup.keepFile.id)"
            >
              <div class="flex items-center gap-2">
                <div class="w-10 h-10 rounded-box overflow-hidden bg-base-content/5 border border-success/30 shrink-0">
                  <img v-if="activeGroup.keepFile.thumbnail" :src="activeGroup.keepFile.thumbnail" class="w-full h-full object-cover" />
                  <div v-else class="w-full h-full skeleton"></div>
                </div>
                <div class="min-w-0 flex-1">
                  <div class="text-[11px] uppercase tracking-widest text-success/80 font-bold">{{ $t('info_panel.dedup.keep_label') }}</div>
                  <div class="text-xs font-semibold text-base-content/75 truncate">{{ activeGroup.keepFile.name }}</div>
                  <div class="text-[11px] text-base-content/50 truncate">{{ getFolderPath(activeGroup.keepFile.file_path) }}</div>
                </div>
              </div>
            </button>

            <button
              v-for="file in activeGroup.duplicateFiles"
              :key="file.id"
              class="w-full rounded-box p-2 border text-left transition-colors cursor-pointer"
              :class="[
                selectedFileId === file.id
                  ? 'border-primary bg-primary/10'
                  : 'border-base-content/10 bg-base-100/40 hover:border-base-content/20 hover:bg-base-100/60',
                isDupSelected(activeGroup.key, file.id) ? '!border-warning/60 !bg-warning/10' : '',
              ]"
              @click="emit('select-file', file.id)"
              @dblclick="emit('preview-file', file.id)"
            >
              <div class="flex items-center gap-2">
                <div class="w-10 h-10 rounded-box overflow-hidden bg-base-content/5 border border-base-content/10 shrink-0">
                  <img v-if="file.thumbnail" :src="file.thumbnail" class="w-full h-full object-cover" />
                  <div v-else class="w-full h-full skeleton"></div>
                </div>
                <div class="min-w-0 flex-1">
                  <div class="text-xs font-semibold text-base-content/75 truncate">{{ file.name }}</div>
                  <div class="text-[11px] text-base-content/50 truncate">{{ getFolderPath(file.file_path) }}</div>
                  <div class="text-[11px] text-base-content/45">
                    {{ formatFileSize(file.size || 0) }}
                    <template v-if="file.width && file.height"> · {{ file.width }} x {{ file.height }}</template>
                  </div>
                </div>
                <label class="flex items-center cursor-pointer shrink-0" @click.stop>
                  <input
                    type="checkbox"
                    class="checkbox checkbox-xs"
                    :checked="isDupSelected(activeGroup.key, file.id)"
                    @change="toggleDupSelected(activeGroup.key, file.id)"
                  />
                </label>
                <button class="btn btn-xs btn-ghost shrink-0" @click.stop="setKeep(activeGroup.key, file.id)">
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
import { computed, ref, watch } from 'vue';
import { formatFileSize, getFolderPath, isMac } from '@/common/utils';
import TButton from '@/components/TButton.vue';
import { IconCheckAll, IconCheckNone, IconClose, IconFiles, IconSimilar, IconSplitOn, IconTrash } from '@/common/icons';

const props = defineProps({
  fileList: {
    type: Array as () => any[],
    default: () => [],
  },
  selectedFileId: {
    type: Number,
    default: -1,
  },
});

const emit = defineEmits<{
  close: [];
  'select-file': [fileId: number];
  'preview-file': [fileId: number];
  'compare-group': [groupKey: string, keepFileId: number];
  'trash-selected-duplicates': [groupKey: string, fileIds: number[], reclaimableBytes: number];
}>();

const keepFileByGroup = ref<Map<string, number>>(new Map());
const selectedDupIdsByGroup = ref<Map<string, Set<number>>>(new Map());

function buildPotentialDupKey(file: any): string | null {
  if (!file || !file.id || !file.size) return null;

  const fileType = Number(file.file_type || 0);
  if (fileType === 1) {
    const width = Number(file.width || 0);
    const height = Number(file.height || 0);
    return `img:${file.size}:${width}x${height}`;
  }
  if (fileType === 2) {
    const duration = Math.round(Number(file.duration || 0));
    return `vid:${file.size}:${duration}`;
  }

  const ext = String(file.name || '').split('.').pop()?.toLowerCase() || 'unknown';
  return `file:${file.size}:${ext}`;
}

const duplicateGroups = computed(() => {
  const groups = new Map<string, any[]>();
  for (const file of props.fileList || []) {
    const key = buildPotentialDupKey(file);
    if (!key) continue;
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key)!.push(file);
  }

  return Array.from(groups.entries())
    .filter(([, files]) => files.length > 1)
    .map(([key, files]) => {
      const sortedFiles = [...files].sort((a, b) => {
        const aName = String(a?.name || '').toLowerCase();
        const bName = String(b?.name || '').toLowerCase();
        return aName.localeCompare(bName);
      });
      const preferredKeepId = keepFileByGroup.value.get(key);
      const keepFile = sortedFiles.find(file => file.id === preferredKeepId) || sortedFiles[0];
      const duplicateFiles = sortedFiles.filter(file => file.id !== keepFile.id);
      const fileSize = Number(sortedFiles[0]?.size || 0);
      return {
        key,
        files: sortedFiles,
        keepFile,
        duplicateFiles,
        reclaimableBytes: Math.max(0, sortedFiles.length - 1) * fileSize,
      };
    })
    .sort((a, b) => {
      if (b.reclaimableBytes !== a.reclaimableBytes) return b.reclaimableBytes - a.reclaimableBytes;
      return b.files.length - a.files.length;
    });
});

const selectedGroupKey = ref('');

watch(
  duplicateGroups,
  (groups) => {
    if (groups.length === 0) {
      selectedGroupKey.value = '';
      return;
    }
    const exists = groups.some(group => group.key === selectedGroupKey.value);
    if (!exists) selectedGroupKey.value = groups[0].key;
  },
  { immediate: true }
);

const activeGroup = computed(() =>
  duplicateGroups.value.find(group => group.key === selectedGroupKey.value) || duplicateGroups.value[0]
);

const activeGroupIndex = computed(() => {
  if (!activeGroup.value) return -1;
  return duplicateGroups.value.findIndex(group => group.key === activeGroup.value.key);
});

const duplicateFileCount = computed(() =>
  duplicateGroups.value.reduce((sum, group) => sum + group.files.length, 0)
);

const reclaimableBytes = computed(() =>
  duplicateGroups.value.reduce((sum, group) => sum + group.reclaimableBytes, 0)
);

const selectedDeleteCount = computed(() => {
  if (!activeGroup.value) return 0;
  return activeGroup.value.duplicateFiles.filter((file: any) => isDupSelected(activeGroup.value.key, file.id)).length;
});

const selectedDeleteBytes = computed(() => {
  if (!activeGroup.value) return 0;
  return activeGroup.value.duplicateFiles.reduce((sum: number, file: any) => {
    return isDupSelected(activeGroup.value.key, file.id) ? sum + Number(file.size || 0) : sum;
  }, 0);
});

function getDupSelectedSet(groupKey: string): Set<number> {
  const existing = selectedDupIdsByGroup.value.get(groupKey);
  if (existing) return existing;
  const set = new Set<number>();
  selectedDupIdsByGroup.value.set(groupKey, set);
  return set;
}

function isDupSelected(groupKey: string, fileId: number) {
  return getDupSelectedSet(groupKey).has(fileId);
}

function toggleDupSelected(groupKey: string, fileId: number) {
  const set = getDupSelectedSet(groupKey);
  if (set.has(fileId)) set.delete(fileId);
  else set.add(fileId);
}

function setKeep(groupKey: string, fileId: number) {
  keepFileByGroup.value.set(groupKey, fileId);
  getDupSelectedSet(groupKey).delete(fileId);
}

function selectGroupDuplicates(groupKey: string, keepFileId: number) {
  const group = duplicateGroups.value.find(g => g.key === groupKey);
  if (!group) return;
  const set = getDupSelectedSet(groupKey);
  const allSelected = group.duplicateFiles.length > 0 && group.duplicateFiles.every(file => set.has(file.id));
  if (allSelected) {
    set.clear();
    return;
  }
  set.clear();
  for (const file of group.duplicateFiles) {
    if (file.id !== keepFileId) set.add(file.id);
  }
}

function isAllGroupDuplicatesSelected(groupKey: string) {
  const group = duplicateGroups.value.find(g => g.key === groupKey);
  if (!group || group.duplicateFiles.length === 0) return false;
  const set = getDupSelectedSet(groupKey);
  return group.duplicateFiles.every(file => set.has(file.id));
}

function trashSelectedDuplicates(groupKey: string, reclaimableBytes: number) {
  const ids = Array.from(getDupSelectedSet(groupKey).values());
  if (ids.length === 0) return;
  emit('trash-selected-duplicates', groupKey, ids, reclaimableBytes);
}

watch(
  duplicateGroups,
  (groups) => {
    const available = new Set(groups.map(group => group.key));
    for (const key of Array.from(keepFileByGroup.value.keys())) {
      if (!available.has(key)) keepFileByGroup.value.delete(key);
    }
    for (const key of Array.from(selectedDupIdsByGroup.value.keys())) {
      if (!available.has(key)) {
        selectedDupIdsByGroup.value.delete(key);
        continue;
      }
      const validIds = new Set((groups.find(g => g.key === key)?.duplicateFiles || []).map((f: any) => f.id));
      const set = selectedDupIdsByGroup.value.get(key);
      if (!set) continue;
      for (const id of Array.from(set.values())) {
        if (!validIds.has(id)) set.delete(id);
      }
    }
  },
  { immediate: true }
);
</script>
