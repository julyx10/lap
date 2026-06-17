<template>
    
  <div class="sidebar-panel">
    <!-- title bar -->
    <div class="sidebar-panel-header" data-tauri-drag-region>
      <div role="tablist" class="sidebar-header-tabs">
        <a
          role="tab"
          :class="['sidebar-header-tab', { 'tab-active': libConfig.search.searchType === 0 }]"
          @click="handleTabClick(0)"
        >
          {{ $t('search.search_images') }}
        </a>
        <a
          role="tab"
          :class="['sidebar-header-tab', { 'tab-active': libConfig.search.searchType === 1 }]"
          @click="handleTabClick(1)"
        >
          {{ $t('search.similar_images') }}
        </a>
        <a
          role="tab"
          :class="['sidebar-header-tab', { 'tab-active': libConfig.search.searchType === 2 }]"
          @click="handleTabClick(2)"
        >
          {{ $t('search.filename_search') }}
        </a>
      </div>
      <ContextMenu :menuItems="searchPanelMenuItems" :iconMenu="IconMore" :smallIcon="true" />
    </div>

    <!-- 0: search text -->
    <template v-if="libConfig.search.searchType === 0">
      <div
        :class="[ 
          'mb-1 p-1 h-10 flex items-center rounded-box whitespace-nowrap cursor-pointer group relative',
          isSearchFocused ? 'text-base-content/70' : 'text-base-content/30 hover:text-base-content/70 hover:bg-base-100',
        ]"
        @click="focusSearchInput"
      >
        <IconSearch 
          :class="[
            'absolute left-2 ml-1 top-1/2 transform -translate-y-1/2 w-4 h-4 cursor-pointer rounded-box z-10',
            isSearchFocused ? 'text-primary group-hover:text-primary' : 'text-base-content/30 group-hover:text-base-content/70' 
          ]"
        />
        <input 
          ref="searchInputRef"
          type="text"
          v-model="searchQuery"
          :placeholder="$t('search.image_search_placeholder')"
          :class="[
            'pl-7 pr-7 w-full input bg-transparent rounded-box',
            isSearchFocused ? 'border-primary' : 'border-base-content/30 group-hover:border-base-content/70 cursor-pointer',
          ]"
          maxlength="255"
          @focus="isSearchFocused = true"
          @keydown.enter = "handleSearch()"
          @keydown.esc = "handleEscKey()"
        >
        <IconClose 
           v-if="searchQuery"
          :class="[
            'absolute right-2 mr-1 top-1/2 transform -translate-y-1/2 w-4 h-4 cursor-pointer rounded-box z-10',
            isSearchFocused ? 'text-primary group-hover:text-primary' : 'text-base-content/30 group-hover:text-base-content/70' 
          ]"
          @click.stop="searchQuery = ''; focusSearchInput()"
        />
      </div>

      <!-- search history -->
      <div class="overflow-y-auto flex-1" >
        <div v-if="libConfig.search.searchHistory.length === 0" class="sidebar-empty text-sm">
          <span class="text-center">{{ $t('search.image_search_tips') }}</span>
        </div>  

        <div v-for="(item, index) in searchHistoryList" :key="index"
          :class="[ 
            'sidebar-item sidebar-item-media text-sm group',
            libConfig.search.searchHistoryIndex === index ? 'sidebar-item-selected' : 'hover:text-base-content hover:bg-base-100/70',
          ]"
          @click="handleSearchHistoryClick(index)"
        >
        <div v-if="typeof item !== 'string' && item.fileId" class="relative w-10 h-10 mr-2 shrink-0 overflow-hidden rounded-box">
           <img 
             v-if="thumbnails[item.fileId]"
             class="w-full h-full object-cover" 
             :src="thumbnails[item.fileId]" 
           />
           <div v-else class="w-full h-full bg-base-300 animate-pulse"></div>
        </div>
          <div
            v-else
            class="w-10 h-10 mr-2 shrink-0 rounded-box border border-base-content/5 flex items-center justify-center"
          >
            <IconError class="w-5 h-5 text-base-content/30" />
          </div>
          
	          <span class="sidebar-item-label">{{ typeof item === 'string' ? item : item.text }}</span>
	          <ContextMenu
	            :class="[
	              'ml-auto flex flex-row items-center text-base-content/30',
	              libConfig.search.searchHistoryIndex != index ? 'invisible group-hover:visible' : ''
	            ]"
	            :iconMenu="IconMore"
	            :menuItems="() => getSearchHistoryMenuItems(index)"
	            :smallIcon="true"
	          />
        </div>  
      </div>

    </template>

    <!-- 1: similar images -->
    <template v-else-if="libConfig.search.searchType === 1">
      <div class="overflow-x-hidden overflow-y-auto flex-1">
        <div v-if="similarImageHistory.length === 0" class="sidebar-empty text-sm">
          <span class="text-center">{{ $t('search.similar_images_tips') }}</span>
        </div>
        
        <div v-for="(fileId, index) in similarImageHistory" :key="index"
          :class="[ 
            'sidebar-item sidebar-item-media text-sm gap-2 group',
            libConfig.search.similarImageHistoryIndex === index ? 'sidebar-item-selected' : 'hover:text-base-content hover:bg-base-100/70',
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
              libConfig.search.similarImageHistoryIndex != index ? 'invisible group-hover:visible' : ''
            ]"
            :iconMenu="IconMore"
            :menuItems="getSimilarHistoryMenuItems(index)"
            :smallIcon="true"
          />
        </div>
      </div>
    </template>

    <!-- 2: filename search -->
    <template v-else-if="libConfig.search.searchType === 2">
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
          v-model="libConfig.search.fileName"
          :placeholder="$t('search.filename_search_tips')"
          :class="[
            'pl-7 pr-7 w-full input bg-transparent rounded-box',
            isSearchFocused ? 'border-primary' : 'border-base-content/30 group-hover:border-base-content/70 cursor-pointer',
          ]"
          maxlength="255"
          @focus="isSearchFocused = true"
          @keydown.enter = "handleEscKey()"
          @keydown.esc = "handleEscKey()"
        >
        <IconClose 
           v-if="libConfig.search.fileName"
          :class="[
            'absolute right-2 mr-1 top-1/2 transform -translate-y-1/2 w-4 h-4 cursor-pointer rounded-box z-10',
            isSearchFocused ? 'text-primary group-hover:text-primary' : 'text-base-content/30 group-hover:text-base-content/70' 
          ]"
          @click.stop="libConfig.search.fileName = ''; focusSearchInput()"
        />
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
import { config, libConfig } from '@/common/config';
import { useUIStore } from '@/stores/uiStore';
import MessageBox from '@/components/MessageBox.vue';
import { getFileInfo, getFileThumbById } from '@/common/api';
import {
  getThumbUrl,
  getThumbnailDataUrl,
  getThumbnailDataUrlInflight,
  isWin,
  setThumbnailDataUrlInflight,
} from '@/common/utils';

import { IconMore, IconTrash, IconSearch, IconDot, IconClose, IconError } from '@/common/icons';
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

const searchPanelMenuItems = computed(() => [
  {
    label: localeMsg.value.menu.search.clear_history,
    icon: IconTrash,
    action: () => showClearConfirmation(),
  },
]);

function getSearchHistoryMenuItems(index: number) {
  return [
    {
      label: localeMsg.value.menu.home.delete,
      icon: IconTrash,
      action: () => {
        deleteHistoryItem(index);
      }
    },
  ];
}

// search query
const searchInputRef = ref<HTMLInputElement | null>(null);
const searchQuery = ref('');
const isSearchFocused = ref(false);

function syncSearchState() {
  if (libConfig.search.searchType === 0) {
    if (libConfig.search.searchHistoryIndex !== -1) {
      const history = libConfig.search.searchHistory as any[];
      const item = history[libConfig.search.searchHistoryIndex];
      if (item) {
        const text = typeof item === 'string' ? item : item.text;
        libConfig.search.searchText = text;
        searchQuery.value = text;
      }
    } else {
      searchQuery.value = libConfig.search.searchText || '';
    }
  }
}

watch(
  () => libConfig.search.searchHistoryIndex,
  () => {
    syncSearchState();
  }
);

function focusSearchInput() {
  searchInputRef.value?.focus();
}

function handleTabClick(type: number) {
  libConfig.search.searchType = type;
  syncSearchState();
  nextTick(() => focusSearchInput());
}

onMounted(() => {
  syncSearchState();
  nextTick(() => focusSearchInput());
});

onUnmounted(() => {
  uiStore.removeInputHandler('ImageSearch');
});

function handleSearchHistoryClick(index: number) {
  isSearchFocused.value = true;
  libConfig.search.searchHistoryIndex = index;
  nextTick(() => focusSearchInput());
}

function clearHistory() {
  if(libConfig.search.searchType === 0) {
    libConfig.search.searchText = '';
    libConfig.search.searchHistory = [];
    libConfig.search.searchHistoryIndex = -1;
  } else if (libConfig.search.searchType === 1) {
    libConfig.search.similarImageHistory = [];
    libConfig.search.similarImageHistoryIndex = -1;
  } else if (libConfig.search.searchType === 2) {
    libConfig.search.fileName = '';
  }

  showClearHistoryMsgbox.value = false;
}

function deleteHistoryItem(index: number) {
  if (index < 0 || index >= libConfig.search.searchHistory.length) return;
  const selectedIndex = libConfig.search.searchHistoryIndex;
  libConfig.search.searchHistory.splice(index, 1);
  if (selectedIndex === index) {
    libConfig.search.searchText = '';
    searchQuery.value = '';
    libConfig.search.searchHistoryIndex = -1;
  } else if (selectedIndex > index) {
    libConfig.search.searchHistoryIndex = selectedIndex - 1;
  }
}

function handleSearch() {
  if (searchQuery.value.trim().length === 0) return;
  
  const query = searchQuery.value.trim();
  const history = libConfig.search.searchHistory as any[];
  
  // Find existing index considering both string and object formats
  const existingIndex = history.findIndex((item: any) => {
    const text = typeof item === 'string' ? item : item.text;
    return text === query;
  });
  
  if (existingIndex !== -1) {
    libConfig.search.searchHistoryIndex = existingIndex;
  } else {
    // Add new item as object
    history.unshift({ text: query, fileId: null });
    libConfig.search.searchHistoryIndex = 0;

    // Limit the history size
    if (history.length > config.search.maxSearchHistory) {
      history.pop();
    }
  }

  libConfig.search.searchText = query;
}

function handleEscKey() {
  searchInputRef.value?.blur();
}

// similar image search history
const historyItems = ref<Record<number, any>>({});
const thumbnails = ref<Record<number, string>>({}); // Shared for both now? Or we should check if we need separate. 
// Ideally we can share the thumbnails cache by ID. 
// But let's check how usage differs. 
// 'thumbnails' is currently keyed by fileId. So it can be shared!

const similarImageHistory = computed(() => libConfig.search.similarImageHistory as number[]);
const searchHistory = computed(() => libConfig.search.searchHistory);
const searchHistoryList = computed(() => libConfig.search.searchHistory as any[]);

const emit = defineEmits(['search-similar-from-history', 'editDataChanged']);

// Watcher for Similar Image History
watch(
  () => libConfig.search.similarImageHistory,
  (newHistory) => {
    const history = newHistory as number[]; 
    fetchThumbnailsForIds(history);
  },
  { immediate: true, deep: true }
);

// Watcher for Text Search History (to fetch thumbnails)
watch(
  () => libConfig.search.searchHistory,
  (newHistory) => {
    const idsToFetch = newHistory
      .filter(item => typeof item !== 'string' && item.fileId)
      .map(item => (item as any).fileId);
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
      let thumbSrc = getThumbUrl(fileId, false, config.settings.thumbnailSize);
      if (isWin && !thumbSrc.startsWith('data:')) {
        const inflight = getThumbnailDataUrlInflight(fileId, config.settings.thumbnailSize);
        const dataUrl = await (inflight || setThumbnailDataUrlInflight(
          fileId,
          config.settings.thumbnailSize,
          getFileThumbById(fileId, config.settings.thumbnailSize, false)
            .then(thumb => getThumbnailDataUrl(thumb, '', false, config.settings.thumbnailSize))
        ));
        thumbSrc = dataUrl || thumbSrc;
      }
      thumbnails.value[fileId] = thumbSrc;
    }
  }
}

function handleSimilarHistoryClick(index: number, fileId: number) {
  if(libConfig.search.similarImageHistoryIndex === index) {
    return;
  }
  libConfig.search.similarImageHistoryIndex = index;
  
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
  (libConfig.search.similarImageHistory as any[]).splice(index, 1);
  if (libConfig.search.similarImageHistoryIndex === index) {
    libConfig.search.similarImageHistoryIndex = -1;
  } else if (libConfig.search.similarImageHistoryIndex > index) {
    libConfig.search.similarImageHistoryIndex--;
  }
}

function showClearConfirmation() {
  showClearHistoryMsgbox.value = true;
}

defineExpose({
  clearHistory,
  showClearConfirmation,
  focusSearchInput,
});

</script>
