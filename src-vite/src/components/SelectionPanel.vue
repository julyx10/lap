<template>
  <div class="w-full h-full rounded-box bg-base-200 flex flex-col overflow-hidden">
    <div class="flex items-center w-full shrink-0 px-2 mb-2">
      <div class="flex-1 pl-1">
        <span class="text-sm font-semibold text-base-content/70">
          {{ $t('toolbar.filter.select_mode') }}
        </span>
      </div>
      <div class="mt-2 flex items-center gap-1">
        <TButton
          :icon="IconClose"
          :tooltip="$t('msgbox.close')"
          :buttonSize="'small'"
          @click.stop="$emit('close')"
        />
      </div>
    </div>

    <div class="mb-2 px-2 flex-1 overflow-y-auto overflow-x-hidden flex flex-col">
      <div class="border-t border-base-content/5 px-1 py-3 space-y-3">
        <div class="flex items-center gap-2">
          <span class="text-[10px] uppercase tracking-widest font-bold text-base-content/30">
            {{ $t('info_panel.select_title') }}
          </span>
          <span
            class="ml-auto min-w-0 truncate text-right text-[11px] font-semibold text-base-content/70"
          >
            {{ selectionSummaryText }}
          </span>
        </div>
        <div class="flex flex-wrap items-center gap-1">
          <PanelActionButton
            :icon="IconCheckAll"
            :disabled="fileCount === 0"
            @click="$emit('selectAll')"
          >
            {{ $t('menu.select.all') }}
          </PanelActionButton>
          <PanelActionButton
            :icon="IconCheckNone"
            :disabled="selectedCount === 0"
            @click="$emit('selectNone')"
          >
            {{ $t('menu.select.none') }}
          </PanelActionButton>
          <PanelActionButton
            :disabled="selectedCount === 0"
            @click="$emit('selectInvert')"
          >
            {{ $t('menu.select.invert') }}
          </PanelActionButton>
        </div>
        <div v-if="selectedFiles.length > 0">
          <div class="mx-2 grid grid-cols-[repeat(auto-fill,minmax(5rem,1fr))] gap-1.5">
            <div
              v-for="file in visibleSelectedFiles"
              :key="file.id"
              class="group/thumb relative h-20 min-w-0 overflow-hidden rounded-box border border-base-content/5 cursor-pointer"
              :title="file.name || file.file_path"
              @click="$emit('unselectFile', file.id)"
            >
              <img
                v-if="file.thumbnail"
                :src="file.thumbnail"
                class="h-full w-full object-cover"
                :style="getSelectedThumbnailStyle(file)"
                loading="lazy"
              />
              <div v-else class="h-full w-full skeleton"></div>
              <div class="absolute inset-0 flex items-center justify-center bg-base-content/5 opacity-0 transition-opacity group-hover/thumb:opacity-100">
                <span class="badge badge-sm badge-primary">{{ $t('info_panel.deselect') }}</span>
              </div>
            </div>
            <div
              v-if="hiddenSelectedCount > 0"
              class="flex h-20 min-w-0 items-center justify-center rounded-box border border-dashed border-base-content/20 bg-base-100/50 text-xs font-semibold text-base-content/70"
              :title="$t('toolbar.filter.select_count', { count: selectedCount.toLocaleString() })"
            >
              +{{ hiddenSelectedCount }}
            </div>
          </div>
        </div>

      </div>

      <!-- Organize -->
      <div class="border-t border-base-content/5 px-1 py-4 space-y-3">
        <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/30">
          {{ $t('info_panel.organize') }}
        </div>
        <FavoriteRatingControl
          :favorite="multiSelectFavorite"
          :rating="multiSelectRating"
          :disabled="selectedCount === 0"
          :favorite-label="$t('info_panel.favorite_all')"
          :unfavorite-label="$t('info_panel.unfavorite_all')"
          label-class="text-base-content/30"
          inactive-rating-class="text-base-content/70"
          @favorite="(favorite) => $emit(favorite ? 'favoriteAll' : 'unfavoriteAll')"
          @rating="(rating) => $emit('setRatingAll', rating)"
        />
        <div class="flex flex-wrap items-center gap-1">
          <PanelActionButton
            :icon="IconTag"
            :disabled="selectedCount === 0"
            @click="$emit('tagAll')"
          >
            {{ $t('menu.meta.tag') }}
          </PanelActionButton>
          <PanelActionButton
            :icon="IconComment"
            :disabled="selectedCount === 0"
            @click="$emit('commentAll')"
          >
            {{ $t('menu.meta.comment') }}
          </PanelActionButton>
          <PanelActionButton
            :icon="IconRotate"
            :disabled="selectedCount === 0"
            @click="$emit('rotateAll')"
          >
            {{ rotateDisplayLabel }}
          </PanelActionButton>
        </div>
      </div>

      <!-- file actions -->
      <div class="border-t border-base-content/5 px-1 py-4 space-y-3">
        <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/30">
          {{ $t('info_panel.file_actions') }}
        </div>
        <div class="flex flex-wrap items-center gap-1">
          <PanelActionButton
            :icon="IconMove"
            :disabled="selectedCount === 0"
            @click="$emit('moveWithinLibrary')"
          >
            {{ $t('menu.file.move_within_library') }}
          </PanelActionButton>
          <PanelActionButton
            :disabled="selectedCount === 0"
            @click="$emit('moveToFolder')"
          >
            {{ $t('menu.file.move_to_folder') }}
          </PanelActionButton>
          <PanelActionButton
            :disabled="selectedCount === 0"
            @click="$emit('copyToFolder')"
          >
            {{ $t('menu.file.copy_to_folder') }}
          </PanelActionButton>
          <PanelActionButton
            :icon="IconTrash"
            :disabled="selectedCount === 0"
            danger
            @click="$emit('trash')"
          >
            {{ $t('menu.file.move_to_trash') }}
          </PanelActionButton>
        </div>

      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, type CSSProperties } from 'vue';
import { useI18n } from 'vue-i18n';
import { formatFileSize, formatTimestamp, isMac } from '@/common/utils';
import {
  IconCheckAll,
  IconCheckNone,
  IconClose,
  IconComment,
  IconMove,
  IconRotate,
  IconTag,
  IconTrash,
} from '@/common/icons';
import TButton from '@/components/TButton.vue';
import FavoriteRatingControl from '@/components/FavoriteRatingControl.vue';
import PanelActionButton from '@/components/PanelActionButton.vue';

const props = defineProps({
  selectedFiles: {
    type: Array as () => any[],
    default: () => [],
  },
  fileCount: {
    type: Number,
    default: 0,
  },
  selectedCount: {
    type: Number,
    default: 0,
  },
  selectedSize: {
    type: Number,
    default: 0,
  },
});

defineEmits([
  'close',
  'selectAll',
  'selectNone',
  'selectInvert',
  'moveWithinLibrary',
  'moveToFolder',
  'copyToFolder',
  'trash',
  'favoriteAll',
  'unfavoriteAll',
  'setRatingAll',
  'tagAll',
  'commentAll',
  'rotateAll',
  'unselectFile',
]);

const { locale, messages, t } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const SELECTED_THUMBNAIL_LIMIT = 19;

const visibleSelectedFiles = computed(() => props.selectedFiles.slice(0, SELECTED_THUMBNAIL_LIMIT));
const hiddenSelectedCount = computed(() => Math.max(0, props.selectedCount - visibleSelectedFiles.value.length));
const selectionSummaryText = computed(() => {
  if (props.selectedCount <= 0) return localeMsg.value.info_panel.select_hint;
  const countText = t('toolbar.filter.select_count', { count: props.selectedCount.toLocaleString() });
  return `${countText} (${formatFileSize(props.selectedSize)})`;
});

function getSelectedThumbnailStyle(file: any): CSSProperties {
  return { rotate: `${Number(file?.rotate || 0)}deg` };
}

const multiSelectDateRange = computed(() => {
  if (props.selectedFiles.length === 0) return '';
  const dates = props.selectedFiles
    .map((f: any) => f.created_at)
    .filter(Boolean)
    .sort();
  if (dates.length === 0) return '';
  const first = formatTimestamp(dates[0], localeMsg.value.format.date);
  const last = formatTimestamp(dates[dates.length - 1], localeMsg.value.format.date);
  return first === last ? first : `${first} - ${last}`;
});

const multiSelectTypeBreakdown = computed(() => {
  const images = props.selectedFiles.filter((f: any) => f.file_type === 1).length;
  const videos = props.selectedFiles.filter((f: any) => f.file_type === 2).length;
  const parts = [];
  if (images > 0) parts.push(`${images} ${images === 1 ? localeMsg.value.info_panel.type_image : localeMsg.value.info_panel.type_images}`);
  if (videos > 0) parts.push(`${videos} ${videos === 1 ? localeMsg.value.info_panel.type_video : localeMsg.value.info_panel.type_videos}`);
  return parts.join(' · ');
});

const multiSelectRating = computed(() => {
  if (!props.selectedFiles.length) return 0;
  const ratings = props.selectedFiles.map((f: any) => Number(f.rating || 0));
  const first = ratings[0];
  return ratings.every((rating: number) => rating === first) ? first : null;
});

const multiSelectFavorite = computed(() => {
  if (!props.selectedFiles.length) return false;
  const favorites = props.selectedFiles.map((f: any) => Boolean(f.is_favorite));
  const first = favorites[0];
  return favorites.every((favorite: boolean) => favorite === first) ? first : null;
});

const multiSelectRotate = computed(() => {
  if (!props.selectedFiles.length) return 0;
  const normalizeRotate = (rotate: number) => {
    const normalized = rotate % 360;
    return normalized < 0 ? normalized + 360 : normalized;
  };
  const rotates = props.selectedFiles.map((f: any) => normalizeRotate(Number(f.rotate || 0)));
  const first = rotates[0];
  return rotates.every((rotate: number) => rotate === first) ? first : null;
});

const rotateDisplayLabel = computed(() => {
  const rotateText = localeMsg.value.menu.meta.rotate;
  if (multiSelectRotate.value === null || multiSelectRotate.value === 0) {
    return rotateText;
  }
  return `${rotateText} (${multiSelectRotate.value}°)`;
});

</script>
