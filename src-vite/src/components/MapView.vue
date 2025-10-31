<template>
  <div class="relative w-full h-[300px] rounded-lg overflow-hidden">
    <div ref="mapEl" style="width:100%; height:100%;"></div>
    <div class="absolute top-2 left-2 flex bg-base-100/20 hover:bg-base-100/80 rounded-lg z-[1000] cursor-pointer transition-colors ease-in-out duration-300 ">
      <TButton
        :icon="IconZoomIn"
        @click="zoomIn"
      />
      <TButton
        :icon="IconZoomOut"
        @click="zoomOut"
      />
      <TButton
        :icon="IconMapCenter"
        @click="zoomCenter"
      />
      <TButton
        :icon="config.map.theme === 0 ? IconMapDefault : IconMapSatellite"
        @click="toggleMap"
      />
    </div>
  </div>
</template>

<script setup>
import { onMounted, onBeforeUnmount, ref, watch } from 'vue'
import { config } from '@/common/utils'

import { IconZoomIn, IconZoomOut, IconMapCenter, IconMapDefault, IconMapSatellite } from '@/common/icons'
import TButton from '@/components/TButton.vue'

import L from 'leaflet'
import 'leaflet/dist/leaflet.css'

// Props: lat/lon from parent; no geolocation fallback
const props = defineProps({
  lat: {
    type: Number, 
    required: false,
    default: 0,
  },
  lon: {
    type: Number, 
    required: false,
    default: 0,
  },
  zoom: {
    type: Number, 
    default: 13,
  },
})

const mapTheme = [
  {
    name: 'standard',
    url: 'https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png',
    attribution: '© OpenStreetMap contributors',
  },
  {
    name: 'satellite',
    url: 'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}',
    attribution: '© ArcGIS',
  },
]

const mapEl = ref(null)
let marker = null
let map = null
let layer = null
let zoom = ref(props.zoom)

onMounted(() => {
  map = L.map(mapEl.value, {
    center: [0, 0],
    zoom: 2,
    // attributionControl: false,
    zoomControl: false,
    maxZoom: 19,
  })
  map.attributionControl.setPrefix('')

  updateTheme()
  updateFromProps()
})

onBeforeUnmount(() => {
  if (map) map.remove()
})

watch(() => [props.lat, props.lon, props.zoom], () => {
  updateFromProps()
})

function updateFromProps() {
  if (!map) return
  if (validLatLon(props.lat, props.lon)) {
    if (marker) {
      marker.setLatLng([props.lat, props.lon])
    } else {
      marker = L.marker([props.lat, props.lon]).addTo(map)
    }
    map.setView([props.lat, props.lon], zoom.value)
  } else {
    // remove marker if any and show world view
    if (marker) {
      map.removeLayer(marker)
      marker = null
    }
    map.setView([0, 0], 2)
  }
}

// validate latitude and longitude
function validLatLon(lat, lon) {
  return lat !== null && lon !== null && lat >= -90 && lat <= 90 && lon >= -180 && lon <= 180
}

function zoomIn() {
  if (map) { 
    map.zoomIn();
    zoom.value = map.getZoom()
  }
}

function zoomOut() {
  if (map) {
    map.zoomOut();
    zoom.value = map.getZoom()
  }
}

function zoomCenter() {
  zoom.value = props.zoom
  updateFromProps()
}

function toggleMap() {
  config.map.theme = config.map.theme === 0 ? 1 : 0;
  updateTheme();
}

function updateTheme() {
  const theme = mapTheme[Number(config.map.theme)] || mapTheme[0]
  if (map) {
    if (layer) {
      map.removeLayer(layer)
      layer = null
    }
    layer = L.tileLayer(theme.url, { attribution: theme.attribution }).addTo(map)
  }
}

</script>
