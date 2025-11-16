<template>
  <div
    :class="[
      'border-2 rounded-box flex flex-col items-center hover:bg-base-100 ease-in-out duration-300 cursor-pointer group',
      isTransitionDisabled ? 'transition-none' : 'transition-all',
      config.content.layout === 0 ? 'p-2' : 'p-0.5 ml-0.5',
      isSelected && !isTransitionDisabled ? (uiStore.inputStack.length > 0 ? 'border-base-content/30' : 'border-primary') : 'border-transparent',
    ]"
    :style="containerStyle"
    @click="$emit('clicked')"
    @dblclick="$emit('dblclicked')"
  >
    <div v-if="file.thumbnail" class="relative rounded-box flex items-center justify-center overflow-hidden">
      <!-- image -->
      <img :src="file.thumbnail"
        class="duration-300"
        :class="{
          'group-hover:scale-115': config.content.layout === 1,
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
      <div class="absolute left-1 top-1 flex items-center text-sm text-base-content/30">
        <!-- video duration -->
        <div v-if="file.file_type===2" class="text-xs border rounded-box px-1 z-10">
          {{ formatDuration(file.duration) }}
        </div>
        <!-- status icons -->
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
          :class="['t-icon-size-sm hover:text-base-content/70', file?.isSelected ? 'text-primary' : 'text-base-content/30']" 
          @click.stop="$emit('select-toggled')"
        />
      </div>

      <!-- context menu -->
      <div v-if="!selectMode" class="absolute right-0 top-0">
        <ContextMenu
          :class="[
            !isSelected ? 'invisible group-hover:visible' : ''
          ]"
          :iconMenu="IconMore"
          :menuItems="contextMenuItems"
          :smallIcon="true"
        />
      </div>
    </div>
    
    <!-- skeleton for loading thumbnail -->
    <div v-else 
      class="skeleton rounded-box flex items-center justify-center"
      :style="{ width: `${config.settings.grid.size}px`, height: `${config.settings.grid.size}px` }"
    ></div>

    <!-- label -->
    <template v-if="config.content.layout === 0">
      <span class="pt-1 text-sm text-center">{{ getGridLabelText(file, config.settings.grid.labelPrimary) }}</span>
      <span class="text-xs text-center">{{ getGridLabelText(file, config.settings.grid.labelSecondary) }}</span>
    </template>

  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import { isMac, shortenFilename, formatFileSize, formatDimensionText, formatDuration, formatTimestamp, formatCaptureSettings } from '@/common/utils';
import ContextMenu from '@/components/ContextMenu.vue';

import { 
  IconMore,
  IconMonitor,
  IconImageEdit,
  IconFavorite,
  IconUnFavorite,
  IconTag,
  IconRotate,
  IconCopy,
  IconRename,
  IconMoveTo,
  IconCopyTo,
  IconTrash,
  IconGoto,
  IconChecked,
  IconUnChecked,
  IconComment,
  IconLocation,
  IconCameraAperture,
  IconRefresh,
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

watch(() => config.content.layout, () => {
  isTransitionDisabled.value = true;
  if (transitionTimeout) {
    clearTimeout(transitionTimeout);
  }
  transitionTimeout = setTimeout(() => {
    isTransitionDisabled.value = false;
  }, 500);
});

const containerStyle = computed(() => {
  if (config.content.layout === 1) {
    const size = config.content.filmStripPaneHeight;
    return {
      width: size + 'px',
      height: size + 'px',
    };
  }
  return {};
});

const layoutStyle = computed(() => {
  if (config.content.layout === 0) {
    return {
      width: config.settings.grid.size + 'px',
      height: config.settings.grid.size + 'px'
    }
  }
  else if (config.content.layout === 1) {
    const size = config.content.filmStripPaneHeight - 8;
    return {
      maxWidth: size + 'px',
      maxHeight: size + 'px',
      minWidth: size + 'px',
      minHeight: size + 'px',
      width: size + 'px',
      height: size + 'px'
    }
  }
})

const uiStore = useUIStore();
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

const contextMenuItems = computed(() => {
  const file = props.file;
  const createAction = (actionName: string) => () => emit('action', actionName);

  return [
    {
      label: localeMsg.value.menu.file.view,
      icon: IconMonitor,
      shortcut: isMac ? '⌘⏎' : 'Ctrl+Enter',
      action: createAction('open')
    },
    {
      label: localeMsg.value.menu.file.edit,
      icon: IconImageEdit,
      shortcut: isMac ? '⌘E' : 'Ctrl+E',
      disabled: file.file_type !== 1,
      action: createAction('edit')
    },
    {
      label: localeMsg.value.menu.file.copy,
      icon: IconCopy,
      shortcut: isMac ? '⌘C' : 'Ctrl+C',
      disabled: file.file_type !== 1,
      action: createAction('copy')
    },
    {
      label: localeMsg.value.menu.file.update_from_file,
      icon: IconRefresh,
      action: createAction('update-from-file')
    },
    {
      label: localeMsg.value.menu.file.goto_album,
      disabled: props.showFolderFiles,
      icon: IconGoto,
      action: createAction('goto-folder')
    },
    {
      label: isMac ? localeMsg.value.menu.file.reveal_in_finder : localeMsg.value.menu.file.reveal_in_file_explorer,
      action: createAction('reveal')
    },
    { label: "-", action: null },
    {
      label: localeMsg.value.menu.file.rename,
      icon: IconRename,
      action: createAction('rename')
    },
    {
      label: localeMsg.value.menu.file.move_to,
      icon: IconMoveTo,
      action: createAction('move-to')
    },
    {
      label: localeMsg.value.menu.file.copy_to,
      icon: IconCopyTo,
      action: createAction('copy-to')
    },
    {
      label: isMac ? localeMsg.value.menu.file.move_to_trash : localeMsg.value.menu.file.delete,
      icon: IconTrash,
      shortcut: isMac ? '⌘⌫' : 'Del',
      action: createAction('trash')
    },
    { label: "-", action: null },
    {
      label: file.is_favorite ? localeMsg.value.menu.meta.unfavorite : localeMsg.value.menu.meta.favorite,
      icon: file.is_favorite ? IconUnFavorite : IconFavorite,
      shortcut: isMac ? '⌘F' : 'Ctrl+F',
      action: createAction('favorite')
    },
    {
      label: localeMsg.value.menu.meta.tag,
      icon: IconTag,
      shortcut: isMac ? '⌘T' : 'Ctrl+T',
      action: createAction('tag')
    },
    {
      label: localeMsg.value.menu.meta.comment,
      icon: IconComment,
      action: createAction('comment')
    },
    {
      label: localeMsg.value.menu.meta.rotate,
      icon: IconRotate,
      shortcut: isMac ? '⌘R' : 'Ctrl+R',
      action: createAction('rotate')
    }
  ];
});

const getGridLabelText = (file, option) => {
  switch (option) {
    case 0: return '';
    case 1: return shortenFilename(file.name);
    case 2: return formatFileSize(file.size);
    case 3: return formatDimensionText(file.width, file.height);
    case 4: return file.duration > 0 ? formatDuration(file.duration): '-';
    case 5: return formatTimestamp(file.created_at, localeMsg.value.format.date_time);
    case 6: return formatTimestamp(file.modified_at, localeMsg.value.format.date_time);
    case 7: return file.e_make && file.e_model ? `${file.e_model}` : '-';
    case 8: return file.e_lens_model ? `${file.e_lens_model}` : '-';
    case 9: return formatCaptureSettings(file.e_focal_length, file.e_exposure_time, file.e_f_number, file.e_iso_speed, file.e_exposure_bias);
    case 10: return file.e_date_time || '-';
    case 11: return file.geo_name || '-';
    default: return '';
  }
};
</script>