<template>
  <ModalDialog :title="isNewAlbum ? $t('album.edit.title_add') : $t('album.edit.title')" @cancel="clickCancel">
    <!-- two column grid layout -->
    <div class="w-full text-sm text-nowrap grid grid-cols-[auto_1fr_auto] gap-x-4 gap-y-1 items-center">
      <!-- Folder -->
      <div class="h-8 flex items-center col-start-1">{{ $t('album.edit.folder') }}</div>
      <div class="h-12 flex items-center justify-between gap-x-2 col-start-2">
        <input v-if="selectedFolder !== ''"
          type="text"
          readonly
          :value="selectedFolder"
          class="p-1 w-full border-none focus:border-none focus:ring-0 focus:outline-none"
        />
        <button v-if="selectedFolder === ''"
          class="btn btn-primary rounded-box"
          @click="clickSelectFolder"
        >
          <IconNewFolder class="w-4 h-4" />
          {{ $t('album.edit.select_folder') }}
        </button>
        <TButton v-if="isNewAlbum && selectedFolder !== ''"
          :icon="IconNewFolder"
          :selected="true"
          @click="clickSelectFolder"
        />
      </div>

      <!-- Cover Image -->
      <div v-if="coverPreview" class="col-start-3 row-span-2">
        <img :src="coverPreview" class="w-20 h-20 object-cover rounded-box shadow-md border border-base-content/10" />
      </div>

      <!-- Name -->
      <div class="flex items-center col-start-1">{{ $t('album.edit.name') }}</div>
      <div class="col-start-2">
        <input
          ref="inputNameRef"
          v-model="inputNameValue"
          type="text"
          maxlength="255"
          :disabled="selectedFolder === ''"
          class="w-full input"
        />
      </div>

      <!-- Description -->
      <div class="flex items-start pt-2 col-start-1">{{ $t('album.edit.description') }}</div>
      <div class="col-start-2 col-span-2">
        <textarea
          v-model="inputDescriptionValue"
          rows="2"
          maxlength="1024"
          :placeholder="$t('album.edit.description_placeholder')"
          :disabled="selectedFolder === ''"
          class="my-2 w-full textarea min-h-[50px] max-h-[200px]"
        ></textarea>
      </div>

      <template v-if="selectedFolder !== ''">
        <!-- Images -->
        <div class="h-8 flex items-center col-start-1">{{ $t('album.edit.images') }}</div>
        <div class="h-8 flex items-center col-start-2 col-span-2">{{ totalImageCount >= 0 ? $t('album.edit.files_count', {count: totalImageCount.toLocaleString(), size: formatFileSize(totalImageSize) }) : $t('album.edit.files_counting') }}</div>
        <!-- Videos -->
        <div class="h-8 flex items-center col-start-1">{{ $t('album.edit.videos') }}</div>
        <div class="h-8 flex items-center col-start-2 col-span-2">{{ totalVideoCount >= 0 ? $t('album.edit.files_count', {count: totalVideoCount.toLocaleString(), size: formatFileSize(totalVideoSize) }) : $t('album.info.files_counting') }}</div>
      </template>
      
      <template v-if="!isNewAlbum">
        <!-- Created At -->
        <div class="h-8 flex items-center col-start-1">{{ $t('album.edit.created_at') }}</div>
        <div class="h-8 flex items-center col-start-2 col-span-2">{{ createdAt }}</div>
        <!-- Modified At -->
        <div class="h-8 flex items-center col-start-1">{{ $t('album.edit.modified_at') }}</div>
        <div class="h-8 flex items-center col-start-2 col-span-2">{{ modifiedAt }}</div>
      </template>
    </div>

    <!-- cancel and OK buttons -->
    <div class="mt-4 flex justify-end space-x-4">
      <button 
        class="px-4 py-1 rounded-box hover:bg-base-100 hover:text-base-content cursor-pointer" 
        @click="clickCancel"
      >
        {{ $t('msgbox.cancel') }}
      </button>
      <button 
        :class="[
          'px-4 py-1 rounded-box', 
          inputNameValue.trim().length > 0 && !isIndexing ? 'hover:bg-primary hover:text-base-100 cursor-pointer' : 'text-base-content/30 cursor-default',
        ]" 
        @click="clickOk"
      >
        {{ $t('msgbox.ok') }}
      </button>
    </div>
    <ToolTip ref="toolTipRef" />
  </ModalDialog>
</template>

<script setup lang="ts">

import { ref, watch, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { countFolder, getAllAlbums, getFileThumb, getFileInfo } from '@/common/api';
import { formatFileSize, openFolderDialog } from '@/common/utils';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';

import ModalDialog from '@/components/ModalDialog.vue';
import TButton from '@/components/TButton.vue';
import ToolTip from '@/components/ToolTip.vue';
import { IconNewFolder } from '@/common/icons';

const props = defineProps({
  isNewAlbum: {
    type: Boolean,
    default: false
  },
  albumId: {
    type: Number,
    required: true
  },
  inputName: { 
    type: String, 
    default: '' 
  },
  inputDescription: { 
    type: String, 
    default: '' 
  },
  albumPath: { 
    type: String, 
    default: '' 
  },
  albumCoverFileId: { 
    type: Number, 
    default: null 
  },
  createdAt: { 
    type: String, 
    default: '' 
  },
  modifiedAt: { 
    type: String, 
    default: '' 
  },
});

const emit = defineEmits(['ok', 'cancel']);
const uiStore = useUIStore();
const { t } = useI18n();

// tooltip
const toolTipRef = ref<InstanceType<typeof ToolTip> | null>(null);

// select folder
const selectedFolder = ref('');

// input 
const inputNameRef = ref<HTMLInputElement | null>(null);
const inputNameValue = ref(props.inputName);
const inputDescriptionValue = ref(props.inputDescription);

// total file count of the album
const totalFolderCount = ref(0);
const totalImageCount = ref(-1);
const totalImageSize = ref(-1);
const totalVideoCount = ref(0);
const totalVideoSize = ref(0);

// cover preview
const coverPreview = ref('');

// image recognition
const isIndexing = ref(false);
const indexedImageCount = ref(0);
// const indexedVideoCount = ref(0);
let shouldStopIndexing = false;

watch(() => selectedFolder.value, (newPath) => {
  if(newPath) {
    if (props.isNewAlbum) {
      // get folder name
      const folderName = newPath.split('/').pop();
      inputNameValue.value = folderName || '';
      inputDescriptionValue.value = '';
    }

    countFolder(newPath).then((res) => {
      [totalFolderCount.value, totalImageCount.value, totalImageSize.value, totalVideoCount.value, totalVideoSize.value] = res;
      console.log('count folder:', res);
    }).catch((err) => {
      console.error('count folder error:', err);
    });
  }
});

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('AlbumEdit');
  
  if (props.isNewAlbum) {
    // clickSelectFolder();
  }
  else {
    selectedFolder.value = props.albumPath;

    if (props.albumCoverFileId) {
      const file = await getFileInfo(props.albumCoverFileId);
      if (file) {
        const thumb = await getFileThumb(file.id, file.file_path, file.file_type, file.e_orientation || 0, config.settings.thumbnailSize, false);
        if (thumb && thumb.error_code === 0) {
          coverPreview.value = `data:image/jpeg;base64,${thumb.thumb_data_base64}`;
        }
      }
    }

    setTimeout(() => {
      inputNameRef.value?.focus();
    }, 50); // 50ms delay
  }
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('AlbumEdit');
  // Stop indexing if component is unmounted
  shouldStopIndexing = true;
});

const clickSelectFolder = async () => {
  const folderPath = await openFolderDialog();
  if (folderPath) {
    selectedFolder.value = folderPath;
  }
};

function handleKeyDown(event: KeyboardEvent) {
  if (!uiStore.isInputActive('AlbumEdit')) return;

  const { key } = event;
  const activeElement = document.activeElement;
  const isTextarea = activeElement?.tagName === 'TEXTAREA';

  switch (key) {
    case 'Enter':
      // Don't trigger OK if user is in a textarea (allow multiline input)
      if (!isTextarea) {
        clickOk();
      }
      break;
    case 'Escape':
      clickCancel();
      break;
    default:
      break;
  }
}

const clickOk = async () => {
  if (inputNameValue.value.trim().length > 0 && selectedFolder.value.length > 0) {
    // Check if album with this path already exists
    if (props.isNewAlbum) {
      const albums = await getAllAlbums(false);
      const exists = albums?.some((album: any) => album.path === selectedFolder.value);
      if (exists) {
        toolTipRef.value?.showTip(t('tooltip.album_exists'));
        return;
      }
    }
    
    emit(
      'ok', 
      selectedFolder.value,
      inputNameValue.value, 
      inputDescriptionValue.value ? inputDescriptionValue.value : '',
      props.isNewAlbum
    );
  }
};

const clickCancel = () => {
  emit('cancel');
};

</script>
