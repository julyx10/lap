<template>
    
  <div class="sidebar-panel">
    <!-- All Files -->
    <div
      :class="[
        'sidebar-item',
        libConfig.album.id === 0 ? 'sidebar-item-selected' : 'sidebar-item-hover',
      ]"
      @click="clickAllFiles"
    >
      <IconFolders class="mx-1 w-5 h-5 shrink-0" />
      <div class="sidebar-item-label">
        <span>{{ $t('album.all_files') }}</span>
      </div>
      <div class="ml-auto flex items-center">
        <span v-if="totalCount > 0" class="px-1 text-xs text-base-content/30">
          {{ totalCount.toLocaleString() }}
        </span>
      </div>
    </div>

    <AlbumList ref="albumListRef"
      :key="albumListKey"
      selectionSource="album"
    />
  </div> 

</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { libConfig } from '@/common/config';

import { IconFolders } from '@/common/icons';
import { getTotalCountAndSum } from '@/common/api';
import AlbumList from '@/components/AlbumList.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const emit = defineEmits(['editDataChanged']);

const totalCount = ref(0);

const albumListRef = ref<InstanceType<typeof AlbumList> | null>(null);
let unlistenLibraryTotalRefreshed: (() => void) | null = null;
let unlistenLibrarySwitched: (() => void) | null = null;

// refresh component
const albumListKey = ref(0);

// Check if there are any albums
const hasAlbums = computed(() => (albumListRef.value?.albums?.length ?? 0) > 0);

const refreshTotalCount = async () => {
  const result = await getTotalCountAndSum();
  totalCount.value = result ? result[0] : 0;
};

onMounted(async () => {
  await refreshTotalCount();
  unlistenLibraryTotalRefreshed = await listen('library-total-refreshed', refreshTotalCount);
  unlistenLibrarySwitched = await listen('library-switched', async () => {
    albumListKey.value++;          // force AlbumList remount
    await refreshTotalCount();
  });
});

onBeforeUnmount(() => {
  if (unlistenLibraryTotalRefreshed) {
    unlistenLibraryTotalRefreshed();
    unlistenLibraryTotalRefreshed = null;
  }
  if (unlistenLibrarySwitched) {
    unlistenLibrarySwitched();
    unlistenLibrarySwitched = null;
  }
});

const clickAllFiles = () => {
    libConfig.album.id = 0;
    libConfig.album.folderId = null;
    libConfig.album.folderPath = '';
    libConfig.album.selected = false;
};

defineExpose({
  albumListRef,
});

</script>
