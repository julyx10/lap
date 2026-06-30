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
            'sidebar-item group',
            libConfig.smartAlbum.type === 'custom' && libConfig.smartAlbum.id === smartAlbum.id ? 'sidebar-item-selected' : 'sidebar-item-hover',
          ]"
          @click="clickCustomSmartAlbum(smartAlbum)"
          @contextmenu.prevent.stop="(event: MouseEvent) => handleSmartAlbumContextMenu(smartAlbum, event)"
        >
          <IconBolt class="mx-1 w-5 h-5 shrink-0" />
          <span class="sidebar-item-label">{{ smartAlbum.name }}</span>
          <div
            :class="[
              'ml-auto flex items-center text-base-content/30',
              libConfig.smartAlbum.type === 'custom' && libConfig.smartAlbum.id === smartAlbum.id ? '' : 'hidden group-hover:flex'
            ]"
          >
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
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { libConfig } from '@/common/config';
import { IconAdd, IconBolt, IconEdit, IconMore, IconTrash } from '@/common/icons';
import TButton from '@/components/TButton.vue';
import ContextMenu from '@/components/ContextMenu.vue';
import SmartAlbumEdit from '@/components/SmartAlbumEdit.vue';
import MessageBox from '@/components/MessageBox.vue';

const customSmartAlbums = computed(() => libConfig.smartAlbums || []);
const { t } = useI18n();
const showEditDialog = ref(false);
const editingSmartAlbum = ref<any | null>(null);
const showDeleteDialog = ref(false);
const deletingSmartAlbum = ref<any | null>(null);
const smartAlbumContextMenus = ref<Record<string, any>>({});

function clickCustomSmartAlbum(smartAlbum: any) {
  libConfig.smartAlbum.type = 'custom';
  libConfig.smartAlbum.id = smartAlbum.id;
}

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
