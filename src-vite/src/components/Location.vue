<template>

  <div class="w-full h-full flex flex-col" style="user-select: none;">
    
    <!-- title bar -->
    <div class="px-1 py-3 h-12 flex items-center justify-end whitespace-nowrap" data-tauri-drag-region>
      <!-- <span class="pl-1 cursor-default" data-tauri-drag-region>{{ titlebar }}</span> -->
      <!-- <TButton :icon="IconRefresh" @click="clickReload"/> -->
      <TButton v-if="config.home.showLeftPane"
        :icon="IconLeftPaneOn"
        @click="config.home.showLeftPane = false"
      />
    </div>

    <!-- location -->
    <div v-if="locations.length > 0" class="flex-1 overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="location in locations">
          <div 
            :class="[
              'mx-1 p-1 h-10 flex items-center rounded-box whitespace-nowrap cursor-pointer hover:bg-base-100 group', 
              config.location.admin1 === location.admin1 && !config.location.name ? 'text-primary' : 'hover:text-base-content',
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
            <div class="flex-1 flex items-center overflow-hidden whitespace-pre text-ellipsis">
              <span>{{ location.admin1 }}</span>
              <span class="text-xs text-base-content/30 ml-1">({{ location.counts.reduce((a, b) => a + b, 0).toLocaleString() }})</span>
            </div>
          </div>
          <ul v-if="location.is_expanded && location.names.length > 0">
            <li v-for="(name, index) in location.names" class="pl-4">
              <div 
                :class="[
                  'ml-3 mr-1 p-1 h-8 flex items-center rounded-box whitespace-nowrap cursor-pointer hover:bg-base-100 group', 
                  config.location.name === name ? 'text-primary' : 'hover:text-base-content',
                ]" 
                @click="clickLocationName(location.admin1, name)"
              >
                <!-- <IconCircle class="mx-1 h-5 shrink-0" /> -->
                <div class="flex-1 flex items-center px-1 whitespace-pre text-ellipsis overflow-hidden">
                  <span>{{ name }}</span>
                  <span class="text-xs text-base-content/30 ml-1">({{ location.counts[index].toLocaleString() }})</span>
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
import { IconLocation, IconLeftPaneOn } from '@/common/icons';
import TButton from '@/components/TButton.vue';

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