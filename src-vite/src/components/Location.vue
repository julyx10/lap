<template>

  <div class="w-full h-full flex flex-col" style="user-select: none;">
    
    <!-- title bar -->
    <div class="p-1 h-12 flex items-start justify-end whitespace-nowrap" data-tauri-drag-region>
      <TButton
        :icon="config.location.sortCount ? IconSortingCount : IconSortingName"
        @click="config.location.sortCount = !config.location.sortCount"
      />
    </div>

    <!-- location -->
    <div v-if="locations.length > 0" class="flex-1 overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="location in sortedLocations">
          <div 
            :class="[
              'mx-1 p-1 h-10 flex items-center rounded-box whitespace-nowrap cursor-pointer hover:bg-base-100 group', 
              config.location.admin1 === location.admin1 && !config.location.name ? 'text-primary bg-base-100' : 'hover:text-base-content',
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
            <span class="flex-1 overflow-hidden whitespace-pre text-ellipsis">{{ location.admin1 }}</span>
            <span class="mx-1 text-xs tabular-nums text-base-content/30">{{ location.counts.reduce((a, b) => a + b, 0).toLocaleString() }}</span>
          </div>
          <ul v-if="location.is_expanded && location.names.length > 0">
            <li v-for="(name, index) in location.names" class="pl-4">
              <div 
                :class="[
                  'ml-3 mr-1 p-1 h-8 flex items-center rounded-box whitespace-nowrap cursor-pointer hover:bg-base-100 group', 
                  config.location.name === name ? 'text-primary bg-base-100' : 'hover:text-base-content',
                ]" 
                @click="clickLocationName(location.admin1, name)"
              >
                <div class="px-1 whitespace-pre text-ellipsis overflow-hidden">
                  <span>{{ name }}</span>
                  <span class="text-[10px] tabular-nums text-base-content/30 ml-1">({{ location.counts[index].toLocaleString() }})</span>
                </div>
              </div>
            </li>
          </ul>
        </li>
      </ul>
    </div>

    <!-- Display message if no data are found -->
    <div v-else class="flex-1 flex flex-col items-center justify-center text-base-content/30">
      <IconLocation class="w-8 h-8" />
      <span class="mt-2">{{ $t('tooltip.not_found.location') }}</span>
    </div>
  </div>

</template>


<script setup lang="ts">

import { ref, onMounted, computed } from 'vue';
import { config } from '@/common/config';
import { getLocationInfo } from '@/common/api';
import { IconLocation, IconRight, IconSortingCount, IconSortingName } from '@/common/icons';
import TButton from '@/components/TButton.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const locations = ref<any[]>([]);

const sortedLocations = computed(() => {
  if (config.location.sortCount) {
    return [...locations.value].sort((a, b) => {
      const countA = (a.counts || []).reduce((sum: number, c: number) => sum + c, 0);
      const countB = (b.counts || []).reduce((sum: number, c: number) => sum + c, 0);
      return countB - countA;
    });
  }
  return locations.value;
});

onMounted(async () => {
  if (locations.value.length === 0) {
    await getLocations();

    if (locations.value.length === 0) {
      (config.location as any).admin1 = null;
      (config.location as any).name = null;
    }
    
    if(config.location.admin1 && config.location.name) {
      let location = locations.value.find((location: any) => location.admin1 === config.location.admin1)
      if(location) {
        location.is_expanded = true;     // expand selected location
      } else {
        (config.location as any).admin1 = null;
        (config.location as any).name = null;
      }
    }
  }
});

/// reload locations
function clickReload() {
  getLocations();
  (config.location as any).admin1 = "";
  (config.location as any).name = "";
};

/// click location icon to expand or collapse names
function clickExpandLocation(location: any) {
  location.is_expanded = !location.is_expanded; 
};

/// click a location to select it
function clickLocationAdmin1(location: any) {
  (config.location as any).admin1 = location.admin1;
  (config.location as any).name = null;

  location.is_expanded = true;
}

/// click a location to select it
function clickLocationName(admin1: string, name: string) {
  (config.location as any).admin1 = admin1;
  (config.location as any).name = name;
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