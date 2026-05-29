<template>
  <div class="absolute inset-0 flex items-center justify-center px-6" data-tauri-drag-region>
    <div class="max-w-3xl w-full text-center">
      <div class="mb-8 flex flex-col items-center gap-3">
        <img :src="iconLogo" class="w-14 h-14 select-none [-webkit-app-region:no-drag]" draggable="false" />
        <div>
          <h2 class="text-xl font-semibold text-base-content">
            {{ $t('welcome.title') }}
          </h2>
          <p class="mt-2 text-sm text-base-content/55">
            {{ $t('welcome.description') }}
          </p>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-3 text-left">
        <div class="rounded-box border border-base-content/5 bg-base-100/60 p-4">
          <IconFolder class="w-5 h-5 mb-3" />
          <h3 class="text-sm font-medium text-base-content">{{ $t('welcome.add_album_title') }}</h3>
          <p class="mt-2 text-xs leading-5 text-base-content/55">{{ $t('welcome.add_album_description') }}</p>
          <button class="mt-4 btn btn-primary btn-sm rounded-box" @click="requestAddAlbum">
            <IconAdd class="w-4 h-4" />
            {{ $t('menu.album.add') }}
          </button>
        </div>
        <div class="rounded-box border border-base-content/5 bg-base-100/60 p-4">
          <IconSearch class="w-5 h-5 mb-3" />
          <h3 class="text-sm font-medium text-base-content">{{ $t('welcome.search_title') }}</h3>
          <p class="mt-2 text-xs leading-5 text-base-content/55">{{ $t('welcome.search_description') }}</p>
        </div>
        <div class="rounded-box border border-base-content/5 bg-base-100/60 p-4">
          <IconDragDrop class="w-5 h-5 mb-3" />
          <h3 class="text-sm font-medium text-base-content">{{ $t('welcome.drag_title') }}</h3>
          <p class="mt-2 text-xs leading-5 text-base-content/55">{{ $t('welcome.drag_description') }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { emit as tauriEmit } from '@tauri-apps/api/event';
import { IconAdd, IconFolder, IconSearch, IconDragDrop } from '@/common/icons';
import iconLogo from '@/assets/images/icon.png';

const requestAddAlbum = () => {
  tauriEmit('add-album-requested');
};
</script>
