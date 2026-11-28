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
          class="cursor-pointer mr-1 p-1 rounded-box text-base-content/30 hover:text-base-content/70"
          @click.stop="search = ''; searchInputRef?.focus()"
        >
          <IconClose class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- list view -->
    <div v-if="sortedLocations.length > 0" class="flex-1 overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="country in countries" :key="country.cc">
          <div :class="['sidebar-item', libConfig.location.cc === country.cc && libConfig.location.admin1 === null ? 'sidebar-item-selected' : 'sidebar-item-hover']" @click="selectCountry(country.cc)">
            <IconRight v-if="country.children.length" :class="['p-1 w-6 h-6 shrink-0 transition-transform', countryExpanded(country.cc) ? 'rotate-90' : '']" @click.stop="toggleCountry(country.cc)" />
            <span v-else class="w-6 shrink-0"></span>
            <span class="sidebar-item-label">{{ country.label }}</span>
            <span class="sidebar-item-count">{{ country.count.toLocaleString() }}</span>
          </div>
          <ul v-if="countryExpanded(country.cc)">
        <li v-for="child in country.children" :key="child.key" class="pl-4">
          <div v-if="child.kind === 'place'"
            :class="['sidebar-item sidebar-item-compact', libConfig.location.cc === country.cc && libConfig.location.admin1 === '' && libConfig.location.name === child.name ? 'sidebar-item-selected' : 'sidebar-item-hover']"
            @click="clickLocationName(child.location, child.name)">
            <span class="w-6 h-6 shrink-0 flex items-center justify-center"><IconLocation class="w-4 h-4" /></span>
            <span class="sidebar-item-label">{{ child.name }}</span>
            <span class="sidebar-item-count">{{ child.count.toLocaleString() }}</span>
          </div>
          <template v-else>
          <template v-for="location in [child.location]" :key="location.admin1">
          <div
            v-if="location.admin1"
            :class="[
              'sidebar-item',
              libConfig.location.cc === location.cc && libConfig.location.admin1 === location.admin1 && libConfig.location.name === null ? 'sidebar-item-selected' : 'sidebar-item-hover',
            ]"
            @click="clickLocationAdmin1(location)"
          >
            <IconRight
              v-if="location.names.some((name: string) => !!name)"
              :class="[
                'p-1 w-6 h-6 shrink-0 transition-transform',
                (location.is_expanded || hasSearch) ? 'rotate-90' : ''
              ]"
              @click.stop="clickExpandLocation(location)"
            />
            <span v-else class="w-6 shrink-0"></span>
            <span class="sidebar-item-label">{{ location.admin1 || localeMsg.search.unknown_location }}</span>
            <span class="sidebar-item-count">{{ location.counts.reduce((a: number, b: number) => a + b, 0).toLocaleString() }}</span>
          </div>
          <ul v-if="!location.admin1 || location.is_expanded || hasSearch">
            <li v-for="(name, index) in location.names" :key="name" :class="location.admin1 ? 'pl-4' : ''">
              <div v-if="name"
                :class="[
                  'sidebar-item sidebar-item-compact ml-2',
                  libConfig.location.cc === location.cc && libConfig.location.admin1 === location.admin1 && libConfig.location.name === name ? 'sidebar-item-selected' : 'sidebar-item-hover',
                ]"
                @click="clickLocationName(location, name)"
              >
                <IconLocation class="mx-1 w-4 h-4 shrink-0" />
                <span class="sidebar-item-label">{{ name || localeMsg.search.unknown_location }}</span>
                <span class="sidebar-item-count">{{ location.counts[index].toLocaleString() }}</span>
              </div>
            </li>
          </ul>
          </template>
          </template>
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
import { useLibraryChangeRefresh } from '@/composables/useLibraryChangeRefresh';
import { SIDEBAR } from '@/common/constants';
import { getCountryName } from '@/common/utils';
import { IconLocation, IconRight, IconSearch, IconClose } from '@/common/icons';
import SortMenuButton from './SortMenuButton.vue';
import { buildCountryChildren } from '@/common/locationTree.js';

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
    const country = String(getCountryName(loc.cc, locale.value) || localeMsg.value.search.unknown_location).toLowerCase();
    if (admin1.includes(q) || country.includes(q)) {
      out.push(loc);
      continue;
    }
    const names = Array.isArray(loc.names) ? loc.names : [];
    const counts = Array.isArray(loc.counts) ? loc.counts : [];
    const keep: number[] = [];
    names.forEach((n: string, i: number) => { if (String(n || '').toLowerCase().includes(q)) keep.push(i); });
    if (keep.length > 0) {
      out.push({ ...loc, names: keep.map((i) => names[i]), counts: keep.map((i) => counts[i]) });
    }
  }
  return out;
});
const expandedCountries = ref(new Set<string>());
const countries = computed(() => {
  const groups = new Map<string, any>();
  for (const region of sortedLocations.value) {
    if (!groups.has(region.cc)) groups.set(region.cc, {
      cc: region.cc, label: getCountryName(region.cc, locale.value) || localeMsg.value.search.unknown_location,
      regions: [], count: 0,
    });
    const country = groups.get(region.cc);
    country.regions.push(region);
    country.count += region.counts.reduce((sum: number, count: number) => sum + count, 0);
  }
  const sort = config.settings.locationSort;
  for (const country of groups.values()) country.children = buildCountryChildren(country.regions, sort, locale.value);
  return [...groups.values()].sort((a, b) => {
    const label = a.label.localeCompare(b.label, locale.value);
    if (sort === 2) return a.count - b.count || label;
    if (sort === 3) return b.count - a.count || label;
    return sort === 1 ? -label : label;
  });
});
function countryExpanded(cc: string) { return hasSearch.value || expandedCountries.value.has(cc); }
function toggleCountry(cc: string) {
  if (expandedCountries.value.has(cc)) expandedCountries.value.delete(cc);
  else expandedCountries.value.add(cc);
}
function selectCountry(cc: string) {
  libConfig.location.cc = cc;
  libConfig.location.admin1 = null;
  libConfig.location.name = null;
  expandedCountries.value.add(cc);
}
const hasSearch = computed(() => !!search.value.trim());

// Header counts the individual places (the deepest level), not countries or regions.
const placeCount = computed(() =>
  sortedLocations.value.reduce((sum: number, loc: any) => sum + (Array.isArray(loc.names) ? loc.names.filter(Boolean).length : 0), 0),
);

onMounted(async () => {
  if (locations.value.length === 0) {
    if (!await getLocations()) return;

    if (locations.value.length === 0) {
      (libConfig.location as any).cc = null;
      (libConfig.location as any).admin1 = null;
      (libConfig.location as any).name = null;
    }
    
    if(libConfig.location.cc !== null && libConfig.location.admin1 !== null && libConfig.location.name !== null) {
      let location = locations.value.find((location: any) => location.admin1 === libConfig.location.admin1 && location.cc === libConfig.location.cc)
      if(location) {
        location.is_expanded = true;     // expand selected location
      } else {
        (libConfig.location as any).cc = null;
        (libConfig.location as any).admin1 = null;
        (libConfig.location as any).name = null;
      }
    }
  }
});

// Keep the current filter even if its last file moves out of this category.
watch(fileInfoRevision, async () => {
  if (libConfig.activePane === 'main' && config.main.sidebarIndex === SIDEBAR.LOCATION) await getLocations(true);
});
useLibraryChangeRefresh(() => getLocations(true));

// Only refresh the active view. Inactive panel data is refreshed on re-entry.
watch(() => [config.settings.locationSort], async () => {
  if (libConfig.activePane === 'main' && config.main.sidebarIndex === SIDEBAR.LOCATION) await getLocations();
});

watch(() => [config.main.sidebarIndex, libConfig.activePane], async () => {
  if (libConfig.activePane === 'main' && config.main.sidebarIndex === SIDEBAR.LOCATION) await getLocations();
});

function restoreLocationSelection() {
  // Empty levels are no longer selectable nodes.
  if (libConfig.location.name === '') libConfig.location.name = null;
  if (libConfig.location.admin1 === '' && libConfig.location.name === null) libConfig.location.admin1 = null;
  if (libConfig.location.cc !== null) expandedCountries.value.add(libConfig.location.cc);
  if (libConfig.location.admin1 === null) return;

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

  if (libConfig.location.name !== null && !location.names.includes(libConfig.location.name)) {
    (libConfig.location as any).name = null;
    if (libConfig.location.admin1 === '') libConfig.location.admin1 = null;
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
      const expanded = new Set(locations.value.filter((location: any) => location.is_expanded)
        .map((location: any) => JSON.stringify([location.cc, location.admin1])));
      locations.value = fetchedLocations.map((location: any) => ({
        ...location,
        is_expanded: expanded.has(JSON.stringify([location.cc, location.admin1])),
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
