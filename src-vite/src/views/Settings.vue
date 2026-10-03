<template>
  <div class="w-screen h-screen flex flex-col bg-base-300 text-base-content/70 overflow-hidden">
    <!-- Title Bar -->
    <TitleBar :titlebar="$t('sidebar.settings')" :resizable="false" viewName="Settings" class="shrink-0 z-50" />

    <div class="flex flex-1 overflow-hidden relative">
      <!-- Sidebar -->
      <div class="w-40 m-1 p-2 bg-base-200/30 flex flex-col rounded-box overflow-y-auto shrink-0 select-none">
        <div
          v-for="tab in settingsTabs"
          :key="tab.id"
          :class="[
            'px-3 py-2 rounded-box cursor-pointer transition-all duration-200 font-medium flex items-center',
            config.settings.tabIndex === tab.id
              ? 'bg-base-100 text-primary' 
              : 'hover:text-base-content hover:bg-base-100/30'
          ]"
          @click="config.settings.tabIndex = tab.id"
        >
          {{ $t(tab.label) }}
        </div>
      </div>

      <!-- Main Content -->
      <div class="p-2 mr-1 mb-2 flex-1 overflow-y-auto scrollbar-hide bg-base-300 cursor-default select-none">
          
        <!-- General Tab -->
        <div v-if="config.settings.tabIndex === SETTINGS_TAB.GENERAL" class="flex flex-col space-y-2">
          
          <!-- languange -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.general.section_language') }}</span>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.general.select_language') }}</div>
                <div v-if="config.settings.language !== 'en'" class="text-xs text-base-content/30">Select language</div>
              </div>
              <select class="select  select-bordered select-sm min-w-32" v-model="config.settings.language">
                <option v-for="(lang, index) in languages" :key="index" :value="lang.value">{{ lang.label }}</option>
              </select>
            </div>
          </div>

          <!-- appearance -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.general.section_appearance') }}</span>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.general.appearance') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.appearance">
                <option v-for="(item, index) in appearanceOptions" :key="index" :value="item.value">{{ item.label }}</option>
              </select>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.general.theme') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="currentTheme">
                <option v-for="(option, index) in themeOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.general.font_size') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.scale">
                <option v-for="(option, index) in scaleOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
          </div>

          <!-- display -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.general.section_interface') }}</span>
            </div>
            <div class="flex items-center justify-between p-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.general.show_tool_tip') }}</div>
              </div>
              <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="config.settings.showToolTip" />
            </div>
            <div class="flex items-center justify-between p-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.general.show_status_bar') }}</div>
              </div>
              <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="config.settings.showStatusBar" />
            </div>
          </div>

          <!-- updates -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.general.section_updates') }}</span>
            </div>
            <div class="flex items-center justify-between p-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.general.auto_check_updates') }}</div>
              </div>
              <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="config.settings.autoCheckUpdates" />
            </div>
          </div>

          <!-- ExifTool -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.general.section_exiftool') }}</span>
            </div>
            <div class="flex flex-col gap-2 p-1">
              <div class="flex items-center justify-between">
                <div class="flex flex-col gap-0.5 text-sm leading-5">
                  <div>{{ $t('settings.general.exiftool_path') }}</div>
                  <div class="text-xs text-base-content/50">{{ $t('settings.general.exiftool_desc') }}</div>
                </div>
                <div class="badge badge-sm" :class="exifToolStatus.available ? 'badge-success text-success-content' : 'badge-ghost text-base-content/40'">
                  {{ exifToolStatus.available ? `${$t('settings.general.exiftool_detected')} (v${exifToolStatus.version})` : $t('settings.general.exiftool_not_found') }}
                </div>
              </div>
              <div class="flex items-center gap-2 mt-1">
                <input
                  type="text"
                  class="input input-sm input-bordered flex-1 text-xs"
                  :placeholder="exifToolStatus.binaryPath || $t('settings.general.exiftool_placeholder')"
                  v-model="config.settings.exiftoolPath"
                  @change="refreshExifToolStatus"
                />
                <button type="button" class="btn btn-sm btn-ghost border border-base-content/10 cursor-pointer" @click="chooseExifToolBinary">
                  {{ $t('settings.general.exiftool_browse') }}
                </button>
                <button v-if="config.settings.exiftoolPath" type="button" class="btn btn-sm btn-ghost text-xs cursor-pointer" @click="resetExifToolPath">
                  {{ $t('settings.general.exiftool_reset') }}
                </button>
              </div>
            </div>
          </div>

        </div>

        <!-- Grid Tab -->
        <div v-else-if="config.settings.tabIndex === SETTINGS_TAB.GRID" class="flex flex-col space-y-2">

          <!-- grid view -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.grid.section_grid') }}</span>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.grid.style') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.grid.style">
                <option v-for="(option, index) in gridStyleOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.grid.scaling') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.grid.scaling" :disabled="config.settings.grid.style !== 0 && config.settings.grid.style !== 1">
                <option v-for="(option, index) in gridScalingOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.grid.thumbnail_corners') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.grid.thumbnailCorners">
                <option v-for="option in thumbnailCornerOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.grid.show_thumbnail_badges') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.grid.thumbnailBadge">
                <option v-for="option in thumbnailBadgeOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.grid.label_primary') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.grid.labelPrimary" :disabled="config.settings.grid.style !== 0">
                  <option v-for="(option, index) in gridLabelOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.grid.label_secondary') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.grid.labelSecondary" :disabled="config.settings.grid.style !== 0">
                  <option v-for="(option, index) in gridLabelOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
          </div>

          <!-- filmstrip -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.grid.filmstrip_view.title') }}</span>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.grid.filmstrip_view.preview_position') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.grid.previewPosition">
                <option v-for="(option, index) in filmStripViewPreviewPositionOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
          </div>

          <!-- map -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.grid.section_map') }}</span>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.grid.map_marker_size') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.mapMarkerSize">
                <option v-for="(option, index) in mapMarkerSizeOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
          </div>

        </div>

        <!-- Viewer Tab -->
        <div v-else-if="config.settings.tabIndex === SETTINGS_TAB.IMAGE_VIEW" class="flex flex-col space-y-2">

          <!-- navigation -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.image_view.section_navigation') }}</span>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.image_view.mouse_wheel') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.mouseWheelMode">
                <option v-for="(item, index) in wheelOptions" :key="index" :value="item.value">
                  {{ item.label }}
                </option>
              </select>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.image_view.navigator_view') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.navigatorViewMode">
                  <option v-for="(option, index) in navigatorViewModeOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.image_view.navigator_view__size') }}</div>
              </div>
                <select class="select select-bordered select-sm min-w-32" v-model="config.settings.navigatorViewSize">
                  <option v-for="(option, index) in navigatorViewSizeOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
          </div>

          <!-- view -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.image_view.section_view') }}</span>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.image_view.view_background') }}</div>
                <div class="text-xs text-base-content/30">
                  {{ $t('settings.image_view.view_background_hint') }}
                </div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.viewBackground">
                <option v-for="option in viewBackgroundOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.image_view.slide_show_transition') }}</div>
              </div>
                <select class="select select-bordered select-sm min-w-32" v-model="config.settings.slideShowTransition">
                  <option v-for="(option, index) in slideShowTransitionOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
          </div>

          <!-- video -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.image_view.section_video') }}</span>
            </div>
            <div class="flex items-center justify-between px-1 h-8 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.image_view.auto_play_video') }}</div>
              </div>
              <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="config.settings.autoPlayVideo" />
            </div>
            <div class="flex items-center justify-between px-1 h-8 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.image_view.loop_video') }}</div>
              </div>
              <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="config.settings.loopVideo" />
            </div>
          </div>

        </div>

        <!-- RAW Tab -->
        <div v-else-if="config.settings.tabIndex === SETTINGS_TAB.RAW" class="flex flex-col space-y-2">
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.raw.section_pairs') }}</span>
            </div>
            <div class="flex items-center justify-between gap-3 px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <label for="raw-group-pairs" class="min-w-0 flex flex-col gap-0.5 text-sm leading-5">
                <span>{{ $t('settings.browse.group_raw_jpeg_pairs') }}</span>
                <span class="text-xs text-base-content/30">{{ $t('settings.browse.group_raw_jpeg_pairs_hint') }}</span>
              </label>
              <input id="raw-group-pairs" type="checkbox" class="toggle toggle-primary toggle-sm shrink-0" v-model="config.settings.groupRawJpegPairs" />
            </div>
            <div v-if="config.settings.groupRawJpegPairs" class="flex items-center justify-between gap-3 px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <label for="raw-pair-source" class="min-w-0 flex flex-col gap-0.5 text-sm leading-5">
                <span>{{ $t('settings.raw.pair_source') }}</span>
                <span class="text-xs text-base-content/30">{{ $t('settings.raw.pair_source_hint') }}</span>
              </label>
              <select id="raw-pair-source" class="select select-bordered select-sm w-40 shrink-0 max-w-full" v-model="config.settings.rawPairDisplaySource">
                <option value="jpeg">{{ $t('settings.raw.pair_jpeg') }}</option>
                <option value="raw">RAW</option>
              </select>
            </div>
          </div>
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span id="raw-preview-heading" class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.raw.preview_mode') }}</span>
            </div>
            <div v-for="field in rawPreviewFields" :key="field.key" class="flex items-center justify-between gap-3 px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <label :for="field.key" class="min-w-0 flex flex-col gap-0.5 text-sm leading-5">
                <span>{{ $t('settings.raw.' + field.key) }}</span>
                <span v-if="field.hint" class="text-xs text-base-content/30">{{ $t('settings.raw.' + field.hint) }}</span>
              </label>
              <select :id="field.key" class="select select-bordered select-sm w-48 shrink-0 max-w-full" v-model="config.settings[field.key]">
                <option v-for="option in field.options" :key="option" :value="option">{{ $t('settings.raw.option_' + option) }}</option>
              </select>
            </div>
          </div>
        </div>

        <!-- Search Tab -->
        <div v-else-if="config.settings.tabIndex === SETTINGS_TAB.IMAGE_SEARCH" class="flex flex-col overflow-hidden space-y-2">

          <!-- image search -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.image_search.search_image') }}</span>
            </div>
            <div class="flex items-start justify-between gap-4 px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="min-w-0 flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.image_search.search_model') }}</div>
                <div class="text-xs text-base-content/30">
                  {{ imageSearchModelHint }}
                </div>
              </div>
              <select
                class="select select-bordered select-sm min-w-36 shrink-0"
                :value="config.settings.imageSearch.model"
                :disabled="isDownloadingMultilingualModel"
                @change="onImageSearchModelChange"
              >
                <option
                  v-for="option in imageSearchModelOptions"
                  :key="option.value"
                  :value="option.value"
                >
                  {{ option.label }}
                </option>
              </select>
            </div>
            <div v-if="isDownloadingMultilingualModel" class="px-1 pt-1 space-y-1">
              <div class="flex items-center justify-between text-xs text-base-content/30">
                <span>{{ $t('settings.image_search.downloading_multilingual_model') }}</span>
                <span>{{ multilingualModelDownloadSizeText }}</span>
              </div>
              <div class="flex items-center gap-2">
                <progress
                  class="progress progress-primary h-1.5 flex-1"
                  :value="multilingualModelDownloadProgress"
                  max="100"
                ></progress>
                <button
                  class="btn btn-ghost btn-xs h-6 min-h-0 w-6 p-0 text-base-content/30 hover:text-base-content"
                  :title="$t('msgbox.cancel')"
                  :aria-label="$t('msgbox.cancel')"
                  @click="cancelMultilingualModelDownload"
                >
                  <IconClose class="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.image_search.similarity') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.image_search.similarity_hint') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.imageSearch.thresholdIndex">
                <option v-for="(option, index) in similarityOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
          </div>

          <!-- similar photos -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.similar_photos.title') }}</span>
            </div>
            <div class="flex items-center justify-between gap-4 px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="min-w-0 flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.similar_photos.grouping_strictness') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.similar_photos.grouping_strictness_hint') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32 shrink-0" v-model="similarPhotoGroupingThresholdIndex">
                <option v-for="(option, index) in similarPhotoGroupingOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
          </div>

          <!-- face recognition -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.face_recognition.title') }}</span>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div class="flex items-center">
                  <div>{{ $t('settings.face_recognition.enable') }}</div>
                  <span class="ml-2 px-1.5 h-5 inline-flex items-center rounded-box text-[10px] font-semibold tracking-[0.08em] text-warning border border-warning/30 bg-warning/10 cursor-default">
                    BETA
                  </span>
                </div>
                <div class="text-xs text-base-content/30">{{ $t('settings.face_recognition.beta_hint') }}</div>
              </div>
              <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="config.settings.face.enabled" />
            </div>
            <div v-if="config.settings.face.enabled" class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div class="flex items-center">
                  <div>{{ $t('settings.face_recognition.similarity') }}</div>
                  <span class="ml-2 px-1.5 h-5 inline-flex items-center rounded-box text-[10px] font-semibold tracking-[0.08em] text-warning border border-warning/30 bg-warning/10 cursor-default">
                    BETA
                  </span>
                </div>
                <div class="text-xs text-base-content/30">{{ $t('settings.face_recognition.cluster_threshold_hint') }}</div>
              </div>
                <select class="select select-bordered select-sm min-w-32" v-model="config.settings.face.clusterThresholdIndex" :disabled="!config.settings.face.enabled">
                  <option v-for="(option, index) in faceClusterOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
          </div>
        </div>

        <!-- Browse Tab -->
        <div v-else-if="config.settings.tabIndex === SETTINGS_TAB.BROWSE" class="flex flex-col space-y-2">

          <!-- album -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.browse.section_album') }}</span>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.browse.show_subfolder_files') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.browse.show_subfolder_files_hint') }}</div>
              </div>
              <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="config.settings.showSubfolderFiles" />
            </div>
          </div>

          <!-- sorting -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.browse.section_sorting') }}</span>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.browse.folder_sort') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.browse.folder_sort_hint') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-40" v-model="config.settings.folderSort">
                <option v-for="option in folderSortOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.browse.calendar_sort') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.browse.calendar_sort_hint') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-40" v-model="config.settings.calendarSort">
                <option v-for="option in calendarSortOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.browse.category_sort') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.browse.category_sort_hint') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-40" v-model="config.settings.categorySort">
                <option v-for="option in categorySortOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
          </div>

        </div>

        <!-- Advanced Tab -->
        <div v-else-if="config.settings.tabIndex === SETTINGS_TAB.ADVANCED" class="flex flex-col space-y-2">

          <!-- thumbnail cache -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.advanced.section_thumbnail_cache') }}</span>
            </div>
            <div class="flex items-center justify-between gap-4 px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="min-w-0 flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.advanced.thumbnail_quality') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.advanced.thumbnail_quality_hint') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-40 shrink-0" :value="config.settings.thumbnailSize" @change="onThumbnailSizeChange">
                <option v-for="option in thumbnailQualityOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
          </div>

          <!-- map -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.advanced.section_map') }}</span>
            </div>
            <div class="flex items-center justify-between gap-4 px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="min-w-0 flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.advanced.map_provider') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.advanced.map_provider_hint') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-40 shrink-0" v-model="config.settings.mapProvider">
                <option v-for="option in mapProviderOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <div v-if="config.settings.mapProvider === 'tianditu'" class="flex items-center justify-between gap-4 px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="min-w-0 flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.advanced.tianditu_token') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.advanced.tianditu_token_hint') }}</div>
              </div>
              <div class="flex shrink-0 items-center gap-2">
                <div class="relative">
                  <input
                    v-model="tiandituTokenInput"
                    class="input input-bordered input-sm min-w-40 w-48"
                    type="text"
                    spellcheck="false"
                    autocomplete="off"
                    :placeholder="$t('settings.advanced.tianditu_token_placeholder')"
                    @input="onTiandituTokenInput"
                    @keydown.enter.prevent="commitTiandituToken"
                    @blur="commitTiandituToken"
                  >
                </div>
                <span
                  v-if="tiandituTokenStatus !== 'idle'"
                  class="min-w-20 text-xs whitespace-nowrap"
                  :class="tiandituTokenStatusClass"
                >
                  {{ tiandituTokenStatusLabel }}
                </span>
              </div>
            </div>
          </div>

          <!-- data -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.database.section_storage') }}</span>
            </div>

            <div class="flex items-center justify-between gap-4 px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="min-w-0 flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.database.current_location') }}</div>
                <div class="text-xs text-base-content/30 truncate" :title="dbStorageDir || ''">
                  {{ hasCustomDbStorage ? (dbStorageDir || '-') : $t('settings.database.system_default') }}
                </div>
              </div>
              <div class="shrink-0 flex items-center gap-2">
                <button
                  class="btn btn-sm btn-ghost rounded-box bg-base-100 border border-base-content/30 text-base-content/70 hover:text-base-content"
                  :disabled="isChangingDbStorage"
                  @click="selectDbStorageDir"
                >
                  {{ isChangingDbStorage ? $t('tooltip.loading') : $t('settings.database.change_location') }}
                </button>
                <TButton
                  v-if="hasCustomDbStorage"
                  :icon="IconRestore"
                  :buttonSize="'small'"
                  :disabled="isChangingDbStorage"
                  :tooltip="$t('settings.database.restore_default_location')"
                  @click="restoreDefaultDbStorageDir"
                />
              </div>
            </div>

            <div class="flex items-center justify-between gap-4 px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.database.backup_title') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.database.backup_hint') }}</div>
              </div>
              <button
                class="btn btn-sm btn-ghost rounded-box bg-base-100 border border-base-content/30 text-base-content/70 hover:text-base-content"
                @click="showBackupDialog = true"
              >
                {{ $t('settings.database.backup') }}
              </button>
            </div>

            <div class="flex items-center justify-between gap-4 px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.database.restore_title') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.database.restore_hint') }}</div>
              </div>
              <button
                class="btn btn-sm btn-ghost rounded-box bg-base-100 border border-base-content/30 text-base-content/70 hover:text-base-content"
                @click="showRestoreDialog = true"
              >
                {{ $t('settings.database.restore') }}
              </button>
            </div>
          </div>

          <!-- diagnostics -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.advanced.section_diagnostics') }}</span>
            </div>
            <div class="flex items-center justify-between p-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.advanced.debug_mode') }}</div>
              </div>
              <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="config.settings.debugMode" />
            </div>
          </div>
        </div>

        <!-- Shortcuts Tab -->
        <div v-else-if="config.settings.tabIndex === SETTINGS_TAB.SHORTCUTS" class="flex flex-col space-y-2">
          <div
            v-for="section in shortcutSections"
            :key="section.key"
            class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm"
          >
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ section.title }}</span>
            </div>
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-x-4 gap-y-1">
              <div
                v-for="item in section.items"
                :key="item.actionId"
                class="min-h-9 flex items-center justify-between gap-4 px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200"
              >
                <div class="min-w-0 text-sm leading-5 truncate">{{ item.label }}</div>
                <div class="shrink-0 flex items-center gap-1">
                  <span
                    v-for="(key, keyIndex) in item.keys"
                    :key="`${item.actionId}-${keyIndex}-${key}`"
                    class="min-w-7 h-7 px-2 inline-flex items-center justify-center rounded-box border border-base-content/10 bg-base-100/40 text-xs font-semibold text-base-content/30 shadow-sm"
                  >
                    {{ key }}
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
        <!-- About Tab -->
        <div v-else-if="config.settings.tabIndex === SETTINGS_TAB.ABOUT" class="py-2">
            <SettingsAbout />
        </div>

      </div>
    </div>

    <MessageBox
      v-if="showChangeDbStorageDialog"
      :title="$t('settings.database.prechange_title')"
      :message="$t('settings.database.prechange_message')"
      :OkText="$t('settings.database.change_location_confirm')"
      :cancelText="$t('msgbox.cancel')"
      @ok="chooseDbStorageDir"
      @cancel="showChangeDbStorageDialog = false"
    />

    <MessageBox
      v-if="showResetDbStorageDialog"
      :title="$t('settings.database.restore_default_confirm_title')"
      :message="$t('settings.database.restore_default_confirm_message')"
      :OkText="$t('settings.database.restore_default_confirm_ok')"
      :cancelText="$t('msgbox.cancel')"
      @ok="confirmResetDbStorageDir"
      @cancel="showResetDbStorageDialog = false"
    />

    <BackupDialog
      v-if="showBackupDialog"
      @done="showBackupDialog = false"
      @cancel="showBackupDialog = false"
    />

    <RestoreDialog
      v-if="showRestoreDialog"
      @done="onRestoreDone"
      @cancel="showRestoreDialog = false"
    />
  </div>
</template>

<script setup lang="ts">

import { ref, watch, computed, onMounted, onUnmounted } from 'vue';
import { LogicalSize } from '@tauri-apps/api/dpi';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { emit } from '@tauri-apps/api/event';
import { ask, open as openDialog } from '@tauri-apps/plugin-dialog';
import { useI18n } from 'vue-i18n';
import { config, libConfig } from '@/common/config';
import { normalizeThumbnailSize } from '@/common/thumbnailProfiles';
import { THUMBNAIL_BADGE, MAP_MARKER_SIZES, SETTINGS_TAB } from '@/common/constants';
import {
  getDbStorageDir,
  changeDbStorageDir,
  resetDbStorageDir,
  isFaceIndexing,
  isUsingCustomDbStorage,
  getImageSearchModelStatus,
  setImageSearchModel,
  downloadMultilingualImageSearchModel,
  cancelMultilingualImageSearchModelDownload,
  listenImageSearchModelDownloadProgress,
  checkExifToolStatus,
} from '@/common/api';
import { formatFileSize, isLinux, isMac, setTheme, SCALE_VALUES } from '@/common/utils';
import { getShortcutLabels, ShortcutActionId, ShortcutPlatform } from '@/common/shortcuts';
import { useToast } from '@/common/toast';
import { IconClose, IconRestore } from '@/common/icons';

import TitleBar from '@/components/TitleBar.vue';
import SettingsAbout from '@/components/SettingsAbout.vue';
import MessageBox from '@/components/MessageBox.vue';
import BackupDialog from '@/components/BackupDialog.vue';
import RestoreDialog from '@/components/RestoreDialog.vue';
import TButton from '@/components/TButton.vue';

/// i18n
const { locale, messages, t } = useI18n();
const localeMsg = computed(() => messages.value[config.settings.language] as any);
const toast = useToast();
const shortcutPlatform: ShortcutPlatform = isMac ? 'mac' : (isLinux ? 'linux' : 'windows');
const settingsTabs = [
  { id: SETTINGS_TAB.GENERAL, label: 'settings.general.title' },
  { id: SETTINGS_TAB.BROWSE, label: 'settings.browse.title' },
  { id: SETTINGS_TAB.GRID, label: 'settings.grid.title' },
  { id: SETTINGS_TAB.IMAGE_VIEW, label: 'settings.image_view.title' },
  { id: SETTINGS_TAB.RAW, label: 'settings.raw.title' },
  { id: SETTINGS_TAB.IMAGE_SEARCH, label: 'settings.image_search.title' },
  { id: SETTINGS_TAB.ADVANCED, label: 'settings.advanced.title' },
  { id: SETTINGS_TAB.SHORTCUTS, label: 'settings.shortcuts.title' },
  { id: SETTINGS_TAB.ABOUT, label: 'settings.about.title' },
];
const rawPreviewFields = [
  { key: 'rawPreviewSource', hint: '', options: ['embedded', 'rendered'] },
  { key: 'rawRenderBrightness', hint: 'brightness_hint', options: ['original', 'brightened'] },
];

const appWindow = getCurrentWebviewWindow()
let unlistenCloseRequested: (() => void) | null = null;
const SETTINGS_BASE_WIDTH = 600;
const SETTINGS_BASE_HEIGHT = 620;
const dbStorageDir = ref('');
const isChangingDbStorage = ref(false);
const hasCustomDbStorage = ref(false);
const showChangeDbStorageDialog = ref(false);
const showResetDbStorageDialog = ref(false);
const showBackupDialog = ref(false);
const showRestoreDialog = ref(false);
const isDownloadingMultilingualModel = ref(false);
const isCancelingMultilingualModelDownload = ref(false);
const multilingualModelDownloadProgress = ref(0);
const multilingualModelDownloadedBytes = ref(0);
const multilingualModelTotalBytes = ref(0);
const isMultilingualModelAvailable = ref(false);
const tiandituTokenInput = ref(String(config.settings.tiandituToken || ''));
const tiandituTokenStatus = ref<'idle' | 'saved' | 'empty'>('idle');
let unlistenImageSearchModelDownloadProgress: (() => void) | null = null;

const exifToolStatus = ref<{ available: boolean; version: string | null; binaryPath: string | null }>({
  available: false,
  version: null,
  binaryPath: null,
});

async function refreshExifToolStatus() {
  exifToolStatus.value = await checkExifToolStatus(config.settings.exiftoolPath || null);
}

async function chooseExifToolBinary() {
  const result = await openDialog({
    title: localeMsg.value.settings?.general?.exiftool_select || 'Select ExifTool Executable',
    multiple: false,
    directory: false,
  });
  if (result && typeof result === 'string') {
    config.settings.exiftoolPath = result;
    await refreshExifToolStatus();
  }
}

function resetExifToolPath() {
  config.settings.exiftoolPath = '';
  void refreshExifToolStatus();
}

watch(() => config.settings.exiftoolPath, refreshExifToolStatus);

const onRestoreDone = () => {
  showRestoreDialog.value = false;
  emit('libraries-changed');
};

const languages = [
  { label: 'English', value: 'en' },
  { label: 'Deutsch', value: 'de' },
  { label: 'Español', value: 'es' },
  { label: 'Français', value: 'fr' },
  { label: 'Italiano', value: 'it' },
  { label: 'Magyar', value: 'hu' },
  { label: 'Nederlands', value: 'nl' },
  { label: 'Polski', value: 'pl' },
  { label: 'Português', value: 'pt' },
  { label: 'Русский', value: 'ru' },
  { label: 'Українська', value: 'uk' },
  { label: '中文简体', value: 'zh' },
  { label: '中文繁體', value: 'zh-TW' },
  { label: '日本語', value: 'ja' },
  { label: '한국어', value: 'ko' },
];

const appearanceOptions = computed(() => {
  const options = localeMsg.value.settings.general.appearance_options;
  return Array.from({ length: options.length }, (_, i) => ({
    label: options[i],
    value: i,
  }));
});

// Define the theme options
const themeOptions = computed(() => {
  const options = config.settings.appearance === 0 
    ? localeMsg.value.settings.general.theme_options_light 
    : localeMsg.value.settings.general.theme_options_dark;

  const result = [];
  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }
  return result;
});

const currentTheme = computed({
  get() {
    return config.settings.appearance === 0 ? config.settings.lightTheme : config.settings.darkTheme;
  },
  set(value) {
    config.settings.appearance === 0 ? config.settings.lightTheme = value : config.settings.darkTheme = value;
  }
});

const scaleOptions = computed(() => {
  const options = localeMsg.value.settings.general.font_size_options;
  const values = [0.8, 0.9, 1, 1.1, 1.2];
  return values.map((value, index) => ({
    value,
    label: options[index] ?? String(value),
  }));
});

const folderSortOptions = computed(() => {
  const options = localeMsg.value.settings.browse.folder_sort_options || [];
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

const calendarSortOptions = computed(() => {
  const options = localeMsg.value.settings.browse.calendar_sort_options || [];
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

const categorySortOptions = computed(() => {
  const options = localeMsg.value.settings.browse.category_sort_options || [];
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

// Define the wheel options using computed to react to language changes
const wheelOptions = computed(() => {
  const options = localeMsg.value.settings.image_view.mouse_wheel_options; // returns an array
  return [
    { label: options[0], value: 0 },  // 0: previous / next
    { label: options[1], value: 1 },  // 1: zoom in / out
  ];
});

const thumbnailQualityOptions = computed(() => {
  const labels = localeMsg.value.settings.advanced.thumbnail_quality_options || [
    'Low(256px)',
    'Standard(512px)',
    'High(1024px)',
  ];
  return [
    { value: 256, label: labels[0] },
    { value: 512, label: labels[1] },
    { value: 1024, label: labels[2] },
  ];
});

const mapProviderOptions = computed(() => {
  const labels = localeMsg.value.settings.advanced.map_provider_options || [
    'Global (default)',
    'China (Tianditu)',
  ];
  return [
    { value: 'global', label: labels[0] },
    { value: 'tianditu', label: labels[1] },
  ];
});

const tiandituTokenStatusLabel = computed(() => {
  const labels: Record<string, string> = {
    saved: t('settings.advanced.tianditu_token_status_saved'),
    empty: t('settings.advanced.tianditu_token_status_empty'),
  };
  return labels[tiandituTokenStatus.value] || '';
});

const tiandituTokenStatusClass = computed(() => {
  if (tiandituTokenStatus.value === 'saved') return 'text-success';
  return 'text-base-content/40';
});

function onTiandituTokenInput() {
  tiandituTokenStatus.value = 'idle';
}

function normalizeTiandituToken(value: string) {
  const token = value.trim();
  const queryToken = token.match(/[?&]tk=([^&#\s]+)/i)?.[1];
  const normalizedToken = queryToken || token.replace(/^tk=/i, '');
  try {
    return decodeURIComponent(normalizedToken);
  } catch {
    return normalizedToken;
  }
}

function commitTiandituToken() {
  const token = normalizeTiandituToken(tiandituTokenInput.value);
  tiandituTokenInput.value = token;
  config.settings.tiandituToken = token;
  tiandituTokenStatus.value = token ? 'saved' : 'empty';
}

function onThumbnailSizeChange(event: Event) {
  const next = normalizeThumbnailSize((event.target as HTMLSelectElement).value);
  if (normalizeThumbnailSize(config.settings.thumbnailSize) === next) return;
  config.settings.thumbnailSize = next;
}

// Define the grid scaling options
const gridScalingOptions = computed(() => {
  const options = localeMsg.value.settings.grid.scaling_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

const thumbnailCornerOptions = computed(() => {
  const options = localeMsg.value.settings.grid.thumbnail_corner_options;
  return options.map((label: string, index: number) => ({ label, value: index }));
});

// Define the grid style options
const gridStyleOptions = computed(() => {
  const options = localeMsg.value.settings.grid.style_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

const mapMarkerSizeOptions = computed(() => {
  const labels = localeMsg.value.settings.grid.map_marker_size_options;
  return MAP_MARKER_SIZES.map((size, index) => ({ label: labels[index], value: size }));
});

// Define the grid label options
const gridLabelOptions = computed(() => {
  const options = localeMsg.value.settings.grid.label_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

const thumbnailBadgeOptions = computed(() => {
  const options = localeMsg.value.settings.grid.thumbnail_badge_options;
  const values = [
    THUMBNAIL_BADGE.EMPTY,
    THUMBNAIL_BADGE.FILE_FORMAT,
    THUMBNAIL_BADGE.ISO,
    THUMBNAIL_BADGE.SHUTTER_SPEED,
    THUMBNAIL_BADGE.APERTURE,
    THUMBNAIL_BADGE.FOCAL_LENGTH,
    THUMBNAIL_BADGE.EXPOSURE,
  ];
  return options.map((label: string, index: number) => ({
    label,
    value: values[index],
  }));
});

// Define the navigator view mode options
const navigatorViewModeOptions = computed(() => {
  const options = localeMsg.value.settings.image_view.navigator_view_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

// Define the navigator view size options
const navigatorViewSizeOptions = computed(() => {
  const options = localeMsg.value.settings.image_view.navigator_view_size_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: parseInt(options[i].split('(')[1].split('px')[0]) });
  }

  return result;
});

const viewBackgroundOptions = computed(() => {
  const options = localeMsg.value.settings.image_view.view_background_options;
  return options.map((label: string, value: number) => ({ label, value }));
});
const slideShowTransitionOptions = computed(() => {
  const options = localeMsg.value.settings.image_view.slide_show_transition_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

const filmStripViewPreviewPositionOptions = computed(() => {
  const options = localeMsg.value.settings.grid.filmstrip_view.preview_position_options;
  return options.map((label, i) => ({ label, value: i }));
});

// Define the similarity options
const similarityOptions = computed(() => {
  const options = localeMsg.value.settings.image_search.similarity_options;
  // Use getter to retrieve thresholds
  const values = config.imageSearchThresholds ?? [0.32, 0.29, 0.26, 0.255];
  // Map index dummy as the value since v-model is thresholdIndex
  return values.map((val, i) => ({ label: options[i], value: i }));
});

const similarPhotoGroupingThresholdIndex = computed({
  get: () => config.settings.similarPhotos?.groupingThresholdIndex ?? 1,
  set: (value) => {
    if (!config.settings.similarPhotos) config.settings.similarPhotos = { groupingThresholdIndex: 1 };
    config.settings.similarPhotos.groupingThresholdIndex = Number(value);
  },
});

const similarPhotoGroupingOptions = computed(() => {
  const options = localeMsg.value.settings.similar_photos.grouping_strictness_options;
  const values = config.similarPhotoGroupingThresholds ?? [0.97, 0.93, 0.9, 0.85];
  return values.map((value, index) => ({ label: options[index], value: index }));
});

const imageSearchModelOptions = computed(() => {
  const options = localeMsg.value.settings.image_search.search_model_options || ['Default', 'Multilingual model'];
  return options.map((label: string, i: number) => ({ label, value: i }));
});

const imageSearchModelHint = computed(() => {
  return Number(config.settings.imageSearch.model || 0) === 1
    ? localeMsg.value.settings.image_search.multilingual_model_hint
    : localeMsg.value.settings.image_search.default_model_hint;
});

const multilingualModelDownloadSizeText = computed(() => {
  const downloaded = multilingualModelDownloadedBytes.value;
  const total = multilingualModelTotalBytes.value;
  if (total > 0) {
    return `${formatFileSize(downloaded)} / ${formatFileSize(total)}`;
  }
  return formatFileSize(downloaded);
});

const syncImageSearchModelStatus = async () => {
  const status = await getImageSearchModelStatus();
  if (!status) return;

  isMultilingualModelAvailable.value = Boolean(status.multilingualAvailable);
  if (Number(config.settings.imageSearch.model || 0) === 1 && !isMultilingualModelAvailable.value) {
    return;
  }

  try {
    await setImageSearchModel(config.settings.imageSearch.model || 0);
  } catch (error) {
    console.error('Failed to activate image search model:', error);
  }
};

// Define the face cluster threshold options
const faceClusterOptions = computed(() => {
  const options = localeMsg.value.settings.face_recognition?.cluster_threshold_options || 
    ['Very High', 'High', 'Medium', 'Low'];
  // Map index as value since v-model is clusterThresholdIndex
  return options.map((label: string, i: number) => ({ label, value: i }));
});

type ShortcutDisplayItem = {
  actionId: ShortcutActionId;
  labelKey: string;
  keys?: string[];
  backgroundValue?: number;
};

const shortcutDisplaySections: Array<{ key: string; items: ShortcutDisplayItem[] }> = [
  {
    key: 'global',
    items: [
      { actionId: 'app.sidebar.toggle', labelKey: 'toggle_sidebar' },
      { actionId: 'app.preferences', labelKey: 'open_settings' },
      { actionId: 'app.scale.increase', labelKey: 'font_increase' },
      { actionId: 'app.scale.decrease', labelKey: 'font_decrease' },
      { actionId: 'app.scale.reset', labelKey: 'font_reset' },
      { actionId: 'app.search', labelKey: 'search' },
    ],
  },
  {
    key: 'image_browsing',
    items: [
      { actionId: 'view.zoomIn', labelKey: 'thumbnail_increase' },
      { actionId: 'view.zoomOut', labelKey: 'thumbnail_decrease' },
      { actionId: 'view.previous', labelKey: 'previous_image' },
      { actionId: 'view.next', labelKey: 'next_image' },
      { actionId: 'view.first', labelKey: 'first_image' },
      { actionId: 'view.last', labelKey: 'last_image' },
      { actionId: 'view.pageUp', labelKey: 'page_up' },
      { actionId: 'view.pageDown', labelKey: 'page_down' },
      { actionId: 'view.quickPreview', labelKey: 'quick_preview' },
      { actionId: 'view.close', labelKey: 'close_viewer' },
      { actionId: 'file.openNewWindow', labelKey: 'open_new_window' },
      { actionId: 'file.openExternalApp', labelKey: 'open_external_app' },
      { actionId: 'file.editImage', labelKey: 'edit_image' },
      { actionId: 'file.searchSimilar', labelKey: 'search_similar' },
    ],
  },
  {
    key: 'viewing',
    items: [
      { actionId: 'view.zoomIn', labelKey: 'zoom_in' },
      { actionId: 'view.zoomOut', labelKey: 'zoom_out' },
      { actionId: 'view.zoomFit', labelKey: 'zoom_fit' },
      { actionId: 'view.cycleBackground', labelKey: 'cycle_background' },
      { actionId: 'view.backgroundTheme', labelKey: 'cycle_background', backgroundValue: 0 },
      { actionId: 'view.backgroundBlack', labelKey: 'cycle_background', backgroundValue: 1 },
      { actionId: 'view.backgroundDarkGray', labelKey: 'cycle_background', backgroundValue: 2 },
      { actionId: 'view.backgroundMediumGray', labelKey: 'cycle_background', backgroundValue: 3 },
      { actionId: 'view.backgroundLightGray', labelKey: 'cycle_background', backgroundValue: 4 },
      { actionId: 'view.backgroundWhite', labelKey: 'cycle_background', backgroundValue: 5 },
      { actionId: 'slideshow.toggle', labelKey: 'toggle_slideshow' },
    ],
  },
  {
    key: 'file_actions',
    items: [

      { actionId: 'file.rename', labelKey: 'rename_file' },
      { actionId: 'file.moveTo', labelKey: 'move_within_library' },
      { actionId: 'file.moveToFolder', labelKey: 'move_to_folder' },
      { actionId: 'file.copy', labelKey: 'copy_file' },
      { actionId: 'file.paste', labelKey: 'paste_file' },
      { actionId: 'file.reveal', labelKey: 'reveal_in_file_manager' },
      { actionId: 'file.trash', labelKey: 'move_to_trash' },
    ],
  },
  {
    key: 'selection',
    items: [
      { actionId: 'file.selectAll', labelKey: 'select_all' },
      { actionId: 'file.selectNone', labelKey: 'select_none' },
      { actionId: 'file.invertSelection', labelKey: 'invert_selection' },
    ],
  },
  {
    key: 'metadata',
    items: [
      { actionId: 'meta.favorite', labelKey: 'toggle_favorite' },
      { actionId: 'meta.rating.clear', labelKey: 'set_clear_rating', keys: ['0 ~ 5'] },
      { actionId: 'meta.culling.pick', labelKey: 'mark_pick' },
      { actionId: 'meta.culling.reject', labelKey: 'mark_rejected' },
      { actionId: 'meta.culling.unreviewed', labelKey: 'mark_unreviewed' },
      { actionId: 'meta.tag', labelKey: 'edit_tags' },
      { actionId: 'meta.collection', labelKey: 'edit_collections' },
      { actionId: 'meta.comment', labelKey: 'edit_comment' },
      { actionId: 'meta.rotate', labelKey: 'rotate' },
      { actionId: 'meta.rotateCounterclockwise', labelKey: 'rotate_counterclockwise' },
      { actionId: 'meta.info', labelKey: 'show_info' },
    ],
  },
];

const shortcutSections = computed(() => {
  const shortcutMessages = localeMsg.value.settings.shortcuts;
  return shortcutDisplaySections.map((section) => ({
    key: section.key,
    title: shortcutMessages.sections[section.key],
    items: section.items
      .map((item) => ({
        actionId: item.actionId,
        label: item.backgroundValue === undefined
          ? shortcutMessages.actions[getShortcutActionLabelKey(item)]
          : `${localeMsg.value.settings.image_view.view_background}: ${localeMsg.value.settings.image_view.view_background_options[item.backgroundValue]}`,
        keys: item.keys ?? getDisplayShortcutKeys(item.actionId),
      }))
      .filter((item) => item.keys.length > 0),
  }));
});

function getShortcutActionLabelKey(item: ShortcutDisplayItem): string {
  if (item.actionId === 'file.reveal' && shortcutPlatform === 'mac') {
    return 'reveal_in_finder';
  }
  return item.labelKey;
}

function getDisplayShortcutKeys(actionId: ShortcutActionId): string[] {
  const labels = getShortcutLabels(actionId, shortcutPlatform);
  const label = getPreferredShortcutLabel(actionId, labels);
  return splitShortcutLabel(label);
}

function getPreferredShortcutLabel(actionId: ShortcutActionId, labels: string[]): string {
  if (actionId === 'app.scale.increase') {
    return labels.find((label) => label.includes('+')) || labels[0] || '';
  }
  return labels[0] || '';
}

function splitShortcutLabel(label: string): string[] {
  if (!label) return [];
  if (shortcutPlatform === 'mac') {
    return splitMacShortcutLabel(label);
  }

  let normalized = label
    .replace(/←/g, 'Left')
    .replace(/→/g, 'Right')
    .replace(/↑/g, 'Up')
    .replace(/↓/g, 'Down');

  normalized = normalized
    .replace(/\+\+$/, '+Plus')
    .replace(/\+=$/, '+=')
    .replace(/\+-$/, '+Minus')
    .replace(/\+0$/, '+0')
    .replace(/\+,/g, '+Comma');

  return normalized
    .split('+')
    .filter(Boolean)
    .map((key) => {
      key = key.trim();
      if (key === 'Plus') return '+';
      if (key === 'Minus') return '-';
      if (key === 'Comma') return ',';
      if (key === 'Del') return 'Delete';
      return key;
    });
}

function splitMacShortcutLabel(label: string): string[] {
  const modifierKeys = new Set(['⌘', '⌥', '⇧', '⌃']);
  const keys: string[] = [];
  let remaining = label;

  while (remaining.length > 0 && modifierKeys.has(remaining[0])) {
    keys.push(remaining[0]);
    remaining = remaining.slice(1);
  }

  if (remaining.startsWith('Fn')) {
    keys.push('Fn');
    remaining = remaining.slice(2);
  }

  if (remaining.length > 0) {
    keys.push(remaining);
  }

  return keys;
}

const onImageSearchModelChange = async (event: Event) => {
  const select = event.target as HTMLSelectElement;
  const nextModel = Number(select.value || 0);
  const previousModel = Number(config.settings.imageSearch.model || 0);

  if (nextModel !== 1) {
    try {
      await setImageSearchModel(nextModel);
      config.settings.imageSearch.model = nextModel;
    } catch (error) {
      select.value = String(previousModel);
      toast.error(error?.message || String(error));
    }
    return;
  }

  if (isMultilingualModelAvailable.value) {
    try {
      await setImageSearchModel(nextModel);
      config.settings.imageSearch.model = nextModel;
    } catch (error) {
      select.value = String(previousModel);
      toast.error(error?.message || String(error));
    }
    return;
  }

  select.value = String(previousModel);
  const shouldDownload = await ask(
    localeMsg.value.settings.image_search.multilingual_model_download_message,
    {
      title: localeMsg.value.settings.image_search.multilingual_model_download_title,
      kind: 'info',
      okLabel: localeMsg.value.settings.image_search.download,
      cancelLabel: localeMsg.value.msgbox?.cancel || 'Cancel',
    },
  );

  if (!shouldDownload) {
    return;
  }

  await startMultilingualModelDownload(previousModel);
};

const startMultilingualModelDownload = async (previousModel: number) => {
  if (isDownloadingMultilingualModel.value) return;

  isDownloadingMultilingualModel.value = true;
  isCancelingMultilingualModelDownload.value = false;
  multilingualModelDownloadProgress.value = 0;
  multilingualModelDownloadedBytes.value = 0;
  multilingualModelTotalBytes.value = 0;

  try {
    await downloadMultilingualImageSearchModel();
    isDownloadingMultilingualModel.value = false;
    isMultilingualModelAvailable.value = true;
    await setImageSearchModel(1);
    config.settings.imageSearch.model = 1;
    multilingualModelDownloadProgress.value = 100;
    if (multilingualModelTotalBytes.value > 0) {
      multilingualModelDownloadedBytes.value = multilingualModelTotalBytes.value;
    }
  } catch (error) {
    if (isCancelingMultilingualModelDownload.value || String(error).includes('Download canceled')) {
      isCancelingMultilingualModelDownload.value = false;
      isDownloadingMultilingualModel.value = false;
      config.settings.imageSearch.model = previousModel;
      multilingualModelDownloadProgress.value = 0;
      multilingualModelDownloadedBytes.value = 0;
      multilingualModelTotalBytes.value = 0;
      return;
    }
    isDownloadingMultilingualModel.value = false;
    config.settings.imageSearch.model = previousModel;
    const errorMessage = typeof error === 'string' ? error : error?.message;
    toast.error(errorMessage || localeMsg.value.settings.image_search.multilingual_model_download_failed);
  }
};

const cancelMultilingualModelDownload = async () => {
  if (!isDownloadingMultilingualModel.value) return;

  isCancelingMultilingualModelDownload.value = true;
  isDownloadingMultilingualModel.value = false;
  multilingualModelDownloadProgress.value = 0;
  multilingualModelDownloadedBytes.value = 0;
  multilingualModelTotalBytes.value = 0;
  await cancelMultilingualImageSearchModelDownload();
};

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown);
  if (!settingsTabs.some(tab => tab.id === config.settings.tabIndex)) {
    config.settings.tabIndex = SETTINGS_TAB.GENERAL;
  }
  if (typeof config.settings.imageSearch.model !== 'number') {
    config.settings.imageSearch.model = 0;
  }
  unlistenImageSearchModelDownloadProgress = await listenImageSearchModelDownloadProgress((event: any) => {
    const progress = Number(event?.payload?.progress ?? 0);
    multilingualModelDownloadProgress.value = Math.max(0, Math.min(100, progress));
    multilingualModelDownloadedBytes.value = Math.max(0, Number(event?.payload?.downloadedBytes ?? 0));
    multilingualModelTotalBytes.value = Math.max(0, Number(event?.payload?.totalBytes ?? 0));
  });
  await syncImageSearchModelStatus();
  applyWindowScale(Number(config.settings.scale || 1));
  dbStorageDir.value = (await getDbStorageDir()) || '';
  hasCustomDbStorage.value = await isUsingCustomDbStorage();
  void refreshExifToolStatus();

  
  // Show window after mount
  await appWindow.show();

  // Destroy the window on close (rather than merely `close()`, which leaves the
  // label registered) so reopening always creates a fresh window. Re-showing a
  // closed transparent window can fail silently on Windows.
  unlistenCloseRequested = await appWindow.onCloseRequested(async (event) => {
    event.preventDefault();
    await appWindow.destroy();
  });
});

onUnmounted(() => {
  if (unlistenCloseRequested) {
    unlistenCloseRequested();
    unlistenCloseRequested = null;
  }
  if (isDownloadingMultilingualModel.value) {
    void cancelMultilingualImageSearchModelDownload();
  }
  if (unlistenImageSearchModelDownloadProgress) {
    unlistenImageSearchModelDownloadProgress();
    unlistenImageSearchModelDownloadProgress = null;
  }
  document.documentElement.style.fontSize = '';
  window.removeEventListener('keydown', handleKeyDown);
});

// general settings
watch(() => config.settings.tabIndex, (newValue) => {
  emit('settings-settingsTabIndex-changed', newValue);
});
watch(() => config.settings.appearance, (newValue) => {
  setTheme(newValue, newValue === 0 ? config.settings.lightTheme : config.settings.darkTheme);
  emit('settings-appearance-changed', newValue);
});
watch(() => config.settings.lightTheme, (newValue) => {
  setTheme(config.settings.appearance, newValue);
  emit('settings-lightTheme-changed', newValue);
});
watch(() => config.settings.darkTheme, (newValue) => {
  setTheme(config.settings.appearance, newValue);
  emit('settings-darkTheme-changed', newValue);
});
watch(() => config.settings.scale, (newValue) => {
  applyWindowScale(Number(newValue || 1));
  updateSettingsWindowSize(Number(newValue || 1));
  emit('settings-scale-changed', newValue);
});
watch(() => config.settings.language, (newValue) => {
  locale.value = newValue;
  emit('settings-language-changed', newValue);
}, { immediate: true });
watch(() => config.settings.showToolTip, (newValue) => {
  emit('settings-showToolTip-changed', newValue);
});
watch(() => config.settings.showStatusBar, (newValue) => {
  emit('settings-showStatusBar-changed', newValue);
});
watch(() => config.settings.autoCheckUpdates, (newValue) => {
  emit('settings-autoCheckUpdates-changed', newValue);
});
// watch(() => config.settings.showComment, (newValue) => {
//   emit('settings-showComment-changed', newValue);
// });
watch(() => config.settings.debugMode, (newValue) => {
  emit('settings-debugMode-changed', newValue);
});
watch(() => config.settings.folderSort, (newValue) => {
  emit('settings-folderSort-changed', newValue);
});
watch(() => config.settings.calendarSort, (newValue) => {
  emit('settings-calendarSort-changed', newValue);
});
watch(() => config.settings.categorySort, (newValue) => {
  emit('settings-categorySort-changed', newValue);
});
watch(() => config.settings.showSubfolderFiles, (newValue) => {
  emit('settings-showSubfolderFiles-changed', newValue);
});
watch(() => config.settings.groupRawJpegPairs, (newValue) => {
  emit('settings-groupRawJpegPairs-changed', newValue);
});

watch(() => config.settings.rawPairDisplaySource, (newValue) => {
  emit('settings-rawPairDisplaySource-changed', newValue);
});
watch(() => config.settings.rawPreviewSource, (newValue) => {
  emit('settings-rawPreviewSource-changed', newValue);
});
watch(() => config.settings.rawRenderBrightness, (newValue) => {
  emit('settings-rawRenderBrightness-changed', newValue);
});

// grid view settings
watch(() => config.settings.thumbnailSize, (newValue) => {
  emit('settings-thumbnailSize-changed', newValue);
});

watch(() => config.settings.mapProvider, (newValue) => {
  emit('settings-mapProvider-changed', newValue);
});
watch(() => config.settings.tiandituToken, (newValue) => {
  emit('settings-tiandituToken-changed', newValue);
});
watch(() => config.settings.grid.style, (newValue) => {
  emit('settings-gridStyle-changed', newValue);
});
watch(() => config.settings.grid.scaling, (newValue) => {
  emit('settings-gridScaling-changed', newValue);
});
watch(() => config.settings.grid.thumbnailCorners, (newValue) => {
  emit('settings-gridThumbnailCorners-changed', newValue);
});
watch(() => config.settings.grid.labelPrimary, (newValue) => {
  emit('settings-gridLabelPrimary-changed', newValue);
});
watch(() => config.settings.grid.labelSecondary, (newValue) => {
  emit('settings-gridLabelSecondary-changed', newValue);
});
watch(() => config.settings.grid.thumbnailBadge, (newValue) => {
  emit('settings-gridThumbnailBadge-changed', newValue);
});
watch(() => config.settings.grid.previewPosition, (newValue) => {
  emit('settings-filmStripViewPreviewPosition-changed', newValue);
});
watch(() => config.settings.mapMarkerSize, (newValue) => {
  emit('settings-mapMarkerSize-changed', newValue);
});
// image viewer settings
watch(() => config.settings.mouseWheelMode, (newValue) => {
  emit('settings-mouseWheelMode-changed', newValue);
});
watch(() => config.settings.navigatorViewMode, (newValue) => {
  emit('settings-navigatorViewMode-changed', newValue);
});
watch(() => config.settings.navigatorViewSize, (newValue) => {
  emit('settings-navigatorViewSize-changed', newValue);
});
watch(() => config.settings.viewBackground, (newValue) => {
  emit('settings-viewBackground-changed', newValue);
});
watch(() => config.settings.slideShowTransition, (newValue) => {
  emit('settings-slideShowTransition-changed', newValue);
});
watch(() => config.settings.autoPlayVideo, (newValue) => {
  emit('settings-autoPlayVideo-changed', newValue);
});
watch(() => config.settings.loopVideo, (newValue) => {
  emit('settings-loopVideo-changed', newValue);
});

// image search settings
watch(() => config.settings.imageSearch.model, (newValue) => {
  emit('settings-imageSearchModel-changed', newValue);
});
watch(() => config.settings.imageSearch.thresholdIndex, (newValue) => {
  emit('settings-imageSearchThresholdIndex-changed', newValue);
});
watch(similarPhotoGroupingThresholdIndex, (newValue) => {
  emit('settings-similarPhotoGroupingThresholdIndex-changed', newValue);
});

// face settings
watch(() => config.settings.face.enabled, (newValue) => {
  emit('settings-faceEnabled-changed', newValue);
});
watch(() => config.settings.face.clusterThresholdIndex, (newValue) => {
  emit('settings-faceClusterThresholdIndex-changed', newValue);
});

// Handle keyboard shortcuts
function handleKeyDown(event: KeyboardEvent) {
  const navigationKeys = ['Tab', 'Escape'];

  // Disable default behavior for certain keys
  if (navigationKeys.includes(event.key)) {
    event.preventDefault();
  }

  switch (event.key) {
    case 'Tab': {
      const current = settingsTabs.findIndex(tab => tab.id === config.settings.tabIndex);
      config.settings.tabIndex = settingsTabs[(current + 1) % settingsTabs.length].id;
      break;
    }
    case 'Escape':
      // Close the topmost dialog first
      if (showBackupDialog.value) { showBackupDialog.value = false; return; }
      if (showRestoreDialog.value) { showRestoreDialog.value = false; return; }
      if (showChangeDbStorageDialog.value) { showChangeDbStorageDialog.value = false; return; }
      if (showResetDbStorageDialog.value) { showResetDbStorageDialog.value = false; return; }
      appWindow.close(); // Close the window
      break;
  }
}

async function selectDbStorageDir() {
  if (Number(libConfig.index.status || 0) === 1) {
    toast.error(localeMsg.value.settings?.database?.busy_library_indexing || 'Cannot change the data location while library indexing is running.');
    return;
  }

  const faceIndexState = await isFaceIndexing();
  if (Array.isArray(faceIndexState) && faceIndexState[0] === true) {
    toast.error(localeMsg.value.settings?.database?.busy_face_indexing || 'Cannot change the data location while face indexing is running.');
    return;
  }

  showChangeDbStorageDialog.value = true;
}

async function chooseDbStorageDir() {
  showChangeDbStorageDialog.value = false;

  const result = await openDialog({
    title: localeMsg.value.settings?.database?.change_location || 'Move data to another folder',
    multiple: false,
    directory: true,
  });

  if (!result || Array.isArray(result) || isChangingDbStorage.value) return;

  try {
    isChangingDbStorage.value = true;
    const newPath = await changeDbStorageDir(result);
    dbStorageDir.value = String(newPath.path || result);
    hasCustomDbStorage.value = true;
    if (newPath.cleanupWarnings?.length) {
      toast.warning(`${localeMsg.value.settings?.database?.cleanup_warning || 'Library data was moved, but some old files could not be removed:'}\n${newPath.cleanupWarnings.join('\n')}`, { duration: 10000 });
    } else {
      toast.success(localeMsg.value.settings?.database?.change_success || 'Library data has been moved successfully');
    }
  } catch (error: any) {
    toast.error(error?.message || String(error));
  } finally {
    isChangingDbStorage.value = false;
  }
}

async function restoreDefaultDbStorageDir() {
  if (Number(libConfig.index.status || 0) === 1) {
    toast.error(localeMsg.value.settings?.database?.busy_library_indexing || 'Cannot change the data location while library indexing is running.');
    return;
  }

  const faceIndexState = await isFaceIndexing();
  if (Array.isArray(faceIndexState) && faceIndexState[0] === true) {
    toast.error(localeMsg.value.settings?.database?.busy_face_indexing || 'Cannot change the data location while face indexing is running.');
    return;
  }

  showResetDbStorageDialog.value = true;
}

async function confirmResetDbStorageDir() {
  showResetDbStorageDialog.value = false;

  try {
    isChangingDbStorage.value = true;
    const newPath = await resetDbStorageDir();
    dbStorageDir.value = String(newPath.path || '');
    hasCustomDbStorage.value = false;
    if (newPath.cleanupWarnings?.length) {
      toast.warning(`${localeMsg.value.settings?.database?.cleanup_warning || 'Library data was moved, but some old files could not be removed:'}\n${newPath.cleanupWarnings.join('\n')}`, { duration: 10000 });
    } else {
      toast.success(localeMsg.value.settings?.database?.restore_default_success || 'Library data has been moved back to the default location');
    }
  } catch (error: any) {
    toast.error(error?.message || String(error));
  } finally {
    isChangingDbStorage.value = false;
  }
}

function normalizeScale(value: number) {
  return SCALE_VALUES.find((item) => item === Number(value)) ?? 1;
}

function applyWindowScale(scale: number) {
  const normalizedScale = normalizeScale(scale);
  document.documentElement.style.fontSize = `${normalizedScale * 16}px`;
}

async function updateSettingsWindowSize(scale: number) {
  const normalizedScale = normalizeScale(scale);
  const width = Math.round(SETTINGS_BASE_WIDTH * normalizedScale);
  const height = Math.round(SETTINGS_BASE_HEIGHT * normalizedScale);
  const size = new LogicalSize(width, height);

  await appWindow.setMinSize(size);
  await appWindow.setSize(size);
}

</script>
