<template>
    
  <div class="w-full h-full flex flex-col select-none">

    <!-- title bar -->
    <div class="p-1 h-12 flex items-start justify-end whitespace-nowrap" data-tauri-drag-region>
      <!-- <span class="pl-1 cursor-default" data-tauri-drag-region>{{ titlebar }}</span> -->
      <div role="tablist" class="tabs-sm tabs-border" >
        <a 
          role="tab"
          class="tab"
          :class="config.search.searchType === 0 ? 'tab-active' : ''" 
          @click="handleTabClick(0)"
        >
          {{ $t('search.search_images') }}
        </a>
        <a 
          role="tab"
          class="tab"
          :class="config.search.searchType === 1 ? 'tab-active' : ''" 
          @click="handleTabClick(1)"
        >
          {{ $t('search.similar_images') }}
        </a>
        <a 
          role="tab"
          class="tab"
          :class="config.search.searchType === 2 ? 'tab-active' : ''" 
          @click="handleTabClick(2)"
        >
          {{ $t('search.filename_search') }}
        </a>
      </div>
      <ContextMenu 
        :iconMenu="IconMore" 
        :menuItems="moreMenuItems"
      />
    </div>

    <!-- 0: search text -->
    <template v-if="config.search.searchType === 0">
      <div
        :class="[ 
          'p-1 h-10 flex items-center rounded-box whitespace-nowrap cursor-pointer group relative',
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
        <div v-if="config.search.searchHistory.length === 0" class="m-2 flex flex-col items-center justify-center text-sm text-base-content/30">
          <span class="text-center">{{ $t('search.image_search_tips') }}</span>
        </div>  

        <div v-for="(item, index) in searchHistoryList" :key="index"
          class="mx-1 p-1 text-sm rounded-box flex items-center"
          :class="[ 
            'h-12 flex items-center rounded-box whitespace-nowrap cursor-pointer group', 
            config.search.searchHistoryIndex === index ? 'text-primary bg-base-100 hover:bg-base-100' : 'hover:text-base-content hover:bg-base-100/70',
          ]"
          @click="handleSearchHistoryClick(index, item)"
        >
          <div v-if="typeof item !== 'string' && item.file_id" class="relative w-10 h-10 mr-2 shrink-0 overflow-hidden rounded-box">
             <img 
               v-if="thumbnails[item.file_id]"
               class="w-full h-full object-cover" 
               :src="thumbnails[item.file_id]" 
             />
             <div v-else class="w-full h-full bg-base-300 animate-pulse"></div>
          </div>
          <IconDot v-else class="w-4 h-4 mr-1 shrink-0" />
          
          <span class="overflow-hidden whitespace-pre text-ellipsis">{{ typeof item === 'string' ? item : item.text }}</span>
          <ContextMenu
            :class="[
              'ml-auto flex flex-row items-center text-base-content/30',
              config.search.searchHistoryIndex != index ? 'invisible group-hover:visible' : ''
            ]"
            :iconMenu="IconMore"
            :menuItems="searchHistoryMenuItems"
            :smallIcon="true"
          />
        </div>  
      </div>

    </template>

    <!-- 1: similar images -->
    <template v-else-if="config.search.searchType === 1">
      <div class="overflow-x-hidden overflow-y-auto flex-1">
        <div v-if="similarImageHistory.length === 0" class="m-2 flex items-center justify-center text-sm text-base-content/30">
          <span class="text-center">{{ $t('search.similar_images_tips') }}</span>
        </div>
        
        <div v-for="(fileId, index) in similarImageHistory" :key="index"
          class="mx-1 p-1 text-sm rounded-box flex items-center"
          :class="[ 
            'h-12 flex items-center gap-2 rounded-box whitespace-nowrap cursor-pointer group', 
            config.search.similarImageHistoryIndex === index ? 'text-primary bg-base-100 hover:bg-base-100' : 'hover:text-base-content hover:bg-base-100/70',
          ]"
          @click="handleSimilarHistoryClick(index, fileId)"
        >
          <div class="relative w-10 h-10 shrink-0 overflow-hidden rounded-box">
             <img 
               v-if="thumbnails[fileId]"
               class="w-full h-full object-cover" 
               :src="thumbnails[fileId]" 
             />
             <div v-else class="w-full h-full bg-base-300 animate-pulse"></div>
          </div>
          <div class="flex-1 flex flex-col justify-center overflow-hidden">
             <span class="font-medium truncate">{{ historyItems[fileId]?.name || $t('tooltip.loading') }}</span>
             <!-- <span class="text-xs opacity-70 truncate">{{ historyItems[fileId]?.file_path }}</span> -->
          </div>
          <ContextMenu
            :class="[
              'ml-auto flex flex-row items-center text-base-content/30',
              config.search.similarImageHistoryIndex != index ? 'invisible group-hover:visible' : ''
            ]"
            :iconMenu="IconMore"
            :menuItems="getSimilarHistoryMenuItems(index)"
            :smallIcon="true"
          />
        </div>
      </div>
    </template>

    <!-- 2: filename search -->
    <template v-else-if="config.search.searchType === 2">
      <div
        :class="[ 
          'p-1 h-10 flex items-center rounded-box whitespace-nowrap cursor-pointer group relative',
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
          v-model="config.search.fileName"
          :placeholder="$t('search.filename_search_tips')"
          :class="[
            'pl-8 w-full input bg-transparent rounded-box',
            isSearchFocused ? 'border-primary' : 'border-base-content/30 group-hover:border-base-content/70 cursor-pointer',
          ]"
          maxlength="255"
          @focus="isSearchFocused = true"
          @keydown.enter = "handleEscKey()"
          @keydown.esc = "handleEscKey()"
        >
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

function syncSearchState() {
  if (config.search.searchType === 0) {
    if (config.search.searchHistoryIndex === -1) {
      searchQuery.value = '';
      config.search.searchText = '';
      nextTick(() => {
        focusSearchInput();
      });
    } else {
      const history = config.search.searchHistory as any[];
      const item = history[config.search.searchHistoryIndex];
      if (item) {
        // Handle both string and object formats
        const text = typeof item === 'string' ? item : item.text;
        searchQuery.value = text;
        config.search.searchText = text;
      }
    }
  } else if (config.search.searchType === 2) {
    nextTick(() => {
      focusSearchInput();
    });
  }
}

watch(
  () => config.search.searchHistoryIndex,
  () => {
    syncSearchState();
  }
);

function focusSearchInput() {
  searchInputRef.value?.focus();
}

function handleTabClick(type: number) {
  config.search.searchType = type;
  syncSearchState();
}

let unlistenKeydown: () => void;

onMounted(async () => {
  unlistenKeydown = await listen('global-keydown', handleKeyDown);
  syncSearchState();
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

function handleSearchHistoryClick(index: number, item: any) {
  isSearchFocused.value = true;
  config.search.searchHistoryIndex = index;
  // watcher will sync searchQuery and searchText
}

function clearHistory() {
  if(config.search.searchType === 0) {
    config.search.searchText = '';
    config.search.searchHistory = [];
    config.search.searchHistoryIndex = -1;
  } else if (config.search.searchType === 1) {
    config.search.similarImageHistory = [];
    config.search.similarImageHistoryIndex = -1;
  } else if (config.search.searchType === 2) {
    config.search.fileName = '';
  }

  showClearHistoryMsgbox.value = false;
}

function deleteHistoryItem() {
  config.search.searchHistory.splice(config.search.searchHistoryIndex, 1);
  config.search.searchHistoryIndex = -1;
}

function handleSearch() {
  if (searchQuery.value.trim().length === 0) return;
  
  const query = searchQuery.value.trim();
  const history = config.search.searchHistory as any[];
  
  // Find existing index considering both string and object formats
  const existingIndex = history.findIndex((item: any) => {
    const text = typeof item === 'string' ? item : item.text;
    return text === query;
  });
  
  if (existingIndex !== -1) {
    config.search.searchHistoryIndex = existingIndex;
  } else {
    // Add new item as object
    history.unshift({ text: query, file_id: null });
    config.search.searchHistoryIndex = 0;

    // Limit the history size
    if (history.length > config.search.maxSearchHistory) {
      history.pop();
    }
  }

  config.search.searchText = query;
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
const thumbnails = ref<Record<number, string>>({}); // Shared for both now? Or we should check if we need separate. 
// Ideally we can share the thumbnails cache by ID. 
// But let's check how usage differs. 
// 'thumbnails' is currently keyed by fileId. So it can be shared!

const similarImageHistory = computed(() => config.search.similarImageHistory as number[]);
const searchHistory = computed(() => config.search.searchHistory);
const searchHistoryList = computed(() => config.search.searchHistory as any[]);

const emit = defineEmits(['search-similar-from-history']);


// Watcher for Similar Image History
watch(
  () => config.search.similarImageHistory,
  (newHistory) => {
    const history = newHistory as number[]; 
    fetchThumbnailsForIds(history);
  },
  { immediate: true, deep: true }
);

// Watcher for Text Search History (to fetch thumbnails)
watch(
  () => config.search.searchHistory,
  (newHistory) => {
    const idsToFetch = newHistory
      .filter(item => typeof item !== 'string' && item.file_id)
      .map(item => (item as any).file_id);
    fetchThumbnailsForIds(idsToFetch);
  },
  { immediate: true, deep: true }
);

async function fetchThumbnailsForIds(ids: number[]) {
  for (const fileId of ids) {
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
        const thumb = await getFileThumb(file.id, file.file_path, file.file_type || 1, file.e_orientation || 0, config.settings.thumbnailSize, false);
        if (thumb && thumb.error_code === 0) {
          thumbnails.value[file.id] = `data:image/jpeg;base64,${thumb.thumb_data_base64}`;
        }
      } catch (e) {
        console.error('Failed to load thumbnail for history item', fileId, e);
      }
    }
  }
}

function handleSimilarHistoryClick(index: number, fileId: number) {
  if(config.search.similarImageHistoryIndex === index) {
    return;
  }
  config.search.similarImageHistoryIndex = index;
  
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
  (config.search.similarImageHistory as any[]).splice(index, 1);
  if (config.search.similarImageHistoryIndex === index) {
    config.search.similarImageHistoryIndex = -1;
  } else if (config.search.similarImageHistoryIndex > index) {
    config.search.similarImageHistoryIndex--;
  }
}

</script>
