<template>
    
  <div class="w-full h-full flex flex-col select-none">

    <!-- All Files -->
    <div 
      :class="[ 
        'mx-1 p-1 h-10 flex items-center rounded-box whitespace-nowrap cursor-pointer',
        (libConfig.album.id === 0 ? 'text-primary bg-base-100 hover:bg-base-100' : 'hover:text-base-content hover:bg-base-100/30'),
      ]"
      @click="clickAllFiles"
    >
      <IconPhotoAll class="mx-1 w-5 h-5 shrink-0" />
      <div class="overflow-hidden whitespace-pre text-ellipsis">
        <span>{{ $t('album.all_files') }}</span>
      </div>

      <!-- Right side: Count -->
      <div class="ml-auto flex items-center">
        <span v-if="totalCount > 0" class="px-1 text-xs text-base-content/30">
          {{ totalCount.toLocaleString() }}
        </span>
      </div>
    </div>

    <AlbumList ref="albumListRef" 
      :key="albumListKey"
      selectionSource="album"
      @editDataChanged="(val) => emit('editDataChanged', val)"
    />
  </div> 

</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { libConfig } from '@/common/config';

import { IconPhotoAll } from '@/common/icons';
import { getTotalCountAndSum } from '@/common/api';
import AlbumList from '@/components/AlbumList.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const emit = defineEmits(['editDataChanged']);

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const totalCount = ref(0);

const albumListRef = ref<InstanceType<typeof AlbumList> | null>(null);

// refresh component
const albumListKey = ref(0);

onMounted(async () => {
  // get total count
  getTotalCountAndSum().then((result) => {
    if(result) {
      totalCount.value = result[0];
    }
  });
});

const clickAllFiles = () => {
    libConfig.album.id = 0;
    libConfig.album.folderId = null;
    libConfig.album.folderPath = '';
    libConfig.album.selected = false;
};

defineExpose({
  albumListRef,
});

</script>
