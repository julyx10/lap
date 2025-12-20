<template>
    
  <div class="w-full h-full flex flex-col select-none">

    <!-- title bar -->
    <div class="p-1 h-12 flex items-start justify-end whitespace-nowrap" data-tauri-drag-region>
      <!-- <span class="pl-1 cursor-default" data-tauri-drag-region>{{ titlebar }}</span> -->
      <ContextMenu 
        :iconMenu="IconMore" 
        :menuItems="moreMenuItems"
      />
    </div>

    <!-- all files -->
    <div 
      :class="[ 
        'mx-1 p-1 h-10 flex items-center rounded-box whitespace-nowrap cursor-pointer hover:bg-base-100 group',
        config.home.optionIndex === 0 ? 'text-primary bg-base-100' : 'hover:text-base-content',
      ]"
      @click="config.home.optionIndex = 0; isSearchFocused = false; config.home.searchHistoryIndex = -1"
    >
      <IconHome class="mx-1 w-5 h-5 shrink-0" />
      <div class="overflow-hidden whitespace-pre text-ellipsis">
        {{ $t('home.all_files') }}
      </div>
    </div>

    <!-- on this day -->
    <div 
      :class="[ 
        'mx-1 p-1 h-10 flex items-center rounded-box whitespace-nowrap cursor-pointer hover:bg-base-100 group',
        config.home.optionIndex === 1 ? 'text-primary bg-base-100' : 'hover:text-base-content',
      ]"
      @click="config.home.optionIndex = 1; isSearchFocused = false; config.home.searchHistoryIndex = -1"
    >
      <IconHistory class="mx-1 w-5 h-5 shrink-0" />
      <div class="overflow-hidden whitespace-pre text-ellipsis">
        {{ $t('home.on_this_day') }}
      </div>
    </div>

    <!-- search similar images -->
    <div 
      :class="[ 
        'mx-1 p-1 h-10 flex items-center rounded-box whitespace-nowrap cursor-pointer hover:bg-base-100 group',
        config.home.optionIndex === 2 ? 'text-primary bg-base-100' : 'hover:text-base-content',
      ]"
      @click="config.home.optionIndex = 2; isSearchFocused = false; config.home.searchHistoryIndex = -1"
    >
      <IconImageSearch class="mx-1 w-5 h-5 shrink-0" />
      <div class="overflow-hidden whitespace-pre text-ellipsis">
        {{ $t('home.similar_images') }}
      </div>
    </div>

    <!-- image search -->
    <div class="px-2 h-10 flex items-center text-sm text-base-content/30 cursor-default whitespace-nowrap">
      {{ $t('home.image_search') }}
    </div>

    <div
      :class="[ 
        'mx-1 p-1 h-10 flex items-center rounded-box whitespace-nowrap cursor-pointer group relative',
        isSearchFocused ? 'text-base-content/70 bg-base-100' : 'text-base-content/30 hover:text-base-content/70 hover:bg-base-100',
      ]"
      @click="focusSearchInput"
    >
      <IconBolt 
        :class="[
          'absolute left-2 top-1/2 transform -translate-y-1/2 w-5 h-5 cursor-pointer rounded-box z-10',
          isSearchFocused ? 'text-primary group-hover:text-primary' : 'text-base-content/30 group-hover:text-base-content/70' 
        ]"
      />
      <input 
        ref="searchInputRef"
        type="text"
        v-model="searchQuery"
        :placeholder="$t('home.image_search_placeholder')"
        :class="[
          'pl-6 w-full input bg-transparent rounded-box',
          isSearchFocused ? 'border-primary' : 'border-base-content/30 group-hover:border-base-content/70 cursor-pointer',
        ]"
        maxlength="255"
        @focus="isSearchFocused = true"
        @keydown.enter = "handleSearch()"
        @keydown.esc = "handleEscKey()"
      >
    </div>

    <!-- search history -->
    <div class="overflow-y-auto" >
      <div v-for="(item, index) in config.home.searchHistory" :key="index"
        class="mx-2 p-2 text-sm rounded-box flex items-center"
        :class="[ 
          'mx-1 p-1 h-8 flex items-center rounded-box whitespace-nowrap cursor-pointer hover:bg-base-100 group', 
          config.home.searchHistoryIndex === index ? 'text-primary bg-base-100' : 'hover:text-base-content',
        ]"
        @click="handleSearchHistoryClick(index, item)"
      >
        <span class="overflow-hidden whitespace-pre text-ellipsis">- {{ item }}</span>
        <ContextMenu
          :class="[
            'ml-auto flex flex-row items-center text-base-content/30',
            config.home.searchHistoryIndex != index ? 'invisible group-hover:visible' : ''
          ]"
          :iconMenu="IconMore"
          :menuItems="searchHistoryMenuItems"
          :smallIcon="true"
        />
      </div>  
    </div>

    <!-- tips -->
    <div v-if="config.home.searchHistory.length === 0" class="px-2 mt-2 text-sm text-base-content/30">
      {{ $t('home.image_search_tips') }}
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
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { listen } from '@tauri-apps/api/event';
import { useUIStore } from '@/stores/uiStore';
import MessageBox from '@/components/MessageBox.vue';

import { IconMore, IconTrash, IconCalendarDay, IconHome, IconBolt, IconSimilar, IconImageSearch, IconHistory } from '@/common/icons';
import ContextMenu from '@/components/ContextMenu.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);
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
  config.home.optionIndex = 10;
  searchInputRef.value?.focus();

  config.home.searchHistoryIndex = -1;
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
  config.home.optionIndex = 10;
  isSearchFocused.value = true;
  config.home.searchHistoryIndex = index;
  
  searchQuery.value = item;
  config.home.searchText = item;
}

function clearHistory() {
  config.home.searchHistory = [];
  config.home.searchHistoryIndex = -1;
  showClearHistoryMsgbox.value = false;
}

function deleteHistoryItem() {
  config.home.searchHistory.splice(config.home.searchHistoryIndex, 1);
  config.home.searchHistoryIndex = -1;
}

function handleSearch() {
  if (searchQuery.value.trim().length === 0) return;
  
  const query = searchQuery.value.trim();
  const existingIndex = config.home.searchHistory.indexOf(query);
  
  if (existingIndex !== -1) {
    config.home.searchHistoryIndex = existingIndex;
  } else {
    config.home.searchHistory.unshift(query);
    config.home.searchHistoryIndex = 0;
  }

  config.home.searchText = query;
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

</script>
