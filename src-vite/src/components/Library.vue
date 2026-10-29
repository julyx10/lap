<template>

  <div class="sidebar-panel">
    <div class="sidebar-panel-header">
      <span class="sidebar-panel-header-title flex-1">{{ titlebar }}</span>
    </div>

    <div class="min-h-0 flex-1 overflow-x-hidden overflow-y-auto">
      <template v-for="item in libraryItems" :key="item.id">
      <div
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
          <span v-if="item.count && item.count > 0" class="sidebar-item-count">
            {{ item.count.toLocaleString() }}
          </span>
        </div>
      </div>
      </template>

      <div class="sidebar-item sidebar-item-hover" @click="toggleRatings">
        <IconRight
          :class="[
            'p-1 w-6 h-6 shrink-0 transition-transform',
            libConfig.library.ratingsExpanded ? 'rotate-90' : '',
          ]"
          @click.stop="toggleRatings"
        />
        <span class="sidebar-item-label">
          {{ localeMsg.rating.title }}
        </span>
      </div>

      <Transition
        @before-enter="onBeforeEnter"
        @enter="onEnter"
        @after-enter="onAfterEnter"
        @leave="onLeave"
      >
        <div v-if="libConfig.library.ratingsExpanded" class="overflow-hidden">
          <ul class="mb-2">
            <li class="pl-4">
              <div
                :class="[
                  'sidebar-item sidebar-item-compact ml-2',
                  libConfig.library.item === LIB_ITEM.RATINGS && libConfig.rating.item === RATE.ALL ? 'sidebar-item-selected' : 'sidebar-item-hover',
                ]"
                @click="selectRating(RATE.ALL)"
              >
                <IconStarFilled class="mx-1 w-4 h-4 shrink-0" />
                <span class="sidebar-item-label">{{ localeMsg.rating.rated }}</span>
                <span v-if="ratedCount" class="text-[10px] tabular-nums text-base-content/30 mr-2">{{ ratedCount.toLocaleString() }}</span>
              </div>
            </li>
            <li v-for="rating in [5, 4, 3, 2, 1]" :key="rating" class="pl-4">
              <div
                :class="[
                  'sidebar-item sidebar-item-compact ml-2',
                  libConfig.library.item === LIB_ITEM.RATINGS && libConfig.rating.item === rating ? 'sidebar-item-selected' : 'sidebar-item-hover',
                ]"
                @click="selectRating(rating)"
              >
                <div class="mx-1 flex items-center gap-0.5">
                  <IconStarFilled
                    v-for="index in rating"
                    :key="index"
                    class="w-4 h-4 shrink-0"
                  />
                </div>
                <span v-if="ratingCounts[rating]" class="ml-auto text-[10px] tabular-nums text-base-content/30 mr-2">{{ ratingCounts[rating].toLocaleString() }}</span>
              </div>
            </li>
            <li class="pl-4">
              <div
                :class="[
                  'sidebar-item sidebar-item-compact ml-2',
                  libConfig.library.item === LIB_ITEM.RATINGS && libConfig.rating.item === RATE.UNRATED ? 'sidebar-item-selected' : 'sidebar-item-hover',
                ]"
                @click="selectRating(RATE.UNRATED)"
              >
                <IconStar class="mx-1 w-4 h-4 shrink-0" />
                <span class="sidebar-item-label">{{ localeMsg.rating.unrated }}</span>
                <span v-if="unratedCount" class="text-[10px] tabular-nums text-base-content/30 mr-2">{{ unratedCount.toLocaleString() }}</span>
              </div>
            </li>
          </ul>
        </div>
      </Transition>

      <div class="sidebar-item sidebar-item-hover" @click="toggleCulling">
        <IconRight
          :class="[
            'p-1 w-6 h-6 shrink-0 transition-transform',
            libConfig.library.cullingExpanded ? 'rotate-90' : '',
          ]"
          @click.stop="toggleCulling"
        />
        <span class="sidebar-item-label">{{ localeMsg.culling.title }}</span>
      </div>

      <Transition
        @before-enter="onBeforeEnter"
        @enter="onEnter"
        @after-enter="onAfterEnter"
        @leave="onLeave"
      >
        <div v-if="libConfig.library.cullingExpanded" class="overflow-hidden">
          <ul class="mb-2">
            <li v-for="item in cullingItems" :key="item.id" class="pl-4">
              <div
                :class="[
                  'sidebar-item sidebar-item-compact ml-2',
                  libConfig.library.item === LIB_ITEM.CULLING && libConfig.culling.item === item.id ? 'sidebar-item-selected' : 'sidebar-item-hover',
                ]"
                @click="selectCulling(item.id)"
              >
                <component :is="item.icon" class="mx-1 w-4 h-4 shrink-0" />
                <span class="sidebar-item-label">{{ item.label }}</span>
                <span v-if="item.count" class="ml-auto text-[10px] tabular-nums text-base-content/30 mr-2">{{ item.count.toLocaleString() }}</span>
              </div>
            </li>
          </ul>
        </div>
      </Transition>

      <div class="sidebar-item sidebar-item-hover" @click="toggleSubjects">
        <IconRight
          :class="[
            'p-1 w-6 h-6 shrink-0 transition-transform',
            libConfig.library.subjectsExpanded ? 'rotate-90' : '',
          ]"
          @click.stop="toggleSubjects"
        />
        <span class="sidebar-item-label">
          {{ localeMsg.subject.title }}
        </span>
      </div>

      <Transition
        @before-enter="onBeforeEnter"
        @enter="onEnter"
        @after-enter="onAfterEnter"
        @leave="onLeave"
      >
        <div v-if="libConfig.library.subjectsExpanded" class="overflow-hidden">
          <ul class="mb-2">
            <li v-for="item in smartTagItems" :key="item.id" class="pl-4">
              <div
                :class="[
                  'sidebar-item sidebar-item-compact ml-2',
                  libConfig.library.item === LIB_ITEM.SUBJECTS && libConfig.library.smartId === item.id ? 'sidebar-item-selected' : 'sidebar-item-hover',
                ]"
                @click="selectSmartTag(item.id)"
              >
                <IconBolt class="mx-1 w-4 h-4 shrink-0" />
                <span class="sidebar-item-label">{{ item.label }}</span>
                <span v-if="item.count" class="text-[10px] tabular-nums text-base-content/30 mr-2">{{ formatSearchResultCount(item.count) }}</span>
              </div>
            </li>
          </ul>
        </div>
      </Transition>
    </div>

  </div>

</template>

<script setup lang="ts">
import { computed, onMounted, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { config, libConfig } from '@/common/config';
import { useUIStore } from '@/stores/uiStore';
import { createEmptyLibraryCounts } from '@/stores/libraryStore';
import { CULLING, LIB_ITEM, RATE, SIDEBAR, type LibItem } from '@/common/constants';

import { IconFiles, IconHeartFilled, IconRight, IconBolt, IconFlag, IconFlagFilled, IconFlagOff, IconStar, IconStarFilled, IconHistory } from '@/common/icons';
import { SMART_TAG_CATEGORIES } from '@/common/smartTags';
import { getLibraryVisibleCounts } from '@/common/api';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();
const libraryCounts = computed(() => libConfig.library.counts || createEmptyLibraryCounts());
const subjectCounts = computed(() => libConfig.library.subjectCounts || {});
const totalCount = computed(() => Number(libraryCounts.value.all || 0));
const favoriteCount = computed(() => Number(libraryCounts.value.favorite || 0));
const todayCount = computed(() => Number(libraryCounts.value.today || 0));
const unratedCount = computed(() => Number(libraryCounts.value.unrated || 0));
const ratedCount = computed(() => Number(libraryCounts.value.rated || 0));
const cullingCounts = computed(() => libraryCounts.value.culling);
const ratingCounts = computed(() => libraryCounts.value.ratings);
let libraryCountRequest = 0;
async function refreshLibraryCounts() {
  const request = ++libraryCountRequest;
  const libraryId = libConfig._libraryId;
  const counts = await getLibraryVisibleCounts();
  if (
    !counts
    || request !== libraryCountRequest
    || libraryId !== libConfig._libraryId
  ) return;
  libConfig.library.counts = {
    all: counts.all, favorite: counts.favorite, today: counts.today,
    rated: counts.rated, unrated: counts.unrated,
    ratings: { 1: counts.rating1, 2: counts.rating2, 3: counts.rating3, 4: counts.rating4, 5: counts.rating5 },
    culling: { [CULLING.PICK]: counts.pick, [CULLING.REJECT]: counts.reject, [CULLING.UNREVIEWED]: counts.unreviewed },
  };
}
onMounted(() => { void refreshLibraryCounts(); });
watch(() => config.settings.groupRawJpegPairs, () => { void refreshLibraryCounts(); });
const isActiveLibraryView = () => libConfig.activePane === 'main' && config.main.sidebarIndex === SIDEBAR.LIBRARY;
watch(() => [config.main.sidebarIndex, libConfig.activePane], () => {
  if (isActiveLibraryView()) void refreshLibraryCounts();
});
const libraryItems = computed(() => [
  {
    id: LIB_ITEM.ALL,
    label: localeMsg.value.library.all_files,
    icon: IconFiles,
    count: totalCount.value,
  },
  {
    id: LIB_ITEM.TODAY,
    label: localeMsg.value.library.on_this_day,
    icon: IconHistory,
    count: todayCount.value,
  },
  {
    id: LIB_ITEM.FAV,
    label: localeMsg.value.favorite.files,
    icon: IconHeartFilled,
    count: favoriteCount.value,
  },
]);

const smartTagItems = computed(() =>
  SMART_TAG_CATEGORIES.map(category => {
    const item = category.items[0];
    return {
      id: item.id,
      label: localeMsg.value.subject.items?.[item.id] || item.id,
      count: Number(subjectCounts.value[item.id] || 0),
    };
  })
);

const cullingItems = computed(() => [
  { id: CULLING.PICK, label: localeMsg.value.culling.picks, icon: IconFlagFilled, count: cullingCounts.value[CULLING.PICK] },
  { id: CULLING.REJECT, label: localeMsg.value.culling.rejected, icon: IconFlagOff, count: cullingCounts.value[CULLING.REJECT] },
  { id: CULLING.UNREVIEWED, label: localeMsg.value.culling.unreviewed, icon: IconFlag, count: cullingCounts.value[CULLING.UNREVIEWED] },
]);

function formatSearchResultCount(count: number) {
  return count.toLocaleString();
}

function selectItem(item: LibItem) {
  libConfig.library.item = item;
  uiStore.requestCountUpdate({ source: 'library', item });
  libConfig.library.activateTick = Number(libConfig.library.activateTick || 0) + 1;
}

function toggleSubjects() {
  libConfig.library.subjectsExpanded = !libConfig.library.subjectsExpanded;
}

function toggleRatings() {
  libConfig.library.ratingsExpanded = !libConfig.library.ratingsExpanded;
}

function toggleCulling() {
  libConfig.library.cullingExpanded = !libConfig.library.cullingExpanded;
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
  uiStore.requestCountUpdate({ source: 'library', item: LIB_ITEM.RATINGS, rating });
  libConfig.library.activateTick = Number(libConfig.library.activateTick || 0) + 1;
}

function selectCulling(item: string) {
  libConfig.library.item = LIB_ITEM.CULLING;
  libConfig.culling.item = item;
  uiStore.requestCountUpdate({ source: 'library', item: LIB_ITEM.CULLING, cullingItem: item });
  libConfig.library.activateTick = Number(libConfig.library.activateTick || 0) + 1;
}

function selectSmartTag(smartId: string) {
  libConfig.library.item = LIB_ITEM.SUBJECTS;
  libConfig.library.smartId = smartId;
  uiStore.requestCountUpdate({ source: 'library', item: LIB_ITEM.SUBJECTS, smartId });
  libConfig.library.activateTick = Number(libConfig.library.activateTick || 0) + 1;
}

</script>
