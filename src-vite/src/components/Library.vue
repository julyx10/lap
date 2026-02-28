<template>
    
  <div class="sidebar-panel">

    <!-- All Files -->
    <div
      v-if="hasAlbums"
      :class="[
        'sidebar-item',
        libConfig.album.id === 0 ? 'sidebar-item-selected' : 'sidebar-item-hover',
      ]"
      @click="clickAllFiles"
    >
      <IconPhotoAll class="mx-1 w-5 h-5 shrink-0" />
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
      @editDataChanged="(val) => emit('editDataChanged', val)"
    />
  </div> 

</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { libConfig } from '@/common/config';

import { IconPhotoAll } from '@/common/icons';
import { getTotalCountAndSum } from '@/common/api';
import AlbumList from '@/components/AlbumList.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const emit = defineEmits(['editDataChanged']);

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const totalCount = ref(0);

const albumListRef = ref<InstanceType<typeof AlbumList> | null>(null);

// refresh component
const albumListKey = ref(0);

// Check if there are any albums
const hasAlbums = computed(() => (albumListRef.value?.albums?.length ?? 0) > 0);

onMounted(async () => {
  // get total count
  getTotalCountAndSum().then((result) => {
    if(result) {
      totalCount.value = result[0];
    }
  });
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
