<template>
  <div class="sidebar-panel">
    <div class="sidebar-panel-header">
      <span class="sidebar-panel-header-title flex-1">
        {{ $t('album.smart_album_list') }}<template v-if="customSmartAlbums.length > 0"> ({{ customSmartAlbums.length.toLocaleString() }})</template>
      </span>
      <TButton
        :icon="IconAdd"
        :buttonSize="'small'"
        :tooltip="$t('album.add_smart_album')"
        @click="clickAddSmartAlbum"
      />
    </div>

    <div class="mx-1 mb-2 px-1 shrink-0">
      <div
        :class="[
          'h-8 flex items-center rounded-box transition-colors bg-base-100/40',
          isSmartAlbumSearchFocused ? 'border-2 border-primary' : 'border border-base-content/10 hover:border-base-content/30',
          customSmartAlbums.length === 0 ? 'opacity-50' : '',
        ]"
      >
        <IconSearch class="ml-2 w-4 h-4 shrink-0" :class="isSmartAlbumSearchFocused ? 'text-primary/70' : 'text-base-content/30'" />
        <input
          v-model="smartAlbumSearch"
          type="text"
          :disabled="customSmartAlbums.length === 0"
          :placeholder="$t('album.search_smart_albums')"
          class="w-full min-w-0 bg-transparent border-none focus:ring-0 px-2 text-sm placeholder-base-content/30 focus:outline-none disabled:opacity-50"
          @focus="isSmartAlbumSearchFocused = true"
          @blur="isSmartAlbumSearchFocused = false"
        />
        <button
          v-if="smartAlbumSearch"
          type="button"
          :disabled="customSmartAlbums.length === 0"
          class="mr-1 p-1 rounded-box text-base-content/30 hover:text-base-content/70 disabled:opacity-30"
          @click="smartAlbumSearch = ''"
        >
          <IconClose class="w-4 h-4" />
        </button>
      </div>
    </div>

    <VueDraggable
      v-if="filteredSmartAlbums.length > 0"
      v-model="customSmartAlbums"
      tag="ul"
      :disabled="reorderingSmartAlbumId === null || Boolean(smartAlbumSearch.trim())"
      :handle="'.smart-album-drag-handle'"
      :animation="200"
      @start="onDragStart"
      @end="onDragEnd"
      @drop.stop
      class="flex-1 overflow-x-hidden overflow-y-auto rounded-box select-none"
    >
      <li v-for="smartAlbum in filteredSmartAlbums" :key="smartAlbum.id">
        <div
          :data-reordering-smart-album="isReorderingSmartAlbum(smartAlbum) ? 'true' : undefined"
          :class="[
            'sidebar-item sidebar-item-media group',
            libConfig.smartAlbum.type === 'custom' && libConfig.smartAlbum.id === smartAlbum.id ? 'sidebar-item-selected' : 'sidebar-item-hover',
          ]"
          @click="clickCustomSmartAlbum(smartAlbum)"
          @contextmenu.prevent.stop="(event: MouseEvent) => handleSmartAlbumContextMenu(smartAlbum, event)"
        >
          <IconDragHandle
            v-if="isReorderingSmartAlbum(smartAlbum)"
            class="smart-album-drag-handle p-1 w-6 h-6 shrink-0 cursor-move text-base-content/70 hover:text-base-content"
            :title="$t('album.smart_edit.reorder')"
            @click.stop
          />
          <span v-else-if="reorderingSmartAlbumId !== null" class="p-1 w-6 h-6 shrink-0"></span>
          <div class="w-10 h-10 mr-2 rounded-box shrink-0 overflow-hidden border border-base-content/5 bg-base-content/5">
            <img
              v-if="getSmartAlbumCount(smartAlbum) > 0 && getSmartAlbumCoverSrc(smartAlbum) && Number(smartAlbumCoverErrors[smartAlbum.id]) !== Number(smartAlbum.coverFileId)"
              :src="getSmartAlbumCoverSrc(smartAlbum)"
              class="w-full h-full object-cover"
              @error="smartAlbumCoverErrors[smartAlbum.id] = Number(smartAlbum.coverFileId)"
            />
            <div v-else class="w-full h-full flex items-center justify-center text-base-content/30">
              <IconFolderCog class="w-5 h-5" />
            </div>
          </div>
          <span class="sidebar-item-label">{{ smartAlbum.name }}</span>
          <div class="ml-auto flex flex-row items-center text-base-content/30">
            <span
              v-if="hasSmartAlbumCount(smartAlbum)"
              class="sidebar-item-count shrink-0"
            >{{ getSmartAlbumCount(smartAlbum).toLocaleString() }}</span>
            <div :class="[isSmartAlbumSelected(smartAlbum) ? '' : 'hidden group-hover:flex']">
              <ContextMenu
                :ref="(element: any) => { if (element) smartAlbumContextMenus[smartAlbum.id] = element }"
                :iconMenu="IconMore"
                :menuItems="getMoreMenuItems(smartAlbum)"
                :smallIcon="true"
              />
            </div>
          </div>
        </div>
      </li>
    </VueDraggable>
    <div v-else-if="customSmartAlbums.length > 0" class="sidebar-empty text-sm">
      <span class="text-center">{{ $t('album.no_smart_albums_found') }}</span>
    </div>
    <div v-else class="mt-2 px-2 flex flex-col items-center justify-center text-base-content/30">
      <span class="text-sm text-center">{{ $t('album.no_smart_albums.description') }}</span>
    </div>

    <SmartAlbumEdit
      v-if="showEditDialog"
      :smartAlbum="editingSmartAlbum"
      @ok="saveSmartAlbum"
      @cancel="closeEditDialog"
    />
    <MessageBox
      v-if="showDeleteDialog"
      :title="$t('album.smart_edit.delete')"
      :message="$t('album.smart_edit.delete_message', { name: deletingSmartAlbum?.name || '' })"
      :OkText="$t('album.smart_edit.delete')"
      :cancelText="$t('msgbox.cancel')"
      warningOk
      @ok="confirmDeleteSmartAlbum"
      @cancel="closeDeleteDialog"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { listen } from '@tauri-apps/api/event';
import { VueDraggable } from 'vue-draggable-plus';
import { config, libConfig } from '@/common/config';
import { useUIStore } from '@/stores/uiStore';
import { getThumbUrl, getThumbnailDataUrl, getThumbnailDataUrlInflight, isWin, setThumbnailDataUrlInflight } from '@/common/utils';
import { getFileThumbById, getSmartQueryCountAndSum, listenIndexFinished } from '@/common/api';
import { IconAdd, IconClose, IconDragHandle, IconEdit, IconMore, IconFolderCog, IconOrder, IconSearch, IconTrash } from '@/common/icons';
import TButton from '@/components/TButton.vue';
import ContextMenu from '@/components/ContextMenu.vue';
import SmartAlbumEdit from '@/components/SmartAlbumEdit.vue';
import MessageBox from '@/components/MessageBox.vue';

const customSmartAlbums = ref<any[]>([]);
const uiStore = useUIStore();
const { t } = useI18n();
const showEditDialog = ref(false);
const editingSmartAlbum = ref<any | null>(null);
const showDeleteDialog = ref(false);
const deletingSmartAlbum = ref<any | null>(null);
const smartAlbumContextMenus = ref<Record<string, any>>({});
const smartAlbumCoverErrors = ref<Record<string, number>>({});
const smartAlbumCoverUrls = ref<Record<string, string>>({});
const reorderingSmartAlbumId = ref<string | null>(null);
const smartAlbumSearch = ref('');
const isSmartAlbumSearchFocused = ref(false);
let smartAlbumCoverLoadToken = 0;
let smartCountRequest = 0;
let smartCountsDisposed = false;
const unlistenSmartCounts: (() => void)[] = [];

const filteredSmartAlbums = computed(() => {
  const query = smartAlbumSearch.value.trim().toLowerCase();
  if (!query) return customSmartAlbums.value;
  return customSmartAlbums.value.filter(smartAlbum => smartAlbum.name.toLowerCase().includes(query));
});

watch(() => libConfig.smartAlbums, (albums) => {
  customSmartAlbums.value = Array.isArray(albums) ? [...albums] : [];
}, { immediate: true });

// Counts are computed per smart query. Re-fetch whenever the set of albums or
// their queries change (id/updatedAt), so each
// album shows its own count instead of the currently-viewed query's count.
const smartAlbumCountSignature = computed(() =>
  (libConfig.smartAlbums || []).map((album: any) => `${album.id}:${album.updatedAt ?? album.createdAt ?? ''}`).join('|')
);

async function refreshSmartAlbumCounts() {
  const request = ++smartCountRequest;
  const libraryId = libConfig._libraryId;
  const albums = customSmartAlbums.value.filter((album: any) => {
    const rules = Array.isArray(album?.query?.rules) ? album.query.rules : [];
    return rules.length > 0;
  });
  if (albums.length === 0) return;

  const results = await Promise.all(albums.map(async (album: any) => {
    const query = album.query;
    try {
      const params = {
        version: Number(query.version || 1),
        match: query.match === 'any' ? 'any' : 'all',
        rules: query.rules,
        sortType: Number(album?.sort?.type ?? 0),
        sortOrder: Number(album?.sort?.order ?? 1),
        folderSort: Number(config.settings.folderSort || 0),
        calendarSort: Number(config.settings.calendarSort || 0),
        // Count/sum query is not grouped, so category ordering is irrelevant here.
        categorySort: 0,
      };
      const result = await getSmartQueryCountAndSum(params);
      return [String(album.id), Number(result?.[0] || 0)] as const;
    } catch {
      return [String(album.id), 0] as const;
    }
  }));

  if (smartCountsDisposed || request !== smartCountRequest || libraryId !== libConfig._libraryId) return;
  const countMap = Object.fromEntries(results);
  libConfig.smartAlbums = (libConfig.smartAlbums || []).map((album: any) => ({
    ...album,
    count: countMap[String(album.id)] ?? 0,
  }));
}

watch(
  () => [smartAlbumCountSignature.value, libConfig._libraryId],
  () => { void refreshSmartAlbumCounts(); },
  { immediate: true },
);

watch(smartAlbumSearch, () => {
  reorderingSmartAlbumId.value = null;
});

const getSmartAlbumCoverSrc = (smartAlbum: any) => {
  const coverFileId = Number(smartAlbum?.coverFileId || 0);
  if (!coverFileId) return '';
  return isWin ? smartAlbumCoverUrls.value[String(smartAlbum.id)] || '' : getThumbUrl(coverFileId, false, config.settings.thumbnailSize);
};

const loadWindowsSmartAlbumCovers = async (albums: any[]) => {
  if (!isWin) return;
  const loadToken = ++smartAlbumCoverLoadToken;
  const libraryId = libConfig._libraryId;

  const covers = await Promise.all(albums.map(async (smartAlbum) => {
    const coverFileId = Number(smartAlbum?.coverFileId || 0);
    if (!coverFileId) return [String(smartAlbum.id), ''] as const;

    const inflight = getThumbnailDataUrlInflight(coverFileId, config.settings.thumbnailSize);
    const url = await (inflight || setThumbnailDataUrlInflight(
      coverFileId,
      config.settings.thumbnailSize,
      getFileThumbById(coverFileId, config.settings.thumbnailSize, false, libraryId)
        .then(thumb => libraryId === libConfig._libraryId ? getThumbnailDataUrl(thumb, '', false, config.settings.thumbnailSize) : '')
    ));
    return [String(smartAlbum.id), url || ''] as const;
  }));

  if (loadToken !== smartAlbumCoverLoadToken || libraryId !== libConfig._libraryId) return;
  smartAlbumCoverUrls.value = Object.fromEntries(covers.filter(([, url]) => url));
};

watch(
  () => customSmartAlbums.value.map((smartAlbum: any) => [smartAlbum.id, smartAlbum.coverFileId]),
  () => { void loadWindowsSmartAlbumCovers(customSmartAlbums.value); },
  { immediate: true },
);

function clickCustomSmartAlbum(smartAlbum: any) {
  libConfig.smartAlbum.type = 'custom';
  libConfig.smartAlbum.id = smartAlbum.id;
}

const isSmartAlbumSelected = (smartAlbum: any) => libConfig.smartAlbum.type === 'custom' && libConfig.smartAlbum.id === smartAlbum.id;
const getSmartAlbumCount = (smartAlbum: any) => {
  return Number(smartAlbum?.count || 0);
};
const hasSmartAlbumCount = (smartAlbum: any) => getSmartAlbumCount(smartAlbum) > 0;

function clickAddSmartAlbum() {
  editingSmartAlbum.value = null;
  showEditDialog.value = true;
}

function handleSmartAlbumContextMenu(smartAlbum: any, event: MouseEvent) {
  clickCustomSmartAlbum(smartAlbum);
  smartAlbumContextMenus.value[smartAlbum.id]?.open?.(event.clientX, event.clientY);
}

const getMoreMenuItems = (smartAlbum: any) => [
  {
    label: t('album.smart_edit.title_edit'),
    icon: IconEdit,
    action: () => clickEditSmartAlbum(smartAlbum),
  },
  {
    label: t('album.smart_edit.reorder'),
    icon: IconOrder,
    action: () => {
      reorderingSmartAlbumId.value = reorderingSmartAlbumId.value === smartAlbum.id ? null : smartAlbum.id;
    },
  },
  {
    label: "-",
    action: null
  },
  {
    label: t('album.smart_edit.delete'),
    icon: IconTrash,
    action: () => clickDeleteSmartAlbum(smartAlbum),
  },
];

const isReorderingSmartAlbum = (smartAlbum: any) => reorderingSmartAlbumId.value === smartAlbum.id;

function handleReorderOutsidePointerDown(event: PointerEvent) {
  if (event.button !== 0 || reorderingSmartAlbumId.value === null) return;
  if (event.target instanceof Element && event.target.closest('[data-reordering-smart-album="true"]')) return;
  reorderingSmartAlbumId.value = null;
}

onMounted(async () => {
  document.addEventListener('pointerdown', handleReorderOutsidePointerDown, true);
  const stopAlbum = await listen('album-updated', (event: any) => {
    if (event.payload?.filtersChanged) void refreshSmartAlbumCounts();
  });
  if (smartCountsDisposed) { stopAlbum(); return; }
  unlistenSmartCounts.push(stopAlbum);
  const stopIndex = await listenIndexFinished(() => { void refreshSmartAlbumCounts(); });
  if (smartCountsDisposed) { stopIndex(); return; }
  unlistenSmartCounts.push(stopIndex);
});

onBeforeUnmount(() => {
  smartCountsDisposed = true;
  ++smartCountRequest;
  unlistenSmartCounts.forEach(stop => stop());
  document.removeEventListener('pointerdown', handleReorderOutsidePointerDown, true);
});

function onDragStart() {
  uiStore.removeInputHandler('SmartAlbumListDrag');
  uiStore.pushInputHandler('SmartAlbumListDrag');
}

function onDragEnd() {
  setTimeout(() => uiStore.removeInputHandler('SmartAlbumListDrag'), 0);
  libConfig.smartAlbums = [...customSmartAlbums.value];
}

function clickEditSmartAlbum(smartAlbum: any) {
  editingSmartAlbum.value = smartAlbum;
  showEditDialog.value = true;
}

function closeEditDialog() {
  showEditDialog.value = false;
  editingSmartAlbum.value = null;
}

function saveSmartAlbum(smartAlbum: any) {
  const albums = [...(libConfig.smartAlbums || [])];
  const index = albums.findIndex((item: any) => item.id === smartAlbum.id);
  if (index >= 0) {
    albums[index] = smartAlbum;
  } else {
    albums.push(smartAlbum);
  }
  libConfig.smartAlbums = albums;
  libConfig.smartAlbum.type = 'custom';
  libConfig.smartAlbum.id = smartAlbum.id;
  closeEditDialog();
}

function clickDeleteSmartAlbum(smartAlbum: any) {
  deletingSmartAlbum.value = smartAlbum;
  showDeleteDialog.value = true;
}

function closeDeleteDialog() {
  showDeleteDialog.value = false;
  deletingSmartAlbum.value = null;
}

function confirmDeleteSmartAlbum() {
  const smartAlbum = deletingSmartAlbum.value;
  if (!smartAlbum) return;
  libConfig.smartAlbums = (libConfig.smartAlbums || []).filter((item: any) => item.id !== smartAlbum.id);
  if (libConfig.smartAlbum.type === 'custom' && libConfig.smartAlbum.id === smartAlbum.id) {
    libConfig.smartAlbum.type = 'system';
    libConfig.smartAlbum.id = null;
  }
  closeDeleteDialog();
}
</script>
