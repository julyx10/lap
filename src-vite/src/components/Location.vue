<template>

  <div class="w-full flex flex-col" style="user-select: none;">
    
    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between whitespace-nowrap" data-tauri-drag-region>
      <span class="cursor-default" data-tauri-drag-region>{{ titlebar }}</span>
      <!-- <TButton :icon="IconRefresh" @click="clickReload"/> -->
    </div>

    <!-- location -->
    <div v-if="locations.length > 0" class="flex-1 overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="location in locations">
          <div 
            :class="[
              'my-1 mr-1 h-8 flex items-center rounded border-l-2 border-base-200 hover:bg-base-content/10 whitespace-nowrap cursor-pointer', 
              { 
                'text-base-content': config.location.admin1 === location.admin1, 
                'bg-base-content/10 border-primary'  : config.location.admin1 === location.admin1 && !config.location.name 
              }
            ]"
            @click="clickLocationAdmin1(location)"
          >
            <!-- <component :is="location.is_expanded ? IconFolderExpanded : IconFolder" class="mx-1 h-5  shrink-0" @click.stop="clickExpandLocation(location)"/> -->
            <IconLocation
              :class="[
                'mx-1 h-5 shrink-0 transition-transform', 
              ]"
              @click.stop="clickExpandLocation(location)"
            />
            <div class="overflow-hidden whitespace-pre text-ellipsis">
             {{ location.admin1 }}
            </div>
          </div>
          <ul v-if="location.is_expanded && location.names.length > 0">
            <li v-for="name in location.names" class="pl-4">
              <div 
                :class="[
                  'm-1 pl-3 flex items-center rounded border-l-2 border-base-200 hover:bg-base-content/10 whitespace-nowrap cursor-pointer', 
                  config.location.name === name ? 'bg-base-content/10 border-primary' : ''
                ]" 
                @click="clickLocationName(location.admin1, name)"
              >
                <!-- <IconCircle class="mx-1 h-5 shrink-0" /> -->
                <div class="overflow-hidden whitespace-pre text-ellipsis">
                  {{ name }}
                </div>
              </div>
            </li>
          </ul>
        </li>
      </ul>
    </div>

    <!-- Display message if no data are found -->
    <div v-else class="mt-10 flex flex-col items-center justify-center text-base-content/30">
      <IconLocation class="w-8 h-8" />
      <span class="mt-2">{{ $t('tooltip.not_found.location') }}</span>
    </div>
  </div>

</template>


<script setup lang="ts">

import { ref, onMounted } from 'vue';
import { config } from '@/common/config';
import { getLocationInfo } from '@/common/api';
import { IconRight, IconLocation } from '@/common/icons';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const locations = ref([]);

onMounted(async () => {
  if (locations.value.length === 0) {
    await getLocations();

    if(config.location.admin1 && config.location.name) {
      let location = locations.value.find(location => location.admin1 === config.location.admin1)
      if(location) {
        location.is_expanded = true;     // expand selected location
      } else {
        config.location.admin1 = null;
        config.location.name = null;
      }
    }
  }
});

/// reload locations
function clickReload() {
  getLocations();
  config.location.admin1 = "";
  config.location.name = "";
};

/// click location icon to expand or collapse names
function clickExpandLocation(location) {
  location.is_expanded = !location.is_expanded; 
};

/// click a location to select it
function clickLocationAdmin1(location) {
  config.location.admin1 = location.admin1;
  config.location.name = "";

  location.is_expanded = true;
}

/// click a location to select it
function clickLocationName(admin1, name) {
  config.location.admin1 = admin1;
  config.location.name = name;
}

/// get locations from db
async function getLocations() {
  const fetchedLocations = await getLocationInfo();
  if (fetchedLocations) {
    locations.value = fetchedLocations.map(location => ({
      ...location, 
      is_expanded: false,
    }));
  }
};

</script>