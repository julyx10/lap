<template>
  <div class="w-full h-full rounded-box bg-base-200 flex flex-col overflow-hidden">
    <!-- Header & Close -->
    <div class="my-2 px-2 flex items-center w-full shrink-0">
      <div class="flex-1 pl-1">
        <span class="text-sm font-semibold text-primary/70">
          {{ $t('file_info.title') }}
        </span>
      </div>
      <div class="flex items-center gap-1">
        <TButton
          :icon="showDelayedLoading ? LoadingIcon : IconRefresh"
          :tooltip="$t(showDelayedLoading ? 'tooltip.loading' : 'toolbar.tooltip.refresh')"
          :buttonSize="'small'"
          :disabled="!fileInfo || fileInfoBusy"
          @click.stop="refreshInfo"
        />
        <TButton
          :icon="IconClose"
          :tooltip="$t('msgbox.close')"
          :buttonSize="'small'"
          @click.stop="$emit('close')"
        />
      </div>
    </div>

    <!-- Info Content -->
    <div v-if="fileInfo" class="mb-2 px-2 flex-1 overflow-y-auto overflow-x-hidden flex flex-col [overflow-anchor:none]">

      <!-- Preview Section -->
      <div class="group/thumbnail shrink-0 border-t border-base-content/5 px-1 py-3">
        <div
          class="flex items-center gap-1 cursor-pointer text-base-content/70 hover:text-base-content transition-all duration-200 ease-in-out"
          @click.stop="togglePreview"
        >
          <IconRight
            class="w-3 h-3 transition-transform duration-200"
            :class="{ 'rotate-90': showPreviewPanel }"
            @click.stop="togglePreview"
          />
          <span class="py-1 font-bold mr-auto uppercase text-xs tracking-wide text-base-content/30">
            {{ $t('file_info.preview') }}
          </span>
          <button v-if="showPreviewPanel && isVideoFile" type="button"
            class="tab h-6 text-xs shrink-0 hover:text-primary cursor-pointer disabled:opacity-40 disabled:cursor-wait"
            :disabled="!canChangeThumbnail || changingThumbnail || fileInfoBusy || !changeThumbnail || isOriginalUnavailable(fileInfo)"
            :aria-busy="changingThumbnail"
            @click.stop="changePreviewThumbnail"
          >{{ $t('file_info.change_thumbnail') }}{{ changingThumbnail ? '…' : '' }}</button>
          <div v-else-if="showPreviewPanel" role="tablist" class="tabs tabs-xs shrink-0">
            <button
              role="tab"
              :class="['tab', !isHistogramPreview ? 'tab-active text-primary' : '']"
              @click.stop="setPreviewMode('thumbnail')"
            >{{ $t('file_info.thumbnail') }}</button>
            <button
              v-if="canShowHistogram"
              role="tab"
              :class="['tab', isHistogramPreview ? 'tab-active text-primary' : '']"
              @click.stop="setPreviewMode('histogram')"
            >{{ $t('file_info.histogram') }}</button>
          </div>
        </div>

        <SmoothHeight>
          <div v-if="showPreviewPanel" class="pt-3 overflow-hidden">
            <div
              v-if="!isHistogramPreview"
              class="relative w-full overflow-hidden rounded-box border border-base-content/5 shadow-sm transition-[padding-top] duration-200 ease-out"
              :style="{ paddingTop: `${75 * previewScale}%` }"
              @pointerenter="playVideoPreviewOnHover"
              @pointerleave="stopPreviewVideo"
            >
              <span v-if="previewFormatLabel" class="thumb-badge pointer-events-none absolute right-1.5 top-1.5 z-20">
                <IconLivePhoto v-if="isLivePhoto || isMotionPhoto" class="h-3.5 w-3.5 shrink-0" />
                <IconVideo v-else-if="isVideoFile" class="h-3.5 w-3.5 shrink-0" />
                {{ previewFormatLabel }}
              </span>
              <div
                class="absolute top-2 left-2 flex bg-base-100/30 hover:bg-base-100/70 rounded-box z-20 cursor-pointer opacity-0 pointer-events-none transition-opacity duration-150 group-hover/thumbnail:opacity-100 group-hover/thumbnail:pointer-events-auto"
              >
                <TButton
                  :icon="IconZoomOut"
                  :tooltip="$t('map.zoom_out')"
                  :disabled="previewScale <= previewScaleOptions[previewScaleOptions.length - 1]"
                  @click.stop="decreasePreviewScale"
                />
                <TButton
                  :icon="IconZoomIn"
                  :tooltip="$t('map.zoom_in')"
                  :disabled="previewScale >= previewScaleOptions[0]"
                  @click.stop="increasePreviewScale"
                />
              </div>
              <div class="absolute" :style="previewContentStyle">
                <ImageNavigator
                  v-if="hasPreviewNavigator"
                  class="h-full w-full"
                  :source="fileInfo.thumbnail"
                  :viewport="navigatorViewport"
                  :sharpness="config.settings.navigatorSharpnessGrid"
                  @navigate="emit('navigator-navigate', $event)"
                  @zoom="emit('navigator-zoom', $event)"
                  @toggle-fit="emit('navigator-toggle-fit')"
                />
                <img
                  v-else-if="fileInfo?.thumbnail"
                  :src="fileInfo.thumbnail"
                  @load="onPreviewImageLoad"
                  class="h-full w-full object-contain"
                  :style="previewImageStyle"
                  draggable="false"
                />
                <video
                  v-if="showVideoPreview"
                  ref="previewVideoRef"
                  class="pointer-events-none absolute inset-0 h-full w-full object-contain"
                  :class="isVideoPreviewReady ? 'opacity-100' : 'opacity-0'"
                  :style="previewImageStyle"
                  :poster="fileInfo?.thumbnail"
                  muted
                  autoplay
                  loop
                  playsinline
                  preload="metadata"
                  @canplay="isVideoPreviewReady = true"
                  @playing="isVideoPreviewReady = true"
                  @error="stopPreviewVideo"
                ></video> 
                <div v-if="!fileInfo?.thumbnail && !showVideoPreview" class="flex h-full w-full items-center justify-center bg-base-content/5">
                  <component
                    :is="fileInfo?.file_type === 2 ? IconVideo : IconPhoto"
                    class="w-10 h-10 text-base-content/30"
                  />
                </div>
              </div>
            </div>
            <div v-else class="rounded-box border border-base-content/5 bg-base-300/30 p-3 shadow-sm">
              <ImageHistogram :source="fileInfo?.thumbnail || ''" />
            </div>
          </div>
        </SmoothHeight>
      </div>

      <div class="shrink-0 border-t border-base-content/5 px-1 py-3">
        <button type="button" class="flex w-full items-center gap-1 cursor-pointer text-base-content/70 hover:text-base-content" :aria-expanded="showOrganizationPanel" @click="toggleOrganization">
          <IconRight class="w-3 h-3 transition-transform duration-200" :class="{ 'rotate-90': showOrganizationPanel }" />
          <span class="font-bold uppercase text-xs tracking-wide text-base-content/30">{{ $t('file_info.organization') }}</span>
        </button>
        <SmoothHeight>
          <div v-if="showOrganizationPanel" class="pt-3 space-y-1.5">
            <FavoriteRatingControl class="pl-4"
              :favorite="Boolean(fileInfo?.is_favorite)"
              :rating="Number(fileInfo?.rating || 0)"
              :culling="Number(fileInfo?.culling_flag ?? fileInfo?.cullingFlag ?? 0)"
              label-class="text-base-content/30"
              inactive-rating-class="text-base-content/70"
              @favorite="emit('toggleFavorite')"
              @rating="(rating) => emit('setRating', rating)"
              @culling="(cullingFlag) => emit('setCulling', cullingFlag)"
            />

            <!-- Tags -->
            <template v-if="displayTags.length">
              <div class="pl-4 flex items-center min-h-8 gap-1">
                <div class="text-[12px] text-base-content/70 flex flex-wrap gap-1 flex-1 min-w-0 cursor-pointer" @click.stop="emit('quickEditTag')">
                  <span
                    v-for="tag in displayTags"
                    :key="tag.id"
                    :title="tag.group_name"
                    class="badge badge-sm badge-outline border-base-content/5 bg-base-content/5 font-medium text-base-content/70 hover:text-base-content"
                  >{{ tag.name }}</span>
                </div>
              </div>
            </template>

          <div class="pl-4 grid grid-cols-[84px_minmax(0,1fr)] gap-y-1.5 gap-x-4 text-xs">
            <!-- Collections -->
            <template v-if="fileCollections.length">
              <div class="flex items-center text-[11px] text-base-content/30 min-h-6 py-1.5">{{ $t('collection.title') }}</div>
              <div class="group/field flex items-center min-h-6 gap-1">
                <div class="flex items-center min-h-6 gap-x-3 gap-y-1 flex-wrap flex-1 min-w-0">
                  <button
                    v-for="collection in fileCollections"
                    :key="collection.id"
                    type="button"
                    class="inline-flex items-center gap-1 text-[12px] font-medium text-base-content/70 transition-colors hover:text-base-content cursor-pointer"
                    @click.stop="emit('quickEditCollection')"
                  >
                    <IconStack class="h-4 w-4 shrink-0" />
                    {{ collection.name }}
                  </button>
                </div>
              </div>
            </template>

            <!-- Editable description -->
            <template v-if="fileInfo?.comments?.trim()">
              <div class="flex items-center text-[11px] text-base-content/30 py-1.5">{{ $t('file_info.comment') }}</div>
              <div class="group/field flex items-center gap-1">
                <button type="button" class="text-left text-[12px] leading-5 text-base-content/70 wrap-break-word whitespace-pre-wrap flex-1 min-w-0 cursor-pointer hover:text-base-content" @click.stop="emit('quickEditComment')">{{ fileInfo?.comments ?? '' }}</button>
              </div>
            </template>

            <!-- Rotate Display -->
            <template v-if="fileInfo?.rotate && fileInfo?.rotate !== 0">
              <div class="flex items-center text-[11px] text-base-content/30 h-6">{{ $t('menu.meta.rotate') }}</div>
              <div class="flex items-center gap-2 min-h-6">
                <span class="text-[12px] text-base-content/70">{{ normalizedRotate }}°</span>
                <TButton
                  :icon="IconRotate"
                  :tooltip="$t('menu.meta.rotate')"
                  :buttonSize="'small'"
                  @click.stop="emit('rotate')"
                />
              </div>
            </template>

            <!-- People -->
            <template v-if="config.settings.face.enabled && filePersons.length">
              <div class="flex items-center text-[11px] text-base-content/30 min-h-6 py-1.5">{{ $t('sidebar.people') }}</div>
              <div class="flex items-center min-h-6 gap-x-3 gap-y-1 flex-wrap">
                <button
                  v-for="person in filePersons"
                  :key="person.id"
                  type="button"
                  class="inline-flex items-center gap-1.5 text-[12px] font-medium text-base-content/70 transition-colors hover:text-base-content cursor-pointer"
                  @click.stop="navigatePerson(person)"
                >
                  <span class="w-5 h-5 rounded-full overflow-hidden bg-base-300/70 ring-1 ring-base-content/5 shrink-0 flex items-center justify-center">
                    <img
                      v-if="person.thumbnail"
                      :src="'data:image/jpeg;base64,' + person.thumbnail"
                      class="w-full h-full object-cover"
                      draggable="false"
                    />
                    <IconPerson v-else class="w-3.5 h-3.5 text-base-content/30" />
                  </span>
                  {{ person.name || person.id }}
                </button>
              </div>
            </template>

          </div>
          </div>
        </SmoothHeight>
      </div>

      <!-- File Info Section -->
      <div class="group/general shrink-0 border-t border-base-content/5 px-1 py-3">

        <div class="flex items-center gap-1 cursor-pointer text-base-content/70 hover:text-base-content transition-all duration-200 ease-in-out" 
          @click.stop="toggleBasicInfo"
        >
          <IconRight
            class="w-3 h-3 transition-transform duration-200"
            :class="{ 'rotate-90': showBasicInfoPanel }"
            @click.stop="toggleBasicInfo"
          />
          <span class="py-1 font-bold mr-auto uppercase text-xs tracking-wide text-base-content/30">{{ $t('file_info.file') }}</span>
          <div v-if="showBasicInfoPanel && isRawJpegPair" role="tablist" class="tabs tabs-xs shrink-0">
            <button
              role="tab"
              :class="['tab', generalInfoTab === 'raw' ? 'tab-active text-primary' : '']"
              @click.stop="generalInfoTab = 'raw'"
            >RAW</button>
            <button
              role="tab"
              :class="['tab', generalInfoTab === 'companion' ? 'tab-active text-primary' : '']"
              :disabled="!rawJpegCompanion"
              @click.stop="generalInfoTab = 'companion'"
            >{{ rawJpegCompanionLabel }}</button>
          </div>
        </div>

        <SmoothHeight>
          <div v-if="showBasicInfoPanel" class="pt-3 overflow-hidden">
            <div class="pl-4 grid grid-cols-[84px_minmax(0,1fr)] gap-y-1.5 gap-x-4 text-xs">
            <!-- Name -->
            <div class="flex items-center text-[11px] text-base-content/30 h-6">{{ $t('file_info.name') }}</div>
            <div class="group/field flex items-center gap-1">
              <div
                v-if="isPrimaryGeneralInfo && isRenaming"
                class="flex items-center w-full min-w-0"
              >
                <input
                  ref="renameInputRef"
                  v-model="renamingName"
                  class="text-[12px] text-base-content input input-xs input-bordered p-1 h-6 leading-6 w-full min-w-0"
                  @blur="finishRename"
                  @keydown.enter.stop="finishRename"
                  @keydown.esc.stop="cancelRename"
                  @click.stop
                />
                <span
                  v-if="renamingExt"
                  class="ml-1 text-[12px] text-base-content/70 whitespace-nowrap"
                >.{{ renamingExt }}</span>
              </div>
              <span v-else
                class="flex items-center gap-1 text-[12px] font-medium text-base-content/70 hover:text-base-content flex-1 min-w-0"
                :class="{ 'cursor-text': isPrimaryGeneralInfo }"
                @dblclick.stop="isPrimaryGeneralInfo && startRename()"
              >
                <span class="min-w-0 break-all">{{ generalFileInfo?.name }}</span>
                <IconExclamation v-if="isOriginalUnavailable(generalFileInfo)" class="h-3.5 w-3.5 shrink-0 text-warning/70" :aria-label="t('offline.original_unavailable')" />
              </span>
            </div>

            <!-- Path -->
            <div class="flex items-center text-[11px] text-base-content/30 h-6">{{ $t('file_info.folder') }}</div>
            <Breadcrumb
              :icon="IconFolder"
              :items="generalFolderBreadcrumbs"
              @navigate="(path: string) => emit('navigateFolder', path)"
            />

            <template v-if="generalFileInfo?.album_name?.trim()">
            <!-- Album -->
            <div class="flex items-center text-[11px] text-base-content/30 h-6">{{ $t('file_info.album_name') }}</div>
            <div class="flex items-center min-w-0 text-[12px] text-base-content/70 break-all">{{ generalFileInfo?.album_name }}</div>
            </template>

            <template v-if="generalFileInfo?.size != null">
            <!-- Size -->
            <div class="flex items-center text-[11px] text-base-content/30 h-6">{{ $t('file_info.size') }}</div>
            <div class="flex items-center text-[12px] text-base-content/70">{{ formatFileSize(generalFileInfo?.size) }}</div>
            </template>

            <template v-if="generalFileInfo?.width && generalFileInfo?.height">
            <!-- Dimension -->
            <div class="flex items-center text-[11px] text-base-content/30 h-6">{{ $t('file_info.dimension') }}</div>
            <div class="flex items-center text-[12px] text-base-content/70">{{ formatDimensionText(generalFileInfo?.width, generalFileInfo?.height, true) }}</div>
            </template>

            <template v-for="row in imageDetailRows" :key="row.key">
              <div class="text-[11px] text-base-content/30 min-h-6 py-1">{{ row.label }}</div>
              <div class="min-h-6 text-[12px] text-base-content/70 wrap-break-word py-1">{{ row.value }}</div>
            </template>
            <!-- Duration -->
            <template v-if="generalFileInfo?.file_type === 2">
              <div class="flex items-center text-[11px] text-base-content/30 h-6">{{ $t('file_info.duration') }}</div>
              <div class="flex items-center text-[12px] text-base-content/70">{{ Number.isFinite(generalFileInfo?.duration) && generalFileInfo.duration > 0 ? formatDuration(generalFileInfo.duration) : $t('file_info.unknown_duration') }}</div>
            </template>

            <div class="col-span-2">
              <div class="grid grid-cols-[84px_1fr] gap-y-1.5 gap-x-4">
                <template v-if="generalFileInfo?.created_at != null">
                <!-- Created At -->
                <div class="flex items-center text-[11px] text-base-content/30 h-6">{{ $t('file_info.created_at') }}</div>
                <div class="flex items-center text-[12px] text-base-content/70">{{ formatTimestamp(generalFileInfo?.created_at, 'date_time') }}</div>
                </template>

                <template v-if="generalFileInfo?.modified_at != null">
                <!-- Modified At -->
                <div class="flex items-center text-[11px] text-base-content/30 h-6">{{ $t('file_info.modified_at') }}</div>
                <div class="flex items-center text-[12px] text-base-content/70">{{ formatTimestamp(generalFileInfo?.modified_at, 'date_time') }}</div>
                </template>

                <!-- Last Scan -->
                <template v-if="generalFileInfo?.last_scan_time && generalFileInfo.last_scan_time > 0">
                  <div class="flex items-center text-[11px] text-base-content/30 h-6">{{ $t('file_info.last_scan_time') }}</div>
                  <div class="flex min-h-6 items-center gap-2">
                    <!-- <span class="text-[12px] text-base-content/70">{{ formatTimestamp(fileInfo.last_scan_time / 1000, 'date_time') }}</span> -->
                    <span class="text-[11px] text-base-content/30">{{ formatRelativeTime(generalFileInfo.last_scan_time / 1000, $t) }}</span>
                  </div>
                </template>
              </div>
            </div>

            
            </div>
          </div>
        </SmoothHeight>
      </div>

      <!-- Capture information -->
      <SmoothHeight>
      <div v-if="hasCaptureInfo" class="border-t border-base-content/5 px-1 py-4">

        <div class="flex items-center gap-1 cursor-pointer text-base-content/70 hover:text-base-content" @click.stop="toggleMetadata">
          <IconRight
            class="w-3 h-3 transition-transform duration-200"
            :class="{ 'rotate-90': showMetadataPanel }"
            @click.stop="toggleMetadata"
          />
          <span class="font-bold mr-auto uppercase text-xs tracking-wide text-base-content/30">{{ $t('file_info.capture') }}</span>
        </div>

          <div v-if="showMetadataPanel" class="pt-3 pl-4 grid grid-cols-[84px_1fr] gap-y-1.5 gap-x-4 text-xs overflow-hidden">
            <template v-if="captureTimeText">
              <!-- Taken At -->
            <div class="flex items-center text-[11px] text-base-content/30 h-6">{{ $t('file_info.taken_at') }}</div>
            <div class="flex items-center text-[12px] text-base-content/70">{{ captureTimeText }}</div>
            </template>

            <template v-if="cameraText.trim()">
            <!-- Camera -->
            <div class="flex items-center text-[11px] text-base-content/30 h-6">{{ $t('file_info.camera') }}</div>
            <div
              :class="['flex items-center text-[12px] text-base-content/70', hasCamera ? 'cursor-pointer hover:text-base-content' : '']"
              @click.stop="navigateCamera"
            >{{ cameraText }}</div>
            </template>

            <template v-if="fileInfo?.e_lens_model?.trim()">
            <!-- Lens -->
            <div class="flex items-center text-[11px] text-base-content/30 h-6">{{ $t('file_info.lens') }}</div>
            <div
              :class="['flex items-center text-[12px] text-base-content/70', hasLens ? 'cursor-pointer hover:text-base-content' : '']"
              @click.stop="navigateLens"
            >{{ fileInfo?.e_lens_model }}</div>
            </template>

            <template v-if="exposureText.trim()">
            <!-- Capture Settings -->
            <div class="flex items-center text-[11px] text-base-content/30 h-6">{{ $t('file_info.capture_settings') }}</div>
            <div class="flex items-center text-[12px] text-base-content/70">{{ exposureText }}</div>
            </template>



            <template v-for="row in captureDetailRows" :key="row.key">
              <div class="text-[11px] text-base-content/30 min-h-6 py-1">{{ row.label }}</div>
              <div class="text-[12px] text-base-content/70 wrap-break-word py-1">{{ row.value }}</div>
            </template>

        </div>
      </div>
      </SmoothHeight>

      <SmoothHeight>
      <div v-if="hasLocationInfo" class="border-t border-base-content/5 px-1 py-3">
        <button type="button" class="flex w-full items-center gap-1 cursor-pointer text-base-content/70 hover:text-base-content" :aria-expanded="showMapPanel" @click="toggleMapPanel">
          <IconRight class="w-3 h-3 transition-transform duration-200" :class="{ 'rotate-90': showMapPanel }" />
          <span class="font-bold uppercase text-xs tracking-wide text-base-content/30">{{ $t('file_info.geo_location') }}</span>
        </button>
        <div v-if="showMapPanel" class="pt-3 pl-4 grid grid-cols-[84px_minmax(0,1fr)] gap-y-1.5 gap-x-4 text-xs">
            <template v-if="formatGeoLocation().trim()">
            <!-- Geo Location -->
            <div class="flex items-center text-[11px] text-base-content/30 h-6">{{ $t('file_info.geo_location') }}</div>
            <div
              :class="['flex items-center text-[12px] text-base-content/70', hasLocation ? 'cursor-pointer hover:text-base-content' : '']"
              @click.stop="navigateLocation"
            >{{ formatGeoLocation() }}</div>
            </template>

            <template v-if="hasCoordinates || hasAltitude">
              <div class="flex items-center text-[11px] text-base-content/30 h-6">GPS</div>
              <div class="text-[12px] text-base-content/70 wrap-break-word">
                <span v-if="hasCoordinates">{{ formatGpsCoordinates(fileInfo.gps_latitude, fileInfo.gps_longitude) }}</span>
                <span v-if="hasCoordinates && hasAltitude"> · </span>
                <span v-if="hasAltitude">{{ Number(Number(fileInfo.gps_altitude).toFixed(2)) }} m</span>
              </div>
            </template>
            <template v-for="row in locationDetailRows" :key="row.key">
              <div class="text-[11px] text-base-content/30 min-h-6 py-1">{{ row.label }}</div>
              <div class="text-[12px] text-base-content/70 wrap-break-word py-1">{{ row.value }}</div>
            </template>
            <template v-if="hasCoordinates">
              <div class="col-span-2 h-75 rounded-box overflow-hidden relative z-0 border border-base-content/5">
                <MapView :lat="Number(fileInfo.gps_latitude)" :lon="Number(fileInfo.gps_longitude)" :label="fileInfo.geo_name || fileInfo.name || 'Lap'" />
              </div>
            </template>
        </div>
      </div>
      </SmoothHeight>

      <SmoothHeight>
      <div v-if="hasSourceMetadata" class="border-t border-base-content/5 px-1 py-3">
        <button type="button" class="flex w-full items-center gap-1 cursor-pointer text-base-content/70 hover:text-base-content" :aria-expanded="showSourceMetadataPanel" @click="toggleSourceMetadata">
          <IconRight class="w-3 h-3 transition-transform duration-200" :class="{ 'rotate-90': showSourceMetadataPanel }" />
          <span class="font-bold uppercase text-xs tracking-wide text-base-content/30">{{ $t('file_info.source_metadata') }}</span>
        </button>
        <div v-if="showSourceMetadataPanel" class="pt-3 pl-4 grid grid-cols-[84px_minmax(0,1fr)] gap-y-1.5 gap-x-4 text-xs">
            <template v-for="row in sourceDetailRows" :key="row.key">
              <div class="text-[11px] text-base-content/30 min-h-6 py-1">{{ row.label }}</div>
              <div class="text-[12px] text-base-content/70 whitespace-pre-wrap wrap-break-word py-1">{{ row.value }}</div>
            </template>

        </div>
      </div>
      </SmoothHeight>

      <button v-if="detailsError" type="button" class="px-1 text-left text-xs text-warning" @click="loadMetadataDetails(true)">{{ $t('file_info.metadata_details_retry') }}</button>
    </div>

    <div v-else class="mb-2 px-2 flex-1 overflow-y-auto overflow-x-hidden flex flex-col">
      <div class="p-4 flex-1 flex items-center justify-center">
        <div class="text-center text-base-content/30 space-y-3 max-w-65">
          <IconFile class="w-8 h-8 mx-auto text-base-content/30" />
          <p class="text-xs font-medium">{{ $t('file_info.empty_title') }}</p>
          <p class="text-xs text-base-content/30">{{ $t('file_info.empty_desc') }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import ImageNavigator from '@/components/ImageNavigator.vue';
import SmoothHeight from '@/components/SmoothHeight.vue';
import { invoke } from '@tauri-apps/api/core';
import { navigatorLayout } from '@/common/imageNavigator';
import { h, ref, nextTick, computed, watch, onBeforeUnmount, onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { useToast } from '@/common/toast';
import { useUIStore } from '@/stores/uiStore';
import { config, libConfig } from '@/common/config';
import { isWebViewVideoPlaybackDisabled, getGStreamerAvailability } from '@/common/video';
import { isOriginalUnavailable } from '@/common/availability';
import { getTagsForFile, renameFile, editImage, getAlbum, getFileCollections, getFileInfo, getMotionPhotoVideoPath, revealPath, getFacesForFile, getPersonThumbnail, isCloudOnlyFile } from '@/common/api';
import { 
  extractFileName, 
  getFileExtension,
  getFolderPath,
  buildFolderBreadcrumbs,
  formatDimensionText, 
  formatFileSize, 
  formatTimestamp,
  formatRelativeTime,
  formatDuration,
  formatCaptureSettings,
  formatCameraInfo,
  formatGpsCoordinates,
  getCountryName,
  combineFileName,
  isValidFileName,
  getAssetSrc,
} from '@/common/utils';
import {
  IconClose,
  IconRefresh,
  IconRight,
  IconFile,
  IconExclamation,
  IconFolder,
  IconPhoto,
  IconRotate,
  IconVideo,
  IconZoomIn,
  IconZoomOut,
  IconLivePhoto,
  IconStack,
  IconPerson,
} from '@/common/icons';
import Breadcrumb from '@/components/Breadcrumb.vue';
import TButton from '@/components/TButton.vue';
import FavoriteRatingControl from '@/components/FavoriteRatingControl.vue';
import ImageHistogram from '@/components/ImageHistogram.vue';
import MapView from '@/components/MapView.vue';

const props = defineProps({
  navigatorViewport: { type: Object, default: null },
  refreshFile: { type: Function, default: null },
  changeThumbnail: { type: Function, default: null },
  fileInfo: {
    type: Object,
    required: false
  },
});

const { locale, messages, t } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();

const emit = defineEmits([
  'navigator-visible',
  'navigator-navigate',
  'navigator-zoom',
  'navigator-toggle-fit',
  'close',
  'success',
  'toggleFavorite',
  'setRating',
  'setCulling',
  'rotate',
  'quickEditTag',
  'quickEditCollection',
  'quickEditComment',
  'navigateFolder',
  'navigateMetadata',
  'navigatePerson',
]);

const displayTags = ref<any[]>([]);
let tagsRequest = 0;
let tagsDisposed = false;
let stopTags: (() => void) | undefined;
async function refreshTags() {
  const id = props.fileInfo?.id; const library = libConfig._libraryId; const request = ++tagsRequest;
  if (!id) { displayTags.value = []; return; }
  const tags = await getTagsForFile(id);
  if (!tagsDisposed && request === tagsRequest && props.fileInfo?.id === id && libConfig._libraryId === library && tags) displayTags.value = tags;
}
watch(() => [props.fileInfo?.id, props.fileInfo?.tags, libConfig._libraryId], () => {
  displayTags.value = props.fileInfo?.tags || []; void refreshTags();
}, { immediate: true });
onMounted(async () => { const stop = await listen('tags-changed', refreshTags); if (tagsDisposed) stop(); else stopTags = stop; });
onBeforeUnmount(() => { tagsDisposed = true; tagsRequest++; stopTags?.(); });

const toast = useToast();
const showPreviewPanel = computed(() => config.infoPanel.showPreview);
const previewScaleOptions = [1, 0.75, 0.5, 0.25];
const previewScale = computed({
  get: () => Number(config.infoPanel.previewScale || 1),
  set: (value: number | string) => {
    const numeric = Number(value);
    config.infoPanel.previewScale = previewScaleOptions.includes(numeric) ? numeric : 1;
  },
});
type MetadataDetail = { values: string[]; source: string };
const metadataDetails = ref<Record<string, MetadataDetail>>({});
type DetailSection = 'source' | 'capture' | 'location';
const sectionNames: DetailSection[] = ['source', 'capture', 'location'];
function emptyDetailSections() {
  return Object.fromEntries(sectionNames.map(key => [key, { loaded: false, loading: false, error: false, stale: false, revalidate: false, token: null as string | null, request: 0 }])) as Record<DetailSection, { loaded: boolean; loading: boolean; error: boolean; stale: boolean; revalidate: boolean; token: string | null; request: number }>;
}
// Snapshots belong to the target file only; backend fingerprints still validate them.
const detailSnapshots = new Map<string, { data: Record<string, MetadataDetail>; loaded: DetailSection[]; bytes: number }>();
function detailSnapshotKey() {
  return JSON.stringify([libConfig._libraryId, props.fileInfo?.id, props.fileInfo?.file_path,
    props.fileInfo?.modified_at, props.fileInfo?.last_scan_time]);
}
function rememberDetailSnapshot() {
  const key = detailSnapshotKey();
  detailSnapshots.delete(key);
  detailSnapshots.set(key, { data: metadataDetails.value, bytes: JSON.stringify(metadataDetails.value).length * 2,
    loaded: sectionNames.filter(section => detailSections.value[section].loaded && !detailSections.value[section].error) });
  while (detailSnapshots.size > 16 || [...detailSnapshots.values()].reduce((bytes, snapshot) => bytes + snapshot.bytes, 0) > 4 * 1024 * 1024) {
    detailSnapshots.delete(detailSnapshots.keys().next().value!);
  }
}
const detailSections = ref(emptyDetailSections());
const detailsError = computed(() => sectionNames.some(section => sectionOpen(section) && detailSections.value[section].error));
const detailClientId = crypto.randomUUID();
let detailsRequest = 0;
function cancelPendingDetails(sections = sectionNames) {
  const requestIds = sections.flatMap(section => {
    const state = detailSections.value[section];
    const token = state.token;
    state.token = null;
    return token ? [token] : [];
  });
  if (requestIds.length) void invoke('cancel_file_metadata_requests', { requestIds }).catch(() => {});
}
const sourceKeys = ['title', 'creator', 'copyright', 'description', 'credit', 'source', 'usageTerms', 'rightsUrl', 'software'];
const detailKeys: Record<DetailSection, string[]> = {
  source: sourceKeys,
  capture: ['flash', 'whiteBalance', 'exposureMode', 'exposureProgram', 'meteringMode', 'focalLength35mm', 'timeZone'],
  location: ['direction', 'directionRef'],
};
const sourceFallbacks: Record<string, string> = { creator: 'e_artist', copyright: 'e_copyright', description: 'e_description', software: 'e_software' };
function detailRows(keys: string[]) {
  return keys.flatMap(key => {
    const detail = metadataDetails.value[key];
    const fallback = sourceFallbacks[key] ? props.fileInfo?.[sourceFallbacks[key]] : key === 'flash' ? props.fileInfo?.e_flash : null;
    let value = detail ? detail.values.filter(v => v.trim()).join('\n') : fallback;
    if (key === 'direction' && value && Number.isFinite(Number(value))) {
      const degrees = Number(value);
      if (degrees < 0 || degrees > 360) return [];
      const compass = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW'][Math.round(degrees / 45) % 8];
      value = `${compass} ${Number(degrees.toFixed(2))}°`;
    }
    if (key === 'directionRef' && value) {
      const reference = String(value).trim().toUpperCase();
      if (reference === 'T' || reference === 'TRUE NORTH') value = t('file_info.true_north');
      if (reference === 'M' || reference === 'MAGNETIC NORTH') value = t('file_info.magnetic_north');
    }
    return value && String(value).trim() ? [{ key, value, label: t(`file_info.detail_fields.${key}`) }] : [];
  });
}
const sourceDetailRows = computed(() => detailRows(sourceKeys));
const captureDetailRows = computed(() => detailRows(detailKeys.capture));
const imageDetailRows = computed(() => [
  { key: 'iccProfile', value: generalFileInfo.value?.icc_profile },
  { key: 'colorSpace', value: generalFileInfo.value?.color_space },
  { key: 'bitDepth', value: generalFileInfo.value?.bit_depth > 0 ? `${generalFileInfo.value.bit_depth} bit` : '' },
].filter(row => row.value != null && String(row.value).trim() !== '')
  .map(row => ({ ...row, label: t(`file_info.detail_fields.${row.key}`) })));
const locationDetailRows = computed(() => detailRows(detailKeys.location));
function sectionOpen(section: DetailSection) {
  return section === 'source' ? showSourceMetadataPanel.value : section === 'capture' ? showMetadataPanel.value
    : showMapPanel.value;
}
async function loadDetailSection(section: DetailSection, force = false) {
  const fileId = Number(props.fileInfo?.id);
  const state = detailSections.value[section];
  if (!fileId || !sectionOpen(section) || state.loading || (state.loaded && !state.stale && !state.revalidate && !force)) return;
  force ||= state.stale;
  const generation = detailsRequest;
  const request = ++state.request;
  const libraryId = libConfig._libraryId;
  const requestId = crypto.randomUUID();
  state.token = requestId;
  state.loading = true;
  state.error = false;
  const current = () => generation === detailsRequest && request === state.request
    && libraryId === libConfig._libraryId && sectionOpen(section);
  try {
    const result = await invoke<Record<string, MetadataDetail>>('get_file_metadata_details', { fileId, section, requestId, force, refreshKey: force ? `${detailClientId}:${generation}` : null });
    if (!current()) return;
    // Replace one complete section in a single assignment, including removed fields.
    const next = { ...metadataDetails.value };
    for (const key of detailKeys[section]) delete next[key];
    for (const key of detailKeys[section]) if (result[key]) next[key] = result[key];
    metadataDetails.value = next;
    state.loaded = true;
    state.stale = false;
    state.revalidate = false;
    rememberDetailSnapshot();
  } catch {
    if (current()) state.error = true;
  } finally {
    if (generation === detailsRequest && request === state.request) { state.loading = false; state.token = null; }
  }
}
function invalidateDetailSections() {
  cancelPendingDetails();
  detailSnapshots.delete(detailSnapshotKey());
  detailsRequest++;
  for (const section of sectionNames) {
    const state = detailSections.value[section];
    state.request++;
    state.loading = false;
    state.error = false;
    state.stale = true;
  }
}
function loadMetadataDetails(force = false) {
  for (const section of sectionNames) if (sectionOpen(section)) void loadDetailSection(section, force);
}
const LoadingIcon = { render: () => h('span', { class: 'loading loading-spinner' }) };
const refreshingFile = ref<string | null>(null);
const activeInfoKey = computed(() => `${libConfig._libraryId}:${props.fileInfo?.id ?? ''}`);
const fileInfoBusy = computed(() => refreshingFile.value === activeInfoKey.value
  || sectionNames.some(section => detailSections.value[section].loading));
const showDelayedLoading = ref(false);
let loadingTimer: ReturnType<typeof setTimeout> | undefined;
watch(() => [fileInfoBusy.value, activeInfoKey.value], () => {
  clearTimeout(loadingTimer);
  showDelayedLoading.value = false;
  if (fileInfoBusy.value) loadingTimer = setTimeout(() => { showDelayedLoading.value = true; }, 200);
}, { immediate: true });
async function refreshInfo() {
  const file = props.fileInfo;
  if (!file || fileInfoBusy.value) return;
  const key = activeInfoKey.value;
  refreshingFile.value = key;
  invalidateDetailSections();
  try {
    if (props.refreshFile) await props.refreshFile(file);
    if (activeInfoKey.value !== key) return;
    await nextTick();
    await Promise.all(sectionNames.filter(sectionOpen).map(section => loadDetailSection(section, true)));
  } catch {
    if (activeInfoKey.value === key) toast.error(t('tooltip.update_image.failed'));
  } finally {
    if (refreshingFile.value === key) refreshingFile.value = null;
  }
}
const changingThumbnail = ref(false);
const canChangeThumbnail = computed(() => Number(props.fileInfo?.duration || 0) > 10);
async function changePreviewThumbnail() {
  if (!canChangeThumbnail.value || changingThumbnail.value || !props.changeThumbnail || !props.fileInfo) return;
  changingThumbnail.value = true;
  try {
    await props.changeThumbnail(props.fileInfo);
  } catch (error) {
    console.error('Failed to change video thumbnail:', error);
    toast.error(t('tooltip.update_image.failed'));
  } finally {
    changingThumbnail.value = false;
  }
}
onBeforeUnmount(() => clearTimeout(loadingTimer));
const showOrganizationPanel = computed(() => config.infoPanel.showOrganization ?? true);
const showSourceMetadataPanel = computed(() => config.infoPanel.showSourceMetadata ?? false);
const hasCoordinates = computed(() => {
  const lat = props.fileInfo?.gps_latitude;
  const lon = props.fileInfo?.gps_longitude;
  return lat != null && lon != null && lat !== '' && lon !== ''
    && Number.isFinite(Number(lat)) && Number.isFinite(Number(lon))
    && Math.abs(Number(lat)) <= 90 && Math.abs(Number(lon)) <= 180;
});
const hasAltitude = computed(() => {
  const altitude = props.fileInfo?.gps_altitude;
  return altitude != null && String(altitude).trim() !== '' && Number.isFinite(Number(altitude));
});
const cameraText = computed(() => formatCameraInfo(props.fileInfo?.e_make, props.fileInfo?.e_model) || '');
const exposureText = computed(() => formatCaptureSettings(props.fileInfo?.e_focal_length, props.fileInfo?.e_exposure_time,
  props.fileInfo?.e_f_number, props.fileInfo?.e_iso_speed, props.fileInfo?.e_exposure_bias) || '');
const captureTimeText = computed(() => props.fileInfo?.e_date_time?.trim() && props.fileInfo?.taken_date != null
  ? formatTimestamp(props.fileInfo.taken_date, 'date_time') : '');
const hasCaptureInfo = computed(() => (!detailSections.value.capture.loaded || detailSections.value.capture.loading || detailSections.value.capture.error) || !!(cameraText.value.trim() || props.fileInfo?.e_lens_model?.trim()
  || exposureText.value.trim() || captureTimeText.value || captureDetailRows.value.length));
const hasLocationInfo = computed(() => (!detailSections.value.location.loaded || detailSections.value.location.loading || detailSections.value.location.error) || !!(locationDetailRows.value.length || hasCoordinates.value || hasAltitude.value || formatGeoLocation().trim()));
const hasSourceMetadata = computed(() => (!detailSections.value.source.loaded || detailSections.value.source.loading || detailSections.value.source.error) || sourceDetailRows.value.length > 0);
const showBasicInfoPanel = computed(() => config.infoPanel.showBasicInfo);
const showMetadataPanel = computed(() => config.infoPanel.showMetadata);
const showMapPanel = computed(() => config.infoPanel.showMap);
watch(() => [libConfig._libraryId, props.fileInfo?.id, props.fileInfo?.file_path, props.fileInfo?.last_scan_time, props.fileInfo?.modified_at], (values, previous) => {
  const sameFile = previous && values[0] === previous[0] && values[1] === previous[1];
  if (sameFile) {
    invalidateDetailSections();
  } else {
    cancelPendingDetails();
    detailsRequest++;
    const snapshot = detailSnapshots.get(detailSnapshotKey());
    metadataDetails.value = snapshot ? { ...snapshot.data } : {};
    detailSections.value = emptyDetailSections();
    for (const section of snapshot?.loaded || []) {
      detailSections.value[section].loaded = true;
      detailSections.value[section].revalidate = true;
    }
  }
  // The explicit refresh waits for catalog updates before fetching open sections.
  if (refreshingFile.value !== activeInfoKey.value) loadMetadataDetails();
}, { immediate: true });
for (const section of sectionNames) {
  watch(() => sectionOpen(section), open => {
    const state = detailSections.value[section];
    if (open) void loadDetailSection(section);
    else {
      cancelPendingDetails([section]);
      if (state.loading) state.stale = true;
      state.request++;
      state.loading = false;
    }
  });
}
onBeforeUnmount(() => { cancelPendingDetails(); detailsRequest++; });
const isVideoFile = computed(() => Number(props.fileInfo?.file_type || 0) === 2);
const isLivePhoto = computed(() => (
  props.fileInfo?.media_subtype === 'live_photo' && !!props.fileInfo?.live_photo_video_path
));
const isMotionPhoto = computed(() => props.fileInfo?.media_subtype === 'motion_photo');
const isRawJpegPair = computed(() => (
  config.settings.groupRawJpegPairs && props.fileInfo?.media_subtype === 'raw_jpeg_pair' && !!props.fileInfo?.live_photo_video_id
));
const rawJpegCompanion = ref<any>(null);
const generalInfoTab = ref<'raw' | 'companion'>('raw');
let rawJpegCompanionRequestSeq = 0;
const rawJpegCompanionLabel = computed(() => {
  const extension = getFileExtension(
    rawJpegCompanion.value?.name || props.fileInfo?.live_photo_video_path || '',
  ).toLowerCase();
  return ['heic', 'heif', 'hif'].includes(extension) ? 'HEIC' : 'JPEG';
});
const generalFileInfo = computed(() => (
  generalInfoTab.value === 'companion' && rawJpegCompanion.value
    ? rawJpegCompanion.value
    : props.fileInfo
));
const isPrimaryGeneralInfo = computed(() => generalInfoTab.value === 'raw');
const motionPhotoVideoPath = ref<string | null>(null);
let motionPhotoVideoRequestSeq = 0;
const previewVideoPath = computed(() => {
  if (isLivePhoto.value) return props.fileInfo?.live_photo_video_path;
  if (isMotionPhoto.value) return motionPhotoVideoPath.value;
  return props.fileInfo?.file_path;
});
const canPreviewVideo = computed(() => (
  (isVideoFile.value || isLivePhoto.value || isMotionPhoto.value)
  && !!previewVideoPath.value
  && !isWebViewVideoPlaybackDisabled(previewVideoPath.value)
));
const canShowHistogram = computed(() => !isVideoFile.value);
const activePreviewMode = computed(() => canShowHistogram.value ? config.infoPanel.previewMode : 'thumbnail');
const isHistogramPreview = computed(() => activePreviewMode.value === 'histogram');
const hasPreviewNavigator = computed(() => !!props.fileInfo?.thumbnail
  && !isVideoFile.value && !!props.navigatorViewport
  && props.navigatorViewport.fileId === props.fileInfo?.id);
const histogramChannelLabel = computed(() => {
  const storedMask = Number(config.infoPanel.histogramChannels);
  const mask = storedMask === 16 || !Number.isInteger(storedMask) || storedMask < 0 || storedMask > 15
    ? 15
    : storedMask;
  const labels = [
    { bit: 1, label: 'L' },
    { bit: 2, label: 'R' },
    { bit: 4, label: 'G' },
    { bit: 8, label: 'B' },
  ];
  return labels
    .filter((item) => Boolean(mask & item.bit))
    .map((item) => item.label)
    .join('');
});
const previewVideoRef = ref<HTMLVideoElement | null>(null);
const showVideoPreview = ref(false);
const isVideoPreviewReady = ref(false);
watch(() => showPreviewPanel.value && !isHistogramPreview.value && hasPreviewNavigator.value && !showVideoPreview.value,
  visible => emit('navigator-visible', visible), { immediate: true, flush: 'post' });
onBeforeUnmount(() => emit('navigator-visible', false));

const fileCollections = ref<Array<{ id: number; name: string }>>([]);
let fileCollectionsRequestSeq = 0;
const normalizedRotate = computed(() => {
  const rotate = Number(props.fileInfo?.rotate || 0) % 360;
  return rotate < 0 ? rotate + 360 : rotate;
});
const previewNaturalSize = ref<{ width: number; height: number } | null>(null);
watch(() => props.fileInfo?.thumbnail, () => { previewNaturalSize.value = null; });
function onPreviewImageLoad(event: Event) {
  const image = event.target as HTMLImageElement;
  if (image.naturalWidth > 0 && image.naturalHeight > 0) {
    previewNaturalSize.value = { width: image.naturalWidth, height: image.naturalHeight };
  }
}
const previewLayout = computed(() => navigatorLayout({
  sourceWidth: previewNaturalSize.value?.width || Number(props.fileInfo?.width) || 4,
  sourceHeight: previewNaturalSize.value?.height || Number(props.fileInfo?.height) || 3,
  rotate: normalizedRotate.value,
  scale: 1, normX: 0.5, normY: 0.5, viewportWidth: 0, viewportHeight: 0, pannable: false,
}, { width: 100, height: 75 * previewScale.value }));
const previewContentStyle = computed(() => {
  if (hasPreviewNavigator.value) return { inset: '0' };
  const layout = previewLayout.value;
  const height = 75 * previewScale.value;
  return { left: `${layout.left}%`, top: `${layout.top / height * 100}%`,
    width: `${layout.width}%`, height: `${layout.height / height * 100}%` };
});
const previewImageStyle = computed(() => {
  const layout = previewLayout.value;
  return {
    position: 'absolute' as const,
    left: `${(layout.width - layout.imageWidth) / 2 / layout.width * 100}%`,
    top: `${(layout.height - layout.imageHeight) / 2 / layout.height * 100}%`,
    width: `${layout.imageWidth / layout.width * 100}%`,
    height: `${layout.imageHeight / layout.height * 100}%`,
    maxWidth: 'none',
    transform: `rotate(${normalizedRotate.value}deg)`,
    transformOrigin: 'center center',
  };
});
const previewFormatLabel = computed(() => {
  if (props.fileInfo?.media_subtype === 'live_photo') {
    return t('image_viewer.live');
  }
  if (props.fileInfo?.media_subtype === 'motion_photo') {
    return t('image_viewer.motion');
  }

  const formatLabel = (props.fileInfo?.format_label || '').trim();
  const isRaw = Number(props.fileInfo?.file_type || 0) === 3;
  if (formatLabel && !(isRaw && formatLabel.toUpperCase() === 'RAW')) {
    return formatLabel.toUpperCase();
  }

  const name = props.fileInfo?.name || '';
  const filePath = props.fileInfo?.file_path || '';
  const extension = getFileExtension(name || filePath).trim();
  if (!extension) return '';
  return extension.toUpperCase();
});
const previewTagLabel = computed(() => (
  isHistogramPreview.value ? histogramChannelLabel.value : previewFormatLabel.value
));
function togglePreview() {
  config.infoPanel.showPreview = !config.infoPanel.showPreview;
}

function setPreviewMode(mode: 'thumbnail' | 'histogram') {
  if (mode === 'histogram' && !canShowHistogram.value) return;
  config.infoPanel.previewMode = mode;
}

let videoPreviewRequest = 0;
async function playPreviewVideo() {
  const request = ++videoPreviewRequest;
  const available = await getGStreamerAvailability();
  if (request !== videoPreviewRequest || !available) return;
  if (!canPreviewVideo.value || !previewVideoPath.value || showVideoPreview.value) return;
  // Playing an online-only video would download it.
  if (await isCloudOnlyFile(previewVideoPath.value) || request !== videoPreviewRequest) return;
  isVideoPreviewReady.value = false;
  showVideoPreview.value = true;
  await nextTick();

  const video = previewVideoRef.value;
  if (!video || request !== videoPreviewRequest) return;

  video.src = getAssetSrc(previewVideoPath.value);
  video.muted = true;

  try {
    await video.play();
  } catch {
    stopPreviewVideo();
  }
}

function playVideoPreviewOnHover() {
  if (canPreviewVideo.value) {
    void playPreviewVideo();
  }
}

function stopPreviewVideo() {
  videoPreviewRequest++;
  const video = previewVideoRef.value;
  if (video) {
    video.pause();
    video.removeAttribute('src');
    video.load();
  }

  isVideoPreviewReady.value = false;
  showVideoPreview.value = false;
}

watch(
  () => [props.fileInfo?.id, props.fileInfo?.media_subtype, props.fileInfo?.modified_at] as const,
  async ([fileId, mediaSubtype]) => {
    const requestSeq = ++motionPhotoVideoRequestSeq;
    motionPhotoVideoPath.value = null;
    if (!fileId || mediaSubtype !== 'motion_photo') return;
    // Extracting the embedded video reads the file; an online-only one would download.
    if (await isCloudOnlyFile(props.fileInfo?.file_path || '') || requestSeq !== motionPhotoVideoRequestSeq) return;
    try {
      const path = await getMotionPhotoVideoPath(Number(fileId));
      if (requestSeq === motionPhotoVideoRequestSeq) {
        motionPhotoVideoPath.value = path;
      }
    } catch (error) {
      if (requestSeq === motionPhotoVideoRequestSeq) {
        console.error('Failed to prepare motion photo video:', error);
      }
    }
  },
  { immediate: true },
);

watch(
  () => [
    props.fileInfo?.id,
    props.fileInfo?.file_path,
    props.fileInfo?.live_photo_video_path,
    props.fileInfo?.media_subtype,
    showPreviewPanel.value,
    isHistogramPreview.value,
  ],
  stopPreviewVideo
);

onBeforeUnmount(stopPreviewVideo);

watch(
  () => [props.fileInfo?.id, Boolean(props.fileInfo?.has_collections), props.fileInfo?.collectionVersion] as const,
  async ([fileId, hasCollections]) => {
    const requestSeq = ++fileCollectionsRequestSeq;
    fileCollections.value = [];
    if (!fileId || !hasCollections) return;
    const collections = await getFileCollections(Number(fileId));
    if (requestSeq !== fileCollectionsRequestSeq || Number(props.fileInfo?.id) !== Number(fileId)) return;
    fileCollections.value = Array.isArray(collections) ? collections : [];
  },
  { immediate: true }
);

function increasePreviewScale() {
  const index = previewScaleOptions.indexOf(previewScale.value);
  if (index > 0) {
    previewScale.value = previewScaleOptions[index - 1];
  }
}

function decreasePreviewScale() {
  const index = previewScaleOptions.indexOf(previewScale.value);
  if (index >= 0 && index < previewScaleOptions.length - 1) {
    previewScale.value = previewScaleOptions[index + 1];
  }
}

function toggleOrganization() {
  config.infoPanel.showOrganization = !showOrganizationPanel.value;
}

function toggleSourceMetadata() {
  config.infoPanel.showSourceMetadata = !showSourceMetadataPanel.value;
}

function toggleBasicInfo() {
  config.infoPanel.showBasicInfo = !config.infoPanel.showBasicInfo;
}

function toggleMetadata() {
  config.infoPanel.showMetadata = !config.infoPanel.showMetadata;
}

function toggleMapPanel() {
  config.infoPanel.showMap = !config.infoPanel.showMap;
}

const quickSave = async (): Promise<boolean> => {
  if (!props.fileInfo || isOriginalUnavailable(props.fileInfo)) return false;
  if (uiStore.activeAdjustments.filePath !== props.fileInfo.file_path) return true;

  const adj = uiStore.activeAdjustments as any;
  const ext = getFileExtension(props.fileInfo.name).toLowerCase();
  const outputFormat = ['jpg', 'jpeg', 'jfif'].includes(ext) ? 'jpg' : ext;

  const editParams = {
    sourceFilePath: props.fileInfo.file_path,
    destFilePath: props.fileInfo.file_path,
    outputFormat,
    quality: 80,
    orientation: props.fileInfo.e_orientation || 1,
    flipHorizontal: false,
    flipVertical: false,
    rotate: 0,
    crop: { x: 0, y: 0, width: 0, height: 0 },
    resize: {
      width: adj.resize?.width ?? props.fileInfo.width,
      height: adj.resize?.height ?? props.fileInfo.height,
    },
    filter: adj.filter || null,
    brightness: adj.brightness ? adj.brightness : null,
    contrast: adj.contrast ? adj.contrast : null,
    blur: adj.blur ? adj.blur : null,
    hue_rotate: adj.hue ? adj.hue : null,
    saturation: adj.saturation !== 100 ? adj.saturation / 100.0 : null,
  };

  try {
    const success = await editImage(editParams);
    if (!success) {
      toast.error(localeMsg.value.tooltip.save_image.failed);
      return false;
    }

    uiStore.updateFileVersion(props.fileInfo.file_path);
    uiStore.clearActiveAdjustments();
    emit('success');
    toast.success(localeMsg.value.tooltip.save_image.success);
    return true;
  } catch {
    toast.error(localeMsg.value.tooltip.save_image.failed);
    return false;
  }
};

// Rename logic
const isRenaming = ref(false);
const renamingName = ref('');
const renamingExt = ref('');
const renameInputRef = ref<HTMLInputElement | null>(null);
const albumRootPath = ref('');
let albumRootRequestSeq = 0;

const generalFolderBreadcrumbs = computed(() => {
  const folderPath = getFolderPath(generalFileInfo.value?.file_path);
  if (!folderPath) return [];
  return buildFolderBreadcrumbs(folderPath, albumRootPath.value);
});
function revealFileInFolder() {
  if (props.fileInfo?.file_path && !isOriginalUnavailable(props.fileInfo)) {
    revealPath(props.fileInfo.file_path);
  }
}

watch(
  () => props.fileInfo?.album_id,
  async (albumId) => {
    const requestSeq = ++albumRootRequestSeq;
    albumRootPath.value = '';
    if (!albumId) return;
    const album = await getAlbum(albumId);
    if (requestSeq !== albumRootRequestSeq) return;
    if (props.fileInfo?.album_id !== albumId) return;
    albumRootPath.value = album?.path || '';
  },
  { immediate: true }
);

watch(
  () => [props.fileInfo?.id, props.fileInfo?.media_subtype, props.fileInfo?.live_photo_video_id, config.settings.groupRawJpegPairs] as const,
  async ([fileId, mediaSubtype, companionId, groupPairs]) => {
    const requestSeq = ++rawJpegCompanionRequestSeq;
    generalInfoTab.value = 'raw';
    rawJpegCompanion.value = null;
    if (!groupPairs || !fileId || mediaSubtype !== 'raw_jpeg_pair' || !companionId) return;
    const companion = await getFileInfo(Number(companionId));
    if (requestSeq !== rawJpegCompanionRequestSeq || Number(props.fileInfo?.id) !== Number(fileId)) return;
    rawJpegCompanion.value = companion || null;
  },
  { immediate: true },
);

const startRename = () => {
  if (isOriginalUnavailable(props.fileInfo)) return;
  if (!props.fileInfo) return;
  
  const { name, ext } = extractFileName(props.fileInfo.name);
  renamingName.value = name;
  renamingExt.value = ext;
  isRenaming.value = true;
  uiStore.pushInputHandler('FileInfo-rename');
  
  nextTick(() => {
    if (renameInputRef.value) {
      renameInputRef.value.focus();
      renameInputRef.value.select();
    }
  });
};

const cancelRename = () => {
  isRenaming.value = false;
  renamingExt.value = '';
  uiStore.removeInputHandler('FileInfo-rename');
};

const finishRename = async () => {
  if (!isRenaming.value || !props.fileInfo) return;

  const newName = renamingName.value.trim();
  const { ext } = extractFileName(props.fileInfo.name);
  
  // Validation
  if (!newName || !isValidFileName(newName)) {
    // Optionally show error toast
    console.warn('Invalid filename');
    cancelRename();
    return;
  }

  const fullNewName = combineFileName(newName, ext);
  
  // If no change, just cancel
  if (fullNewName === props.fileInfo.name) {
    cancelRename();
    return;
  }

  // Call API
  let newPath;
  try {
    newPath = await renameFile(props.fileInfo.id, props.fileInfo.file_path, fullNewName);
  } catch (error) {
    toast.error(`${t('msgbox.rename_file.error')}: ${String(error)}`);
    cancelRename();
    return;
  }
  
  if (newPath) {
    // Update local props to reflect change immediately (assuming parent passes object ref)
    props.fileInfo.name = fullNewName;
    props.fileInfo.file_path = newPath;
  } else {
    // Optionally show error
    console.error('Rename failed');
  }

  cancelRename();
};

function formatGeoLocation() {
  const info = props.fileInfo;
  if (!info) return "";

  const fields = [
    info.geo_name,
    info.geo_admin2,
    info.geo_admin1,
    info.geo_cc ? getCountryName(info.geo_cc, locale.value) : info.geo_cc,
  ];

  return fields.filter(Boolean).join(", ");
}

// Clickable metadata values: jump to the corresponding sidebar view.
const hasCamera = computed(() => !!(props.fileInfo?.e_make || props.fileInfo?.e_model));
const hasLens = computed(() => !!props.fileInfo?.e_lens_model);
const hasLocation = computed(() => !!(props.fileInfo?.geo_cc || props.fileInfo?.geo_admin1 || props.fileInfo?.geo_name));

function navigateCamera() {
  if (!hasCamera.value) return;
  emit('navigateMetadata', { type: 'camera', make: props.fileInfo?.e_make || null, model: props.fileInfo?.e_model || null });
}

function navigateLens() {
  if (!hasLens.value) return;
  emit('navigateMetadata', { type: 'lens', lensMake: props.fileInfo?.e_lens_make || null, lensModel: props.fileInfo?.e_lens_model || null });
}

function navigateLocation() {
  if (!hasLocation.value) return;
  emit('navigateMetadata', { type: 'location', cc: props.fileInfo?.geo_cc || null, admin1: props.fileInfo?.geo_admin1 || null, name: props.fileInfo?.geo_name || null });
}

// People recognized in this file (deduplicated by person id).
const filePersons = ref<Array<{ id: number; name: string; thumbnail: string }>>([]);
let filePersonsRequestSeq = 0;

async function loadFilePersons(fileId: number) {
  const seq = ++filePersonsRequestSeq;
  if (!fileId || fileId <= 0) {
    if (seq === filePersonsRequestSeq) filePersons.value = [];
    return;
  }
  const faces = await getFacesForFile(fileId);
  if (seq !== filePersonsRequestSeq) return;

  // Dedupe persons by id, keeping the name from the face.
  const personsById = new Map<number, string>();
  for (const face of faces || []) {
    const id = Number(face?.person_id || 0);
    if (id > 0 && !personsById.has(id)) {
      personsById.set(id, face?.person_name || '');
    }
  }
  if (personsById.size === 0) {
    filePersons.value = [];
    return;
  }

  // Fetch each person's face thumbnail from the backend (People list is
  // paginated on the frontend, so we can't rely on locally loaded thumbnails).
  const persons = await Promise.all(
    Array.from(personsById.entries()).map(async ([id, name]) => {
      const thumbnail = (await getPersonThumbnail(id)) || '';
      return { id, name, thumbnail };
    }),
  );
  if (seq !== filePersonsRequestSeq) return;
  filePersons.value = persons;
}

watch(() => props.fileInfo?.id, (id) => {
  void loadFilePersons(Number(id || 0));
}, { immediate: true });

function navigatePerson(person: { id: number; name: string }) {
  if (!person?.id) return;
  emit('navigatePerson', { personId: person.id, personName: person.name });
}

defineExpose({
  quickSave
});
</script>
