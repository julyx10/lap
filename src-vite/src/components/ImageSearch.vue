<template>
    
  <div class="w-full h-full flex flex-col select-none">

    <!-- title bar -->
    <div class="p-1 h-12 flex items-start justify-end whitespace-nowrap" data-tauri-drag-region>
      <!-- <span class="pl-1 cursor-default" data-tauri-drag-region>{{ titlebar }}</span> -->
      <div role="tablist" class="tabs-sm tabs-border" >
        <a 
          role="tab"
          class="tab"
          :class="config.imageSearch.searchType === 0 ? 'tab-active' : ''" 
          @click="config.imageSearch.searchType = 0"
        >
          {{ $t('search.search_images') }}
        </a>
        <a 
          role="tab"
          class="tab"
          :class="config.imageSearch.searchType === 1 ? 'tab-active' : ''" 
          @click="config.imageSearch.searchType = 1"
        >
          {{ $t('search.similar_images') }}
        </a>
      </div>
      <ContextMenu 
        :iconMenu="IconMore" 
        :menuItems="moreMenuItems"
      />
    </div>

    <!-- search text -->
    <template v-if="config.imageSearch.searchType === 0">
      <div
        :class="[ 
          'mx-1 p-1 h-10 flex items-center rounded-box whitespace-nowrap cursor-pointer group relative',
          isSearchFocused ? 'text-base-content/70' : 'text-base-content/30 hover:text-base-content/70 hover:bg-base-100',
        ]"
        @click="focusSearchInput"
      >
        <IconSearch 
          :class="[
            'absolute left-2 mx-1 top-1/2 transform -translate-y-1/2 w-4 h-4 cursor-pointer rounded-box z-10',
            isSearchFocused ? 'text-primary group-hover:text-primary' : 'text-base-content/30 group-hover:text-base-content/70' 
          ]"
        />
        <input 
          ref="searchInputRef"
          type="text"
          v-model="searchQuery"
          :placeholder="$t('search.image_search_placeholder')"
          :class="[
            'pl-8 w-full input bg-transparent rounded-box',
            isSearchFocused ? 'border-primary' : 'border-base-content/30 group-hover:border-base-content/70 cursor-pointer',
          ]"
          maxlength="255"
          @focus="isSearchFocused = true"
          @keydown.enter = "handleSearch()"
          @keydown.esc = "handleEscKey()"
        >
      </div>

      <!-- search history -->
      <div class="overflow-y-auto flex-1" >
        <div v-if="config.imageSearch.searchHistory.length === 0" class="px-2 mt-2 text-sm text-base-content/30">
          {{ $t('search.image_search_tips') }}
        </div>  

        <div v-for="(item, index) in config.imageSearch.searchHistory" :key="index"
          class="mx-2 p-2 text-sm rounded-box flex items-center"
          :class="[ 
            'mx-1 p-1 h-8 flex items-center rounded-box whitespace-nowrap cursor-pointer hover:bg-base-100 group', 
            config.imageSearch.searchHistoryIndex === index ? 'text-primary bg-base-100' : 'hover:text-base-content',
          ]"
          @click="handleSearchHistoryClick(index, item)"
        >
          <IconDot class="w-4 h-4 mr-1 shrink-0" />
          <span class="overflow-hidden whitespace-pre text-ellipsis">{{ item }}</span>
          <ContextMenu
            :class="[
              'ml-auto flex flex-row items-center text-base-content/30',
              config.imageSearch.searchHistoryIndex != index ? 'invisible group-hover:visible' : ''
            ]"
            :iconMenu="IconMore"
            :menuItems="searchHistoryMenuItems"
            :smallIcon="true"
          />
        </div>  
      </div>

    </template>

    <!-- similar images -->
    <template v-else>
      <div class="overflow-x-hidden overflow-y-auto flex-1">
        <div v-if="similarImageHistory.length === 0" class="px-2 mt-2 text-sm text-base-content/30">
          {{ $t('search.similar_images_tips') }}
        </div>
        
        <div v-for="(fileId, index) in similarImageHistory" :key="index"
          class="mx-2 p-2 text-sm rounded-box flex items-center"
          :class="[ 
            'mx-1 p-1 h-16 flex items-center gap-2 rounded-box whitespace-nowrap cursor-pointer hover:bg-base-100 group', 
            config.imageSearch.similarImageHistoryIndex === index ? 'text-primary bg-base-100' : 'hover:text-base-content',
          ]"
          @click="handleSimilarHistoryClick(index, fileId)"
        >
          <img :src="thumbnails[fileId]" 
            class="w-12 h-12 object-cover rounded-box shrink-0" 
            loading="lazy" 
          />
          <span class="overflow-hidden whitespace-pre text-ellipsis">{{ historyItems[fileId]?.name }}</span>
          <ContextMenu
            :class="[
              'ml-auto flex flex-row items-center text-base-content/30',
              config.imageSearch.similarImageHistoryIndex != index ? 'invisible group-hover:visible' : ''
            ]"
            :iconMenu="IconMore"
            :menuItems="getSimilarHistoryMenuItems(index)"
            :smallIcon="true"
          />
        </div>
      </div>
    </template>
  </div>

  <!-- clear history messagebox -->
  <MessageBox
    v-if="showClearHistoryMsgbox"
    :title="$t('msgbox.clear_search_history.title')"
    :message="`${$t('msgbox.clear_search_history.content')}`"
    :OkText="$t('msgbox.clear_search_history.ok')"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    @ok="clearHistory()"
    @cancel="showClearHistoryMsgbox = false"
  />

</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { listen, emit as tauriEmit } from '@tauri-apps/api/event';
import { useUIStore } from '@/stores/uiStore';
import MessageBox from '@/components/MessageBox.vue';
import { getFileThumb, getFileInfo } from '@/common/api';

import { IconMore, IconTrash, IconSearch, IconDot } from '@/common/icons';
import ContextMenu from '@/components/ContextMenu.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();

const showClearHistoryMsgbox = ref(false);

const searchHistoryMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu.home.delete,
      icon: IconTrash,
      action: () => {
        deleteHistoryItem();
      }
    },
  ];
});

// search query
const searchInputRef = ref<HTMLInputElement | null>(null);
const searchQuery = ref('');
const isSearchFocused = ref(false);

function focusSearchInput() {
  searchInputRef.value?.focus();
  config.imageSearch.searchHistoryIndex = -1;
}

let unlistenKeydown: () => void;

onMounted(async () => {
  unlistenKeydown = await listen('global-keydown', handleKeyDown);
});

onUnmounted(() => {
  if (unlistenKeydown) {
    unlistenKeydown();
  }
  uiStore.popInputHandler();
});

// more menuitems
const moreMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu.home.clear_history,
      icon: IconTrash,
      action: () => {
        showClearHistoryMsgbox.value = true;
      }
    },
  ];
});

function handleSearchHistoryClick(index: number, item: string) {
  isSearchFocused.value = true;
  config.imageSearch.searchHistoryIndex = index;
  
  searchQuery.value = item;
  config.imageSearch.searchText = item;
}

function clearHistory() {
  if(config.imageSearch.searchType === 0) {
    config.imageSearch.searchHistory = [];
    config.imageSearch.searchHistoryIndex = -1;
    config.imageSearch.searchText = '';
  } else {
    config.imageSearch.similarImageHistory = [];
    config.imageSearch.similarImageHistoryIndex = -1;
  }

  showClearHistoryMsgbox.value = false;
}

function deleteHistoryItem() {
  config.imageSearch.searchHistory.splice(config.imageSearch.searchHistoryIndex, 1);
  config.imageSearch.searchHistoryIndex = -1;
}

function handleSearch() {
  if (searchQuery.value.trim().length === 0) return;
  
  const query = searchQuery.value.trim();
  const existingIndex = config.imageSearch.searchHistory.indexOf(query);
  
  if (existingIndex !== -1) {
    config.imageSearch.searchHistoryIndex = existingIndex;
  } else {
    config.imageSearch.searchHistory.unshift(query);
    config.imageSearch.searchHistoryIndex = 0;

    // Limit the history size
    if (config.imageSearch.searchHistory.length > config.imageSearch.maxSearchHistory) {
      config.imageSearch.searchHistory.pop();
    }
  }

  config.imageSearch.searchText = query;
  searchQuery.value = '';
}

function handleEscKey() {
  searchInputRef.value?.blur();
}

function handleKeyDown(event: any) {
  if (event.payload.key === 'Escape') {
    console.log('handleKeyDown');
  }
};


// similar image search history
const historyItems = ref<Record<number, any>>({});
const thumbnails = ref<Record<number, string>>({});

const similarImageHistory = computed(() => config.imageSearch.similarImageHistory as number[]);
const emit = defineEmits(['search-similar-from-history']);

watch(
  () => config.imageSearch.similarImageHistory,
  async (newHistory) => {
    // We treat 'newHistory' as number[]
    const history = newHistory as number[]; 
    for (const fileId of history) {
      if (!fileId) continue;

      if (!historyItems.value[fileId]) {
        try {
           const info = await getFileInfo(fileId);
           if (info) {
             historyItems.value[fileId] = info;
           }
        } catch (e) {
          console.error('Failed to load file info', fileId, e);
        }
      }

      if (historyItems.value[fileId] && !thumbnails.value[fileId]) {
        try {
          const file = historyItems.value[fileId];
          // getFileThumb(fileId, filePath, fileType, orientation, thumbnailSize, forceRegenerate)
          const thumb = await getFileThumb(file.id, file.file_path, file.file_type || 1, file.e_orientation || 0, 200, false);
          if (thumb && thumb.error_code === 0) {
            thumbnails.value[file.id] = `data:image/jpeg;base64,${thumb.thumb_data_base64}`;
          }
        } catch (e) {
          console.error('Failed to load thumbnail for history item', fileId, e);
        }
      }
    }
  },
  { immediate: true, deep: true }
);

function handleSimilarHistoryClick(index: number, fileId: number) {
  if(config.imageSearch.similarImageHistoryIndex === index) {
    return;
  }
  config.imageSearch.similarImageHistoryIndex = index;
  
  if (historyItems.value[fileId]) {
    nextTick(() => {
      emit('search-similar-from-history', historyItems.value[fileId]);
    });
  }
}

function getSimilarHistoryMenuItems(index: number) {
  return [
    {
      label: localeMsg.value.menu.home.delete,
      icon: IconTrash,
      action: () => {
        deleteSimilarHistoryItem(index);
      }
    },
  ];
}

function deleteSimilarHistoryItem(index: number) {
  (config.imageSearch.similarImageHistory as any[]).splice(index, 1);
  if (config.imageSearch.similarImageHistoryIndex === index) {
    config.imageSearch.similarImageHistoryIndex = -1;
  } else if (config.imageSearch.similarImageHistoryIndex > index) {
    config.imageSearch.similarImageHistoryIndex--;
  }
}

</script>
