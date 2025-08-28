<template>

  <div class="w-full flex flex-col select-none">

    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between whitespace-nowrap" data-tauri-drag-region>
      <span class="cursor-default" data-tauri-drag-region>{{ titlebar }}</span>
    </div>

    <AlbumList ref="albumListRef" 
      :key="albumListKey"
      v-model:albumId="config.trashAlbumId"
      v-model:folderId="config.trashFolderId"
      v-model:folderPath="config.trashFolderPath"
      :componentId="2"
    />

  </div>

</template>

<script setup lang="ts">

import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/utils';

import AlbumList from '@/components/AlbumList.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

const albumListRef = ref<AlbumList | null>(null);

// refresh component
const albumListKey = ref(0);

</script>