<template>
  <div class="sidebar-panel">
    <div class="sidebar-panel-header">
      <span class="sidebar-panel-header-title flex-1">{{ $t('album.smart_album_list') }}</span>
      <TButton
        :icon="IconAdd"
        :buttonSize="'small'"
        :tooltip="$t('album.add_smart_album')"
        @click="clickAddSmartAlbum"
      />
    </div>

    <ul
      v-if="customSmartAlbums.length > 0"
      class="flex-1 overflow-x-hidden overflow-y-auto rounded-box select-none"
    >
      <li v-for="smartAlbum in customSmartAlbums" :key="smartAlbum.id">
        <div
          :class="[
            'sidebar-item sidebar-item-media group',
            libConfig.smartAlbum.type === 'custom' && libConfig.smartAlbum.id === smartAlbum.id ? 'sidebar-item-selected' : 'sidebar-item-hover',
          ]"
          @click="clickCustomSmartAlbum(smartAlbum)"
          @contextmenu.prevent.stop="(event: MouseEvent) => handleSmartAlbumContextMenu(smartAlbum, event)"
        >
          <div class="w-10 h-10 mr-2 rounded-box shrink-0 overflow-hidden border border-base-content/5 bg-base-content/5">
            <img
              v-if="getSmartAlbumCoverSrc(smartAlbum) && Number(smartAlbumCoverErrors[smartAlbum.id]) !== Number(smartAlbum.coverFileId)"
              :src="getSmartAlbumCoverSrc(smartAlbum)"
              class="w-full h-full object-cover"
              @error="smartAlbumCoverErrors[smartAlbum.id] = Number(smartAlbum.coverFileId)"
            />
            <div v-else class="w-full h-full flex items-center justify-center text-base-content/30">
              <IconPhoto class="w-5 h-5" />
            </div>
          </div>
          <span class="sidebar-item-label">{{ smartAlbum.name }}</span>
          <div class="ml-auto">
            <span
              v-if="hasSmartAlbumCount(smartAlbum)"
              class="sidebar-item-count"
              :class="isSmartAlbumSelected(smartAlbum) ? 'hidden' : 'group-hover:hidden'"
            >{{ getSmartAlbumCount(smartAlbum).toLocaleString() }}</span>
          </div>
          <div :class="['flex items-center text-base-content/30', isSmartAlbumSelected(smartAlbum) ? '' : 'hidden group-hover:flex']">
            <ContextMenu
              :ref="(element: any) => { if (element) smartAlbumContextMenus[smartAlbum.id] = element }"
              :iconMenu="IconMore"
              :menuItems="getMoreMenuItems(smartAlbum)"
              :smallIcon="true"
            />
          </div>
        </div>
      </li>
    </ul>
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
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { config, libConfig } from '@/common/config';
import { useUIStore } from '@/stores/uiStore';
import { getThumbUrl, getThumbnailDataUrl, getThumbnailDataUrlInflight, isWin, setThumbnailDataUrlInflight } from '@/common/utils';
import { getFileThumbById } from '@/common/api';
import { IconAdd, IconEdit, IconMore, IconPhoto, IconTrash } from '@/common/icons';
import TButton from '@/components/TButton.vue';
import ContextMenu from '@/components/ContextMenu.vue';
import SmartAlbumEdit from '@/components/SmartAlbumEdit.vue';
import MessageBox from '@/components/MessageBox.vue';

const customSmartAlbums = computed(() => libConfig.smartAlbums || []);
const uiStore = useUIStore();
const { t } = useI18n();
const showEditDialog = ref(false);
const editingSmartAlbum = ref<any | null>(null);
const showDeleteDialog = ref(false);
const deletingSmartAlbum = ref<any | null>(null);
const smartAlbumContextMenus = ref<Record<string, any>>({});
const smartAlbumCoverErrors = ref<Record<string, number>>({});
const smartAlbumCoverUrls = ref<Record<string, string>>({});
let smartAlbumCoverLoadToken = 0;

const getSmartAlbumCoverSrc = (smartAlbum: any) => {
  const coverFileId = Number(smartAlbum?.coverFileId || 0);
  if (!coverFileId) return '';
  return isWin ? smartAlbumCoverUrls.value[String(smartAlbum.id)] || '' : getThumbUrl(coverFileId, false, config.settings.thumbnailSize);
};

const loadWindowsSmartAlbumCovers = async (albums: any[]) => {
  if (!isWin) return;
  const loadToken = ++smartAlbumCoverLoadToken;

  const covers = await Promise.all(albums.map(async (smartAlbum) => {
    const coverFileId = Number(smartAlbum?.coverFileId || 0);
    if (!coverFileId) return [String(smartAlbum.id), ''] as const;

    const inflight = getThumbnailDataUrlInflight(coverFileId, config.settings.thumbnailSize);
    const url = await (inflight || setThumbnailDataUrlInflight(
      coverFileId,
      config.settings.thumbnailSize,
      getFileThumbById(coverFileId, config.settings.thumbnailSize, false)
        .then(thumb => getThumbnailDataUrl(thumb, '', false, config.settings.thumbnailSize))
    ));
    return [String(smartAlbum.id), url || ''] as const;
  }));

  if (loadToken !== smartAlbumCoverLoadToken) return;
  smartAlbumCoverUrls.value = Object.fromEntries(covers.filter(([, url]) => url));
};

watch(
  () => customSmartAlbums.value.map((smartAlbum: any) => [smartAlbum.id, smartAlbum.coverFileId]),
  () => { void loadWindowsSmartAlbumCovers(customSmartAlbums.value); },
  { immediate: true },
);

function clickCustomSmartAlbum(smartAlbum: any) {
  uiStore.smartAlbumCountRequestedFor = String(smartAlbum.id);
  uiStore.smartAlbumCountRequestTick++;
  libConfig.smartAlbum.type = 'custom';
  libConfig.smartAlbum.id = smartAlbum.id;
}

const isSmartAlbumSelected = (smartAlbum: any) => libConfig.smartAlbum.type === 'custom' && libConfig.smartAlbum.id === smartAlbum.id;
const getSmartAlbumCount = (smartAlbum: any) => Number(smartAlbum?.count || 0);
const hasSmartAlbumCount = (smartAlbum: any) => smartAlbum?.count !== null && smartAlbum?.count !== undefined;

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
    label: t('album.smart_edit.delete'),
    icon: IconTrash,
    action: () => clickDeleteSmartAlbum(smartAlbum),
  },
];

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
