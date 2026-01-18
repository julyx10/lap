<template>

  <div class="w-full h-full flex flex-col" style="user-select: none;">
    <!-- location -->
    <div v-if="locations.length > 0" class="flex-1 overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="location in sortedLocations">
          <div 
            :class="[
              'mx-1 p-1 h-10 flex items-center rounded-box whitespace-nowrap cursor-pointer group', 
              libConfig.location.admin1 === location.admin1 && !libConfig.location.name ? 'text-primary bg-base-100 hover:bg-base-100' : 'hover:text-base-content hover:bg-base-100/30',
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
            <span class="flex-1 overflow-hidden whitespace-pre text-ellipsis">{{ location.admin1 + (location.cc ? ', ' + getCountryName(location.cc, locale) : '') }}</span>
            <span class="mx-1 text-xs tabular-nums text-base-content/30">{{ location.counts.reduce((a, b) => a + b, 0).toLocaleString() }}</span>
          </div>
          <ul v-if="location.is_expanded && location.names.length > 0">
            <li v-for="(name, index) in location.names" class="pl-4">
              <div 
                :class="[
                  'ml-3 mr-1 p-1 h-8 flex items-center rounded-box whitespace-nowrap cursor-pointer group', 
                  libConfig.location.name === name ? 'text-primary bg-base-100 hover:bg-base-100' : 'hover:text-base-content hover:bg-base-100/30',
                ]" 
                @click="clickLocationName(location, name)"
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
    <div v-else class="mt-8 px-2 flex flex-col items-center justify-center text-base-content/30">
      <!-- <IconLocation class="w-8 h-8" /> -->
      <span class="text-sm">{{ $t('tooltip.not_found.location') }}</span>
    </div>
  </div>

</template>


<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { config, libConfig } from '@/common/config';
import { getLocationInfo } from '@/common/api';
import { getCountryName } from '@/common/utils';
import { IconRight } from '@/common/icons';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const { locale } = useI18n(); // get locale for country name translation

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