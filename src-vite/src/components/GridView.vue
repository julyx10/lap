<template>
  <div ref="scrollable" class="mb-1 flex-1 overflow-auto t-scrollbar">
    <div id="gridView" 
      class="px-2 grid gap-2"
      :style="{ gridTemplateColumns: `repeat(auto-fit, minmax(${gridSize}px, 1fr))` }"
    >
      <div 
        v-for="(file, index) in fileList" 
        :key="index"
        :id="'item-' + index"
        :class="[
          'p-2 border-2 rounded-lg hover:text-gray-300 hover:bg-gray-600 cursor-pointer transition duration-200', 
          index === selectedIndex ? 'border-sky-500' : 'border-gray-800'
        ]"
        @click="clickItem(index)"
        @dblclick="openItem(true)"
      >
        <div class="relative flex flex-col items-center">

          <!-- action buttons -->
          <div v-if="index === selectedIndex" 
            class="absolute z-10 left-0 w-full flex flex-row items-center justify-between"
          >
            <div class="flex">
              <IconFavorite v-if="file.is_favorite" class="t-icon-size-sm"></IconFavorite>
              <IconRotate v-if="file.rotate % 360 > 0"
                class="t-icon-size-sm"
                :style="{ 
                  transform: `rotate(${file.rotate}deg)`, 
                  transition: 'transform 0.3s ease-in-out' 
                }"
              />
              <!-- <IconDelete class="t-icon-size-sm"></IconDelete> -->
            </div>
            <!-- <IconChecked class="t-icon-size t-icon-focus"></IconChecked> -->
          </div>

          <img v-if="file.thumbnail"
            :src="file.thumbnail"
            :class="[
              'rounded transition duration-200',
              config.thumbnailImageOption === 0 ? 'object-contain' : '',
              config.thumbnailImageOption === 1 ? 'object-cover' : '',
              config.thumbnailImageOption === 2 ? 'object-fill' : ''
            ]"
            :style="{ 
              width: `${gridSize}px`, height: `${gridSize}px`, 
              transform: `rotate(${file.rotate}deg)`, 
              transition: 'transform 0.3s ease-in-out' 
            }"
            loading="lazy"
          />
          <div v-else 
            class="rounded flex items-center justify-center"
            :style="{ width: `${gridSize}px`, height: `${gridSize}px` }"
          >
            <IconPhoto class="size-1/2"/>
          </div>
          <span class="pt-1 text-sm">{{ getThumbnailText(file, config.thumbnailPrimaryOption) }}</span>
          <span class="text-sm">{{ getThumbnailText(file, config.thumbnailSecondaryOption) }}</span>
        </div>
      </div>

    </div>
  </div>
</template>


<script setup lang="ts">

import { ref, watch, computed, onMounted, onUnmounted } from 'vue';
import { emit, listen } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { useConfigStore } from '@/stores/configStore';
import { shortenFilename, formatFileSize, formatTimestamp } from '@/common/utils';

import IconPhoto from '@/assets/photo.svg';
import IconFavorite from '@/assets/heart-solid.svg';
import IconRotate from '@/assets/rotate-right.svg';
import IconDelete from '@/assets/trash.svg';
import IconUnChecked from '@/assets/checkbox-unchecked.svg';
import IconChecked from '@/assets/checkbox-checked.svg';

const props = defineProps({
  modelValue: {     // selecte item index(v-model value) 
    type: Number,
    required: true,
  },
  fileList: {
    type: Array,
    required: true,
  },
  gridSize: {
    type: Number,
    default: 200,     // from 100 to 400
  },
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

// config store
const config = useConfigStore();

const selectedIndex = ref(props.modelValue);
const emitUpdate = defineEmits(['update:modelValue']);

const scrollable = ref(null); // Ref for the scrollable element

const thumbnailPrimaryText = ref('');
const thumbnailSecondaryText = ref('');

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

listen('message-from-image-viewer', (event) => {
  const { message } = event.payload;
  console.log('GriView.vue: message-from-image-viewer:', message);
  switch (message) {
    case 'rotate':
      props.fileList[selectedIndex.value].rotate = event.payload.rotate;
      break;
    case 'favorite':
      props.fileList[selectedIndex.value].is_favorite = event.payload.favorite;
      break;
    default:
      break;
  }
});

watch(() => props.modelValue, (newValue) => { 
  selectedIndex.value = newValue; 
});

watch(() => props.fileList, () => {
  selectedIndex.value = - 1;

  const element = scrollable.value; // Get the scrollable element
  element.scrollTop = 0;
});

watch(() => selectedIndex.value, (newValue) => {
  openItem(false);
  scrollToItem(newValue);

  emitUpdate('update:modelValue', newValue);
});

function clickItem(index: number) {
  selectedIndex.value = index;
}

function handleKeyDown(event) {
  if (keyActions[event.key]) {
    event.preventDefault(); // Prevent the default action
    keyActions[event.key](); 
  }
}

// function to get the text for the thumbnail
const getThumbnailText = (file, option) => {
  switch (option) {
    case 0:   // empty
      return '';
    case 1:   // name
      return shortenFilename(file.name);
    case 2:   // resolution
      return `${file.width}x${file.height}`;
    case 3:   // size
      return formatFileSize(file.size);
    case 4:   // created time
      return formatTimestamp(file.created_at, localeMsg.value.date_time_format);
    case 5:   // modified time
      return formatTimestamp(file.modified_at, localeMsg.value.date_time_format);
    case 6:   // date taken
      return file.e_date_time || '-';
    default:
      return '';
  }
};

// key actions
const keyActions = {
  ArrowDown: ()  => selectedIndex.value = Math.min(selectedIndex.value + getColumnCount(), props.fileList.length - 1),
  ArrowRight: () => selectedIndex.value = Math.min(selectedIndex.value + 1, props.fileList.length - 1),
  ArrowUp: ()    => selectedIndex.value = Math.max(selectedIndex.value - getColumnCount(), 0),
  ArrowLeft: ()  => selectedIndex.value = Math.max(selectedIndex.value - 1, 0),
  Home: ()       => selectedIndex.value = 0,
  End: ()        => selectedIndex.value = props.fileList.length - 1,
  Enter: ()      => openItem(true),
};

// open the selected item in the image viewer
function openItem(openNewViewer = false) {
  if (openNewViewer) {
    emit('message-from-grid-view', { message: 'open-image-viewer' });
  } else {
    emit('message-from-grid-view', { message: 'update-image-viewer' });
  }
};

// make the selected item always visible in a scrollable container
function scrollToItem(index) {
  const item = document.getElementById(`item-${index}`);
  if (item) {
    item.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }
};

// function to get the number of columns in the grid
function getColumnCount() {
  // get the first element with the id 'gridView'
  const gridContainer = document.querySelector('#gridView');

  const computedStyle = getComputedStyle(gridContainer);
  const gridTemplateColumns = computedStyle.gridTemplateColumns;

  // Split by space to account for grid definitions
  const columnCount = gridTemplateColumns.split(' ').length;

  return columnCount;
}

</script>

<style scoped>
* {
  user-select: none;
}
</style>
