<template>
  <div class="w-full h-full rounded-box bg-base-200 flex flex-col overflow-hidden">
    <!-- Tabs & Close -->
    <div class="flex items-center w-full shrink-0 px-2 mb-2">
      <div role="tablist" class="tabs tabs-sm tabs-border flex-1">
        <a role="tab" :class="['tab mx-1', { 'tab-active': activeTab === 'info' }]" @click="activeTab = 'info'">{{ $t('info_panel.tabs[0]') }}</a>
        <a role="tab" :class="['tab mx-1', { 'tab-active': activeTab === 'edit' }]" @click="activeTab = 'edit'">{{ $t('image_viewer.toolbar.edit') }}</a>
      </div>
      <div class="mt-2">
        <TButton
          :icon="IconClose"
          :buttonSize="'small'"
          @click.stop="$emit('close')"
        />
      </div>
    </div>

    <!-- Info Content -->
    <div v-if="fileInfo" v-show="activeTab === 'info'" class="mb-2 px-2 flex-1 overflow-y-auto overflow-x-hidden space-y-2 flex flex-col">

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
      <div class="rounded-box p-2 space-y-3 border border-base-content/5 shadow-sm">

        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content transition-all duration-200 ease-in-out" 
          @click.stop="config.infoPanel.showBasicInfo = !config.infoPanel.showBasicInfo"
        >
          <component :is="fileInfo?.file_type === 2 ? IconFile : IconFile" class="w-4 h-4" />
          <span class="font-bold mr-auto">{{ $t('file_info.title') }}</span>
          <TButton
            :icon="config.infoPanel.showBasicInfo ? IconArrowUp : IconArrowDown"
            :buttonSize="'small'"
          />
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @leave="onLeave"
        >
          <div v-if="config.infoPanel.showBasicInfo" class="grid grid-cols-[100px_1fr] gap-y-3 gap-x-4 text-sm overflow-hidden">
            <!-- Name (Moved here) -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.name') }}</div>
            <div class="flex items-center">
              <input 
                v-if="isRenaming"
                ref="renameInputRef"
                v-model="renamingName" 
                class="font-bold text-sm text-base-content input input-xs input-bordered p-1 h-6 leading-6 w-full"
                @blur="finishRename"
                @keydown.enter="finishRename"
                @keydown.esc="cancelRename"
                @click.stop
              />
              <span v-else 
                class="font-bold text-sm text-base-content/70 break-all cursor-text hover:bg-base-content/10 rounded px-1 -mx-1 transition-colors"
                @click.stop="startRename"
              >{{ fileInfo?.name }}</span>
            </div>

            <!-- Album -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.album_name') }}</div>
            <div class="text-base-content/70 break-all">{{ fileInfo?.album_name }}</div>

            <!-- Path -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.folder') }}</div>
            <div class="text-base-content/70 break-all">{{ getFolderPath(fileInfo?.file_path) }}</div>

            <!-- Size -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.size') }}</div>
            <div class="text-base-content/70">{{ formatFileSize(fileInfo?.size) }}</div>

            <!-- Dimension -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.dimension') }}</div>
            <div class="text-base-content/70">{{ formatDimensionText(fileInfo?.width, fileInfo?.height) }}</div>

            <!-- Duration -->
            <template v-if="fileInfo?.file_type === 2">
              <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.duration') }}</div>
              <div class="text-base-content/70">{{ formatDuration(fileInfo?.duration) }}</div>
            </template>

            <!-- Created At -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.created_at') }}</div>
            <div class="text-base-content/70">{{ formatTimestamp(fileInfo?.created_at, $t('format.date_time')) }}</div>

            <!-- Modified At -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.modified_at') }}</div>
            <div class="text-base-content/70">{{ formatTimestamp(fileInfo?.modified_at, $t('format.date_time')) }}</div>

            <!-- Tags -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.tags') }}</div>
            <div class="text-base-content/70 flex flex-wrap gap-1">
              <template v-if="fileInfo?.tags && fileInfo.tags.length">
                <span v-for="tag in fileInfo.tags" :key="tag.id" class="badge">{{ tag.name }}</span>
              </template>
            </div>

            <!-- Comment -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.comment') }}</div>
            <div class="text-base-content/70 wrap-break-words whitespace-pre-wrap">{{ fileInfo?.comments }}</div>
          </div>
        </Transition>
      </div>

      <!-- Metadata Section -->
      <div class="rounded-box p-2 space-y-3 border border-base-content/5 shadow-sm">

        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content" @click.stop="config.infoPanel.showMetadata = !config.infoPanel.showMetadata">
          <IconCameraAperture class="w-4 h-4 " /> 
          <span class="font-bold mr-auto">{{ $t('file_info.metadata') }}</span>
          <TButton
            :icon="config.infoPanel.showMetadata ? IconArrowUp : IconArrowDown"
            :buttonSize="'small'"
          />
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @leave="onLeave"
        >
          <div v-if="config.infoPanel.showMetadata" class="grid grid-cols-[100px_1fr] gap-y-3 gap-x-4 text-sm overflow-hidden">
            <!-- Camera -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.camera') }}</div>
            <div class="text-base-content/70">{{ formatCameraInfo(fileInfo?.e_make, fileInfo?.e_model) }}</div>

            <!-- Lens -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.lens') }}</div>
            <div class="text-base-content/70">{{ fileInfo?.e_lens_model }}</div>

            <!-- Capture Settings -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.capture_settings') }}</div>
            <div class="text-base-content/70">{{ formatCaptureSettings(fileInfo?.e_focal_length, fileInfo?.e_exposure_time, fileInfo?.e_f_number, fileInfo?.e_iso_speed, fileInfo?.e_exposure_bias) }}</div>

            <!-- Software -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.software') }}</div>
            <div class="text-base-content/70">{{ fileInfo?.e_software }}</div>

            <!-- Taken By -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.taken_by') }}</div>
            <div class="text-base-content/70">{{ fileInfo?.e_artist }}</div>

            <!-- Copyright -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.copyright') }}</div>
            <div class="text-base-content/70">{{ fileInfo?.e_copyright }}</div>

            <!-- Taken At -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.taken_at') }}</div>
            <div class="text-base-content/70">{{ fileInfo?.e_date_time }}</div>

            <!-- Description -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.description') }}</div>
            <div class="text-base-content/70 wrap-break-words">{{ fileInfo?.e_description }}</div>

            <!-- Geo Location -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.geo_location') }}</div>
            <div class="text-base-content/70">{{ formatGeoLocation() }}</div>
          </div>
        </Transition>
      </div>

      <!-- Map View -->
      <div v-if="fileInfo?.gps_latitude && fileInfo?.gps_longitude" 
        class="rounded-box p-2 space-y-3 border border-base-content/5 shadow-sm flex flex-col transition-[flex-grow]" 
        :class="{ 'flex-1 min-h-[300px]': config.infoPanel.showMap }">
        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content shrink-0" @click.stop="config.infoPanel.showMap = !config.infoPanel.showMap">
          <IconLocation class="w-4 h-4 " /> 
          <span class="font-bold mr-auto">{{ $t('file_info.location') }}</span>
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
            <div class="w-full rounded-box overflow-hidden relative z-0 flex-1 min-h-[200px]">
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
    <div v-if="fileInfo && fileInfo.file_type === 1" v-show="activeTab === 'edit'" class="mb-2 px-2 flex-1 overflow-y-auto overflow-x-hidden space-y-2 flex flex-col min-h-0">

      <!-- Adjust Section -->
      <div class="rounded-box p-2 space-y-3 border border-base-content/5 shadow-sm">
        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content"
          @click.stop="config.infoPanel.showAdjust = !config.infoPanel.showAdjust"
        >
          <IconPalette class="w-4 h-4" />
          <span class="font-bold mr-auto">{{ $t('msgbox.image_editor.tab_edit') }}</span>
          <div class="flex items-center gap-1">
            <TButton v-if="hasAdjustments" 
              :icon="IconRestore" 
              :buttonSize="'small'" 
              :selected="true"
              :tooltip="$t('msgbox.image_editor.reset')"
              @click.stop="resetAdjustments">
            </TButton>
            <TButton
              :icon="config.infoPanel.showAdjust ? IconArrowUp : IconArrowDown"
              :buttonSize="'small'"
            />
          </div>
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @leave="onLeave"
        >
          <div v-if="config.infoPanel.showAdjust" class="space-y-4 overflow-hidden">
            <div class="grid grid-cols-[100px_1fr] gap-x-4 items-center">
              <label class="font-medium text-base-content/30 tracking-wide text-sm">{{ $t('msgbox.image_editor.presets.title') }}</label>
              <select v-model="selectedPreset" class="select select-bordered select-xs w-full h-7">
                <option v-for="option in presetOptions" :value="option.value" :key="option.value">{{ option.label }}</option>
              </select>
            </div>
            
            <div class="space-y-3">
              <div v-for="adj in adjustmentSliders" :key="adj.key" class="grid grid-cols-[100px_1fr] gap-x-4 items-center">
                <div class="font-medium text-base-content/30 tracking-wide text-sm">{{ adj.label }}</div>
                <div class="flex items-center gap-2 pr-2">
                  <SliderInput 
                    v-model="adj.model.value" 
                    :min="adj.min" 
                    :max="adj.max" 
                    :step="adj.step" 
                    class="flex-1"
                  />
                  <span class="text-xs text-base-content/70 w-8 text-right shrink-0">{{ adj.valueDisplay }}</span>
                </div>
              </div>
            </div>
          </div>
        </Transition>
      </div>

      <!-- Resize Section -->
      <div class="rounded-box p-2 space-y-3 border border-base-content/5 shadow-sm">
        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content"
          @click.stop="config.infoPanel.showResize = !config.infoPanel.showResize"
        >
          <IconResize class="w-4 h-4" />
          <span class="font-bold mr-auto">{{ $t('msgbox.image_editor.resize') }}</span>
          <TButton
            :icon="config.infoPanel.showResize ? IconArrowUp : IconArrowDown"
            :buttonSize="'small'"
          />
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @leave="onLeave"
        >
          <div v-if="config.infoPanel.showResize" class="space-y-3 overflow-hidden">
            <div class="grid grid-cols-[100px_1fr] gap-x-4 items-center">
              <div class="font-medium text-base-content/30 tracking-wide text-sm">{{ $t('msgbox.image_editor.width') }}</div>
              <input type="number" class="input input-bordered input-xs h-7 w-full font-bold text-base-content/70"
                v-model.number="resizedWidth"
                @blur="handleResizeInput('width')"
              />
            </div>
            <div class="grid grid-cols-[100px_1fr] gap-x-4 items-center">
              <div class="font-medium text-base-content/30 tracking-wide text-sm">{{ $t('msgbox.image_editor.height') }}</div>
              <input type="number" class="input input-bordered input-xs h-7 w-full font-bold text-base-content/70"
                v-model.number="resizedHeight"
                @blur="handleResizeInput('height')"
              />
            </div>
            <div class="grid grid-cols-[100px_1fr] gap-x-4 items-center">
              <div class="font-medium text-base-content/30 tracking-wide text-sm">{{ $t('msgbox.image_editor.percentage') }}</div>
              <div class="join w-full">
                <input type="number" class="input input-bordered input-xs h-7 join-item flex-1 font-bold text-base-content/70"
                  v-model.number="resizedPercentage"
                  @blur="handleResizeInput('percentage')"
                />
                <span class="btn btn-xs h-7 join-item no-animation pointer-events-none opacity-50 px-2">%</span>
              </div>
            </div>
          </div>
        </Transition>
      </div>

      <!-- Save Section -->
      <div class="rounded-box p-2 space-y-3 border border-base-content/5 shadow-sm">
        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content"
          @click.stop="config.infoPanel.showSettings = !config.infoPanel.showSettings"
        >
          <IconSave class="w-4 h-4" />
          <span class="font-bold mr-auto">{{ $t('msgbox.image_editor.options') }}</span>
          <TButton
            :icon="config.infoPanel.showSettings ? IconArrowUp : IconArrowDown"
            :buttonSize="'small'"
          />
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @leave="onLeave"
        >
          <div v-if="config.infoPanel.showSettings" class="space-y-4 overflow-hidden">
            <div class="grid grid-cols-[100px_1fr] gap-x-4 items-center">
              <label class="font-medium text-base-content/30 tracking-wide text-sm">{{ $t('msgbox.image_editor.save_as') }}</label>
              <select v-model="config.imageEditor.saveAs" class="select select-bordered select-xs h-7 w-full font-bold text-base-content/70">
                <option v-for="option in fileSaveAsOptions" :value="option.value" :key="option.value">{{ option.label }}</option>
              </select>
            </div>
            
            <div class="grid grid-cols-[100px_1fr] gap-x-4 items-center" v-if="config.imageEditor.saveAs !== 0">
              <label class="font-medium text-base-content/30 tracking-wide text-sm">{{ $t('msgbox.image_editor.format') }}</label>
              <select v-model="config.imageEditor.format" class="select select-bordered select-xs h-7 w-full font-bold text-base-content/70">
                <option v-for="option in fileFormatOptions" :value="option.value" :key="option.value">{{ option.label }}</option>
              </select>
            </div>
            
            <div v-if="config.imageEditor.saveAs !== 0 && config.imageEditor.format == 0" class="grid grid-cols-[100px_1fr] gap-x-4 items-center">
              <label class="font-medium text-base-content/30 tracking-wide text-sm">{{ $t('msgbox.image_editor.quality') }}</label>
              <select v-model="config.imageEditor.quality" class="select select-bordered select-xs h-7 w-full font-bold text-base-content/70">
                <option v-for="option in fileQualityOptions" :value="option.value" :key="option.value">{{ option.label }}</option>
              </select>
            </div>
          </div>
        </Transition>
      </div>

      <!-- Action Button -->
      <div class="flex flex-col gap-2 p-2">
         <button class="btn btn-primary btn-sm w-full" :disabled="isProcessing" @click="clickSave">
           {{ config.imageEditor.saveAs === 1 ? $t('msgbox.image_editor.save_as_new') : $t('msgbox.image_editor.overwrite') }}
         </button>
      </div>
    </div>

    <!-- Overwrite Confirm -->
    <MessageBox v-if="showOverwriteConfirm"
      :title="$t('msgbox.image_editor.overwrite')"
      :message="$t('msgbox.image_editor.overwrite_confirm')"
      :warningOk="true"
      :OkText="$t('msgbox.ok')"
      :cancelText="$t('msgbox.cancel')"
      @ok="handleOverwriteConfirm"
      @cancel="showOverwriteConfirm = false"
    />

    <ToolTip ref="toolTipRef" />
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, computed, watch, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import { renameFile, editImage, checkFileExists } from '@/common/api';
import { 
  formatTimestamp, formatFileSize, formatDuration, formatDimensionText, 
  getFolderPath, formatCaptureSettings, formatCameraInfo, getCountryName,
  extractFileName, combineFileName, isValidFileName, getSelectOptions,
  getFileExtension, getFullPath
} from '@/common/utils';
import { 
  IconClose, IconLocation, IconArrowDown, IconArrowUp, IconCameraAperture, 
  IconFavorite, IconFile, IconRestore, IconImageEdit, IconCrop, IconSave, IconPalette, IconResize
} from '@/common/icons';
import TButton from '@/components/TButton.vue';
import SliderInput from '@/components/SliderInput.vue';
import MessageBox from '@/components/MessageBox.vue';
import ToolTip from '@/components/ToolTip.vue';
import MapView from '@/components/MapView.vue';

const props = defineProps({
  fileInfo: {
    type: Object,
    required: false
  },
});

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();


const emit = defineEmits([
  'close',
  'success'
]);

// Tabs logic
const activeTab = computed({
  get: () => config.infoPanel.activeTab || 'info',
  set: (val) => config.infoPanel.activeTab = val
});
const toolTipRef = ref<InstanceType<typeof ToolTip> | null>(null);
const isProcessing = ref(false);

// Adjustment state
const brightness = ref(0);
const contrast = ref(0);
const saturation = ref(100);
const hue = ref(0);
const blur = ref(0);
const selectedFilter = ref('');
const selectedPreset = ref('natural');

const presets: Record<string, any> = {
  natural: { brightness: 0, contrast: 0, saturation: 100, hue: 0, blur: 0, filter: '' },
  vivid: { brightness: 0, contrast: 10, saturation: 120, hue: 0, blur: 0, filter: '' },
  muted: { brightness: 0, contrast: -10, saturation: 80, hue: 0, blur: 0, filter: '' },
  warm: { brightness: 5, contrast: 0, saturation: 100, hue: 5, blur: 0, filter: '' },
  cool: { brightness: 5, contrast: 0, saturation: 100, hue: -5, blur: 0, filter: '' },
  bw: { brightness: 0, contrast: 0, saturation: 0, hue: 0, blur: 0, filter: 'grayscale' },
  vintage: { brightness: 10, contrast: -10, saturation: 60, hue: 0, blur: 0, filter: 'sepia' },
  invert: { brightness: 0, contrast: 0, saturation: 100, hue: 0, blur: 0, filter: 'invert' },
  kodak: { brightness: 10, contrast: 15, saturation: 120, hue: -5, blur: 0, filter: '' },
  toyo: { brightness: 5, contrast: 0, saturation: 110, hue: 5, blur: 0, filter: '' },
  cinematic: { brightness: 0, contrast: 20, saturation: 80, hue: 0, blur: 0, filter: '' },
  dramatic: { brightness: 0, contrast: 30, saturation: 110, hue: 0, blur: 0, filter: '' },
  cyberpunk: { brightness: 10, contrast: 20, saturation: 130, hue: -15, blur: 0, filter: '' },
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

// Save options
const fileSaveAsOptions = computed(() => getSelectOptions(localeMsg.value.msgbox.image_editor.save_as_options));
const fileFormatOptions = computed(() => getSelectOptions(localeMsg.value.msgbox.image_editor.format_options));
const fileQualityOptions = computed(() => getSelectOptions(localeMsg.value.msgbox.image_editor.quality_options));

const showOverwriteConfirm = ref(false);

const thumbnailStyle = computed(() => ({
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
}));

const hasAdjustments = computed(() => 
  brightness.value !== 0 || contrast.value !== 0 || blur.value !== 0 || hue.value !== 0 || saturation.value !== 100 || selectedFilter.value !== ''
);

const adjustmentSliders = computed(() => [
  { key: 'brightness', label: localeMsg.value.msgbox.image_editor.brightness, model: brightness, min: -100, max: 100, step: 1, valueDisplay: `${brightness.value > 0 ? '+' : ''}${brightness.value}` },
  { key: 'contrast', label: localeMsg.value.msgbox.image_editor.contrast, model: contrast, min: -100, max: 100, step: 1, valueDisplay: `${contrast.value > 0 ? '+' : ''}${contrast.value}` },
  { key: 'saturation', label: localeMsg.value.msgbox.image_editor.saturation, model: saturation, min: 0, max: 200, step: 1, valueDisplay: `${saturation.value}%` },
  { key: 'hue', label: localeMsg.value.msgbox.image_editor.hue_rotate, model: hue, min: -180, max: 180, step: 1, valueDisplay: `${hue.value}°` },
]);

let isApplyingPreset = false;
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
});

const initEditState = () => {
  if (!props.fileInfo) return;
  resizedWidth.value = props.fileInfo.width;
  resizedHeight.value = props.fileInfo.height;
  resizedPercentage.value = 100;
  newFileName.value = extractFileName(props.fileInfo.name).name;
  
  const ext = getFileExtension(props.fileInfo.name).toLowerCase();
  if (['jpg', 'jpeg'].includes(ext)) config.imageEditor.format = 0;
  else if (ext === 'png') config.imageEditor.format = 1;
  else if (ext === 'webp') config.imageEditor.format = 2;
  
  resetAdjustments();
};

const resetAdjustments = () => {
  selectedPreset.value = 'natural';
};

const handleResizeInput = (type: string) => {
  if (!props.fileInfo) return;
  const originalWidth = props.fileInfo.width;
  const originalHeight = props.fileInfo.height;

  if (type === 'width') {
    resizedPercentage.value = Math.round((resizedWidth.value / originalWidth) * 100);
    resizedHeight.value = Math.round((originalHeight * resizedPercentage.value) / 100);
  } else if (type === 'height') {
    resizedPercentage.value = Math.round((resizedHeight.value / originalHeight) * 100);
    resizedWidth.value = Math.round((originalWidth * resizedPercentage.value) / 100);
  } else if (type === 'percentage') {
    resizedWidth.value = Math.round((originalWidth * resizedPercentage.value) / 100);
    resizedHeight.value = Math.round((originalHeight * resizedPercentage.value) / 100);
  }
};

const setEditParams = (overrides: any = {}) => {
  if (!props.fileInfo) return null;
  
  const ext = fileFormatOptions.value[config.imageEditor.format].label.toLowerCase();
  const outputFormat = overrides.outputFormat || (ext === 'jpg' || ext === 'jpeg' ? 'jpg' : ext);
  
  return {
    sourceFilePath: props.fileInfo.file_path,
    destFilePath: overrides.destFilePath || props.fileInfo.file_path,
    outputFormat: outputFormat,
    quality: [90, 80, 60][config.imageEditor.quality] || 80,
    resize: {
      width: resizedWidth.value,
      height: resizedHeight.value,
    },
    // adjustments
    filter: selectedFilter.value || null,
    brightness: brightness.value !== 0 ? brightness.value : null,
    contrast: contrast.value !== 0 ? contrast.value : null,
    blur: blur.value > 0 ? blur.value : null,
    hue_rotate: hue.value !== 0 ? hue.value : null,
    saturation: saturation.value !== 100 ? saturation.value / 100.0 : null,
  };
};

const executeSave = async (overrides: any = {}) => {
  isProcessing.value = true;
  let success = false;
  let params = null;
  try {
    params = setEditParams(overrides);
    if (params) success = await editImage(params);
  } finally {
    isProcessing.value = false;
    if (success && params && props.fileInfo) {
      uiStore.updateFileVersion(props.fileInfo.file_path);
      emit('success');
      toolTipRef.value?.showTip(localeMsg.value.tooltip.save_image.success);
    } else {
      toolTipRef.value?.showTip(localeMsg.value.tooltip.save_image.failed, true);
    }
  }
};

const clickSave = async () => {
  if (isProcessing.value || !props.fileInfo) return;

  if (config.imageEditor.saveAs === 1) { // Save as new
    isProcessing.value = true;
    try {
      const folderPath = getFolderPath(props.fileInfo.file_path);
      const ext = fileFormatOptions.value[config.imageEditor.format].label.toLowerCase();
      let baseName = newFileName.value;
      let counter = 1;
      let candidateName = `${baseName}_${counter}`;
      let candidatePath = getFullPath(folderPath, combineFileName(candidateName, ext));
      
      while (await checkFileExists(candidatePath)) {
        counter++;
        candidateName = `${baseName}_${counter}`;
        candidatePath = getFullPath(folderPath, combineFileName(candidateName, ext));
      }
      await executeSave({ fileName: candidateName, destFilePath: candidatePath });
    } catch(err) {
      isProcessing.value = false;
    }
  } else {
    showOverwriteConfirm.value = true;
  }
};

const handleOverwriteConfirm = () => {
  showOverwriteConfirm.value = false;
  executeSave();
};

watch(() => props.fileInfo, () => {
  if (activeTab.value === 'edit') initEditState();
}, { deep: true, immediate: true });

onMounted(() => {
  if (activeTab.value === 'edit') initEditState();
});

watch(activeTab, (newVal) => {
  if (newVal === 'edit') initEditState();
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

</script>