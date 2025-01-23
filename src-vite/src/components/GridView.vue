<template>
  <div ref="scrollable" class="mb-1 flex-1 overflow-auto t-scrollbar select-none">
    <div id="gridView" 
      class="px-2 grid gap-2"
      :style="{ gridTemplateColumns: `repeat(auto-fit, minmax(${config.thumbnailSize}px, 1fr))` }"
    >
      <div 
        v-for="(file, index) in fileList" 
        :key="index"
        :id="'item-' + index"
        :class="[
          'p-2 border-2 rounded-lg hover:text-gray-300 hover:bg-gray-600 cursor-pointer transition duration-200 group', 
          !selectMode && index === selectedIndex ? 'border-sky-500' : 'border-gray-800',
          selectMode && file.isSelected ? 'border-sky-500' : 'border-gray-800',
        ]"
        @click="clickItem(index)"
        @dblclick="openItem(true)"
      >
        <div class="relative flex flex-col items-center">

          <!-- action buttons -->
          <div 
            :class="['absolute z-10 left-0 w-full flex flex-row items-center justify-between',
          ]"
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
            </div>

            <component v-if="selectMode"
              :is="file?.isSelected ? IconChecked : IconUnChecked" 
              :class="['t-icon-size t-icon-hover', file?.isSelected ? 'text-sky-500' : 'text-gray-500']" 
              @click.stop="selectItem(index)"
            />
            <DropDownMenu v-else-if="index === selectedIndex && !selectMode"
              :iconMenu="IconMore"
              :menuItems="moreMenuItems"
              :alignRight="true"
              @click="clickItem(index)"
            />

          </div>

          <img v-if="file.thumbnail"
            :src="file.thumbnail"
            :class="[
              'rounded transition duration-200',
              config.thumbnailScalingOption === 0 ? 'object-contain' : '',
              config.thumbnailScalingOption === 1 ? 'object-cover' : '',
              config.thumbnailScalingOption === 2 ? 'object-fill' : ''
            ]"
            :style="{ 
              width: `${config.thumbnailSize}px`, height: `${config.thumbnailSize}px`, 
              transform: `rotate(${file.rotate}deg)`, 
              transition: 'transform 0.3s ease-in-out' 
            }"
            loading="lazy"
          />
          <div v-else 
            class="rounded flex items-center justify-center"
            :style="{ width: `${config.thumbnailSize}px`, height: `${config.thumbnailSize}px` }"
          >
            <IconImagePlaceHolder class="size-1/2"/>
          </div>
          <span class="pt-1 text-sm text-center">{{ getThumbnailText(file, config.thumbnailLabelPrimaryOption) }}</span>
          <span class="text-sm text-center">{{ getThumbnailText(file, config.thumbnailLabelSecondaryOption) }}</span>
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
import { shortenFilename, formatFileSize, formatTimestamp, getFolderPath, openFileExplorer } from '@/common/utils';
import DropDownMenu from '@/components/DropDownMenu.vue';

import IconImagePlaceHolder from '@/assets/photo.svg';
import IconChecked from '@/assets/checkbox-checked.svg';
import IconUnChecked from '@/assets/checkbox-unchecked.svg';

import IconMore from '@/assets/more.svg';
import IconOpen from '@/assets/open.svg';
import IconEdit from '@/assets/edit.svg';
import IconFavorite from '@/assets/heart-solid.svg';
import IconUnFavorite from '@/assets/heart.svg';
import IconRotate from '@/assets/rotate-right.svg';
import IconCopy from '@/assets/copy.svg';
import IconRename from '@/assets/rename.svg';
import IconCopyTo from '@/assets/copy-to.svg';
import IconMoveTo from '@/assets/move-to.svg';
import IconDelete from '@/assets/trash.svg';
import IconOpenFolder from '@/assets/folder-open.svg';

const props = defineProps({
  modelValue: {     // selecte item index(v-model value) 
    type: Number,
    required: true,
  },
  fileList: {
    type: Array,
    required: true,
  },
  selectMode: {
    type: Boolean,
    default: false,
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

watch(() => selectedIndex.value, (newValue) => {
  openItem(false);
  scrollToItem(newValue);

  emitUpdate('update:modelValue', newValue);
});

// Define menu items with labels and actions
// more menuitems
const moreMenuItems = computed(() => {
  if (selectedIndex.value < 0 || selectedIndex.value >= props.fileList.length) {
    return [];
  }

  const file = props.fileList[selectedIndex.value];
  return [
    {
      label: localeMsg.value.menu_item_open,
      shortcut: 'Enter',
      action: () => {
        openItem(true);
      }
    },
    {
      label: localeMsg.value.menu_item_edit,
      icon: IconEdit,
      action: () => {
        console.log('Edit:', selectedIndex.value);
      }
    },
    {
      label: localeMsg.value.menu_item_copy,
      icon: IconCopy,
      shortcut: 'Ctrl+C',
      action: () => {
        console.log('Copy:', selectedIndex.value);
      }
    },
    {
      label: localeMsg.value.menu_item_rename,
      icon: IconRename,
      shortcut: 'F2',
      action: () => {
        console.log('Rename:', selectedIndex.value);
      }
    },
    {
      label: localeMsg.value.menu_item_delete,
      icon: IconDelete,
      shortcut: 'Del',
      action: () => {
        deleteItem(selectedIndex.value);
      }
    },
    {
      label: localeMsg.value.menu_item_properties,
      action: () => {
        console.log('Show properties:', selectedIndex.value);
      }
    },
    {
      label: "-",   // separator
      action: () => {}
    },
    {
      label: file.is_favorite ? localeMsg.value.menu_item_unfavorite : localeMsg.value.menu_item_favorite,
      icon: file.is_favorite ? IconUnFavorite : IconFavorite,
      shortcut: 'F',
      action: () => {
        toggleFavorite();
      }
    },
    {
      label: localeMsg.value.menu_item_rotate,
      icon: IconRotate,
      shortcut: 'R',
      action: () => {
        rotateImage();
      }
    },
    {
      label: "-",   // separator
      action: null
    },
    {
      label: localeMsg.value.menu_item_move_to,
      icon: IconMoveTo,
      action: () => {
        console.log('Move to:', selectedIndex.value);
      }
    },
    {
      label: localeMsg.value.menu_item_copy_to,
      // icon: IconCopyTo,
      action: () => {
        console.log('Copy to:', selectedIndex.value);
      }
    },
    {
      label: localeMsg.value.menu_item_open_folder,
      icon: IconOpenFolder,
      action: () => {
        openFileExplorer(getFolderPath(file.file_path));
      }
    },
  ];
});

function clickItem(index: number) {
  selectedIndex.value = index;
  if(props.selectMode) {
    selectItem(index);
  }
}

function selectItem(index: number) {
  if (props.fileList[index].isSelected) {
    props.fileList[index].isSelected = false;
  } else {
    props.fileList[index].isSelected = true;
  }
}

function handleKeyDown(event) {
  const key = event.key.toLowerCase(); // Convert key to lowercase
  if (keyActions[key]) {
    event.preventDefault(); // Prevent the default action
    keyActions[key](); 
  }
}

// function to get the text for the thumbnail
const getThumbnailText = (file, option) => {
  switch (option) {
    case 0:   // empty
      return '';
    case 1:   // name
      return shortenFilename(file.name);
    case 2:   // size
      return formatFileSize(file.size);
    case 3:   // resolution
      return `${file.width}x${file.height}`;
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
  f: ()          => toggleFavorite(),
  r: ()          => rotateImage(),
  F2: ()         => console.log('Rename:', selectedIndex.value),
  Delete: ()     => deleteItem(),
};

function toggleFavorite() {
  if (selectedIndex.value < 0 || selectedIndex.value >= props.fileList.length) {
    return;
  }
  props.fileList[selectedIndex.value].is_favorite = !props.fileList[selectedIndex.value].is_favorite;
};

function rotateImage() {
  if (selectedIndex.value < 0 || selectedIndex.value >= props.fileList.length) {
    return;
  }
  props.fileList[selectedIndex.value].rotate += 90;
};

// open the selected item in the image viewer
function openItem(openNewViewer = false) {
  if (openNewViewer) {
    emit('message-from-grid-view', { message: 'open-image-viewer' });
  } else {
    emit('message-from-grid-view', { message: 'update-image-viewer' });
  }
};

// delete the selected item
function deleteItem() {
  if (selectedIndex.value < 0 || selectedIndex.value >= props.fileList.length) {
    return;
  }
  props.fileList.splice(selectedIndex.value, 1);
  selectedIndex.value = Math.min(selectedIndex.value, props.fileList.length - 1);
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
</style>
