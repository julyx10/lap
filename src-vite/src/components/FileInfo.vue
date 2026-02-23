<template>
  <div class="w-full h-full rounded-box bg-base-200 flex flex-col overflow-hidden">
    <!-- Tabs & Close -->
    <div class="flex items-center w-full shrink-0 px-2 mb-2">
      <div role="tablist" class="tabs tabs-sm tabs-border flex-1">
        <a role="tab" :class="['tab mx-1', { 'tab-active': activeTab === 'info' }]" @click="activeTab = 'info'">{{ $t('info_panel.tabs[0]') }}</a>
        <a role="tab" :class="['tab mx-1', { 'tab-active': activeTab === 'edit' && !multiSelect, 'opacity-30 cursor-not-allowed': multiSelect }]" @click="!multiSelect && (activeTab = 'edit')">{{ $t('info_panel.tabs[1]') }}</a>
      </div>
      <div class="mt-2 flex items-center gap-1">
        <!-- Buttons -->
        <template v-if="activeTab === 'edit' && hasChanges && !isSaving && !multiSelect">
          <TButton
            :icon="IconRestore"
            :tooltip="$t('msgbox.image_editor.reset')"
            :buttonSize="'small'"
            :selected="true"
            @click.stop="resetAll"
          />
          <TButton
            :icon="IconSave"
            :tooltip="$t('msgbox.image_editor.save')"
            :buttonSize="'small'"
            :selected="true"
            :loading="isProcessing"
            @click.stop="quickSave"
          />
          <div class="w-px h-4 bg-base-content/10 mx-1"></div>
        </template>
        <TButton
          :icon="IconClose"
          :buttonSize="'small'"
          @click.stop="$emit('close')"
        />
      </div>
    </div>

    <!-- Info Content -->
    <!-- Multi-Select: No files selected yet -->
    <div v-if="multiSelect && selectedFiles.length === 0" v-show="activeTab === 'info'" class="mb-2 px-2 flex-1 flex items-center justify-center bg-base-200/50">
      <div class="text-center text-base-content/40 space-y-3 max-w-[200px]">
        <IconFiles class="w-8 h-8 mx-auto text-base-content/30" />
        <p class="text-xs font-medium leading-relaxed">{{ $t('info_panel.select_hint') }}</p>
      </div>
    </div>

    <!-- Multi-Select Summary View -->
    <div v-if="multiSelect && selectedFiles.length > 0" v-show="activeTab === 'info'" class="mb-2 px-2 flex-1 overflow-y-auto overflow-x-hidden space-y-3 flex flex-col bg-base-200/50">
      <div class="rounded-box p-3 space-y-3 bg-base-300/30 border border-base-content/5 shadow-sm">
        <!-- Header -->
        <div class="flex items-center gap-2 text-base-content/70">
          <IconCheckAll class="w-4 h-4" />
          <span class="font-bold uppercase text-xs tracking-wide">{{ $t('toolbar.filter.select_count', { count: selectedFiles.length }) + ' (' + formatFileSize(multiSelectTotalSize) + ')' }}</span>
        </div>

        <!-- Thumbnail Grid -->
        <div class="grid grid-cols-3 gap-1">
          <div v-for="(file, idx) in selectedFiles.slice(0, selectedFiles.length <= 20 ? 20 : 19)" :key="file.id || idx"
            class="aspect-square rounded-md overflow-hidden bg-base-content/5 border border-base-content/5 cursor-pointer relative group"
            @click="emit('deselect', file)">
            <img v-if="file.thumbnail" :src="file.thumbnail" class="w-full h-full object-cover" />
            <div v-else class="w-full h-full skeleton"></div>
            <div class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
              <IconClose class="w-4 h-4 text-white" />
            </div>
          </div>
          <div v-if="selectedFiles.length > 20"
            class="aspect-square rounded-md overflow-hidden bg-base-content/5 border border-base-content/5 flex items-center justify-center">
            <span class="text-xs font-bold text-base-content/40">+{{ selectedFiles.length - 19 }}</span>
          </div>
        </div>

        <!-- Summary Info -->
        <div class="space-y-2 text-xs">
          <!-- File type breakdown -->
          <div class="flex items-center gap-1 text-base-content/50 font-medium">
            <span>{{ multiSelectTypeBreakdown }}</span>
          </div>

          <!-- Date range -->
          <div v-if="multiSelectDateRange" class="flex items-center gap-1 text-base-content/50 font-medium">
            <span>{{ multiSelectDateRange }}</span>
          </div>
        </div>

        <!-- Quick Actions -->
        <div class="flex gap-1 pt-1">
          <button class="btn btn-xs btn-ghost gap-1 text-base-content/50 hover:text-base-content" @click="emit('favoriteAll')">
            <IconFavorite class="w-3.5 h-3.5" />
            <span>{{ $t('info_panel.favorite_all') }}</span>
          </button>
          <button class="btn btn-xs btn-ghost gap-1 text-base-content/50 hover:text-base-content" @click="emit('unfavoriteAll')">
            <IconUnFavorite class="w-3.5 h-3.5" />
            <span>{{ $t('info_panel.unfavorite_all') }}</span>
          </button>
          <button class="btn btn-xs btn-ghost gap-1 text-base-content/50 hover:text-base-content" @click="emit('tagAll')">
            <IconTag class="w-3.5 h-3.5" />
            <span>{{ $t('info_panel.tag_all') }}</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Single-File Info Content -->
    <div v-if="fileInfo && !multiSelect" v-show="activeTab === 'info'" class="mb-2 px-2 flex-1 overflow-y-auto overflow-x-hidden space-y-3 flex flex-col bg-base-200/50">

      <!-- Title / Thumbnail -->
      <!-- <div class="flex justify-center p-2">
        <div class="relative w-16 h-16 rounded-box shrink-0 shadow-sm border border-base-content/5">
          <img v-if="fileInfo?.thumbnail" :src="fileInfo.thumbnail" :style="thumbnailStyle" class="w-full h-full object-cover rounded-box" />
          <div v-else-if="fileInfo" class="w-full h-full skeleton object-cover rounded-box"></div>
          <div v-else class="w-full h-full bg-base-content/5 rounded-box"></div>
          
          <div v-if="fileInfo?.is_favorite" class="absolute -bottom-1 -right-1 drop-shadow-md">
            <IconFavorite class="t-icon-size-xs" />
          </div>
        </div>
      </div> -->

      <!-- File Info Section -->
      <div class="rounded-box p-3 space-y-3 bg-base-300/30 border border-base-content/5 shadow-sm">

        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content transition-all duration-200 ease-in-out" 
          @click.stop="config.infoPanel.showBasicInfo = !config.infoPanel.showBasicInfo"
        >
          <IconFile class="w-4 h-4" />
          <span class="font-bold mr-auto uppercase text-xs tracking-wide">{{ $t('file_info.title') }}</span>
          <TButton
            :icon="config.infoPanel.showBasicInfo ? IconArrowUp : IconArrowDown"
            :buttonSize="'small'"
          />
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @after-enter="onAfterEnter"
          @leave="onLeave"
        >
          <div v-if="config.infoPanel.showBasicInfo" class="grid grid-cols-[80px_1fr] gap-y-3 gap-x-4 text-xs overflow-hidden">
            <!-- Name -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25 py-1">{{ $t('file_info.name') }}</div>
            <div class="flex items-center">
              <input 
                v-if="isRenaming"
                ref="renameInputRef"
                v-model="renamingName" 
                class="font-bold text-xs text-base-content input input-xs input-bordered p-1 h-6 leading-6 w-full"
                @blur="finishRename"
                @keydown.enter="finishRename"
                @keydown.esc="cancelRename"
                @click.stop
              />
              <span v-else 
                class="font-semibold text-xs text-base-content/65 break-all cursor-text hover:bg-base-content/10 rounded px-1 -mx-1 transition-colors"
                @click.stop="startRename"
              >{{ fileInfo?.name }}</span>
            </div>

            <!-- Album -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.album_name') }}</div>
            <div class="text-xs font-semibold text-base-content/65 break-all">{{ fileInfo?.album_name }}</div>

            <!-- Path -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.folder') }}</div>
            <div class="text-xs font-semibold text-base-content/65 break-all">{{ getFolderPath(fileInfo?.file_path) }}</div>

            <!-- Size -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.size') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ formatFileSize(fileInfo?.size) }}</div>

            <!-- Dimension -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.dimension') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ formatDimensionText(fileInfo?.width, fileInfo?.height) }}</div>

            <!-- Duration -->
            <template v-if="fileInfo?.file_type === 2">
              <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.duration') }}</div>
              <div class="text-xs font-semibold text-base-content/65">{{ formatDuration(fileInfo?.duration) }}</div>
            </template>

            <!-- Created At -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.created_at') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ formatTimestamp(fileInfo?.created_at, $t('format.date_time')) }}</div>

            <!-- Modified At -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.modified_at') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ formatTimestamp(fileInfo?.modified_at, $t('format.date_time')) }}</div>

            <!-- Tags -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.tags') }}</div>
            <div class="text-xs font-semibold text-base-content/65 flex flex-wrap gap-1">
              <template v-if="fileInfo?.tags && fileInfo.tags.length">
                <span v-for="tag in fileInfo.tags" :key="tag.id" class="badge badge-sm badge-ghost font-medium">{{ tag.name }}</span>
              </template>
            </div>

            <!-- Comment -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.comment') }}</div>
            <div class="text-xs font-semibold text-base-content/65 wrap-break-words whitespace-pre-wrap">{{ fileInfo?.comments }}</div>
          </div>
        </Transition>
      </div>

      <!-- Metadata Section -->
      <div class="rounded-box p-3 space-y-3 bg-base-300/30 border border-base-content/5 shadow-sm">

        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content" @click.stop="config.infoPanel.showMetadata = !config.infoPanel.showMetadata">
          <IconCameraAperture class="w-4 h-4 " /> 
          <span class="font-bold mr-auto uppercase text-xs tracking-wide">{{ $t('file_info.metadata') }}</span>
          <TButton
            :icon="config.infoPanel.showMetadata ? IconArrowUp : IconArrowDown"
            :buttonSize="'small'"
          />
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @after-enter="onAfterEnter"
          @leave="onLeave"
        >
          <div v-if="config.infoPanel.showMetadata" class="grid grid-cols-[80px_1fr] gap-y-3 gap-x-4 text-xs overflow-hidden">
            <!-- Camera -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.camera') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ formatCameraInfo(fileInfo?.e_make, fileInfo?.e_model) }}</div>

            <!-- Lens -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.lens') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ fileInfo?.e_lens_model }}</div>

            <!-- Capture Settings -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.capture_settings') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ formatCaptureSettings(fileInfo?.e_focal_length, fileInfo?.e_exposure_time, fileInfo?.e_f_number, fileInfo?.e_iso_speed, fileInfo?.e_exposure_bias) }}</div>

            <!-- Software -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.software') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ fileInfo?.e_software }}</div>

            <!-- Taken By -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.taken_by') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ fileInfo?.e_artist }}</div>

            <!-- Copyright -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.copyright') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ fileInfo?.e_copyright }}</div>

            <!-- Taken At -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.taken_at') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ fileInfo?.e_date_time }}</div>

            <!-- Description -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.description') }}</div>
            <div class="text-xs font-semibold text-base-content/65 wrap-break-words">{{ fileInfo?.e_description }}</div>

            <!-- Geo Location -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.geo_location') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ formatGeoLocation() }}</div>
          </div>
        </Transition>
      </div>

      <!-- Map View -->
      <div v-if="fileInfo?.gps_latitude && fileInfo?.gps_longitude" 
        class="rounded-box p-3 space-y-3 bg-base-300/30 border border-base-content/5 shadow-sm flex flex-col transition-[flex-grow]" 
        :class="{ 'flex-1 min-h-[300px]': config.infoPanel.showMap }">
        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content shrink-0" @click.stop="config.infoPanel.showMap = !config.infoPanel.showMap">
          <IconLocation class="w-4 h-4 " /> 
          <span class="font-bold mr-auto uppercase text-xs tracking-wide">{{ $t('file_info.location') }}</span>
          <TButton
            :icon="config.infoPanel.showMap ? IconArrowUp : IconArrowDown"
            :buttonSize="'small'"
          />
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @after-enter="onAfterEnter"
          @leave="onLeave"
        >
          <div v-if="config.infoPanel.showMap" class="overflow-hidden flex-1 flex flex-col min-h-0">
            <div class="w-full rounded-box overflow-hidden relative z-0 flex-1 min-h-[200px] border border-base-content/5">
              <MapView
                :lat="fileInfo.gps_latitude ? Number(fileInfo.gps_latitude) : 0"
                :lon="fileInfo.gps_longitude ? Number(fileInfo.gps_longitude) : 0"
              />
            </div>
          </div>
        </Transition>
      </div>
    </div>

    <!-- Edit Content -->
    <!-- Multi-select: Adjust tab disabled message -->
    <div v-if="multiSelect" v-show="activeTab === 'edit'" class="mb-2 px-2 flex-1 flex items-center justify-center bg-base-200/50">
      <div class="text-center text-base-content/30 space-y-2">
        <IconAdjustments class="w-8 h-8 mx-auto opacity-30" />
        <p class="text-xs font-medium">{{ $t('info_panel.edit_disabled') }}</p>
      </div>
    </div>

    <div v-if="fileInfo && fileInfo.file_type === 1 && !multiSelect" v-show="activeTab === 'edit'" class="mb-2 px-2 flex-1 overflow-y-auto overflow-x-hidden space-y-3 flex flex-col min-h-0 bg-base-200/50">
      
      <!-- Histogram Section -->
      <div v-if="config.infoPanel.showHistogram" class="rounded-box p-3 bg-base-300/30 border border-base-content/5 shadow-sm">
        <div class="flex items-center justify-between mb-3 px-0.5">
          <span class="text-[10px] uppercase tracking-widest font-bold text-base-content/30">{{ $t('msgbox.image_editor.histogram') || 'Histogram' }}</span>
          <div class="flex gap-2 text-[9px] font-bold text-base-content/60 uppercase tracking-tighter tabular-nums">
            {{ formatCaptureSettings(fileInfo?.e_focal_length, fileInfo?.e_exposure_time, fileInfo?.e_f_number, fileInfo?.e_iso_speed, fileInfo?.e_exposure_bias) }}
          </div>
        </div>
        
        <!-- Refined SVG Histogram -->
        <div class="relative w-full aspect-4/1 mb-1.5 px-0.5">
          <svg viewBox="0 0 256 64" class="w-full h-full text-primary" preserveAspectRatio="none">
            <defs>
              <linearGradient id="histGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="currentColor" stop-opacity="0.6" />
                <stop offset="100%" stop-color="currentColor" stop-opacity="0.1" />
              </linearGradient>
            </defs>
            <!-- Professional Grid -->
            <g class="text-base-content/20">
              <line x1="64" y1="0" x2="64" y2="64" stroke="currentColor" stroke-width="0.5" />
              <line x1="128" y1="0" x2="128" y2="64" stroke="currentColor" stroke-width="0.5" />
              <line x1="192" y1="0" x2="192" y2="64" stroke="currentColor" stroke-width="0.5" />
            </g>
            
            <path 
              :d="generateHistogramPath()" 
              fill="url(#histGradient)" 
              class="transition-all duration-300"
            />
          </svg>
        </div>

        <div class="flex justify-between px-0.5 text-[8px] uppercase tracking-tighter font-black text-base-content/25">
          <span>{{ $t('msgbox.image_editor.shadows') || 'Shadows' }}</span>
          <span>{{ $t('msgbox.image_editor.midtones') || 'Midtones' }}</span>
          <span>{{ $t('msgbox.image_editor.highlights') || 'Highlights' }}</span>
        </div>
      </div>


      <!-- Preset Gallery -->
      <div class="rounded-box p-2 space-y-3 border border-base-content/5 shadow-sm">
        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content"
          @click.stop="config.infoPanel.showPresets = !config.infoPanel.showPresets"
        >
          <IconPalette class="w-4 h-4" />
          <span class="font-bold mr-auto">{{ $t('msgbox.image_editor.presets.title') }}</span>
          <span v-if="config.infoPanel.showPresets" class="text-[10px] text-primary/70 font-bold uppercase mr-1">{{ presetOptions.find(o => o.value === selectedPreset)?.label || selectedPreset }}</span>
          <TButton
            :icon="config.infoPanel.showPresets ? IconArrowUp : IconArrowDown"
            :buttonSize="'small'"
          />
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @after-enter="onAfterEnter"
          @leave="onLeave"
        >
          <div v-if="config.infoPanel.showPresets" class="flex flex-wrap gap-2 pb-1 overflow-hidden">
            <div 
              v-for="option in presetOptions" 
              :key="option.value"
              @click="selectedPreset = option.value"
              class="shrink-0 w-[calc(33.33%-6px)] max-w-[160px] group cursor-pointer"
            >
              <div 
                :class="[
                  'aspect-4/3 rounded-box border-2 transition-all duration-200 flex items-center justify-center overflow-hidden mb-1 relative',
                  selectedPreset === option.value ? 'border-primary ring-2 ring-primary/20' : 'border-base-content/5 hover:border-base-content/20'
                ]"
              >
                <div class="w-full h-full bg-base-300 flex items-center justify-center relative overflow-hidden rounded-[inherit] isolation-isolate">
                  <img 
                    v-if="fileInfo.thumbnail"
                    :src="fileInfo.thumbnail" 
                    class="w-full h-full object-cover pointer-events-none rounded-[inherit] block"
                    :style="{ 
                      ...getPresetThumbnailStyle(option.value),
                      transform: 'translateZ(0)'
                    }"
                  />
                  <IconPalette v-else class="w-4 h-4 text-base-content/10" />
                </div>
              </div>
              <div class="text-[9px] text-center truncate font-medium text-base-content/50 group-hover:text-base-content transition-colors uppercase tracking-tight">
                {{ option.label }}
              </div>
            </div>
          </div>
        </Transition>
      </div>

      <!-- Adjust Section -->
      <div class="rounded-box p-2 space-y-3 border border-base-content/5 shadow-sm">
        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content"
          @click.stop="config.infoPanel.showAdjust = !config.infoPanel.showAdjust"
        >
          <IconAdjustments class="w-4 h-4" />
          <span class="font-bold mr-auto">{{ $t('msgbox.image_editor.adjustments') }}</span>
          <TButton
            :icon="config.infoPanel.showAdjust ? IconArrowUp : IconArrowDown"
            :buttonSize="'small'"
          />
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @after-enter="onAfterEnter"
          @leave="onLeave"
        >
          <div v-if="config.infoPanel.showAdjust" class="space-y-5 overflow-hidden py-1">
            <!-- Light Group -->
            <div class="space-y-3">
              <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/20 mb-2">{{ $t('msgbox.image_editor.light') || 'Light' }}</div>
              <div v-for="adj in lightSliders" :key="adj.key" class="grid grid-cols-[80px_1fr] gap-x-4 items-center">
                <div class="font-medium text-base-content/40 tracking-wide text-xs">{{ adj.label }}</div>
                <div class="flex items-center gap-2 pr-2">
                  <SliderInput 
                    v-model="adj.model.value" 
                    :min="adj.min" 
                    :max="adj.max" 
                    :step="adj.step" 
                    class="flex-1"
                  />
                  <span class="text-[10px] font-mono text-base-content/60 w-6 text-right shrink-0">{{ adj.valueDisplay }}</span>
                </div>
              </div>
            </div>

            <div class="h-px bg-base-content/5 mx-1"></div>

            <!-- Color Group -->
            <div class="space-y-3">
              <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/20 mb-2">{{ $t('msgbox.image_editor.color') || 'Color' }}</div>
              <div v-for="adj in colorSliders" :key="adj.key" class="grid grid-cols-[80px_1fr] gap-x-4 items-center">
                <div class="font-medium text-base-content/40 tracking-wide text-xs">{{ adj.label }}</div>
                <div class="flex items-center gap-2 pr-2">
                  <SliderInput 
                    v-model="adj.model.value" 
                    :min="adj.min" 
                    :max="adj.max" 
                    :step="adj.step" 
                    class="flex-1"
                  />
                  <span class="text-[10px] font-mono text-base-content/60 w-6 text-right shrink-0">{{ adj.valueDisplay }}</span>
                </div>
              </div>
            </div>
          </div>
        </Transition>
      </div>



    </div>


    <ToolTip ref="toolTipRef" />
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, computed, watch, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import { renameFile, editImage } from '@/common/api';
import { 
  extractFileName, 
  getFileExtension, 
  getFolderPath, 
  formatDimensionText, 
  formatFileSize, 
  formatTimestamp,
  formatDuration,
  formatCaptureSettings,
  formatCameraInfo,
  getCountryName,
  combineFileName,
  isValidFileName,
  getSelectOptions,
  getFullPath
} from '@/common/utils';
import { 
  IconClose, IconLocation, IconArrowDown, IconArrowUp, IconCameraAperture, 
  IconFile, IconFiles, IconRestore, IconSave, IconPalette, IconAdjustments,
  IconCheckAll, IconFavorite, IconUnFavorite, IconTag,
} from '@/common/icons';
import TButton from '@/components/TButton.vue';
import SliderInput from '@/components/SliderInput.vue';
import ToolTip from '@/components/ToolTip.vue';
import MapView from '@/components/MapView.vue';

const props = defineProps({
  fileInfo: {
    type: Object,
    required: false
  },
  multiSelect: {
    type: Boolean,
    default: false
  },
  selectedFiles: {
    type: Array as () => any[],
    default: () => []
  },
});

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();


const emit = defineEmits([
  'close',
  'success',
  'deselect',
  'favoriteAll',
  'unfavoriteAll',
  'tagAll'
]);

// Tabs logic
const activeTab = computed({
  get: () => config.infoPanel.activeTab || 'info',
  set: (val) => config.infoPanel.activeTab = val
});
const toolTipRef = ref<InstanceType<typeof ToolTip> | null>(null);

// Multi-select computed
const multiSelectTotalSize = computed(() => {
  return props.selectedFiles.reduce((total: number, f: any) => total + (f.size || 0), 0);
});

const multiSelectDateRange = computed(() => {
  if (props.selectedFiles.length === 0) return '';
  const dates = props.selectedFiles
    .map((f: any) => f.created_at)
    .filter(Boolean)
    .sort();
  if (dates.length === 0) return '';
  const first = formatTimestamp(dates[0], 'YYYY-MM-DD');
  const last = formatTimestamp(dates[dates.length - 1], 'YYYY-MM-DD');
  return first === last ? first : `${first} — ${last}`;
});

const multiSelectTypeBreakdown = computed(() => {
  const images = props.selectedFiles.filter((f: any) => f.file_type === 1).length;
  const videos = props.selectedFiles.filter((f: any) => f.file_type === 2).length;
  const parts = [];
  if (images > 0) parts.push(`${images} ${images === 1 ? localeMsg.value.info_panel.type_image : localeMsg.value.info_panel.type_images}`);
  if (videos > 0) parts.push(`${videos} ${videos === 1 ? localeMsg.value.info_panel.type_video : localeMsg.value.info_panel.type_videos}`);
  return parts.join(' · ');
});
const isProcessing = ref(false);
const isSaving = ref(false);

let isApplyingPreset = false;
let isInitializing = false;

// Adjustment state
const brightness = ref(0);
const contrast = ref(0);
const saturation = ref(100);
const hue = ref(0);
const blur = ref(0);
const selectedFilter = ref('');
const selectedPreset = ref('natural');

const presets: Record<string, any> = {
  natural: { brightness: 0, contrast: 0, saturation: 100, hue: 0, blur: 0, filter: "" },
  vivid: { brightness: 0, contrast: 10, saturation: 120, hue: 0, blur: 0, filter: "" },
  muted: { brightness: 0, contrast: -10, saturation: 80, hue: 0, blur: 0, filter: "" },
  warm: { brightness: 5, contrast: 0, saturation: 100, hue: 5, blur: 0, filter: "" },
  cool: { brightness: 5, contrast: 0, saturation: 100, hue: -5, blur: 0, filter: "" },
  bw: { brightness: 0, contrast: 0, saturation: 0, hue: 0, blur: 0, filter: "grayscale" },
  vintage: { brightness: 10, contrast: -10, saturation: 60, hue: 0, blur: 0, filter: "sepia" },
  invert: { brightness: 0, contrast: 0, saturation: 100, hue: 0, blur: 0, filter: "invert" },
  kodak: { brightness: 10, contrast: 15, saturation: 120, hue: -5, blur: 0, filter: "" },
  toyo: { brightness: 5, contrast: 0, saturation: 110, hue: 5, blur: 0, filter: "" },
  cinematic: { brightness: 0, contrast: 20, saturation: 80, hue: 0, blur: 0, filter: "" },
  dramatic: { brightness: 0, contrast: 30, saturation: 110, hue: 0, blur: 0, filter: "" },
  cyberpunk: { brightness: 10, contrast: 20, saturation: 130, hue: -15, blur: 0, filter: "" },
};

// Histogram state
const histogramData = ref<number[]>(new Array(256).fill(0));

const updateRealHistogram = () => {
  if (!props.fileInfo || !props.fileInfo.thumbnail) return;
  
  const img = new Image();
  img.crossOrigin = "anonymous";
  img.src = props.fileInfo.thumbnail;
  
  img.onload = () => {
    const canvas = document.createElement("canvas");
    const ctx = canvas.getContext("2d", { willReadFrequently: true });
    if (!ctx) return;
    
    // Use a larger size for better resolution (256x256 matches histogram bins)
    const size = 256;
    canvas.width = size;
    canvas.height = size;
    ctx.drawImage(img, 0, 0, size, size);
    
    const imageData = ctx.getImageData(0, 0, size, size).data;
    const hist = new Array(256).fill(0);
    
    for (let i = 0; i < imageData.length; i += 4) {
      // Luminosity calculation (Rec. 709)
      const r = imageData[i];
      const g = imageData[i + 1];
      const b = imageData[i + 2];
      const gray = Math.round(0.2126 * r + 0.7152 * g + 0.0722 * b);
      hist[gray]++;
    }
    
    // Normalize heights (max height is 58 to leave some padding)
    const maxVal = Math.max(...hist);
    if (maxVal > 0) {
      histogramData.value = hist.map(v => (v / maxVal) * 58);
    } else {
      histogramData.value = new Array(256).fill(0);
    }
  };
};

const presetOptions = computed(() => [
  { value: 'custom', label: localeMsg.value.msgbox.image_editor.presets.custom },
  { value: 'natural', label: localeMsg.value.msgbox.image_editor.presets.natural },
  { value: 'vivid', label: localeMsg.value.msgbox.image_editor.presets.vivid },
  { value: 'muted', label: localeMsg.value.msgbox.image_editor.presets.muted },
  { value: 'warm', label: localeMsg.value.msgbox.image_editor.presets.warm },
  { value: 'cool', label: localeMsg.value.msgbox.image_editor.presets.cool },
  { value: 'bw', label: localeMsg.value.msgbox.image_editor.presets.bw },
  { value: 'vintage', label: localeMsg.value.msgbox.image_editor.presets.vintage },
  { value: 'kodak', label: localeMsg.value.msgbox.image_editor.presets.kodak },
  { value: 'toyo', label: localeMsg.value.msgbox.image_editor.presets.toyo },
  { value: 'cinematic', label: localeMsg.value.msgbox.image_editor.presets.cinematic },
  { value: 'dramatic', label: localeMsg.value.msgbox.image_editor.presets.dramatic },
  { value: 'cyberpunk', label: localeMsg.value.msgbox.image_editor.presets.cyberpunk },
  { value: 'invert', label: localeMsg.value.msgbox.image_editor.presets.invert },
]);

// Resize state
const resizedWidth = ref(0);
const resizedHeight = ref(0);
const resizedPercentage = ref(100);
const newFileName = ref('');

const thumbnailStyle = computed(() => {
  return {
    filter: `
      brightness(${100 + brightness.value}%)
      contrast(${100 + contrast.value}%)
      blur(${blur.value}px)
      hue-rotate(${hue.value}deg)
      saturate(${saturation.value}%)
      ${selectedFilter.value === 'grayscale' ? 'grayscale(100%)' : ''}
      ${selectedFilter.value === 'sepia' ? 'sepia(100%)' : ''}
      ${selectedFilter.value === 'invert' ? 'invert(100%)' : ''}
    `
  };
});

const hasChanges = computed(() => uiStore.hasActiveChanges(props.fileInfo));

const lightSliders = computed(() => [
  { key: 'brightness', label: localeMsg.value.msgbox.image_editor.brightness, model: brightness, min: -100, max: 100, step: 1, valueDisplay: `${brightness.value > 0 ? '+' : ''}${brightness.value}` },
  { key: 'contrast', label: localeMsg.value.msgbox.image_editor.contrast, model: contrast, min: -100, max: 100, step: 1, valueDisplay: `${contrast.value > 0 ? '+' : ''}${contrast.value}` },
]);

const colorSliders = computed(() => [
  { key: 'saturation', label: localeMsg.value.msgbox.image_editor.saturation, model: saturation, min: 0, max: 200, step: 1, valueDisplay: `${saturation.value}%` },
  { key: 'hue', label: localeMsg.value.msgbox.image_editor.hue_rotate, model: hue, min: -180, max: 180, step: 1, valueDisplay: `${hue.value}°` },
]);

// Generate a crisp, Adobe-style smooth histogram path
const generateHistogramPath = () => {
  if (!props.fileInfo || !histogramData.value) return "";
  
  const width = 256;
  const height = 64;
  const br = (100 + brightness.value) / 100;
  const ct = (100 + contrast.value) / 100;

  const sampledPoints: {x: number, y: number}[] = [];
  const step = 2; // Higher granularity

  for (let i = 0; i <= 256; i += step) {
    // Tighter average ( Adobe balance) for crisper peaks
    let sum = 0;
    let count = 0;
    const windowSize = 2; 
    for (let j = Math.max(0, i - windowSize); j < Math.min(256, i + windowSize); j++) {
      sum += histogramData.value[j];
      count++;
    }
    const val = count > 0 ? sum / count : 0;
    
    // Multiplicative brightness then contrast (matching CSS filters)
    let x = (i * br - 128) * ct + 128;
    let y = height - val;

    if (x >= -10 && x <= width + 10) {
      sampledPoints.push({ x, y });
    }
  }
  
  if (sampledPoints.length < 2) return "";

  let path = `M 0,${height}`;
  
  for (let i = 0; i < sampledPoints.length; i++) {
    const p = sampledPoints[i];
    if (i === 0) {
      path += ` L ${p.x.toFixed(1)},${p.y.toFixed(1)}`;
    } else {
      const prev = sampledPoints[i - 1];
      const cp1x = prev.x + (p.x - prev.x) / 2;
      const cp1y = prev.y;
      const cp2x = prev.x + (p.x - prev.x) / 2;
      const cp2y = p.y;
      path += ` C ${cp1x.toFixed(1)},${cp1y.toFixed(1)} ${cp2x.toFixed(1)},${cp2y.toFixed(1)} ${p.x.toFixed(1)},${p.y.toFixed(1)}`;
    }
  }
  
  path += ` L ${width},${height} Z`;
  return path;
};

const getPresetThumbnailStyle = (presetKey: string) => {
  if (presetKey === 'custom') return thumbnailStyle.value;
  const p = presets[presetKey];
  if (!p) return {};
  return {
    filter: `
      brightness(${100 + p.brightness}%)
      contrast(${100 + p.contrast}%)
      blur(${p.blur}px)
      hue-rotate(${p.hue}deg)
      saturate(${p.saturation}%)
      ${p.filter === 'grayscale' ? 'grayscale(100%)' : ''}
      ${p.filter === 'sepia' ? 'sepia(100%)' : ''}
      ${p.filter === 'invert' ? 'invert(100%)' : ''}
    `
  };
};

watch(selectedPreset, (newVal) => {
  if (newVal === 'custom') return;
  const p = presets[newVal];
  if (p) {
    isApplyingPreset = true;
    brightness.value = p.brightness;
    contrast.value = p.contrast;
    saturation.value = p.saturation;
    hue.value = p.hue;
    blur.value = p.blur;
    selectedFilter.value = p.filter;
    
    // Sync to store for preview
    if (props.fileInfo && !isInitializing) {
       uiStore.setActiveAdjustments(props.fileInfo.file_path, {
        brightness: brightness.value,
        contrast: contrast.value,
        saturation: saturation.value,
        hue: hue.value,
        blur: blur.value,
         filter: selectedFilter.value,
       });
    }
    
    nextTick(() => isApplyingPreset = false);
  }
});

watch([brightness, contrast, saturation, hue, blur, selectedFilter], () => {
  if (isApplyingPreset) return;
  if (selectedPreset.value !== 'custom') {
    const p = presets[selectedPreset.value];
    if (p && (brightness.value !== p.brightness || contrast.value !== p.contrast || saturation.value !== p.saturation || hue.value !== p.hue || blur.value !== p.blur || selectedFilter.value !== p.filter)) {
      selectedPreset.value = 'custom';
    }
  }
  
  if (isInitializing) return;

  // Sync to store for preview
  if (props.fileInfo) {
    uiStore.setActiveAdjustments(props.fileInfo.file_path, {
      brightness: brightness.value,
      contrast: contrast.value,
      saturation: saturation.value,
      hue: hue.value,
      blur: blur.value,
      filter: selectedFilter.value,
    });
  }
});


const resetAdjustments = () => {
  const p = presets.natural;
  brightness.value = p.brightness;
  contrast.value = p.contrast;
  saturation.value = p.saturation;
  hue.value = p.hue;
  blur.value = p.blur;
  selectedFilter.value = p.filter;
  selectedPreset.value = 'natural';
};

const resetAll = () => {
  uiStore.clearActiveAdjustments();
  initEditState();
};

const quickSave = async (): Promise<boolean> => {
  if (!props.fileInfo) return false;
  
  isSaving.value = true;

  const currentFilePath = props.fileInfo.file_path;
  const currentFileName = props.fileInfo.name;
  const currentWidth = props.fileInfo.width;
  const currentHeight = props.fileInfo.height;
  const currentOrientation = props.fileInfo.e_orientation || 1;

  const ext = getFileExtension(currentFileName).toLowerCase();
  const outputFormat = (ext === 'jpg' || ext === 'jpeg') ? 'jpg' : ext;

  const editParams = {
    sourceFilePath: currentFilePath,
    destFilePath: currentFilePath,
    outputFormat: outputFormat,
    quality: 80,
    orientation: currentOrientation,
    flipHorizontal: false,
    flipVertical: false,
    rotate: 0,
    crop: { x: 0, y: 0, width: 0, height: 0 },
    resize: { width: currentWidth, height: currentHeight },
    filter: selectedFilter.value || null,
    brightness: brightness.value !== 0 ? brightness.value : null,
    contrast: contrast.value !== 0 ? contrast.value : null,
    blur: blur.value > 0 ? blur.value : null,
    hue_rotate: hue.value !== 0 ? hue.value : null,
    saturation: saturation.value !== 100 ? saturation.value / 100.0 : null,
  };

  // Background processing
  try {
    const success = await editImage(editParams);
    if (success) {
      uiStore.updateFileVersion(currentFilePath);
      emit('success');
      toolTipRef.value?.showTip(localeMsg.value.tooltip.save_image.success);
      return true;
    } else {
      isSaving.value = false;
      toolTipRef.value?.showTip(localeMsg.value.tooltip.save_image.failed, true);
      return false;
    }
  } catch {
    isSaving.value = false;
    toolTipRef.value?.showTip(localeMsg.value.tooltip.save_image.failed, true);
    return false;
  }
};

const initEditState = () => {
  if (!props.fileInfo) return;
  
  isInitializing = true;

  // Check if we have active adjustments for this file in store
  if (uiStore.activeAdjustments.filePath === props.fileInfo.file_path) {
    const adj = uiStore.activeAdjustments as any;
    brightness.value = adj.brightness || 0;
    contrast.value = adj.contrast || 0;
    saturation.value = adj.saturation ?? 100;
    hue.value = adj.hue || 0;
    blur.value = adj.blur || 0;
    selectedFilter.value = adj.filter || '';
  } else {
    // New file or no adjustments, reset to defaults
    resetAdjustments();
  }
  
  newFileName.value = extractFileName(props.fileInfo.name).name;
  
  nextTick(() => {
    isInitializing = false;
  });
};


watch(() => props.fileInfo?.id, () => {
  if (activeTab.value === 'edit') {
    initEditState();
    updateRealHistogram();
  }
}, { immediate: true });

// When the thumbnail loads asynchronously (e.g. first item), regenerate histogram
watch(() => props.fileInfo?.thumbnail, (newThumb) => {
  if (newThumb && activeTab.value === 'edit') {
    updateRealHistogram();
  }
});

// Auto-switch to Info tab when entering multi-select mode
watch(() => props.multiSelect, (isMulti) => {
  if (isMulti && activeTab.value === 'edit') {
    activeTab.value = 'info';
  }
});

watch(() => uiStore.activeAdjustments.filePath, (newVal) => {
  if (!newVal) {
    if (activeTab.value === 'edit') {
      initEditState();
    }
    isSaving.value = false;
  }
});

onMounted(() => {
  if (activeTab.value === 'edit') {
    initEditState();
    updateRealHistogram();
  }
});

onBeforeUnmount(() => {
  // We do NOT clear active adjustments here anymore to support persistence across panel toggles
  // uiStore.clearActiveAdjustments();
});

watch(activeTab, (newVal) => {
  if (newVal === 'edit') {
    initEditState();
    updateRealHistogram();
  }
});

// Rename logic
const isRenaming = ref(false);
const renamingName = ref('');
const renameInputRef = ref<HTMLInputElement | null>(null);

const startRename = () => {
  if (!props.fileInfo) return;
  
  const { name } = extractFileName(props.fileInfo.name);
  renamingName.value = name;
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
  const newPath = await renameFile(props.fileInfo.id, props.fileInfo.file_path, fullNewName);
  
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

const onBeforeEnter = (el: any) => {
  el.style.opacity = '0';
  el.style.height = '0';
}

const onEnter = (el: any) => {
  el.style.transition = 'all 0.1s ease';
  // Check scrollHeight to know final height
  el.style.height = el.scrollHeight + 'px';
  el.style.opacity = '1';
}

const onAfterEnter = (el: any) => {
  el.style.height = '';
}

const onLeave = (el: any) => {
  el.style.transition = 'all 0.1s ease';
  // Force height back to explicit pixel value for animation
  el.style.height = el.scrollHeight + 'px';
  // Force repaint to ensure transition triggers
  // eslint-disable-next-line no-unused-expressions
  el.offsetHeight; 
  el.style.height = '0';
  el.style.opacity = '0';
}
defineExpose({
  quickSave
});
</script>