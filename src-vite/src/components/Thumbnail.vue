<template>
  <div
    :class="[
      'border-2 flex flex-col items-center cursor-pointer group',
      isTransitionDisabled ? 'transition-none' : 'transition-all ease-in-out duration-300 ',
      config.settings.grid.style === 0 ? 'm-1 p-1 rounded-box' : '',
      isSelected && !isTransitionDisabled ? (uiStore.inputStack.length > 0 ? 'border-base-content/30' : 'border-primary') : 'border-transparent',
      config.settings.grid.style === 0 && isSelected ? 'bg-base-100 hover:bg-base-100' : 'hover:bg-base-100/30 hover:text-base-content ',
    ]"
    :style="containerStyle"
    @click="$emit('clicked')"
    @dblclick="$emit('dblclicked')"
  >
    <div v-if="file.thumbnail" 
      :class="[
        'relative flex items-center justify-center overflow-hidden', 
        config.settings.grid.style === 0 ? 'rounded-box' : '',
        (config.settings.grid.style === 1 || config.content.showFilmStrip) ? 'w-full h-full' : ''
      ]">
      <!-- image -->
      <img :src="file.thumbnail"
        class="duration-300"
        :class="{
          'group-hover:scale-115': config.settings.grid.style === 1,
          'scale-115': config.settings.grid.style === 1 && isSelected,
          'object-contain': config.settings.grid.scaling === 0,
          'object-cover': config.settings.grid.scaling === 1,
          'object-fill': config.settings.grid.scaling === 2,
          'transition-all': !isTransitionDisabled,
        }"
        :style="{ 
          ...layoutStyle,
          transform: `rotate(${file.rotate}deg)`,
        }"
        loading="lazy"
      />

      <!-- status icons -->
      <div class="absolute left-1 top-1 flex items-center text-sm text-base-content/30 group-hover:bg-base-100/10 rounded-box">
        <!-- video duration -->
        <div v-if="file.file_type===2" class="text-xs border rounded-box px-1 mr-1">
          {{ formatDuration(file.duration) }}
        </div>
        <IconCameraAperture v-if="file.e_model && file.e_model !== ''" class="t-icon-size-xs "></IconCameraAperture>
        <IconLocation v-if="file.geo_name" class="t-icon-size-xs "></IconLocation>
        <IconFavorite v-if="file.is_favorite" class="t-icon-size-xs"></IconFavorite>
        <IconTag v-if="file.has_tags" class="t-icon-size-xs "></IconTag>
        <IconComment v-if="file.comments?.length > 0" class="t-icon-size-xs "></IconComment>
        <IconRotate v-if="file.rotate % 360 > 0"
          class="t-icon-size-xs"
          :style="{ transform: `rotate(${file.rotate}deg)`, transition: 'transform 0.3s ease-in-out' }"
        />
      </div>

      <!-- select checkbox -->
      <div v-if="selectMode" class="absolute right-1 top-0.5">
        <component 
          :is="file?.isSelected ? IconChecked : IconUnChecked" 
          :class="[
            't-icon-size-sm', 
            file?.isSelected && uiStore.inputStack.length === 0 ? 'text-primary hover:text-primary' : 'text-base-content/30 hover:text-base-content/70'
          ]" 
          @click.stop="(event: MouseEvent) => $emit('select-toggled', event.shiftKey)"
        />
      </div>

      <!-- context menu -->
      <div v-if="!selectMode && !config.content.showFilmStrip" class="absolute right-0 top-0">
        <ContextMenu
          :class="[
            !isSelected ? 'invisible group-hover:visible' : ''
          ]"
          :iconMenu="IconMore"
          :menuItems="menuItems"
          :smallIcon="true"
        />
      </div>
    </div>
    
    <!-- skeleton for loading thumbnail -->
    <div v-else 
      :class="[
        'relative flex items-center justify-center overflow-hidden skeleton', 
        config.settings.grid.style === 0 ? 'rounded-box' : '',
      ]"
      :style="layoutStyle"
    ></div>

    <!-- label -->
    <div v-if="!config.content.showFilmStrip && config.settings.grid.style === 0" class="flex flex-col items-center" :class="{ 'text-primary': isSelected }">
      <span class="w-full text-sm text-center whitespace-pre text-nowrap text-ellipsis overflow-hidden">{{ getGridLabelText(file, config.settings.grid.labelPrimary) }}</span>
      <span class="w-full text-xs text-center whitespace-pre text-nowrap text-ellipsis overflow-hidden ">{{ getGridLabelText(file, config.settings.grid.labelSecondary) }}</span>
    </div>

  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, toRef } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import { isMac, shortenFilename, formatFileSize, formatDimensionText, formatDuration, formatTimestamp, formatCaptureSettings, formatCameraInfo } from '@/common/utils';
import ContextMenu from '@/components/ContextMenu.vue';
import { useFileMenuItems } from '@/common/fileMenu';

import { 
  IconMore,
  IconFavorite,
  IconTag,
  IconRotate,
  IconChecked,
  IconUnChecked,
  IconComment,
  IconLocation,
  IconCameraAperture,
} from '@/common/icons';

const props = defineProps({
  file: {
    type: Object,
    required: true,
  },
  isSelected: {
    type: Boolean,
    default: false,
  },
  selectMode: {
    type: Boolean,
    default: false,
  },
  showFolderFiles: {
    type: Boolean,
    default: false,
  }
});

const emit = defineEmits([
    'clicked', 
    'dblclicked', 
    'select-toggled', 
    'action'
]);

const isTransitionDisabled = ref(false);
let transitionTimeout: NodeJS.Timeout | null = null;

watch(() => config.content.showFilmStrip, () => {
  isTransitionDisabled.value = true;
  if (transitionTimeout) {
    clearTimeout(transitionTimeout);
  }
  transitionTimeout = setTimeout(() => {
    isTransitionDisabled.value = false;
  }, 500);
});

const containerStyle = computed(() => {
  if (config.content.showFilmStrip) {
    // For grid.style === 0, account for margin (4px) and padding (4px) = 8px total
    const size = config.settings.grid.style === 0 
      ? config.content.filmStripPaneHeight - 8 
      : config.content.filmStripPaneHeight;
    return {
      width: size + 'px',
      height: size + 'px',
    };
  }
  return {};
});

const layoutStyle = computed(() => {
  if (!config.content.showFilmStrip) {
    if (config.settings.grid.style === 1) {
      const height = config.settings.grid.size;
      return {
        width: '100%',
        height: height + 'px'
      }
    }
    return {
      width: config.settings.grid.size + 'px',
      height: config.settings.grid.size + 'px'
    }
  }
  else if (config.content.showFilmStrip) {
    return {
      width: '100%',
      height: '100%'
    }
  }
})

const uiStore = useUIStore();
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const menuItems = useFileMenuItems(
  toRef(props, 'file'),
  localeMsg,
  isMac,
  toRef(props, 'showFolderFiles'),
  (action) => emit('action', action)
);

const getGridLabelText = (file, option) => {
  switch (option) {
    case 0: return '';
    case 1: return shortenFilename(file.name) || ' ';
    case 2: return formatFileSize(file.size) || ' ';
    case 3: return formatDimensionText(file.width, file.height) || ' ';
    case 4: return formatTimestamp(file.taken_date, localeMsg.value.format.date_time) || ' ';
    case 5: return file.geo_name || ' ';
    case 6: return formatCameraInfo(file.e_make, file.e_model) || ' ';
    case 7: return file.e_lens_model || ' ';
    case 8: return formatCaptureSettings(file.e_focal_length, file.e_exposure_time, file.e_f_number, file.e_iso_speed, file.e_exposure_bias) || ' ';
    default: return '';
  }
};
</script>