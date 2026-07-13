<template>
  <div class="w-screen h-screen flex flex-col bg-base-300 text-base-content/70 overflow-hidden">
    <!-- Title Bar -->
    <TitleBar :titlebar="$t('sidebar.settings')" :resizable="false" viewName="Settings" class="shrink-0 z-50" />

    <div class="flex flex-1 overflow-hidden relative">
      <!-- Sidebar -->
      <div class="w-40 m-1 p-2 bg-base-200/30 flex flex-col rounded-box overflow-y-auto shrink-0 select-none">
        <div
          v-for="(tab, index) in settingsTabs"
          :key="index"
          :class="[
            'px-3 py-2 rounded-box cursor-pointer transition-all duration-200 font-medium flex items-center',
            config.settings.tabIndex === index 
              ? 'bg-base-100 text-primary' 
              : 'hover:text-base-content hover:bg-base-100/30'
          ]"
          @click="config.settings.tabIndex = index"
        >
          {{ $t(tab) }}
        </div>
      </div>

      <!-- Main Content -->
      <div class="p-2 mr-1 mb-2 flex-1 overflow-y-auto scrollbar-hide bg-base-300 cursor-default select-none">
          
        <!-- General Tab -->
        <div v-if="config.settings.tabIndex === 0" class="flex flex-col space-y-2">
          
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

        </div>

        <!-- Library Tab -->
        <div v-else-if="config.settings.tabIndex === 1" class="flex flex-col space-y-2">

          <!-- album -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.library.section_album') }}</span>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.library.show_subfolder_files') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.library.show_subfolder_files_hint') }}</div>
              </div>
              <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="config.settings.showSubfolderFiles" />
            </div>
          </div>

          <!-- collections -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.library.section_collections') }}</span>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.library.show_collections') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.library.show_collections_hint') }}</div>
              </div>
              <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="config.settings.showCollections" />
            </div>
          </div>

          <!-- sorting -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.library.section_sorting') }}</span>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.library.folder_sort') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.library.folder_sort_hint') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-40" v-model="config.settings.folderSort">
                <option v-for="option in folderSortOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.library.calendar_sort') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.library.calendar_sort_hint') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-40" v-model="config.settings.calendarSort">
                <option v-for="option in calendarSortOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.library.category_sort') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.library.category_sort_hint') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-40" v-model="config.settings.categorySort">
                <option v-for="option in categorySortOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
          </div>

        </div>

        <!-- Browse Tab -->
        <div v-else-if="config.settings.tabIndex === 2" class="flex flex-col space-y-2">

          <!-- grid view -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.browse.section_grid') }}</span>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.browse.style') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.grid.style">
                <option v-for="(option, index) in gridStyleOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.browse.scaling') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.grid.scaling" :disabled="config.settings.grid.style !== 0 && config.settings.grid.style !== 1">
                <option v-for="(option, index) in gridScalingOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.browse.show_thumbnail_badges') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.grid.thumbnailBadge">
                <option v-for="option in thumbnailBadgeOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.browse.label_primary') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.grid.labelPrimary" :disabled="config.settings.grid.style !== 0">
                  <option v-for="(option, index) in gridLabelOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.browse.label_secondary') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.grid.labelSecondary" :disabled="config.settings.grid.style !== 0">
                  <option v-for="(option, index) in gridLabelOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
          </div>

          <!-- filmstrip -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.browse.filmstrip_view.title') }}</span>
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.browse.filmstrip_view.enable_filmstrip') }}</div>
              </div>
              <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="config.settings.grid.showFilmStrip" />
            </div>
            <div class="flex items-center justify-between px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.browse.filmstrip_view.preview_position') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.grid.previewPosition" :disabled="!config.settings.grid.showFilmStrip">
                <option v-for="(option, index) in filmStripViewPreviewPositionOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
          </div>

        </div>

        <!-- Viewer Tab -->
        <div v-else-if="config.settings.tabIndex === 3" class="flex flex-col space-y-2">

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
                  {{ $t('settings.image_view.view_background_hint', { shortcut: cycleBackgroundShortcut }) }}
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

        <!-- Search Tab -->
        <div v-else-if="config.settings.tabIndex === 4" class="flex flex-col overflow-hidden space-y-2">

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
          </div>

          <!-- find similar -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.image_search.find_similar') }}</span>
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

        <!-- Shortcuts Tab -->
        <div v-else-if="config.settings.tabIndex === 5" class="flex flex-col space-y-2">
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

        <!-- Advanced Tab -->
        <div v-else-if="config.settings.tabIndex === 6" class="flex flex-col space-y-2">

          <!-- external app -->
          <div class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
            <div class="flex items-center gap-2 text-base-content/30">
              <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.advanced.section_external_apps') }}</span>
            </div>
            <div class="flex items-center justify-between gap-4 px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="min-w-0 flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.advanced.external_image_editor') }}</div>
                <div class="text-xs text-base-content/30 truncate" :title="config.settings.externalImageAppPath || ''">
                  {{ externalImageAppName }}
                </div>
              </div>
              <div class="shrink-0 flex items-center gap-1">
                <button
                  class="btn btn-sm btn-ghost min-w-20 rounded-box bg-base-100 border border-base-content/30 text-base-content/70 hover:text-base-content"
                  @click="selectExternalApp('image')"
                >
                  {{ $t('settings.advanced.choose_app') }}
                </button>
                <TButton v-if="config.settings.externalImageAppPath"
                  :icon="IconTrash"
                  :buttonSize="'small'"
                  :tooltip="$t('settings.advanced.clear_app')"
                  @click="clearExternalApp('image')"
                />
              </div>
            </div>
            <div class="flex items-center justify-between gap-4 px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="min-w-0 flex flex-col gap-0.5 text-sm leading-5">
                <div>{{ $t('settings.advanced.external_video_app') }}</div>
                <div class="text-xs text-base-content/30 truncate" :title="config.settings.externalVideoAppPath || ''">
                  {{ externalVideoAppName }}
                </div>
              </div>
              <div class="shrink-0 flex items-center gap-1">
                <button
                  class="btn btn-sm btn-ghost min-w-20 rounded-box bg-base-100 border border-base-content/30 text-base-content/70 hover:text-base-content"
                  @click="selectExternalApp('video')"
                >
                  {{ $t('settings.advanced.choose_app') }}
                </button>
                <TButton v-if="config.settings.externalVideoAppPath"
                  :icon="IconTrash"
                  :buttonSize="'small'"
                  :tooltip="$t('settings.advanced.clear_app')"
                  @click="clearExternalApp('video')"
                />
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

        <!-- About Tab -->
        <div v-else-if="config.settings.tabIndex === 7" class="py-2">
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
import { THUMBNAIL_BADGE } from '@/common/constants';
import {
  getExternalAppDisplayName,
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
} from '@/common/api';
import { formatFileSize, isLinux, isMac, setTheme, SCALE_VALUES } from '@/common/utils';
import { getShortcutLabels, ShortcutActionId, ShortcutPlatform } from '@/common/shortcuts';
import { useToast } from '@/common/toast';
import { IconTrash, IconRestore, IconClose } from '@/common/icons';

import TitleBar from '@/components/TitleBar.vue';
import SettingsAbout from '@/components/SettingsAbout.vue';
import MessageBox from '@/components/MessageBox.vue';
import BackupDialog from '@/components/BackupDialog.vue';
import RestoreDialog from '@/components/RestoreDialog.vue';
import TButton from '@/components/TButton.vue';

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[config.settings.language] as any);
const toast = useToast();
const shortcutPlatform: ShortcutPlatform = isMac ? 'mac' : (isLinux ? 'linux' : 'windows');
const settingsTabs = [
  'settings.general.title',
  'settings.library.title',
  'settings.browse.title',
  'settings.image_view.title',
  'settings.image_search.title',
  'settings.shortcuts.title',
  'settings.advanced.title',
  'settings.about.title',
];

const appWindow = getCurrentWebviewWindow()
let gridSizeEmitTimer: number | null = null;
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
let unlistenImageSearchModelDownloadProgress: (() => void) | null = null;

const onRestoreDone = () => {
  showRestoreDialog.value = false;
  emit('libraries-changed');
};

const languages = [
  { label: 'English', value: 'en' },
  { label: 'Deutsch', value: 'de' },
  { label: 'Español', value: 'es' },
  { label: 'Français', value: 'fr' },
  { label: 'Português', value: 'pt' },
  { label: 'Русский', value: 'ru' },
  { label: '中文', value: 'zh' },
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
  const options = localeMsg.value.settings.library.folder_sort_options || [];
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

const calendarSortOptions = computed(() => {
  const options = localeMsg.value.settings.library.calendar_sort_options || [];
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

const categorySortOptions = computed(() => {
  const options = localeMsg.value.settings.library.category_sort_options || [];
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

const externalImageAppName = computed(() =>
  String(config.settings.externalImageAppName || '') || localeMsg.value.settings.advanced.external_app_not_selected
);

const externalVideoAppName = computed(() =>
  String(config.settings.externalVideoAppName || '') || localeMsg.value.settings.advanced.external_app_not_selected
);

// Define the wheel options using computed to react to language changes
const wheelOptions = computed(() => {
  const options = localeMsg.value.settings.image_view.mouse_wheel_options; // returns an array
  return [
    { label: options[0], value: 0 },  // 0: previous / next
    { label: options[1], value: 1 },  // 1: zoom in / out
  ];
});

// Define the grid scaling options
const gridScalingOptions = computed(() => {
  const options = localeMsg.value.settings.browse.scaling_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

// Define the grid style options
const gridStyleOptions = computed(() => {
  const options = localeMsg.value.settings.browse.style_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

// Define the grid label options
const gridLabelOptions = computed(() => {
  const options = localeMsg.value.settings.browse.label_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

const thumbnailBadgeOptions = computed(() => {
  const options = localeMsg.value.settings.browse.thumbnail_badge_options;
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
const cycleBackgroundShortcut = computed(() =>
  getShortcutLabels('view.cycleBackground', shortcutPlatform)[0] || 'Shift+B'
);

const slideShowTransitionOptions = computed(() => {
  const options = localeMsg.value.settings.image_view.slide_show_transition_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

const filmStripViewPreviewPositionOptions = computed(() => {
  const options = localeMsg.value.settings.browse.filmstrip_view.preview_position_options;
  return options.map((label, i) => ({ label, value: i }));
});

// Define the similarity options
const similarityOptions = computed(() => {
  const options = localeMsg.value.settings.image_search.similarity_options;
  // Use getter to retrieve thresholds
  const values = config.imageSearchThresholds ?? [0.8, 0.6, 0.4, 0.25]; 
  // Map index dummy as the value since v-model is thresholdIndex
  return values.map((val, i) => ({ label: options[i], value: i }));
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
    config.settings.imageSearch.model = 0;
    await setImageSearchModel(0);
    return;
  }

  try {
    await setImageSearchModel(config.settings.imageSearch.model || 0);
  } catch (error) {
    console.error('Failed to activate image search model:', error);
    config.settings.imageSearch.model = 0;
    await setImageSearchModel(0);
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
      { actionId: 'meta.tag', labelKey: 'edit_tags' },
      { actionId: 'meta.comment', labelKey: 'edit_comment' },
      { actionId: 'meta.rotate', labelKey: 'rotate' },
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
        label: shortcutMessages.actions[getShortcutActionLabelKey(item)],
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
    toast.success(localeMsg.value.settings.image_search.multilingual_model_download_success);
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
    toast.error(error?.message || localeMsg.value.settings.image_search.multilingual_model_download_failed);
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
  if (typeof config.settings.tabIndex !== 'number' || config.settings.tabIndex < 0 || config.settings.tabIndex > 7) {
    config.settings.tabIndex = 0;
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

  if (config.settings.externalImageAppPath) {
    try {
      config.settings.externalImageAppName = await getExternalAppDisplayName(config.settings.externalImageAppPath);
    } catch {
      config.settings.externalImageAppName = '';
    }
  }

  if (config.settings.externalVideoAppPath) {
    try {
      config.settings.externalVideoAppName = await getExternalAppDisplayName(config.settings.externalVideoAppPath);
    } catch {
      config.settings.externalVideoAppName = '';
    }
  }
  
  // Show window after mount
  await appWindow.show();
});

onUnmounted(() => {
  if (isDownloadingMultilingualModel.value) {
    void cancelMultilingualImageSearchModelDownload();
  }
  if (gridSizeEmitTimer) {
    clearTimeout(gridSizeEmitTimer);
    gridSizeEmitTimer = null;
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
watch(() => config.settings.externalImageAppPath, (newValue) => {
  emit('settings-externalImageAppPath-changed', newValue);
});
watch(() => config.settings.externalImageAppName, (newValue) => {
  emit('settings-externalImageAppName-changed', newValue);
});
watch(() => config.settings.externalVideoAppPath, (newValue) => {
  emit('settings-externalVideoAppPath-changed', newValue);
});
watch(() => config.settings.externalVideoAppName, (newValue) => {
  emit('settings-externalVideoAppName-changed', newValue);
});
watch(() => config.settings.language, (newValue) => {
  locale.value = newValue;
  emit('settings-language-changed', newValue);
});
watch(() => config.settings.showToolTip, (newValue) => {
  emit('settings-showToolTip-changed', newValue);
});
watch(() => config.settings.showStatusBar, (newValue) => {
  emit('settings-showStatusBar-changed', newValue);
});
watch(() => config.settings.autoCheckUpdates, (newValue) => {
  emit('settings-autoCheckUpdates-changed', newValue);
});
watch(() => config.settings.showCollections, (newValue) => {
  emit('settings-showCollections-changed', newValue);
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

// grid view settings
watch(() => config.settings.grid.size, (newValue: number) => {
  if (gridSizeEmitTimer) {
    clearTimeout(gridSizeEmitTimer);
  }

  gridSizeEmitTimer = window.setTimeout(() => {
    emit('settings-gridSize-changed', newValue);
    gridSizeEmitTimer = null;
  }, 100);
});
watch(() => config.settings.grid.style, (newValue) => {
  emit('settings-gridStyle-changed', newValue);
});
watch(() => config.settings.grid.scaling, (newValue) => {
  emit('settings-gridScaling-changed', newValue);
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
watch(() => config.settings.grid.showFilmStrip, (newValue) => {
  emit('settings-showFilmStrip-changed', newValue);
});
watch(() => config.settings.grid.previewPosition, (newValue) => {
  emit('settings-filmStripViewPreviewPosition-changed', newValue);
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
watch(() => config.settings.imageSearch.limit, (newValue) => {
  emit('settings-imageSearchLimit-changed', newValue);
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
    case 'Tab':
      config.settings.tabIndex += 1;
      config.settings.tabIndex = config.settings.tabIndex % settingsTabs.length;
      break;
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
    dbStorageDir.value = String(newPath || result);
    hasCustomDbStorage.value = true;
    toast.success(localeMsg.value.settings?.database?.change_success || 'Library data has been moved successfully');
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
    dbStorageDir.value = String(newPath || '');
    hasCustomDbStorage.value = false;
    toast.success(localeMsg.value.settings?.database?.restore_default_success || 'Library data has been moved back to the default location');
  } catch (error: any) {
    toast.error(error?.message || String(error));
  } finally {
    isChangingDbStorage.value = false;
  }
}

async function selectExternalApp(kind: 'image' | 'video') {
  const result = await openDialog({
    title: kind === 'image'
      ? localeMsg.value.settings.image_view.external_image_editor
      : localeMsg.value.settings.image_view.external_video_app,
    multiple: false,
    directory: false,
    ...(isMac
      ? {
          defaultPath: '/Applications',
          filters: [{ name: 'Applications', extensions: ['app'] }],
        }
      : {}),
  });

  if (!result || Array.isArray(result)) return;
  let displayName = '';
  try {
    displayName = await getExternalAppDisplayName(result);
  } catch {}

  if (kind === 'image') {
    config.settings.externalImageAppPath = result;
    config.settings.externalImageAppName = displayName;
  } else {
    config.settings.externalVideoAppPath = result;
    config.settings.externalVideoAppName = displayName;
  }
}

function clearExternalApp(kind: 'image' | 'video') {
  if (kind === 'image') {
    config.settings.externalImageAppPath = '';
    config.settings.externalImageAppName = '';
  } else {
    config.settings.externalVideoAppPath = '';
    config.settings.externalVideoAppName = '';
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
