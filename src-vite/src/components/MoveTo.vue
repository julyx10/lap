<template>
    
  <div class="fixed inset-0 flex items-center justify-center bg-gray-900 bg-opacity-60 z-50 backdrop-blur-0">
    <div class="w-96 p-4 t-color-bg-light rounded shadow-lg">

      <!-- titlebar -->
      <div class="mb-2 flex items-center justify-between">
        {{ title }}
        <IconCancel class="t-icon-size-sm t-icon-hover" @click="clickCancel" />
      </div>

      <!-- message -->
      <div class="mb-2">
        {{ message }}
      </div>

      <!-- select album and folder -->
      <SelectAlbum ref="selectAlbumRef" 
        v-model:albumId="config.destAlbumId"
        v-model:folderId="config.destFolderId"
        v-model:folderPath="config.destFolderPath"
        :componentId="1"
      />

      <!-- action buttons -->
      <div class="flex justify-between mt-4">
        <button v-if="cancelText.length > 0" 
          :class="[
            'mr-auto px-4 py-1 rounded-full t-color-bg-light text-nowrap',
            config.destAlbumId > 0 ? 't-color-bg-hover t-icon-hover' : 't-icon-disabled'
          ]"
          @click="clickNewFolder"
        >{{$t('msgbox_new_folder_title')}}</button>

        <button v-if="cancelText.length > 0" 
          class="items-end px-4 py-1 rounded-full t-color-bg-light t-color-bg-hover t-icon-hover text-nowrap" 
          @click="clickCancel"
        >{{ cancelText }}</button>

        <button 
          :class="[
            'px-4 py-1 rounded-full t-color-bg-light text-nowrap', 
            config.destAlbumId > 0 ? 't-color-bg-hover t-icon-hover' : 't-icon-disabled'
          ]" 
          @click="clickOk"
        >{{ OkText }}</button>

      </div>

    </div>

  </div>

</template>

<script setup lang="ts">
import { useConfigStore } from '@/stores/configStore';

import { ref, computed } from 'vue';
// import { useI18n } from 'vue-i18n';

import SelectAlbum from '@/components/SelectAlbum.vue';

// svg icons
import IconCancel from '@/assets/close.svg';

const props = defineProps({
  title: {
    type: String,
    required: true
  },
  message: {
    type: String,
    required: true
  },
  OkText: {
    type: String, 
    default: 'OK'
  },
  cancelText: { 
    type: String, 
    default: 'Cancel' 
  },  
});

/// i18n
// const { locale, messages } = useI18n();
// const localeMsg = computed(() => messages.value[locale.value]);

// config store
const config = useConfigStore();

const emit = defineEmits(['ok', 'cancel']);

// const selectedAlbumId = ref(0);
// const selectedFolderId = ref(0);


const clickNewFolder = () => {
  selectAlbumRef.value.clickNewFolder();
};

const clickCancel = () => {
  emit('cancel');
};

const clickOk = () => {
  if (selectedAlbumId.value === 0) return;
  emit('ok', { albumId: selectedAlbumId.value, folderId: selectedFolderId.value });
};


/// click a album to select it
const clickAlbum = async (album) => {
  // selectedAlbumId.value = album.id;
  // selectedFolderId.value = album.folderId;
};

/// click folder to select
const clickFolder = async (albumId, folder) => {
  // selectedAlbumId.value = albumId;
  // selectedFolderId.value = folder.id;
};

</script>