<template>
  <div class="w-full h-full rounded-box bg-base-200 flex flex-col overflow-hidden">
    <div class="flex items-center w-full shrink-0 px-2 mb-2">
      <div class="flex-1 pl-1">
        <span class="text-[11px] font-bold uppercase tracking-[0.22em] text-base-content/35">
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

    <div class="mb-2 px-2 flex-1 overflow-y-auto overflow-x-hidden space-y-3 flex flex-col bg-base-200/50">
      <div class="rounded-box p-3 space-y-3 bg-base-300/30 border border-base-content/5 shadow-sm">
        <div class="flex items-center gap-2 text-base-content/70">
          <span class="font-bold uppercase text-xs tracking-wide">
            {{ $t('info_panel.select_title') }}
          </span>
        </div>
        <div class="flex flex-wrap items-center gap-1">
          <button
            class="btn btn-xs btn-ghost"
            :class="fileCount === 0 ? 'text-base-content/30' : 'text-base-content/70 hover:text-base-content'"
            :disabled="fileCount === 0"
            @click="$emit('selectAll')"
          >
            <IconCheckAll class="w-3.5 h-3.5" />
            {{ $t('menu.select.all') }}
          </button>
          <button
            class="btn btn-xs btn-ghost"
            :class="selectedCount === 0 ? 'text-base-content/30' : 'text-base-content/70 hover:text-base-content'"
            :disabled="selectedCount === 0"
            @click="$emit('selectNone')"
          >
            <IconCheckNone class="w-3.5 h-3.5" />
            {{ $t('menu.select.none') }}
          </button>
          <button
            class="btn btn-xs btn-ghost"
            :class="selectedCount === 0 ? 'text-base-content/30' : 'text-base-content/70 hover:text-base-content'"
            :disabled="selectedCount === 0"
            @click="$emit('selectInvert')"
          >
            {{ $t('menu.select.invert') }}
          </button>
        </div>
        <div v-if="showSelectionLimitHint" class="mx-2 text-xs font-medium leading-relaxed text-warning">
          {{ $t('info_panel.select_limit_hint', { count: selectionLimit }) }}
        </div>
        <div class="border-t border-base-content/10"></div>

        <div class="mx-2 pt-1 space-y-2 text-xs text-base-content/50 font-medium">
          <div class="font-semibold">
            <span>{{ selectedCount > 0 ? `${$t('toolbar.filter.select_count', { count: selectedCount.toLocaleString() })} (${formatFileSize(selectedSize)})` : $t('info_panel.select_hint') }}</span>
          </div>
          <div>
            <span>{{ multiSelectTypeBreakdown }}</span>
          </div>
          <div v-if="multiSelectDateRange">
            <span>{{ multiSelectDateRange }}</span>
          </div>
        </div>
      </div>

      <div class="rounded-box p-3 space-y-3 bg-base-300/30 border border-base-content/5 shadow-sm">
        <div class="text-base-content/70">
          <span class="font-bold uppercase text-xs tracking-wide">{{ $t('info_panel.action') }}</span>
        </div>
        <div class="space-y-2">
          <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/30">
            {{ $t('info_panel.file_actions') }}
          </div>
          <div class="flex flex-wrap items-center gap-1">
            <button
              class="btn btn-xs btn-ghost"
              :class="selectedCount === 0 ? 'text-base-content/30' : 'text-base-content/70 hover:text-base-content'"
              :disabled="selectedCount === 0"
              @click="$emit('moveTo')"
            >
              <IconMoveTo class="w-3.5 h-3.5" />
              {{ $t('menu.file.move_to') }}
            </button>
            <button
              class="btn btn-xs btn-ghost"
              :class="selectedCount === 0 ? 'text-base-content/30' : 'text-base-content/70 hover:text-base-content'"
              :disabled="selectedCount === 0"
              @click="$emit('copyTo')"
            >
              <IconCopyTo class="w-3.5 h-3.5" />
              {{ $t('menu.file.copy_to') }}
            </button>
            <button
              class="btn btn-xs btn-ghost"
              :class="selectedCount === 0 ? 'text-base-content/30' : 'text-error'"
              :disabled="selectedCount === 0"
              @click="$emit('trash')"
            >
              <IconTrash class="w-3.5 h-3.5" />
              {{ isMac ? $t('menu.file.move_to_trash') : $t('menu.file.delete') }}
            </button>
          </div>
        </div>
        <div class="border-t border-base-content/10"></div>
        <div class="space-y-2">
          <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/30">
            {{ $t('info_panel.labels_and_display') }}
          </div>
          <div class="rounded-box border border-base-content/10 bg-base-100/30 px-2 py-1.5">
            <div class="flex items-center gap-0.5">
              <button
                class="btn btn-ghost btn-xs min-h-0 h-6 w-6 p-0 mr-1"
                :title="multiSelectFavorite ? $t('info_panel.unfavorite_all') : $t('info_panel.favorite_all')"
                :disabled="selectedCount === 0"
                @click="$emit(multiSelectFavorite ? 'unfavoriteAll' : 'favoriteAll')"
              >
                <component
                  :is="multiSelectFavorite ? IconHeartFilled : IconHeart"
                  class="w-3.5 h-3.5"
                  :class="selectedCount === 0 ? 'text-base-content/30' : (multiSelectFavorite ? 'text-error' : 'text-base-content/70')"
                />
              </button>
              <div class="w-px h-4 bg-base-content/10 mx-1"></div>
              <span class="mr-1 text-[11px] font-medium" :class="selectedCount === 0 ? 'text-base-content/30' : 'text-base-content/70'">{{ $t('favorite.ratings') }}</span>
              <button
                v-for="rating in [1, 2, 3, 4, 5]"
                :key="rating"
                class="btn btn-ghost btn-xs min-h-0 h-6 w-6 p-0"
                :title="getRatingLabel(rating)"
                :disabled="selectedCount === 0"
                @click="$emit('setRatingAll', multiSelectRating === rating ? 0 : rating)"
              >
                <component
                  :is="(multiSelectRating || 0) >= rating ? IconStarFilled : IconStar"
                  class="w-3.5 h-3.5"
                  :class="selectedCount === 0 ? 'text-base-content/30' : ((multiSelectRating || 0) >= rating ? 'text-warning' : 'text-base-content/70')"
                />
              </button>
            </div>
          </div>
          <div class="flex flex-wrap items-center gap-1">
            <button
              class="btn btn-xs btn-ghost gap-1"
              :class="selectedCount === 0 ? 'text-base-content/30' : 'text-base-content/70 hover:text-base-content'"
              :disabled="selectedCount === 0"
              @click="$emit('tagAll')"
            >
              <IconTag class="w-3.5 h-3.5" />
              <span>{{ $t('menu.meta.tag') }}</span>
            </button>
            <button
              class="btn btn-xs btn-ghost gap-1"
              :class="selectedCount === 0 ? 'text-base-content/30' : 'text-base-content/70 hover:text-base-content'"
              :disabled="selectedCount === 0"
              @click="$emit('commentAll')"
            >
              <IconComment class="w-3.5 h-3.5" />
              <span>{{ $t('menu.meta.comment') }}</span>
            </button>
            <button
              class="btn btn-xs btn-ghost gap-1"
              :class="selectedCount === 0 ? 'text-base-content/30' : 'text-base-content/70 hover:text-base-content'"
              :disabled="selectedCount === 0"
              @click="$emit('rotateAll')"
            >
              <IconRotate class="w-3.5 h-3.5" />
              <span>{{ rotateDisplayLabel }}</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { formatFileSize, formatTimestamp, isMac } from '@/common/utils';
import {
  IconCheckAll,
  IconCheckNone,
  IconClose,
  IconComment,
  IconCopyTo,
  IconHeart,
  IconHeartFilled,
  IconMoveTo,
  IconRotate,
  IconStar,
  IconStarFilled,
  IconTag,
  IconTrash,
} from '@/common/icons';
import TButton from '@/components/TButton.vue';

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
  selectionLimit: {
    type: Number,
    default: 400,
  },
  showSelectionLimitHint: {
    type: Boolean,
    default: false,
  },
});

defineEmits([
  'close',
  'selectAll',
  'selectNone',
  'selectInvert',
  'moveTo',
  'copyTo',
  'trash',
  'favoriteAll',
  'unfavoriteAll',
  'setRatingAll',
  'tagAll',
  'commentAll',
  'rotateAll',
]);

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

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
  return favorites.every((favorite: boolean) => favorite === first) ? first : false;
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

function getRatingLabel(rating: number) {
  const keys: Record<number, string> = {
    5: 'five_stars',
    4: 'four_stars',
    3: 'three_stars',
    2: 'two_stars',
    1: 'one_star',
  };
  const key = keys[rating];
  return localeMsg.value.favorite?.[key] || `${rating}★`;
}
</script>
