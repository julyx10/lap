<template>

  <dialog id="albumInfoDialog" class="modal">
    <div class="w-96 p-4 text-base-content/70 bg-base-200 border border-base-content/30 rounded-box">

        <!-- title bar -->
        <div class="mb-2 flex items-center justify-between">
          {{ $t('album.edit.title') }}
          <TButton
            :icon="IconClose"
            :buttonSize="'small'"
            @click="clickCancel"
          />
        </div>

        <!-- two colums table -->
        <table class="mt-4 w-full text-sm text-nowrap">
          <tbody>
            <tr>
              <td>{{ $t('album.edit.name') }}</td>
              <td>
                <input
                  ref="inputNameRef"
                  v-model="inputNameValue"
                  type="text"
                  maxlength="255"
                  class="px-2 py-1 w-full input"
                />
              </td>
            </tr>
            <tr>
              <td>{{ $t('album.edit.description') }}</td>
              <td>
                <textarea
                  v-model="inputDescriptionValue"
                  rows="2"
                  maxlength="1024"
                  :placeholder="$t('album.edit.description_placeholder')"
                  class="px-2 py-1 mt-2 w-full textarea min-h-[30px] max-h-[200px]`"
                ></textarea>
              </td>
            </tr>
            <tr class="h-8 py-2">
              <td>{{ $t('album.edit.hidden') }}</td>
              <td>
                <div class="tooltip" :data-tip="$t('album.edit.hidden_description')">
                  <input type="checkbox" class="toggle toggle-sm" v-model="hiddenAlbumValue" />
                </div>
              </td>
            </tr>
            <tr class="h-8">
              <td>{{ $t('album.edit.folder') }}</td>
              <td>
                <input
                  type="text"
                  readonly
                  :value="albumPath"
                  class="py-1 w-full border-none focus:border-none focus:ring-0 focus:outline-none"
                />
              </td>
            </tr>
            <tr class="h-8">
              <td>{{ $t('album.edit.images') }}</td>
              <td>{{ totalImageCount >= 0 ? $t('album.edit.files_count', {count: totalImageCount.toLocaleString(), size: formatFileSize(totalImageSize) }) : $t('album.edit.files_counting') }}</td>
            </tr>
            <tr class="h-8">
              <td>{{ $t('album.edit.videos') }}</td>
              <td>{{ totalVideoCount >= 0 ? $t('album.edit.files_count', {count: totalVideoCount.toLocaleString(), size: formatFileSize(totalVideoSize) }) : $t('album.info.files_counting') }}</td>
            </tr>
            <tr class="h-8">
              <td>{{ $t('album.edit.created_at') }}</td>
              <td>{{ createdAt }}</td>
            </tr>
            <tr class="h-8">
              <td>{{ $t('album.edit.modified_at') }}</td>
              <td>{{ modifiedAt }}</td>
            </tr>
          </tbody>
        </table>

        <!-- cancel and OK buttons -->
        <div class="mt-4 flex justify-end space-x-4">
          <button 
            class="px-4 py-1 rounded-box hover:bg-base-100 cursor-pointer" 
            @click="clickCancel"
          >
            {{ $t('msgbox.cancel') }}
          </button>
          <button 
            :class="[
              'px-4 py-1 rounded-box', 
              inputNameValue.trim().length > 0 ? 'hover:bg-primary hover:text-base-100 cursor-pointer' : 'text-base-content/30 cursor-default',
            ]" 
            @click="clickOk"
          >
            {{ $t('msgbox.ok') }}
          </button>
        </div>

    </div>

  </dialog>

</template>

<script setup lang="ts">

import { ref, onMounted, onUnmounted } from 'vue';
import { countFolder } from '@/common/api';
import { formatFileSize } from '@/common/utils';
import { listen } from '@tauri-apps/api/event';
import { useUIStore } from '@/stores/uiStore';

import { IconClose } from '@/common/icons';

import TButton from '@/components/TButton.vue';

const props = defineProps({
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
  hiddenAlbum: { 
    type: Boolean, 
    default: false 
  },
  albumPath: { 
    type: String, 
    default: '' 
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

// input 
const inputNameRef = ref<HTMLInputElement | null>(null);
const inputNameValue = ref(props.inputName);
const inputDescriptionValue = ref(props.inputDescription);
const hiddenAlbumValue = ref(props.hiddenAlbum);

// total file count of the album
const totalFolderCount = ref(0);
const totalImageCount = ref(-1);
const totalImageSize = ref(-1);
const totalVideoCount = ref(0);
const totalVideoSize = ref(0);

let unlistenKeydown: () => void;

onMounted(async () => {
  const albumInfoDialog = document.getElementById('albumInfoDialog') as HTMLDialogElement | null;
  albumInfoDialog?.showModal();

  unlistenKeydown = await listen('global-keydown', handleKeyDown);
  uiStore.pushInputHandler('AlbumEdit');
  
  setTimeout(() => {
    inputNameRef.value?.focus();
  }, 50); // 50ms delay

  // count folder
  countFolder(props.albumPath).then((res) => {
    [totalFolderCount.value, totalImageCount.value, totalImageSize.value, totalVideoCount.value, totalVideoSize.value] = res;
    console.log('count folder:', res);
  }).catch((err) => {
    console.error('count folder error:', err);
  });
});

onUnmounted(() => {
  if (unlistenKeydown) {
    unlistenKeydown();
  }
  uiStore.popInputHandler();
});

function handleKeyDown(event) {
  if (!uiStore.isInputActive('AlbumEdit')) return;

  const { key } = event.payload;

  switch (key) {
    case 'Enter':
      clickOk();
      break;
    case 'Escape':
      clickCancel();
      break;
    default:
      break;
  }
}

const clickOk = () => {
  if (inputNameValue.value.trim().length > 0) {
    emit(
      'ok', 
      inputNameValue.value, 
      inputDescriptionValue.value ? inputDescriptionValue.value : '', 
      hiddenAlbumValue.value ? hiddenAlbumValue.value: false
    );
  }
};

const clickCancel = () => {
  emit('cancel');
};

</script>
