<template>
  <div class="flex flex-col flex-1 overflow-hidden">
    <div class="flex flex-1 overflow-hidden">
      <div
        class="group/map relative flex-1 overflow-hidden"
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
          v-if="!loading && points.length > 0"
          class="absolute top-2 right-2 px-2 py-1 rounded-box bg-base-100/60 text-xs text-base-content/70 z-500 pointer-events-none"
        >
          {{ t('map.photo_count', { count: totalCount.toLocaleString() }) }}
        </div>
        <div
          v-if="!loading && points.length === 0"
          class="absolute inset-0 flex items-center justify-center text-base-content/30 px-4"
        >
          <span class="text-sm text-center">{{ $t('tooltip.not_found.location_hint') }}</span>
        </div>
      </div>

      <!-- nearby photos panel: photos visible in the current map view -->
      <div
        class="w-64 shrink-0 flex flex-col overflow-hidden border-l border-base-content/10 bg-base-200"
      >
        <div class="px-2 py-2 text-xs text-base-content/50 shrink-0 border-b border-base-content/10">
          <span v-if="nearbyLoading">{{ t('tooltip.loading') }}</span>
          <span v-else-if="nearbyFiles.length === 0">{{ t('map.no_photos_in_view') }}</span>
          <span v-else-if="nearbyFiles.length >= NEARBY_LIMIT">{{ t('map.photos_in_view_limited', { count: NEARBY_LIMIT }) }}</span>
          <span v-else>{{ t('map.photos_in_view', { count: nearbyFiles.length }) }}</span>
        </div>
        <div class="flex-1 overflow-y-auto p-1">
          <div class="grid grid-cols-3 gap-1">
            <div
              v-for="file in nearbyFiles"
              :key="file.id"
              class="relative aspect-square overflow-hidden rounded-box cursor-pointer bg-base-300"
              @click="openPreview(file)"
              @mouseenter="highlightMarker(file.id)"
              @mouseleave="highlightMarker(null)"
            >
              <img
                class="w-full h-full object-cover"
                :src="getThumbUrl(file.id, false, config.settings.thumbnailSize || 512)"
                loading="lazy"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- lightbox preview -->
    <div
      v-if="previewFile"
      class="fixed inset-0 z-1000 bg-black/80 flex items-center justify-center cursor-pointer"
      @click="previewFile = null"
    >
      <img
        class="max-w-[90%] max-h-[90%] object-contain"
        :src="getPreviewUrl(previewFile.id, previewFile.file_path)"
      />
      <button
        class="absolute top-4 right-4 p-2 rounded-box bg-base-100/30 hover:bg-base-100/70 text-base-content cursor-pointer"
        @click.stop="previewFile = null"
      >
        <IconClose class="w-5 h-5" />
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { config } from '@/common/config'
import { getGpsHeatmapPoints, getQueryFiles } from '@/common/api'
import { getThumbUrl, getPreviewUrl } from '@/common/utils'
import { useUIStore } from '@/stores/uiStore'

import { IconZoomIn, IconZoomOut, IconMapCenter, IconMapDefault, IconMapSatellite, IconClose } from '@/common/icons'
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

const totalCount = computed(() => points.value.reduce((sum, p) => sum + p.count, 0))

const NEARBY_LIMIT = 60

// zoom level from which individual photo markers are drawn on the map
const MARKERS_ZOOM_THRESHOLD = 11

const nearbyFiles = ref([])
const nearbyLoading = ref(false)
const previewFile = ref(null)

let map = null
let tileLayer = null
let heatLayer = null
let markerLayer = null
let resizeObserver = null
let tileErrorFallbackTriggered = false
let nearbyFetchTimer = null
let nearbyFetchToken = 0
const markersByFileId = new Map()

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

  markerLayer = L.layerGroup().addTo(map)

  map.on('zoomend', () => {
    zoom.value = map.getZoom()
    scheduleNearbyFetch()
  })
  map.on('moveend', scheduleNearbyFetch)

  resizeObserver = new ResizeObserver(() => {
    if (map) map.invalidateSize()
  })
  resizeObserver.observe(mapEl.value.parentElement)

  updateTheme()
  window.addEventListener('keydown', handleMapKeyDown, true)

  const gridPoints = await getGpsHeatmapPoints()
  points.value = gridPoints
  loading.value = false

  if (gridPoints.length > 0) {
    // log-scale the per-cell counts so a few very dense spots (e.g. home)
    // don't wash out the rest of the map, then normalize to 0..1
    const weights = gridPoints.map(p => Math.log1p(p.count))
    const maxWeight = Math.max(...weights)
    const heatPoints = gridPoints.map((p, i) => [p.lat, p.lon, weights[i] / maxWeight])

    heatLayer = L.heatLayer(heatPoints, {
      radius: 14,
      blur: 10,
      maxZoom: 17,
      max: 1.0,
      minOpacity: 0.35,
      gradient: { 0.2: '#1d4ed8', 0.4: '#06b6d4', 0.6: '#22c55e', 0.8: '#facc15', 1.0: '#ef4444' },
    }).addTo(map)

    const latLngs = gridPoints.map(p => [p.lat, p.lon])
    map.fitBounds(L.latLngBounds(latLngs), { padding: [20, 20] })
  }

  zoom.value = map.getZoom()
  scheduleNearbyFetch()
})

onBeforeUnmount(() => {
  uiStore.setMapActive(false)
  window.removeEventListener('keydown', handleMapKeyDown, true)
  if (nearbyFetchTimer) clearTimeout(nearbyFetchTimer)
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
  if (markerLayer) markerLayer.eachLayer(m => m.bringToFront())
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

// debounce fetching nearby photos while the user pans/zooms the map
function scheduleNearbyFetch() {
  if (nearbyFetchTimer) clearTimeout(nearbyFetchTimer)
  nearbyFetchTimer = setTimeout(fetchNearbyFiles, 300)
}

async function fetchNearbyFiles() {
  if (!map) return

  const token = ++nearbyFetchToken
  nearbyLoading.value = true

  const bounds = map.getBounds()
  const params = {
    searchFileName: '',
    searchFileType: 0,
    sortType: 0,
    sortOrder: 1,
    searchAllSubfolders: '',
    searchFolder: '',
    startDate: 0,
    endDate: 0,
    calendarSort: 0,
    make: '',
    model: '',
    lensMake: '',
    lensModel: '',
    locationAdmin1: '',
    locationName: '',
    isFavorite: false,
    rating: -1,
    tagId: 0,
    personId: 0,
    gpsMinLat: bounds.getSouth(),
    gpsMaxLat: bounds.getNorth(),
    gpsMinLon: bounds.getWest(),
    gpsMaxLon: bounds.getEast(),
  }

  const files = await getQueryFiles(params, 0, NEARBY_LIMIT)
  if (token !== nearbyFetchToken) return // stale response

  nearbyFiles.value = files || []
  nearbyLoading.value = false
  renderMarkers()
}

function renderMarkers() {
  clearMarkers()
  if (!markerLayer) return
  if (zoom.value < MARKERS_ZOOM_THRESHOLD) return
  for (const file of nearbyFiles.value) {
    if (file.gps_latitude == null || file.gps_longitude == null) continue
    const marker = L.circleMarker([file.gps_latitude, file.gps_longitude], {
      radius: 5,
      weight: 1,
      color: '#ffffff',
      fillColor: '#ef4444',
      fillOpacity: 0.9,
    })
    marker.on('click', () => openPreview(file))
    marker.addTo(markerLayer)
    markersByFileId.set(file.id, marker)
  }
}

function clearMarkers() {
  if (markerLayer) markerLayer.clearLayers()
  markersByFileId.clear()
}

function highlightMarker(fileId) {
  for (const [id, marker] of markersByFileId) {
    marker.setStyle({ radius: id === fileId ? 8 : 5, fillColor: id === fileId ? '#facc15' : '#ef4444' })
  }
}

function openPreview(file) {
  previewFile.value = file
}

function handleMapKeyDown(event) {
  const target = event.target
  if (target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA' || target?.isContentEditable) return
  if (event.key === 'Escape' && previewFile.value) {
    previewFile.value = null
    return
  }
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
