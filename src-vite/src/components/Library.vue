<template>

  <div class="sidebar-panel">
    <div class="sidebar-panel-header">
      <span class="sidebar-panel-header-title flex-1">{{ titlebar }}</span>
    </div>

    <div class="min-h-0 flex-1 overflow-x-hidden overflow-y-auto">
      <div
        v-for="item in libraryItems"
        :key="item.id"
        :class="[
          'sidebar-item',
          libConfig.library.item === item.id ? 'sidebar-item-selected' : 'sidebar-item-hover',
        ]"
        @click="selectItem(item.id)"
      >
        <component :is="item.icon" class="mx-1 w-5 h-5 shrink-0" />
        <div class="sidebar-item-label">
          <span>{{ item.label }}</span>
        </div>
        <div class="ml-auto flex items-center">
          <span v-if="item.count && item.count > 0" class="px-1 text-xs text-base-content/30">
            {{ item.count.toLocaleString() }}
          </span>
        </div>
      </div>

      <div class="sidebar-panel-header cursor-pointer" @click="toggleRatings">
        <span class="sidebar-panel-header-title flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">
          {{ localeMsg.rating.title }}
        </span>
        <TButton
          :icon="libConfig.library.ratingsExpanded ? IconArrowDown : IconArrowUp"
          :buttonSize="'small'"
          @click.stop="toggleRatings"
        />
      </div>

      <Transition
        @before-enter="onBeforeEnter"
        @enter="onEnter"
        @after-enter="onAfterEnter"
        @leave="onLeave"
      >
        <div v-if="libConfig.library.ratingsExpanded" class="overflow-hidden">
          <ul class="mb-2">
            <li>
              <div
                :class="[
                  'sidebar-item group border-2 border-transparent',
                  libConfig.library.item === LIB_ITEM.RATINGS && libConfig.rating.item === RATE.ALL ? 'sidebar-item-selected' : 'sidebar-item-hover',
                ]"
                @click="selectRating(RATE.ALL)"
              >
                <IconStarFilled class="mx-1 w-5 h-5 shrink-0" />
                <span class="sidebar-item-label">{{ localeMsg.rating.all_rated || localeMsg.rating.rated }}</span>
                <span v-if="ratedCount" class="sidebar-item-count ml-auto">{{ ratedCount.toLocaleString() }}</span>
              </div>
            </li>
            <li v-for="rating in [5, 4, 3, 2, 1]" :key="rating">
              <div
                :class="[
                  'sidebar-item group border-2 border-transparent',
                  libConfig.library.item === LIB_ITEM.RATINGS && libConfig.rating.item === rating ? 'sidebar-item-selected' : 'sidebar-item-hover',
                ]"
                @click="selectRating(rating)"
              >
                <div class="mx-1 flex items-center gap-0.5">
                  <IconStarFilled
                    v-for="index in rating"
                    :key="index"
                    class="w-5 h-5 shrink-0"
                  />
                </div>
                <span v-if="ratingCounts[rating]" class="text-[10px] tabular-nums text-base-content/30 ml-auto">
                  {{ ratingCounts[rating].toLocaleString() }}
                </span>
              </div>
            </li>
            <li>
              <div
                :class="[
                  'sidebar-item group border-2 border-transparent',
                  libConfig.library.item === LIB_ITEM.RATINGS && libConfig.rating.item === RATE.UNRATED ? 'sidebar-item-selected' : 'sidebar-item-hover',
                ]"
                @click="selectRating(RATE.UNRATED)"
              >
                <IconStar class="mx-1 w-5 h-5 shrink-0" />
                <span class="sidebar-item-label">{{ localeMsg.rating.unrated }}</span>
                <span v-if="unratedCount" class="sidebar-item-count ml-auto">{{ unratedCount.toLocaleString() }}</span>
              </div>
            </li>
          </ul>
        </div>
      </Transition>

      <div class="sidebar-panel-header cursor-pointer" @click="toggleSubjects">
        <span class="sidebar-panel-header-title flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">
          {{ localeMsg.subject.title }}
        </span>
        <TButton
          :icon="libConfig.library.subjectsExpanded ? IconArrowDown : IconArrowUp"
          :buttonSize="'small'"
          @click.stop="toggleSubjects"
        />
      </div>

      <Transition
        @before-enter="onBeforeEnter"
        @enter="onEnter"
        @after-enter="onAfterEnter"
        @leave="onLeave"
      >
        <div v-if="libConfig.library.subjectsExpanded" class="overflow-hidden">
          <ul class="mb-2">
            <li v-for="item in smartTagItems" :key="item.id">
              <div
                :class="[
                  'sidebar-item group border-2 border-transparent',
                  libConfig.library.item === LIB_ITEM.SUBJECTS && libConfig.library.smartId === item.id ? 'sidebar-item-selected' : 'sidebar-item-hover',
                ]"
                @click="selectSmartTag(item.id)"
              >
                <IconSmartTag class="mx-1 w-5 h-5 shrink-0" />
                <span class="sidebar-item-label">{{ item.label }}</span>
              </div>
            </li>
          </ul>
        </div>
      </Transition>
    </div>

  </div>

</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { libConfig } from '@/common/config';
import { LIB_ITEM, RATE, type LibItem } from '@/common/constants';

import { IconPhotoAll, IconHeartFilled, IconCalendarDay, IconArrowDown, IconArrowUp, IconSmartTag, IconStar, IconStarFilled, IconHistory } from '@/common/icons';
import { getQueryCountAndSum, getTotalCountAndSum } from '@/common/api';
import { SMART_TAG_CATEGORIES } from '@/common/smartTags';
import TButton from '@/components/TButton.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const totalCount = ref(0);
const favoriteCount = ref(0);
const todayCount = ref(0);
const unratedCount = ref(0);
const ratingCounts = ref<Record<number, number>>({
  1: 0,
  2: 0,
  3: 0,
  4: 0,
  5: 0,
});
const ratedCount = computed(() => Object.values(ratingCounts.value).reduce((sum, count) => sum + count, 0));

const buildQueryParams = ({ isFavorite = false, rating = RATE.NONE, startDate = 0, endDate = 0 } = {}) => ({
  searchFileType: 0,
  sortType: 0,
  sortOrder: 0,
  searchFileName: "",
  searchAllSubfolders: "",
  searchFolder: "",
  startDate,
  endDate,
  calendarSort: 0,
  make: "",
  model: "",
  lensMake: "",
  lensModel: "",
  locationAdmin1: "",
  locationName: "",
  isFavorite,
  rating,
  tagId: 0,
  personId: 0,
});

const libraryItems = computed(() => [
  {
    id: LIB_ITEM.ALL,
    label: localeMsg.value.library.all_files,
    icon: IconPhotoAll,
    count: totalCount.value,
  },
  {
    id: LIB_ITEM.FAV,
    label: localeMsg.value.favorite.files,
    icon: IconHeartFilled,
    count: favoriteCount.value,
  },
  {
    id: LIB_ITEM.TODAY,
    label: localeMsg.value.library.on_this_day,
    icon: IconHistory,
    count: todayCount.value,
  },
]);

const smartTagItems = computed(() =>
  SMART_TAG_CATEGORIES.map(category => {
    const item = category.items[0];
    return {
      id: item.id,
      label: localeMsg.value.subject.items?.[item.id] || item.id,
    };
  })
);

const refreshTotalCount = async () => {
  const result = await getTotalCountAndSum();
  totalCount.value = result ? result[0] : 0;
};

const refreshFavoriteCount = async () => {
  const result = await getQueryCountAndSum(buildQueryParams({ isFavorite: true }));
  favoriteCount.value = result ? Number(result[0]) : 0;
};

const refreshTodayCount = async () => {
  const result = await getQueryCountAndSum(buildQueryParams({ startDate: -1, endDate: -1 }));
  todayCount.value = result ? Number(result[0]) : 0;
};

const refreshRatingCounts = async () => {
  const unrated = await getQueryCountAndSum(buildQueryParams({ rating: 0 }));
  unratedCount.value = unrated ? Number(unrated[0]) : 0;

  const entries = await Promise.all(
    [1, 2, 3, 4, 5].map(async (rating) => {
      const result = await getQueryCountAndSum(buildQueryParams({ rating }));
      return [rating, result ? Number(result[0]) : 0] as const;
    }),
  );

  ratingCounts.value = Object.fromEntries(entries) as Record<number, number>;
};

function selectItem(item: LibItem) {
  libConfig.library.item = item;
}

function toggleSubjects() {
  libConfig.library.subjectsExpanded = !libConfig.library.subjectsExpanded;
}

function toggleRatings() {
  libConfig.library.ratingsExpanded = !libConfig.library.ratingsExpanded;
}

function onBeforeEnter(el: Element) {
  const element = el as HTMLElement;
  element.style.opacity = '0';
  element.style.height = '0';
}

function onEnter(el: Element) {
  const element = el as HTMLElement;
  element.style.transition = 'all 0.1s ease';
  element.style.height = `${element.scrollHeight}px`;
  element.style.opacity = '1';
}

function onAfterEnter(el: Element) {
  (el as HTMLElement).style.height = '';
}

function onLeave(el: Element) {
  const element = el as HTMLElement;
  element.style.transition = 'all 0.1s ease';
  element.style.height = `${element.scrollHeight}px`;
  void element.offsetHeight;
  element.style.height = '0';
  element.style.opacity = '0';
}

function selectRating(rating: number) {
  libConfig.library.item = LIB_ITEM.RATINGS;
  libConfig.rating.item = rating;
}

function selectSmartTag(smartId: string) {
  libConfig.library.item = LIB_ITEM.SUBJECTS;
  libConfig.library.smartId = smartId;
}

onMounted(async () => {
  await refreshTotalCount();
  await refreshFavoriteCount();
  await refreshTodayCount();
  await refreshRatingCounts();
});

</script>
