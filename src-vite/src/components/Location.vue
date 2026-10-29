<template>

  <div class="sidebar-panel">
    <div class="sidebar-panel-header">
      <span class="sidebar-panel-header-title flex-1">
        {{ titlebar }}<template v-if="placeCount > 0"> ({{ placeCount.toLocaleString() }})</template>
      </span>
      <SortMenuButton v-model="config.settings.locationSort" kind="category" />
    </div>

    <!-- search -->
    <div v-if="locations.length > 0" class="mx-1 mb-2 px-1 shrink-0">
      <div
        :class="[
          'h-8 flex items-center rounded-box transition-colors bg-base-100/40',
          isSearchFocused ? 'border-2 border-primary' : 'border border-base-content/10 hover:border-base-content/30',
        ]"
        @click="searchInputRef?.focus()"
      >
        <IconSearch
          class="ml-2 w-4 h-4 shrink-0"
          :class="isSearchFocused ? 'text-primary/70' : 'text-base-content/30'"
        />
        <input
          ref="searchInputRef"
          type="text"
          v-model="search"
          :placeholder="$t('menu.location.search')"
          class="w-full min-w-0 bg-transparent border-none focus:ring-0 px-2 text-sm placeholder-base-content/30 focus:outline-none"
          maxlength="255"
          @focus="isSearchFocused = true"
          @blur="isSearchFocused = false"
        />
        <button
          v-if="search"
          type="button"
          class="mr-1 p-1 rounded-box text-base-content/30 hover:text-base-content/70"
          @click.stop="search = ''; searchInputRef?.focus()"
        >
          <IconClose class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- list view -->
    <div v-if="sortedLocations.length > 0" class="flex-1 overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="location in sortedLocations">
          <div
            :class="[
              'sidebar-item',
              libConfig.location.admin1 === location.admin1 && (libConfig.location.cc || '') === location.cc && !libConfig.location.name ? 'sidebar-item-selected' : 'sidebar-item-hover',
            ]"
            @click="clickLocationAdmin1(location)"
          >
            <IconRight
              :class="[
                'p-1 w-6 h-6 shrink-0 transition-transform',
                (location.is_expanded || hasSearch) ? 'rotate-90' : ''
              ]"
              @click.stop="clickExpandLocation(location)"
            />
            <span class="sidebar-item-label">{{ location.admin1 + (location.cc ? ', ' + getCountryName(location.cc, locale) : '') }}</span>
            <span class="sidebar-item-count">{{ (location.parent_count ?? location.counts.reduce((a: number, b: number) => a + b, 0)).toLocaleString() }}</span>
          </div>
          <ul v-if="(location.is_expanded || hasSearch) && location.names.length > 0">
            <li v-for="(name, index) in location.names" class="pl-4">
              <div
                :class="[
                  'sidebar-item sidebar-item-compact ml-2',
                  libConfig.location.name === name && libConfig.location.admin1 === location.admin1 && (libConfig.location.cc || '') === location.cc ? 'sidebar-item-selected' : 'sidebar-item-hover',
                ]"
                @click="clickLocationName(location, name)"
              >
                <IconLocation class="mx-1 w-4 h-4 shrink-0" />
                <span class="sidebar-item-label">{{ name }}</span>
                <span class="sidebar-item-count">{{ location.counts[index].toLocaleString() }}</span>
              </div>
            </li>
          </ul>
        </li>
      </ul>
    </div>

    <!-- Display message if no data are found -->
    <div v-else-if="!isLoadingLocations" class="mt-2 px-2 flex flex-col items-center justify-center text-base-content/30">
        <span class="text-sm text-center">{{ search ? $t('tooltip.not_found.location') : $t('tooltip.not_found.location_hint') }}</span>
    </div>
  </div>

</template>


<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { fileInfoRevision } from '@/common/fileInfoRefresh';
import { config, libConfig } from '@/common/config';
import { getLocationInfo } from '@/common/api';
import { SIDEBAR } from '@/common/constants';
import { getCountryName } from '@/common/utils';
import { IconLocation, IconRight, IconSearch, IconClose } from '@/common/icons';
import SortMenuButton from './SortMenuButton.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const { locale, messages } = useI18n(); // get locale for country name translation
const localeMsg = computed(() => messages.value[locale.value] as any);

const locations = ref<any[]>([]);
const isLoadingLocations = ref(true);
const search = ref('');
const searchInputRef = ref<HTMLInputElement | null>(null);
const isSearchFocused = ref(false);
let isLocationMounted = true;
let locationRequestVersion = 0;

onUnmounted(() => {
  isLocationMounted = false;
  locationRequestVersion++;
});

// Backend returns locations already ordered by locationSort. Search mirrors the
// Tags behavior: a region/country (first-level) match keeps all its places;
// otherwise keep the region with only the matching places (and parallel counts).
const sortedLocations = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return locations.value;
  const out: any[] = [];
  for (const loc of locations.value) {
    const admin1 = String(loc.admin1 || '').toLowerCase();
    const country = String(getCountryName(loc.cc, locale.value) || '').toLowerCase();
    if (admin1.includes(q) || country.includes(q)) {
      out.push(loc);
      continue;
    }
    const names = Array.isArray(loc.names) ? loc.names : [];
    const counts = Array.isArray(loc.counts) ? loc.counts : [];
    const keep: number[] = [];
    names.forEach((n: string, i: number) => { if (String(n).toLowerCase().includes(q)) keep.push(i); });
    if (keep.length > 0) {
      out.push({ ...loc, names: keep.map((i) => names[i]), counts: keep.map((i) => counts[i]) });
    }
  }
  return out;
});
const hasSearch = computed(() => !!search.value.trim());

// Header counts the second level (individual places), not the top-level regions.
const placeCount = computed(() =>
  sortedLocations.value.reduce((sum: number, loc: any) => sum + (Array.isArray(loc.names) ? loc.names.length : 0), 0),
);

onMounted(async () => {
  if (locations.value.length === 0) {
    if (!await getLocations()) return;

    if (locations.value.length === 0) {
      (libConfig.location as any).cc = null;
      (libConfig.location as any).admin1 = null;
      (libConfig.location as any).name = null;
    }
    
    restoreLocationSelection();
  }
});

// Keep the current filter even if its last file moves out of this category.
watch(fileInfoRevision, async () => {
  if (libConfig.activePane === 'main' && config.main.sidebarIndex === SIDEBAR.LOCATION) await getLocations(true);
});

watch(() => config.settings.groupRawJpegPairs, async () => {
  if (libConfig.activePane === 'main' && config.main.sidebarIndex === SIDEBAR.LOCATION) await getLocations(true);
});

// Only refresh the active view. Inactive panel data is refreshed on re-entry.
watch(() => [config.settings.locationSort], async () => {
  if (libConfig.activePane === 'main' && config.main.sidebarIndex === SIDEBAR.LOCATION) await getLocations();
});

watch(() => [config.main.sidebarIndex, libConfig.activePane], async () => {
  if (libConfig.activePane === 'main' && config.main.sidebarIndex === SIDEBAR.LOCATION) await getLocations();
});

function restoreLocationSelection() {
  if (!libConfig.location.admin1) return;

  const location = locations.value.find((item: any) =>
    item.admin1 === libConfig.location.admin1 && item.cc === (libConfig.location.cc || '')
  );

  if (!location) {
    (libConfig.location as any).cc = null;
    (libConfig.location as any).admin1 = null;
    (libConfig.location as any).name = null;
    return;
  }

  location.is_expanded = true;

  if (libConfig.location.name && !location.names.includes(libConfig.location.name)) {
    (libConfig.location as any).name = null;
  }
}

/// click location icon to expand or collapse names
function clickExpandLocation(location: any) {
  location.is_expanded = !location.is_expanded; 
};

/// click a location to select it
function clickLocationAdmin1(location: any) {
  (libConfig.location as any).cc = location.cc;
  (libConfig.location as any).admin1 = location.admin1;
  (libConfig.location as any).name = null;

  location.is_expanded = true;
}

/// click a location to select it
function clickLocationName(location: any, name: string) {
  (libConfig.location as any).cc = location.cc;
  (libConfig.location as any).admin1 = location.admin1;
  (libConfig.location as any).name = name;
}

/// get locations from db
async function getLocations(preserveFilter = false) {
  const requestVersion = ++locationRequestVersion;
  const libraryId = libConfig._libraryId;
  isLoadingLocations.value = true;
  try {
    const fetchedLocations = await getLocationInfo(config.settings.locationSort);
    if (!isLocationMounted || requestVersion !== locationRequestVersion || libraryId !== libConfig._libraryId) return false;
    if (fetchedLocations) {
      locations.value = fetchedLocations.map((location: any) => ({
        ...location,
        is_expanded: false,
      }));
      if (!preserveFilter) restoreLocationSelection();
    }
    return true;
  } finally {
    if (isLocationMounted && requestVersion === locationRequestVersion) {
      isLoadingLocations.value = false;
    }
  }
};

</script>
