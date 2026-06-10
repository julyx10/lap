<template>
  <div class="flex flex-col flex-1 overflow-hidden">
    <div
      class="group/map relative flex-1 border border-base-content/30 rounded-box overflow-hidden m-2"
      @mouseenter="uiStore.setMapActive(true)"
      @mouseleave="uiStore.setMapActive(false)"
    >
      <div v-if="loading" class="absolute inset-0 flex items-center justify-center z-50 bg-base-200/50">
        <span class="loading loading-spinner loading-md text-primary"></span>
      </div>
      <div ref="mapEl" style="width:100%; height:100%;"></div>
      <div class="absolute top-2 left-2 flex bg-base-100/30 hover:bg-base-100/70 rounded-box z-500 cursor-pointer opacity-0 pointer-events-none transition-opacity duration-150 group-hover/map:opacity-100 group-hover/map:pointer-events-auto">
        <TButton :icon="IconZoomOut" :tooltip="t('map.zoom_out')" :disabled="zoom <= 0" @click="zoomOut" />
        <TButton :icon="IconZoomIn" :tooltip="t('map.zoom_in')" :disabled="zoom >= 18" @click="zoomIn" />
        <TButton :icon="IconMapCenter" :tooltip="t('map.zoom_center')" @click="fitBounds" />
        <TButton
          :icon="config.infoPanel.mapTheme === 0 ? IconMapDefault : IconMapSatellite"
          :tooltip="t(config.infoPanel.mapTheme === 0 ? 'map.standard' : 'map.satellite')"
          @click="toggleMap"
        />
      </div>
      <div
        v-if="!loading && points.length === 0"
        class="absolute inset-0 flex items-center justify-center text-base-content/30 px-4"
      >
        <span class="text-sm text-center">{{ $t('tooltip.not_found.location_hint') }}</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { config } from '@/common/config'
import { getGpsCoordinates } from '@/common/api'
import { useUIStore } from '@/stores/uiStore'

import { IconZoomIn, IconZoomOut, IconMapCenter, IconMapDefault, IconMapSatellite } from '@/common/icons'
import TButton from '@/components/TButton.vue'

import L from 'leaflet'
import 'leaflet/dist/leaflet.css'
import 'leaflet.heat'

const { t } = useI18n()
const uiStore = useUIStore()

const mapEl = ref(null)
const loading = ref(true)
const points = ref([])
const zoom = ref(2)

let map = null
let tileLayer = null
let heatLayer = null
let resizeObserver = null
let tileErrorFallbackTriggered = false

const mapThemes = [
  {
    url: 'https://tile.openstreetmap.org/{z}/{x}/{y}.png',
    attribution: 'OpenStreetMap',
  },
  {
    url: 'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}',
    attribution: 'Powered by Esri',
  },
]

onMounted(async () => {
  map = L.map(mapEl.value, {
    center: [20, 0],
    zoom: 2,
    keyboard: false,
    zoomControl: false,
    maxZoom: 19,
  })
  map.attributionControl.setPrefix('')

  map.on('zoomend', () => {
    zoom.value = map.getZoom()
  })

  resizeObserver = new ResizeObserver(() => {
    if (map) map.invalidateSize()
  })
  resizeObserver.observe(mapEl.value.parentElement)

  updateTheme()
  window.addEventListener('keydown', handleMapKeyDown, true)

  const coords = await getGpsCoordinates()
  points.value = coords
  loading.value = false

  if (coords.length > 0) {
    const heatPoints = coords.map(p => [p.lat, p.lon, 1.0])
    heatLayer = L.heatLayer(heatPoints, {
      radius: 20,
      blur: 25,
      maxZoom: 17,
      gradient: { 0.2: 'blue', 0.4: 'cyan', 0.6: 'lime', 0.8: 'yellow', 1.0: 'red' },
    }).addTo(map)

    const latLngs = coords.map(p => [p.lat, p.lon])
    map.fitBounds(L.latLngBounds(latLngs), { padding: [20, 20] })
  }
})

onBeforeUnmount(() => {
  uiStore.setMapActive(false)
  window.removeEventListener('keydown', handleMapKeyDown, true)
  if (map) map.remove()
  if (resizeObserver) resizeObserver.disconnect()
})

watch(() => config.infoPanel.mapTheme, updateTheme)

function updateTheme() {
  const theme = mapThemes[Number(config.infoPanel.mapTheme)] || mapThemes[0]
  if (!map) return
  if (tileLayer) {
    map.removeLayer(tileLayer)
    tileLayer = null
  }
  tileErrorFallbackTriggered = false
  tileLayer = L.tileLayer(theme.url, { attribution: theme.attribution }).addTo(map)
  if (heatLayer) heatLayer.bringToFront()
  tileLayer.on('tileerror', () => {
    if (tileErrorFallbackTriggered) return
    tileErrorFallbackTriggered = true
    if (Number(config.infoPanel.mapTheme) !== 0) {
      config.infoPanel.mapTheme = 0
      updateTheme()
    }
  })
}

function zoomIn() {
  if (map && zoom.value < 18) map.setZoom(zoom.value + 1)
}

function zoomOut() {
  if (map && zoom.value > 0) map.setZoom(zoom.value - 1)
}

function fitBounds() {
  if (!map) return
  if (points.value.length === 0) {
    map.setView([20, 0], 2)
    return
  }
  const latLngs = points.value.map(p => [p.lat, p.lon])
  map.fitBounds(L.latLngBounds(latLngs), { padding: [20, 20] })
}

function toggleMap() {
  config.infoPanel.mapTheme = config.infoPanel.mapTheme === 0 ? 1 : 0
}

function handleMapKeyDown(event) {
  const target = event.target
  if (target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA' || target?.isContentEditable) return
  if (!uiStore.mapActive || event.metaKey || event.ctrlKey || event.altKey) return
  if (event.key === '=') {
    event.preventDefault()
    event.stopPropagation()
    zoomIn()
  } else if (event.key === '-') {
    event.preventDefault()
    event.stopPropagation()
    zoomOut()
  }
}
</script>
