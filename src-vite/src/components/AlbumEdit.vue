<template>
  <ModalDialog :title="isNewAlbum ? $t('album.edit.title_add') : $t('album.edit.title')" @cancel="clickCancel">
    <section>
      <!-- two column grid layout -->
      <div class="w-full grid grid-cols-[80px_1fr] gap-x-4 gap-y-2 items-center text-xs select-none">
        <!-- Folder -->
        <div class="h-8 flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/30">{{ $t('album.edit.folder') }}</div>
        <div class="h-8 flex items-center justify-between gap-x-2">
          <input v-if="selectedFolder !== ''"
            type="text"
            readonly
            :value="selectedFolder"
            class="w-full bg-transparent border-none p-0 text-xs font-semibold text-base-content/30 focus:border-none focus:ring-0 focus:outline-none"
          />
          <button v-if="selectedFolder === ''"
            class="btn btn-primary btn-sm rounded-box"
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

        <!-- Name -->
        <div class="h-8 flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/70">{{ $t('album.edit.name') }}</div>
        <div>
          <input
            ref="inputNameRef"
            v-model="inputNameValue"
            type="text"
            maxlength="255"
            :disabled="selectedFolder === ''"
            class="w-full input input-sm text-xs font-semibold"
          />
        </div>

        <!-- Description -->
        <div class="h-8 flex items-start pt-2 text-[10px] uppercase tracking-widest font-bold text-base-content/70">{{ $t('album.edit.description') }}</div>
        <div>
          <textarea
            v-if="showDescription"
            ref="descriptionRef"
            v-model="inputDescriptionValue"
            rows="2"
            maxlength="1024"
            :placeholder="$t('album.edit.description_placeholder')"
            :disabled="selectedFolder === ''"
            class="w-full textarea textarea-sm min-h-[56px] max-h-[200px] text-xs font-semibold"
          ></textarea>
          <TButton
            v-else
            :icon="IconEdit"
            :buttonSize="'small'"
            :tooltip="$t('album.edit.description')"
            :disabled="selectedFolder === ''"
            @click="showDescriptionInput"
          />
        </div>

        <template v-if="selectedFolder !== ''">
          <!-- Files Breakdown Label (Left Column) -->
          <div class="h-8 flex items-start pt-3 text-[10px] uppercase tracking-widest font-bold text-base-content/30">
            {{ isScanning ? $t('search.index.indexing', { count: '', total: '' }).split('...')[0] : (isNewAlbum ? $t('album.edit.files_to_scan') : $t('album.edit.scanned_files')) }}
          </div>
          
          <!-- Breakdown Content (Right Column) -->
          <div class="py-3 flex flex-col gap-2">
            <!-- Total Summary (Right Aligned) -->
            <div class="flex justify-end items-baseline mb-0.5">
              <span class="text-xs font-bold text-base-content/30" :class="{ 'animate-pulse': totalImageCount < 0 }">
                {{ totalImageCount >= 0 ? $t('album.edit.files_count', { count: (totalImageCount + totalVideoCount).toLocaleString(), size: formatFileSize(totalImageSize + totalVideoSize) }) : $t('album.edit.files_counting') }}
              </span>
            </div>
            
            <template v-if="shouldShowBreakdown">
              <!-- Elegant Progress Bar -->
              <div v-if="totalImageCount >= 0" class="flex w-full h-2 bg-base-content/5 rounded-full overflow-hidden shadow-inner">
                <template v-if="isScanning">
                  <div 
                    class="bg-primary transition-all duration-300 ease-out" 
                    :style="{ width: (indexedCount / (totalCount || 1) * 100) + '%' }"
                  ></div>
                </template>
                <template v-else>
                  <div 
                    class="bg-primary/80 transition-all duration-500 ease-out shadow-[inset_-1px_0_0_rgba(0,0,0,0.1)]" 
                    :style="{ width: (totalImageCount / (totalImageCount + totalVideoCount || 1) * 100) + '%' }"
                    :title="$t('album.edit.images')"
                  ></div>
                  <div 
                    class="bg-primary/30 transition-all duration-500 ease-out" 
                    :style="{ width: (totalVideoCount / (totalImageCount + totalVideoCount || 1) * 100) + '%' }"
                    :title="$t('album.edit.videos')"
                  ></div>
                </template>
              </div>
              <progress v-else class="progress progress-primary/40 w-full h-2"></progress>

              <!-- Metadata Labels (Space Between) -->
              <div class="flex justify-between items-center px-0.5">
                <!-- Images info -->
                <div class="flex flex-col">
                  <span class="text-[10px] font-bold text-primary/70 uppercase tracking-tight">{{ $t('album.edit.images') }}</span>
                  <span class="text-[11px] font-semibold text-base-content/30">
                    {{ totalImageCount >= 0 ? totalImageCount.toLocaleString() : '...' }} 
                    <span v-if="!isScanning" class="text-[9px] font-medium ml-0.5">{{ totalImageCount >= 0 ? formatFileSize(totalImageSize) : '' }}</span>
                  </span>
                </div>
                <!-- Videos info -->
                <div class="flex flex-col items-end">
                  <span class="text-[10px] font-bold text-primary/40 uppercase tracking-tight">{{ $t('album.edit.videos') }}</span>
                  <span class="text-[11px] font-semibold text-base-content/30">
                    {{ totalVideoCount >= 0 ? totalVideoCount.toLocaleString() : '...' }}
                    <span v-if="!isScanning" class="text-[9px] font-medium ml-0.5">{{ totalVideoCount >= 0 ? formatFileSize(totalVideoSize) : '' }}</span>
                  </span>
                </div>
              </div>

              <!-- Scanning Progress Text -->
              <div v-if="isScanning" class="text-[10px] font-bold text-primary text-center uppercase tracking-widest">
                {{ $t('search.index.indexing', { count: indexedCount.toLocaleString(), total: totalCount.toLocaleString() }) }}
              </div>
            </template>
          </div>
        </template>
        
        <template v-if="!isNewAlbum">
          <!-- Created At -->
          <!-- <div class="h-8 flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/30">{{ $t('album.edit.created_at') }}</div>
          <div class="h-8 flex items-center text-xs font-semibold text-base-content/30">{{ createdAt }}</div> -->
          <!-- Modified At -->
          <!-- <div class="h-8 flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/30">{{ $t('album.edit.modified_at') }}</div>
          <div class="h-8 flex items-center text-xs font-semibold text-base-content/30">{{ modifiedAt }}</div> -->
          <!-- Last Scan Time -->
          <div class="h-8 flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/30">{{ $t('album.edit.last_scan_time') }}</div>
          <div class="h-8 flex items-center text-xs font-semibold text-base-content/30">{{ lastScanTime }}</div>
        </template>
      </div>
    </section>

    <!-- cancel and OK buttons -->
    <div class="mt-4 flex justify-end space-x-4">
      <button 
        class="t-button-default" 
        @click="clickCancel"
      >
        {{ $t('msgbox.cancel') }}
      </button>
      <button 
        class="t-button-primary"
        :disabled="inputNameValue.trim().length === 0 || selectedFolder.length === 0"
        @click="clickOk"
      >
        {{ $t('msgbox.ok') }}
      </button>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">

import { ref, watch, onMounted, onUnmounted, computed, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { countFolder, getAllAlbums, listenIndexProgress, listenIndexFinished } from '@/common/api';
import { useToast } from '@/common/toast';
import { formatFileSize, openFolderDialog, getFolderName } from '@/common/utils';
import { useUIStore } from '@/stores/uiStore';
import { useLibraryStore } from '@/stores/libraryStore';
import { getAlbumScanState } from '@/common/scanStatus';

import ModalDialog from '@/components/ModalDialog.vue';
import TButton from '@/components/TButton.vue';
import { IconEdit, IconNewFolder } from '@/common/icons';

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
  lastScanTime: { 
    type: String, 
    default: '' 
  },
});

const emit = defineEmits(['ok', 'cancel']);
const uiStore = useUIStore();
const libStore = useLibraryStore();
const { t } = useI18n();

const toast = useToast();

// select folder
const selectedFolder = ref('');

// input 
const inputNameRef = ref<HTMLInputElement | null>(null);
const descriptionRef = ref<HTMLTextAreaElement | null>(null);
const inputNameValue = ref(props.inputName);
const inputDescriptionValue = ref(props.inputDescription);
const showDescription = ref(inputDescriptionValue.value.trim().length > 0);

// total file count of the album (from disk probe)
const totalFolderCount = ref(0);
const totalImageCount = ref(-1);
const totalImageSize = ref(-1);
const totalVideoCount = ref(0);
const totalVideoSize = ref(0);

// indexing progress
const indexedCount = ref(0);
const totalCount = ref(0);
const isScanning = computed(() => {
  if (props.isNewAlbum) return false;
  return getAlbumScanState({
    albumId: props.albumId,
    albumQueue: libStore.index.albumQueue as any[],
    pausedAlbumIds: libStore.index.pausedAlbumIds as any[],
    status: Number(libStore.index.status || 0),
  }) === 'scanning';
});

const shouldShowBreakdown = computed(() => {
  // If scan (indexing) is active, always show the breakdown
  if (isScanning.value) return true;
  // For new albums, show ONLY total file/size summary (initially)
  if (props.isNewAlbum) return false;
  // For existing albums, show the breakdown by default
  return true;
});

let unlistenIndexProgress: (() => void) | undefined;
let unlistenIndexFinished: (() => void) | undefined;

watch(() => selectedFolder.value, (newPath) => {
  if(newPath) {
    if (props.isNewAlbum) {
      // get folder name
      inputNameValue.value = getFolderName(newPath);
      inputDescriptionValue.value = '';
      showDescription.value = false;
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
  
  // listen for index progress
  unlistenIndexProgress = await listenIndexProgress((event: any) => {
    const { album_id, current, total } = event.payload;
    if (Number(album_id) === Number(props.albumId)) {
      indexedCount.value = current;
      totalCount.value = total;
    }
  });

  // listen for index finished
  unlistenIndexFinished = await listenIndexFinished((event: any) => {
    const { album_id } = event.payload;
    if (Number(album_id) === Number(props.albumId)) {
      // Refresh album info if needed? Usually total counts should be updated.
    }
  });

  if (props.albumPath) {
    selectedFolder.value = props.albumPath;

    setTimeout(() => {
      inputNameRef.value?.focus();
    }, 50); // 50ms delay
  }
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('AlbumEdit');
  
  if (unlistenIndexProgress) unlistenIndexProgress();
  if (unlistenIndexFinished) unlistenIndexFinished();
});

const clickSelectFolder = async () => {
  const folderPath = await openFolderDialog();
  if (folderPath) {
    selectedFolder.value = folderPath;
    // Auto focus name input after folder selected
    setTimeout(() => {
      inputNameRef.value?.focus();
    }, 100);
  }
};

async function showDescriptionInput() {
  showDescription.value = true;
  await nextTick();
  descriptionRef.value?.focus();
}

function handleKeyDown(event: KeyboardEvent) {
  if (!uiStore.isInputActive('AlbumEdit')) return;

  const { key } = event;
  const activeElement = document.activeElement;
  const isInputOrTextarea = activeElement?.tagName === 'INPUT' || activeElement?.tagName === 'TEXTAREA';

  switch (key) {
    case 'Enter':
      // Allow Enter to submit if in a text input (but not a textarea)
      if (activeElement?.tagName === 'INPUT' || !isInputOrTextarea) {
        event.preventDefault();
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
      const albums = await getAllAlbums();
      const exists = albums?.some((album: any) => album.path === selectedFolder.value);
      if (exists) {
        toast.warning(t('tooltip.album_exists'));
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
