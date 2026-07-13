<template>
    
  <div class="sidebar-panel">
    <!-- title bar -->
    <div class="sidebar-panel-header" data-tauri-drag-region>
      <span class="sidebar-panel-header-title flex-1">{{ titlebar }}</span>
      <ContextMenu :menuItems="searchPanelMenuItems" :iconMenu="IconMore" :smallIcon="true" />
    </div>

    <div class="mx-1 mb-2 px-1 shrink-0">
      <div
        :class="[
          'h-8 flex items-center rounded-box transition-colors bg-base-100/40',
          isSearchFocused ? 'border-2 border-primary' : 'border border-base-content/10 hover:border-base-content/30',
        ]"
        @click="focusSearchInput"
      >
        <IconSearch
          class="ml-2 w-4 h-4 shrink-0"
          :class="isSearchFocused ? 'text-primary/70' : 'text-base-content/30'"
        />
        <input 
          ref="searchInputRef"
          type="text"
          v-model="searchQuery"
          :placeholder="$t('search.image_search_placeholder')"
          class="w-full min-w-0 bg-transparent border-none focus:ring-0 px-2 text-sm placeholder-base-content/30 focus:outline-none"
          maxlength="255"
          @focus="isSearchFocused = true"
          @blur="isSearchFocused = false"
          @keydown.enter = "handleSearch()"
          @keydown.esc = "handleEscKey()"
        >
        <button
          v-if="searchQuery"
          type="button"
          class="mr-1 p-1 rounded-box text-base-content/30 hover:text-base-content/70"
          @click.stop="searchQuery = ''; focusSearchInput()"
        >
          <IconClose class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- search history -->
    <div class="overflow-y-auto flex-1" >
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
            <IconSearch class="w-5 h-5 text-base-content/30" />
          </div>
          
          <span class="sidebar-item-label">{{ typeof item === 'string' ? item : item.text }}</span>
          <div class="ml-auto">
            <span
              v-if="hasSearchHistoryCount(item)"
              class="sidebar-item-count"
              :class="libConfig.search.searchHistoryIndex === index ? 'hidden' : 'group-hover:hidden'"
            >{{ formatSearchResultCount(getSearchHistoryCount(item)) }}</span>
          </div>
          <div
            :class="[
              'flex items-center text-base-content/30',
              libConfig.search.searchHistoryIndex === index ? '' : 'hidden group-hover:flex'
            ]"
          >
            <ContextMenu
              :iconMenu="IconMore"
              :menuItems="() => getSearchHistoryMenuItems(index)"
              :smallIcon="true"
            />
          </div>
        </div>  
    </div>

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

import { IconMore, IconTrash, IconSearch, IconClose } from '@/common/icons';
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

watch(
  () => libConfig.search.searchHistoryIndex,
  () => {
    syncSearchState();
  }
);

function focusSearchInput() {
  searchInputRef.value?.focus();
}

onMounted(() => {
  syncSearchState();
  nextTick(() => focusSearchInput());
});

onUnmounted(() => {
  uiStore.removeInputHandler('ImageSearch');
});

function handleSearchHistoryClick(index: number) {
  const item = libConfig.search.searchHistory[index];
  const searchText = typeof item === 'string' ? item : item?.text;
  if (searchText) {
    uiStore.searchCountRequestedFor = searchText;
    uiStore.searchCountRequestTick++;
  }
  libConfig.search.searchHistoryIndex = index;
}

function clearHistory() {
  libConfig.search.searchText = '';
  libConfig.search.searchHistory = [];
  libConfig.search.searchHistoryIndex = -1;

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
  uiStore.searchCountRequestedFor = query;
  uiStore.searchCountRequestTick++;
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
    history.unshift({ text: query, fileId: null, count: null });
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

const historyItems = ref<Record<number, any>>({});
const thumbnails = ref<Record<number, string>>({});
const searchHistoryList = computed(() => libConfig.search.searchHistory as any[]);
const getSearchHistoryCount = (item: any) => Number(item?.count || 0);
const hasSearchHistoryCount = (item: any) => typeof item !== 'string' && item?.count !== null && item?.count !== undefined;
const formatSearchResultCount = (count: number) => {
  const limit = Number(config.settings.imageSearch.limit || 0);
  return limit > 0 && count >= limit
    ? `${limit.toLocaleString()}+`
    : count.toLocaleString();
};

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

function showClearConfirmation() {
  showClearHistoryMsgbox.value = true;
}

defineExpose({
  clearHistory,
  showClearConfirmation,
  focusSearchInput,
});

</script>
