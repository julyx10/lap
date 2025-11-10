<template>
  <div class="relative w-full h-[300px] border border-base-content/30 rounded-lg overflow-hidden">
    <div ref="mapEl" style="width:100%; height:100%;"></div>
    <div class="absolute top-2 left-2 flex bg-base-100/20 hover:bg-base-100/50 rounded-lg z-[1000] cursor-pointer">
      <TButton
        :icon="IconZoomIn"
        :disabled="zoom >= 18"
        @click="zoomIn"
      />
      <TButton
        :icon="IconZoomOut"
        :disabled="zoom <= 0"
        @click="zoomOut"
      />
      <TButton
        :icon="IconMapCenter"
        @click="zoomCenter"
      />
      <TButton
        :icon="config.fileInfo.mapTheme === 0 ? IconMapDefault : IconMapSatellite"
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

// default marker icon
import markerIcon2x from 'leaflet/dist/images/marker-icon-2x.png'
import markerIcon from 'leaflet/dist/images/marker-icon.png'
import markerShadow from 'leaflet/dist/images/marker-shadow.png'

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
    attribution: 'OpenStreetMap', // https://osmfoundation.org/wiki/Licence/Attribution_Guidelines
  },
  {
    name: 'satellite',
    url: 'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}',
    attribution: 'Powered by Esri',
  },
]

const mapEl = ref(null)
let marker = null
let map = null
let layer = null
let zoom = ref(props.zoom)
let resizeObserver = null

onMounted(() => {
  map = L.map(mapEl.value, {
    center: [0, 0],
    zoom: 2,
    // attributionControl: false,
    zoomControl: false,
    maxZoom: 19,
  })
  map.attributionControl.setPrefix('')

  map.on('zoomend', () => {
    zoom.value = map.getZoom()
  })

  // delete default marker icon
  delete L.Icon.Default.prototype._getIconUrl
  // set default marker icon
  L.Icon.Default.mergeOptions({
    iconRetinaUrl: markerIcon2x,
    iconUrl: markerIcon,
    shadowUrl: markerShadow
  })

  resizeObserver = new ResizeObserver(() => {
    if (map) {
      map.invalidateSize()
    }
  })
  resizeObserver.observe(mapEl.value.parentElement)

  updateTheme()
  updateFromProps()
})

onBeforeUnmount(() => {
  if (map) map.remove()
  if (resizeObserver) resizeObserver.disconnect()
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
    if (zoom.value < 18) {
      map.setZoom(zoom.value + 1)
    }
  }
}

function zoomOut() {
  if (map) {
    if (zoom.value > 0) {
      map.setZoom(zoom.value - 1)
    }
  }
}

function zoomCenter() {
  zoom.value = props.zoom
  updateFromProps()
}

function toggleMap() {
  config.fileInfo.mapTheme = config.fileInfo.mapTheme === 0 ? 1 : 0;
  updateTheme();
}

function updateTheme() {
  const theme = mapTheme[Number(config.fileInfo.mapTheme)] || mapTheme[0]
  if (map) {
    if (layer) {
      map.removeLayer(layer)
      layer = null
    }
    layer = L.tileLayer(theme.url, { attribution: theme.attribution }).addTo(map)
  }
}

</script>
