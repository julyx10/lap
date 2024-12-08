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
        <div class="flex flex-col items-center">
          <img v-if="file.thumbnail"
            :src="file.thumbnail" 
            :class="[
              'rounded transition duration-200', 
              isFitWidth ? 'object-cover' : 'object-contain'
            ]"
            :style="{ width: `${gridSize}px`, height: `${gridSize}px` }"
            loading="lazy"
          />
          <div v-else 
            class="rounded flex items-center justify-center"
            :style="{ width: `${gridSize}px`, height: `${gridSize}px` }"
          >
            <!-- <IconFolder class="size-1/2"/> -->
            <IconPhoto class="size-1/2"/>
          </div>
          <p class="pt-1 text-sm text-center">{{ shortenFilename(file.name) }}</p>
          <p class="text-sm">{{ file.width }}x{{ file.height }}</p>
          <!-- <p class="text-sm">{{ file.e_model }}</p> -->
          <!-- <p class="text-sm">{{ formatFileSize(file.size) }}</p> -->
        </div>
      </div>

    </div>
  </div>
</template>


<script setup lang="ts">

import { ref, watch, onMounted, onUnmounted } from 'vue';
import { emit } from '@tauri-apps/api/event';
import { shortenFilename, formatFileSize } from '@/common/utils';

import IconPhoto from '@/assets/photo.svg';

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
  isFitWidth: Boolean,
});

const selectedIndex = ref(props.modelValue);
const emitUpdate = defineEmits(['update:modelValue']);

const scrollable = ref(null); // Ref for the scrollable element

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

watch(() => props.modelValue, (newValue) => { 
  selectedIndex.value = newValue; 
});

watch(() => props.fileList, (newList) => {
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

const keyActions = {
  ArrowDown: ()  => selectedIndex.value = Math.min(selectedIndex.value + getColumnCount(), props.fileList.length - 1),
  ArrowRight: () => selectedIndex.value = Math.min(selectedIndex.value + 1, props.fileList.length - 1),
  ArrowUp: ()    => selectedIndex.value = Math.max(selectedIndex.value - getColumnCount(), 0),
  ArrowLeft: ()  => selectedIndex.value = Math.max(selectedIndex.value - 1, 0),
  Home: ()       => selectedIndex.value = 0,
  End: ()        => selectedIndex.value = props.fileList.length - 1,
  Enter: ()      => openItem(true),
};

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
