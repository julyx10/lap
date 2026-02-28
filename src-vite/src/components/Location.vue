<template>

  <div class="sidebar-panel">
    <div class="sidebar-panel-header">
      <span class="sidebar-panel-header-title">{{ localeMsg.sidebar.location }}</span>
      <ContextMenu class="sidebar-panel-action" :menuItems="locationPanelMenuItems" :iconMenu="IconMore" :smallIcon="true" />
    </div>

    <!-- location -->
    <div v-if="locations.length > 0" class="flex-1 overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="location in sortedLocations">
          <div
            :class="[
              'sidebar-item',
              libConfig.location.admin1 === location.admin1 && !libConfig.location.name ? 'sidebar-item-selected' : 'sidebar-item-hover',
            ]"
            @click="clickLocationAdmin1(location)"
          >
            <IconRight
              :class="[
                'p-1 w-6 h-6 shrink-0 transition-transform',
                location.is_expanded ? 'rotate-90' : ''
              ]"
              @click.stop="clickExpandLocation(location)"
            />
            <span class="sidebar-item-label">{{ location.admin1 + (location.cc ? ', ' + getCountryName(location.cc, locale) : '') }}</span>
            <span class="sidebar-item-count">{{ location.counts.reduce((a: number, b: number) => a + b, 0).toLocaleString() }}</span>
          </div>
          <ul v-if="location.is_expanded && location.names.length > 0">
            <li v-for="(name, index) in location.names" class="pl-4">
              <div
                :class="[
                  'sidebar-item sidebar-item-compact ml-3',
                  libConfig.location.name === name ? 'sidebar-item-selected' : 'sidebar-item-hover',
                ]"
                @click="clickLocationName(location, name)"
              >
                <span class="sidebar-item-label">{{ name }}</span>
                <span class="text-[10px] tabular-nums text-base-content/30 ml-1">({{ location.counts[index].toLocaleString() }})</span>
              </div>
            </li>
          </ul>
        </li>
      </ul>
    </div>

    <!-- Display message if no data are found -->
    <div v-else class="mt-8 px-2 flex flex-col items-center justify-center text-base-content/30">
        <IconLocation class="w-8 h-8 mb-2" />
        <span class="text-sm text-center">{{ $t('tooltip.not_found.location') }}</span>
    </div>
  </div>

</template>


<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { config, libConfig } from '@/common/config';
import { getLocationInfo } from '@/common/api';
import { getCountryName } from '@/common/utils';
import { IconDot, IconLocation, IconMore, IconRight } from '@/common/icons';
import ContextMenu from '@/components/ContextMenu.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const { locale, messages } = useI18n(); // get locale for country name translation
const localeMsg = computed(() => messages.value[locale.value] as any);

const locations = ref<any[]>([]);

const sortedLocations = computed(() => {
  if (config.leftPanel.sortCount) {
    return [...locations.value].sort((a, b) => {
      const countA = (a.counts || []).reduce((sum: number, c: number) => sum + c, 0);
      const countB = (b.counts || []).reduce((sum: number, c: number) => sum + c, 0);
      return countB - countA;
    });
  }
  return locations.value;
});

const locationPanelMenuItems = computed(() => [
  {
    label: localeMsg.value.menu.sort.sort_by_name,
    icon: config.leftPanel.sortCount ? null : IconDot,
    action: () => { config.leftPanel.sortCount = false; },
  },
  {
    label: localeMsg.value.menu.sort.sort_by_count,
    icon: config.leftPanel.sortCount ? IconDot : null,
    action: () => { config.leftPanel.sortCount = true; },
  },
]);

onMounted(async () => {
  if (locations.value.length === 0) {
    await getLocations();

    if (locations.value.length === 0) {
      (libConfig.location as any).cc = null;
      (libConfig.location as any).admin1 = null;
      (libConfig.location as any).name = null;
    }
    
    if(libConfig.location.cc && libConfig.location.admin1 && libConfig.location.name) {
      let location = locations.value.find((location: any) => location.admin1 === libConfig.location.admin1)
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
async function getLocations() {
  const fetchedLocations = await getLocationInfo();
  if (fetchedLocations) {
    locations.value = fetchedLocations.map((location: any) => ({
      ...location, 
      is_expanded: false,
    }));
  }
};

</script>
