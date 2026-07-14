<template>

  <div
    ref="contentRootRef"
    tabindex="-1"
    class="relative flex-1 flex flex-col select-none outline-none"
    :class="{ 'opacity-50 pointer-events-none': uiStore.isInputActive('ManageLibraries') }"
    @focus="activateContentPane"
    @mousedown.capture="activateContentPane"
    @mouseenter="isContentHovered = true"
    @mouseleave="isContentHovered = false"
    @wheel.capture="handleContentWheel"
    @keydown="handleLocalKeyDown"
    @dragover.prevent
  >

    <!-- Loading overlay -->
    <transition name="fade">
      <div v-if="isProcessing" class="absolute inset-0 bg-base-100/50 flex items-center justify-center z-50 rounded-box">
        <span class="loading loading-dots text-primary"></span>
      </div>
    </transition>

    <!-- title bar -->
    <div
      v-if="!showWelcomeContent"
      class="absolute top-0 left-0 right-0 px-2 h-12 flex flex-row flex-nowrap items-center justify-between bg-base-300 z-30 overflow-hidden"
      data-tauri-drag-region
    >
      <!-- title -->
      <div class="mr-1 flex flex-row items-center gap-1 min-w-0 flex-1 overflow-hidden" data-tauri-drag-region>
        <TButton v-if="tempViewMode !== 'none'"
          :icon="IconPrev"
          :buttonSize="'medium'"
          :tooltip="$t('toolbar.tooltip.back')"
          :selected="true"
          @click="exitTempViewMode"
        />
        <component v-if="currentTitleIcon" 
          :is="currentTitleIcon" 
          class="t-icon-size-sm shrink-0"
          :class="{ 'cursor-pointer text-primary': tempViewMode !== 'none' }" 
          @click="handleTitleClick"
        />
        <Breadcrumb
          v-if="contentTitle"
          :items="titleBreadcrumbItems"
          :highlight="tempViewMode !== 'none'"
          size="large"
          class="min-w-0 overflow-hidden"
          @navigate="handleTitleBreadcrumbNavigate"
        />
      </div>

      <!-- toolbar -->
      <div class="flex items-center gap-2 shrink-0">

        <!-- file type options -->
        <DropDownSelect
          :options="fileTypeOptions"
          :multiSelect="true"
          :selectedValues="fileTypeSelectedValues"
          :summaryLabel="fileTypeSummaryLabel"
          :separatorsAfter="[0]"
          :disabled="tempViewMode === 'album' || tempViewMode === 'person' || isScanStreamingMode"
          :selected="activeFileTypeMask !== 0"
          @multi-select="handleFileTypeSelect"
        />

        <!-- sort type options (hidden in AI search mode and similar temp mode) -->
        <DropDownSelect
          v-if="!isAiSearchMode && tempViewMode !== 'similar'"
          :options="sortOptions"
          :defaultIndex="toolbarSortType"
          :extendOptions="sortExtendOptions"
          :defaultExtendIndex="toolbarSortOrder"
          :disabled="tempViewMode === 'album' || tempViewMode === 'person' || isScanStreamingMode"
          @select="handleSortTypeSelect"
        />

        <!-- select and layout section -->
        <div class="flex flex-row items-center">
          <IconSeparator class="t-icon-size text-base-content/30" />
          
          <!-- refresh file list -->
          <!-- Bugfix: need to update album files when the album is updated -->
          <!-- <TButton
            :icon="IconRefresh"
            :tooltip="$t('toolbar.tooltip.refresh')"
            :disabled="isIndexing || showQuickView"
            @click="updateContent()"
          /> -->
          
          <!-- grid size slider -->
          <div class="flex flex-row items-center gap-2 px-2 shrink-0 group">
            <SliderInput v-model="config.settings.grid.size" 
              :min="120" :max="360" :step="1" label="" :slider_width="80" 
            />
          </div>

          <!-- grid styles cycle -->
          <TButton
            :icon="[IconCard, IconTile, IconJustified, IconJustified][config.settings.grid.style]"
            :iconStyle="{
              transform: `rotate(${config.settings.grid.style === 3 ? 90 : 0}deg)`,
            }"
            :tooltip="localeMsg.settings.browse.style_options[config.settings.grid.style]"
            @click="cycleGridStyle"
          />

          <!-- toggle filmstrip -->
          <TButton
            :icon="IconFilmstrip"
            :iconStyle="{ 
              transform: `rotate(${config.settings.grid.previewPosition === 0 ? 180 : (config.settings.grid.previewPosition === 2 ? 90 : (config.settings.grid.previewPosition === 3 ? 270 : 0))}deg)`, 
              transition: 'transform 0.3s ease-in-out' 
            }" 
            :tooltip="localeMsg.settings.browse.filmstrip_view.title"
            :selected="config.settings.grid.showFilmStrip"
            @click="toggleFilmstripView"
          />

          <IconSeparator class="t-icon-size text-base-content/30" />
          <!-- toggle select mode -->
          <TButton
            :icon="IconSelection"
            :tooltip="$t('toolbar.filter.select_mode')"
            :selected="selectMode"
            :disabled="isScanStreamingMode"
            @click="handleSelectMode(!selectMode)"
          />

          <!-- toggle dedup panel -->
          <TButton
            :icon="IconSimilar"
            :tooltip="$t('toolbar.tooltip.open_dedup')"
            :selected="isDedupPanelOpen"
            :disabled="isScanStreamingMode"
            @click="toggleDedupPanel"
          />

          <!-- toggle info panel -->
          <TButton
            :icon="IconInformation"
            :tooltip="isInfoPanelOpen ? $t('toolbar.tooltip.hide_info') : $t('toolbar.tooltip.show_info')"
            :shortcut="shortcut('meta.info')"
            :selected="isInfoPanelOpen"
            @click="toggleInfoPanel"
          />
        </div>
      </div>
    </div>

    <!-- progress bar -->
    <div v-if="showTopProgressBar" class="absolute top-11 left-0 right-0 z-50">
      <ProgressBar :percent="topProgressPercent" />
    </div>

    <!-- content view -->
    <div ref="contentViewDiv" class="relative flex-1 flex flex-row overflow-hidden">
      <div class="relative flex-1 flex flex-row overflow-hidden">
        <div ref="gridViewDiv" 
          :class="[
            'flex-1 flex',
            gridViewLayoutClass,
            config.settings.grid.showFilmStrip && !showWelcomeContent ? (config.settings.showStatusBar ? 'mt-12 mb-8' : 'mt-12 mb-1') : ''
          ]"
        >
          <div class="relative" 
            :class="{ 'flex-1': showWelcomeContent || !config.settings.grid.showFilmStrip }"
            :style="{ 
              height: (config.settings.grid.showFilmStrip && !showWelcomeContent && !isFilmstripVertical) ? itemSize + 'px' : '',
              width: (config.settings.grid.showFilmStrip && !showWelcomeContent && isFilmstripVertical) ? itemWidth + 'px' : ''
            }"
          >
            <!-- grid view -->
            <div ref="gridScrollContainerRef" class="absolute w-full h-full">
              <Welcome v-if="showWelcomeContent" />
              <GridView v-else ref="gridViewRef"
                :selected-item-index="selectedItemIndex"
                :fileList="gridRows"
                :timeline-data="timelineData"
                :sort-type="currentQueryParams.sortType"
                :selectMode="selectMode"
                :content-ready="contentReady"
                :empty-message="emptyFilesMessage"
                :empty-hint="emptyFilesHint"
                :layout-version="layoutVersion"
                :group-by="activeGroupBy"
                :group-selected-counts="groupSelectedCounts"
                :group-selection-loading="groupSelectionLoading"
                :folder-group-roots="folderGroupRoots"
                :query-source="currentQuerySource"
                :dedup-statuses="dedupStatuses"
                @item-clicked="handleItemClicked"
                @item-dblclicked="handleItemDblClicked"
                @item-select-toggled="handleItemSelectToggled"
                @item-action="handleItemAction"
                @item-select-contextmenu="handleSelectionContextMenu"
                @date-group-select="handleDateGroupSelect"
                @group-select-toggled="handleGroupSelectToggled"
                @visible-range-update="handleVisibleRangeUpdate"
                @scroll="handleGridScroll"
                @layout-update="handleLayoutUpdate"
                @item-drag-start="markContentInternalDrag"
                @item-drag="updateContentDragPosition"
                @item-drag-end="clearContentInternalDrag"
              />
              <!-- Navigation buttons -->
              <div v-if="!showWelcomeContent && config.settings.grid.showFilmStrip && fileList.length > 0" 
                class="absolute z-10 inset-1 flex items-center justify-between pointer-events-none"
                :class="{ 'flex-col': isFilmstripVertical }"
              >
                <button 
                  :class="[
                    'p-2 rounded-full pointer-events-auto bg-base-100/30', 
                    selectedItemIndex > 0 ? 'text-base-content/70 hover:text-base-content hover:bg-base-100/70 cursor-pointer' : 'text-base-content/30'
                  ]"
                  @click="requestNavigate('prev')"
                  @dblclick.stop
                >
                  <component :is="isFilmstripVertical ? IconArrowUp : IconLeft" class="w-8 h-8" />
                </button>
                <button 
                  :class="[
                    'p-2 rounded-full pointer-events-auto bg-base-100/30', 
                    selectedItemIndex < fileList.length - 1 ? 'text-base-content/70 hover:text-base-content hover:bg-base-100/70 cursor-pointer' : 'text-base-content/30'
                  ]"
                  @click="requestNavigate('next')" 
                  @dblclick.stop
                >
                  <component :is="isFilmstripVertical ? IconArrowDown : IconRight" class="w-8 h-8" />
                </button> 
              </div>
            </div>
          </div>

          <div v-if="!showWelcomeContent && config.settings.grid.showFilmStrip" 
            :class="isFilmstripVertical ? 'w-1 shrink-0' : 'h-1 shrink-0'"
          ></div>

          <!-- film strip preview -->
          <div v-if="!showWelcomeContent && config.settings.grid.showFilmStrip" ref="previewDiv" 
            class="flex-1 bg-base-200 overflow-hidden"
          >
            <div v-if="selectedItemIndex >= 0 && selectedItemIndex < fileList.length"
              class="w-full h-full flex items-center justify-center"
            >
              <MediaViewer
                ref="filmStripMediaRef"
                :mode="1"
                :isFullScreen="false"
                :file="fileList[selectedItemIndex]"
                :nextFilePath="getNextImagePath(selectedItemIndex)"
                :hasPrevious="selectedItemIndex > 0"
                :hasNext="selectedItemIndex < fileList.length - 1"
                :fileIndex="selectedItemIndex"
                :fileCount="fileList.length"
                :isSlideShow="isSlideShow"
                :canSlideShow="true"
                :imageScale="imageScale"
                :imageMinScale="imageMinScale"
                :imageMaxScale="imageMaxScale"
                v-model:isZoomFit="filmStripZoomFit"
                @prev="performNavigate('prev')"
                @next="performNavigate('next')"
                @toggle-slide-show="toggleSlideShow"
                @scale="onScale"
                @item-action="handleItemAction"
                @slideshow-next="handleSlideshowNext"
              />
            </div>
          </div> <!-- film strip preview -->
        </div> <!-- grid view -->

        <!-- custom scrollbar -->
        <div v-if="!showWelcomeContent && !config.settings.grid.showFilmStrip && fileList.length > 0"
          class="mt-12 shrink-0" 
          :class="[ config.settings.showStatusBar ? 'mb-8' : 'mb-1' ]"
        >
          <ScrollBar
            :total="scrollbarTotal"
            :pageSize="scrollbarPageSize"
            :modelValue="scrollPosition"
            :markers="scrollbarMarkers"
            :selectedIndex="scrollbarSelectedIndex"
            @update:modelValue="handleScrollUpdate"
            @select-item="handleTimelineSelectItem"
          ></ScrollBar>
        </div>

        <!-- Quick View Overlay -->
        <div v-if="showQuickView && fileList[selectedItemIndex]" 
          class="absolute inset-0 z-60 flex items-center justify-center bg-base-200/95 backdrop-blur-lg overflow-hidden"
          :class="[ config.settings.showStatusBar ? 'mt-12 mb-8': 'mt-12' ]"
        >
          <div
            class="relative w-full h-full flex items-center justify-center"
            @mousemove="handleQuickViewMouseMove"
            @mouseleave="handleQuickViewMouseLeave"
          >
            <MediaViewer
              ref="quickViewMediaRef"
              :mode="0"
              :isFullScreen="false"
              :file="fileList[selectedItemIndex]"
              :nextFilePath="getNextImagePath(selectedItemIndex)"
              :hasPrevious="selectedItemIndex > 0"
              :hasNext="selectedItemIndex < fileList.length - 1"
              :fileIndex="selectedItemIndex"
              :fileCount="fileList.length"
              :isSlideShow="isSlideShow"
              :canSlideShow="true"
              :imageScale="imageScale"
              :imageMinScale="imageMinScale"
              :imageMaxScale="imageMaxScale"
              :showOverlayNav="false"
              v-model:isZoomFit="quickViewZoomFit"
              @prev="performNavigate('prev')"
              @next="performNavigate('next')"
              @toggle-slide-show="toggleSlideShow"
              @scale="onScale"
              @item-action="handleItemAction"
              @close="closeQuickPreview()"
              @slideshow-next="handleSlideshowNext"
            />
            <div class="absolute inset-1 z-90 flex items-center justify-between pointer-events-none">
              <button
                :class="[
                  'p-2 rounded-full bg-base-100/30 transition-opacity duration-200',
                  quickViewHoverLeft
                    ? (selectedItemIndex > 0
                      ? 'opacity-100 pointer-events-auto text-base-content/70 hover:text-base-content hover:bg-base-100/70 cursor-pointer'
                      : 'opacity-30 pointer-events-none cursor-default text-base-content/30')
                    : 'opacity-0 pointer-events-none'
                ]"
                :disabled="selectedItemIndex <= 0"
                @pointerdown.stop
                @click.stop="requestNavigate('prev')"
                @dblclick.stop
              >
                <IconLeft class="w-8 h-8" />
              </button>
              <button
                :class="[
                  'p-2 rounded-full bg-base-100/30 transition-opacity duration-200',
                  quickViewHoverRight
                    ? (selectedItemIndex < fileList.length - 1
                      ? 'opacity-100 pointer-events-auto text-base-content/70 hover:text-base-content hover:bg-base-100/70 cursor-pointer'
                      : 'opacity-30 pointer-events-none cursor-default text-base-content/30')
                    : 'opacity-0 pointer-events-none'
                ]"
                :disabled="selectedItemIndex >= fileList.length - 1"
                @pointerdown.stop
                @click.stop="requestNavigate('next')"
                @dblclick.stop
              >
                <IconRight class="w-8 h-8" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- info panel splitter -->
      <div v-if="rightPanelLayoutVisible"
        class="w-1 shrink-0 transition-colors mt-12"
        :class="{
          'mb-8': config.settings.showStatusBar,
          'mb-1': !config.settings.showStatusBar,
          'hover:bg-primary cursor-col-resize': rightPanelLayoutVisible,
          'bg-primary': rightPanelLayoutVisible && isDraggingInfoPanel,
        }" 
        @mousedown="startDraggingInfoPanelSplitter"
      ></div>

      <!-- info panel -->
      <div
        v-if="rightPanelMounted"
        class="relative shrink-0"
        :style="{ width: rightPanelLayoutVisible ? activeRightPanelWidth + 'px' : '0px' }"
      >
        <div
          :class="[
            'absolute right-0 z-40 pr-1 transition-transform duration-200 ease-in-out',
            rightPanelVisualVisible ? 'translate-x-0' : 'translate-x-full pointer-events-none',
          ]"
          :style="{ width: activeRightPanelWidth + 'px', top: '3rem', bottom: config.settings.showStatusBar ? '2rem' : '0.25rem' }"
        >
          <DedupPane
            v-if="!selectMode && config.rightPanel.mode === 'dedup'"
            ref="dedupPaneRef"
            :key="dedupScanKey"
            :file-list="fileList"
            :selected-file-id="fileList[selectedItemIndex]?.id"
            :dedup-scan-key="dedupScanKey"
            :dedup-query-params="dedupQueryParams"
            :dedup-collection-id="dedupCollectionId"
            :dedup-file-ids="dedupFileIds"
            @close="config.rightPanel.show = false"
            @select-file="handleDedupSelectFile"
            @preview-file="handleDedupPreviewFile"
            @trash-selected-duplicates="handleDedupTrashSelectedDuplicates"
            @dedup-status-updated="dedupStatuses = $event"
          />
          <SelectionPanel
            v-else-if="selectMode"
            :file-count="fileList.length"
            :selected-files="selectedFiles"
            :selected-count="selectedCount"
            :selected-size="selectedSize"
            :query-source="currentQuerySource"
            :more-actions="selectionMenuItems"
            @close="handleSelectMode(false)"
            @select-all="selectAllInCurrentList"
            @select-none="selectNoneInCurrentList"
            @select-invert="invertSelectionInCurrentList"
            @move-within-library="showMoveTo = true"
            @move-to-folder="onMoveToFolder"
            @copy-to-folder="onCopyToFolder"
            @remove-from-collection="removeSelectedFromCollection"
            @trash="openTrashMsgbox()"
            @favorite-all="selectModeSetFavorites(true)"
            @unfavorite-all="selectModeSetFavorites(false)"
            @set-rating-all="selectModeSetRatings"
            @tag-all="clickTag"
            @comment-all="openCommentEditor"
            @rotate-all="clickRotate"
            @unselect-file="unselectFileFromSelection"
            @more-action="action => action()"
          />
          <FileInfo
            v-else
            ref="fileInfoRef"
            :fileInfo="fileList[selectedItemIndex]" 
            @close="checkUnsavedChanges(() => config.rightPanel.show = false)" 
            @success="onFileSaved(true, $event)"
            @failed="onFileSaved(false)"
            @toggleFavorite="toggleFavorite"
            @setRating="setSelectedFileRating"
            @rotate="clickRotate"
            @quick-edit-tag="clickTag"
            @quick-edit-comment="openCommentEditor"
            @navigate-folder="handleInfoNavigateFolder"
            @edit-album="openAlbumEdit"
          />
        </div>
      </div>

    </div>

    <StatusBar
      v-if="config.settings.showStatusBar"
      :file-list="fileList"
      :selected-item-index="selectedItemIndex"
      :total-file-count="totalFileCount"
      :total-file-size="totalFileSize"
      :select-mode="selectMode"
      :selected-count="selectedCount"
      :selected-size="selectedSize"
      :show-film-strip="config.settings.grid.showFilmStrip"
      :show-quick-view="showQuickView"
      :image-scale="imageDisplayScale"
      :scan-text="statusBarScanText"
      :show-update-icon="statusBarShowUpdateIcon"
      :is-update-animating="statusBarIsUpdateAnimating"
      :update-icon="statusBarUpdateIcon"
    />
  </div>


  <!-- rename -->
  <MessageBox
    v-if="showRenameMsgbox"
    :title="$t('msgbox.rename_file.title')"
    :showInput="true"
    :inputText="renamingFileName.name"
    :inputPlaceholder="$t('msgbox.rename_file.placeholder')"
    :inputExtension="renamingFileName.ext"
    :needValidateInput="true"
    :OkText="$t('msgbox.rename_file.ok')"
    :cancelText="$t('msgbox.cancel')"
    :errorMessage="errorMessage"
    @ok="onRenameFile"
    @cancel="showRenameMsgbox = false"
    @reset="errorMessage = ''"
  />

  <!-- move to -->
  <MoveTo
    v-if="showMoveTo"
    :title="`${$t('msgbox.move_to.title', { source: selectMode ? $t('toolbar.filter.select_count', { count: selectedCount.toLocaleString() }) : shortenFilename(fileList[selectedItemIndex]?.name || '', 32) })}`"
    :message="$t('msgbox.move_to.content')"
    :OkText="$t('msgbox.move_to.ok')" 
    :cancelText="$t('msgbox.cancel')"
    @ok="onMoveTo"
    @cancel="showMoveTo = false"
  />

  <FileConflictDialog
    v-if="fileConflictDialog.show"
    :name="fileConflictDialog.name"
    :destination="fileConflictDialog.destination"
    :showApplyAll="fileConflictDialog.showApplyAll"
    :allowReplace="fileConflictDialog.allowReplace"
    @resolve="resolveFileConflict"
  />

  <!-- move to trash -->
  <MessageBox
    v-if="showTrashMsgbox"
    :title="trashMsgboxTitle"
    :message="trashMsgboxMessage"
    :OkText="trashMsgboxOkText"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    :checkboxText="$t('msgbox.permanent_delete.checkbox')"
    :checkboxChecked="deletePermanently"
    @ok="onTrashFile"
    @cancel="closeTrashMsgbox"
    @checkbox-change="deletePermanently = $event"
  />

  <!-- opening many files in an external app -->
  <MessageBox
    v-if="showExternalOpenWarningMsgbox"
    :title="$t('msgbox.open_external_many.title', { count: pendingExternalOpen?.paths.length ?? 0 })"
    :message="$t('msgbox.open_external_many.message', { count: pendingExternalOpen?.paths.length ?? 0 })"
    :OkText="$t('msgbox.open_external_many.ok')"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    @ok="confirmExternalOpen"
    @cancel="cancelExternalOpen"
  />

  <!-- tag -->
  <TaggingDialog 
    v-if="showTaggingDialog"
    :fileIds="fileIdsToTag"
    @ok="updateFileHasTags"
    @cancel="showTaggingDialog = false"
  />

  <!-- comment -->
  <MessageBox
    v-if="showCommentMsgbox"
    :title="$t('msgbox.edit_comment.title')"
    :showInput="true"
    :inputText="commentInputText"
    :inputPlaceholder="$t('msgbox.edit_comment.placeholder')"
    :multiLine="true"
    :OkText="$t('msgbox.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="onEditComment"
    @cancel="showCommentMsgbox = false"
  />

  <!-- Unsaved Changes Confirmation -->
  <MessageBox
    v-if="showUnsavedChangesMsgbox"
    :title="$t('msgbox.unsaved_changes.title') || 'Unsaved Changes'"
    :message="$t('msgbox.unsaved_changes.message') || 'You have unsaved changes. Do you want to save them?'"
    :OkText="$t('msgbox.image_editor.save') || 'Save'"
    :cancelText="$t('msgbox.cancel')"
    :thirdText="$t('msgbox.discard') || 'Discard'"
    :warningThird="true"
    @ok="confirmSave"
    @cancel="cancelDiscard"
    @third="confirmDiscard"
  />

  <IndexRecoveryDialog
    v-if="showIndexRecoveryMsgbox"
    :title="indexRecoveryTitle"
    :message="indexRecoveryMessage"
    :fileLabel="indexRecoveryFileLabel"
    :filePath="recoverySkipFilePath"
    :continueText="indexRecoveryOkText"
    :skipLabel="indexRecoverySkipLabel"
    :cancelText="$t('msgbox.cancel')"
    @continue="confirmIndexRecoveryContinue"
    @cancel="cancelIndexRecovery"
  />

  <!-- Drag-drop warning -->
  <MessageBox
    v-if="showDropWarning"
    :title="$t('msgbox.drop_import.title')"
    :message="$t('msgbox.drop_import.message')"
    :cancelText="''"
    @ok="showDropWarning = false"
    @cancel="showDropWarning = false"
  />

  <!-- Drop overlay -->
  <div v-if="isDragOver && acceptDrops" class="drop-overlay">
    <div class="drop-overlay-content">
      <IconDownload class="w-16 h-16"/>
      <span>{{ $t('msgbox.drop_import.overlay') }}</span>
    </div>
  </div>

  <Teleport to="body">
    <div class="print-only">
      <img
        v-if="printImageSrc"
        ref="printImageRef"
        :src="printImageSrc"
        alt=""
      />
    </div>
  </Teleport>

  <!-- Single shared context menu for multi-select right-click. It acts on the
       whole selection, so one instance lives here rather than one per thumbnail.
       The trigger is empty; it's opened at cursor coordinates by
       handleSelectionContextMenu, and its popup teleports to <body>. -->
  <div class="hidden">
    <ContextMenu
      ref="selectionMenuRef"
      :iconMenu="null"
      :menuItems="selectionMenuItems"
    >
      <template #trigger><span></span></template>
    </ContextMenu>
  </div>
</template>

<script setup lang="ts">

import { ref, watch, computed, createVNode, onMounted, onBeforeUnmount, nextTick, render, markRaw } from 'vue';
import { emit as tauriEmit, listen } from '@tauri-apps/api/event';
import { ask, open as openDialog } from '@tauri-apps/plugin-dialog';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useI18n } from 'vue-i18n';
import { useToast } from '@/common/toast';
import { useUIStore } from '@/stores/uiStore';
import { getAlbum, getAllAlbums, recountAlbum, getQueryCountAndSum, getQueryTimeLine, getQueryFiles, getGroupedQueryRows, getGroupFileIds, getQueryFileIds, syncAlbumFolderMtimes,
         getSmartQueryCountAndSum, getSmartQueryTimeLine, getSmartQueryFiles, getSmartGroupedQueryRows, getSmartGroupFileIds, getSmartQueryFileIds, getSmartQueryFilePosition,
         copyImages, renameFile, moveFile, moveFileOutsideLibrary, copyFile, deleteFile, deleteFilePermanently, batchDeleteFiles, editFileComment, getFileThumb, getFileThumbs, getFileInfo,
         setFileRotate, setFileFavorite, setFileRating, batchUpdateFileMetadata, getTagsForFile, searchSimilarImages, generateEmbedding,
         revealPath, getTagName, indexAlbum, listenIndexProgress, listenIndexFinished, setAlbumCover,
         updateFileInfo, importFile, importUrl, importFileBytes, getDragPayload, importClipboard, addFileToDb, checkFileExists, cancelIndexing as cancelIndexingApi, selectFolder, getFacesForFile, listenFaceIndexProgress,
         openFilesWithApp, getAppConfig, getIndexRecoveryInfo, clearIndexRecoveryInfo, setLastSelectedItemIndex,
         dedupDeleteSelected, getQueryFilePosition, getFolderSearchExcluded,
         listCollections, createCollection, addFilesToCollection, removeFilesFromCollection, getCollectionCountAndSum, getCollectionFiles, getCollectionFileIds, fetchFolder } from '@/common/api';
import { config, libConfig } from '@/common/config';
import { getShortcutLabel, matchesShortcut, ShortcutActionId, ShortcutPlatform } from '@/common/shortcuts';
import { getSmartTagById, SMART_TAG_SEARCH_THRESHOLD } from '@/common/smartTags';
import { getAlbumScanState, getAlbumScanIcon, shouldAnimateAlbumScanIcon } from '@/common/scanStatus';
import { DATE_SORT, GROUP, LIB_ITEM, RATE, SIDEBAR } from '@/common/constants';
import { isWin, isMac, isLinux, setTheme, separator,
         formatFileSize, formatDate, getCalendarDateRange, formatFolderBreadcrumb, getThumbnailDataUrl, getAssetSrc, getPreviewUrl,
         getCachedThumbnailDataUrl,
         clearCachedThumbnailDataUrl,
         extractFileName, combineFileName, getFolderPath, getFolderName, getSelectOptions, 
         shortenFilename, getSlideShowInterval, getFullPath, normalizePathForCompare, isWithinRootPath, shouldUseBackendPreview } from '@/common/utils';

import DropDownSelect from '@/components/DropDownSelect.vue';
import ProgressBar from '@/components/ProgressBar.vue';
import GridView  from '@/components/GridView.vue';
import ContextMenu from '@/components/ContextMenu.vue';
import { useFileMenuItems } from '@/common/fileMenu';
import Welcome from '@/components/Welcome.vue';
import MediaViewer from '@/components/MediaViewer.vue';
import MessageBox from '@/components/MessageBox.vue';
import IndexRecoveryDialog from '@/components/IndexRecoveryDialog.vue';
import MoveTo from '@/components/MoveTo.vue';
import TButton from '@/components/TButton.vue';
import TaggingDialog from '@/components/TaggingDialog.vue';
import FileInfo from '@/components/FileInfo.vue';
import Breadcrumb from '@/components/Breadcrumb.vue';
import DedupPane from '@/components/DedupPane.vue';
import SelectionPanel from '@/components/SelectionPanel.vue';
import FileConflictDialog from '@/components/FileConflictDialog.vue';
import ScrollBar from '@/components/ScrollBar.vue';
import SliderInput from '@/components/SliderInput.vue';
import StatusBar from '@/components/StatusBar.vue';

import {
  IconFolders,
  IconHeart,
  IconFiles,
  IconFolder,
  IconTag,
  IconBox,
  IconBolt,
  IconLocation,
  IconCameraAperture,
  IconLeft,
  IconRight,
  IconSeparator,
  IconCard,
  IconTile,
  IconJustified,
  IconFilmstrip,
  IconSelection,
  IconInformation,
  IconSearch,
  IconPhotoSearch,
  IconPersonSearch,
  IconFolderSearch,
  IconCalendarMonth,
  IconCalendarDay,
  IconArrowUp,
  IconArrowDown,
  IconDownload,
  IconPrev,
  IconAdd,
  IconPhotoAll,
  IconStack,
  IconStar,
  IconStarFilled,
  IconSimilar,
  IconCamera,
  IconHeartFilled,
  IconHistory,
  IconBookmark,
  IconSparkles,
  IconFile,
} from '@/common/icons';

const thumbnailPlaceholder = new URL('@/assets/images/image-file.png', import.meta.url).href;

const props = defineProps({
  titlebar: String,
  libraryEmpty: Boolean
});

/// i18n
const { locale, messages, t } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();

const contentTitle = ref("");

const titleSegments = computed(() => {
  if (!contentTitle.value) return [];
  const parts = contentTitle.value.split(' > ');
  return parts.length > 1 ? parts : [contentTitle.value];
});

const titleBreadcrumbItems = computed(() =>
  titleSegments.value.map((label, index) => ({
    label,
    path: String(index),
  }))
);

function handleTitleBreadcrumbNavigate(path: string) {
  const segmentIndex = Number(path);
  if (!Number.isInteger(segmentIndex)) return;

  if (segmentIndex === titleSegments.value.length - 1) {
    handleTitleClick();
    return;
  }

  handleBreadcrumbClick(segmentIndex);
}

function handleBreadcrumbClick(segmentIndex: number) {
  if (tempViewMode.value === 'album') {
    const segments = titleSegments.value;
    if (segmentIndex < 0 || segmentIndex >= segments.length - 1) return;

    const currentPath = currentQueryParams.value.searchFolder || '';
    if (!currentPath) return;

    const pathParts = currentPath.split(separator);
    const levelsToGoUp = segments.length - 1 - segmentIndex;
    const targetPath = pathParts.slice(0, Math.max(0, pathParts.length - levelsToGoUp)).join(separator);

    if (!targetPath || targetPath === currentPath) return;

    // Update breadcrumb title immediately to reflect parent navigation in temp album mode.
    contentTitle.value = segments.slice(0, segmentIndex + 1).join(' > ');

    const requestId = ++currentContentRequestId;
    clearContentRows();
    totalFileCount.value = 0;
    totalFileSize.value = 0;
    scrollPosition.value = 0;
    selectedItemIndex.value = 0;
    if (gridViewRef.value) {
      gridViewRef.value.scrollToPosition(0);
    }

    getFileList({ searchFolder: targetPath }, requestId);
    return;
  }

  const sidebarIndex = config.main.sidebarIndex;

  // Location: clicking parent segment navigates to admin1 only
  if (sidebarIndex === SIDEBAR.LOCATION) {
    if (segmentIndex === 0) {
      libConfig.location.name = null;
    }
    return;
  }

  // Camera: clicking parent segment navigates to make only
  if (sidebarIndex === SIDEBAR.CAMERA) {
    if (segmentIndex === 0) {
      if (!config.camera.isCamera) {
        (libConfig.camera as any).lensModel = null;
      } else {
        libConfig.camera.model = null;
      }
    }
    return;
  }

  // Album folders: navigate to parent folder
  if (sidebarIndex === SIDEBAR.ALBUM) {
    const segments = titleSegments.value;
    // The first segment is the album/folder root name.
    // Remaining segments are relative path components.
    // To navigate to segmentIndex, we rebuild the folder path.
    const currentPath = libConfig.album.folderPath || '';

    if (!currentPath) return;

    // Split the current path to reconstruct the target path
    const pathParts = currentPath.split(separator);
    // segments[0] = root folder name, segments[1..] = path parts after root
    // segmentIndex=0 means navigate to root, so we go up (segments.length - 1 - segmentIndex) levels
    const levelsToGoUp = segments.length - 1 - segmentIndex;
    const targetPath = pathParts.slice(0, pathParts.length - levelsToGoUp).join(separator);

    // Update title immediately for responsive UI, defer folderPath update to
    // the expand-album-folder event handler so folderId stays consistent.
    contentTitle.value = segments.slice(0, segmentIndex + 1).join(' > ');
    tauriEmit('expand-album-folder', { albumId: libConfig.album.id, folderPath: targetPath });
  }
}

// album's folder mode
const isCurrentFolderExcluded = ref(false);

const showFolderFiles = computed(() =>
  Boolean(config.main.sidebarIndex === SIDEBAR.ALBUM && libConfig.album.id && libConfig.album.id > 0 && !libConfig.album.selected)
);

// progress bar
const thumbCount = ref(0);      // thumbnail count (from 0 to fileList.length)
const showProgressBar = ref(false); // show progress bar

// div elements
const contentRootRef = ref<HTMLElement | null>(null);
const contentViewDiv = ref<HTMLDivElement | null>(null);
const gridViewDiv = ref<HTMLDivElement | null>(null);
const isContentHovered = ref(false);

// file list
const fileList = ref<any[]>([]);
const groupedRows = ref<any[]>([]);
const totalFileCount = ref(0);    // total files' count
const totalRowCount = ref(0);     // total render rows' count (group headers + files)
const totalFileSize = ref(0);     // total files' size

const selectedItemIndex = ref(-1);
let pendingInitialSelectedIndex = -1;
let hasRestoredInitialSelection = false;

// mutil select mode
const selectMode = ref(false);
const selectedCount = ref(0);
const selectedSize = ref(0);  // selected files size
const selectionChunkSize = computed(() => Number(config.main?.selectionChunkSize) || 200);
const isRealFileItem = (item: any) => !!item && !item.isPlaceholder && typeof item.id === 'number';
const isPendingFileItem = (item: any) => !item || item.isPlaceholder;

function createVirtualFileSlots(count: number) {
  return new Array(Math.max(0, Number(count || 0))).fill(null);
}
const isGroupRow = (item: any) => item?.type === 'group';
const isItemRow = (item: any) => item?.type === 'item';
const selectedFileIds = markRaw(new Set<number>());
const selectionVersion = ref(0);
const selectedFilesVersion = ref(0);
const setItemSelected = (index: number, selected: boolean) => {
  if (index < 0 || index >= fileList.value.length) return;
  if (!fileList.value[index] && !ensureGroupedFileAtIndex(index)) return;
  const item = fileList.value[index];
  if (!item) return;
  const fileId = Number(item?.id || 0);
  const previous = fileId > 0 ? selectedFileIds.has(fileId) : Boolean(item?.isSelected);
  if (previous === selected) {
    item.isSelected = selected;
    return;
  }
  item.isSelected = selected;
  if (fileId > 0) {
    if (selected) selectedFileIds.add(fileId);
    else selectedFileIds.delete(fileId);
  }
  applySelectionDelta(item, selected ? 1 : -1);
};
const getActionableSelectedItems = () =>
  fileList.value.filter(item => isRealFileItem(item) && (item.isSelected || selectedFileIds.has(Number(item.id))));
const selectedFiles = computed(() => {
  selectedFilesVersion.value;
  return selectMode.value ? getActionableSelectedItems() : [];
});
const getMediaKind = (items: any[]): 'image' | 'video' | 'mixed' | 'empty' => {
  let hasImage = false;
  let hasVideo = false;
  for (const item of items) {
    if (item?.file_type === 1 || item?.file_type === 3) hasImage = true;
    else if (item?.file_type === 2) hasVideo = true;
    if (hasImage && hasVideo) return 'mixed';
  }
  if (hasImage) return 'image';
  if (hasVideo) return 'video';
  return 'empty';
};
const selectionMediaKind = computed(() => getMediaKind(selectedFiles.value));

const selectionMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null);
const selectionMenuIndex = ref(-1);
const selectionMenuItems = useFileMenuItems(
  ref<any>(null),
  localeMsg,
  isMac,
  t,
  (action: string) => handleItemAction({ action, index: selectionMenuIndex.value }),
  {
    selectMode: ref(true),
    selectionMediaKind,
    selectionCount: selectedCount,
  },
);

// Opens the shared selection menu for a right-clicked thumbnail. Mirrors the
// per-thumbnail behavior: when a selection already exists, only open from an
// item that's part of it; when nothing is selected, select the clicked item
// first so the menu has a target, then wait a tick for that selection to flow
// into the menu before opening.
async function handleSelectionContextMenu({ x, y, index, isSelected }: { x: number; y: number; index: number; isSelected: boolean }) {
  if (!selectMode.value) return;
  if (!isSelected) {
    if (selectedCount.value > 0) return;
    handleItemClicked(index, false);
    await nextTick();
  }
  selectionMenuIndex.value = index;
  // Skip when there's no visible entry (empty/non-media selection), otherwise the
  // menu would render as an empty box.
  const hasVisibleItem = (selectionMenuItems.value ?? []).some((m: any) => !m.hidden);
  if (!hasVisibleItem) return;
  selectionMenuRef.value?.open?.(x, y);
}

const groupedModeActive = ref(false);
const selectedFolderHasChildren = ref(true);
const gridRows = computed(() => groupedModeActive.value ? groupedRows.value : fileList.value);
const groupFileIdsCache = new Map<string, number[]>();
const groupedTimelineGroups = ref<any[]>([]);
const folderGroupRoots = ref<Array<{ path: string; name?: string }>>([]);
const groupSelectedCountMap = ref<Record<string, number>>({});
const groupSelectedSizeMap = ref<Record<string, number>>({});
const groupSelectedCounts = computed(() => groupSelectedCountMap.value);
const groupSelectionLoading = ref<Record<string, boolean>>({});
const fileIdToGroupId = new Map<number, string>();
const groupMetaMap = new Map<string, { count: number; size: number }>();
const LARGE_BATCH_CONFIRM_THRESHOLD = 1000;
const FILE_OPERATION_CONCURRENCY = 8;

function clearSelectionForFileListUpdate() {
  selectMode.value = false;
  resetSelectionSummary();
  lastSelectedIndex.value = -1;
  keyboardSelectionAnchorIndex.value = -1;
  groupSelectedCountMap.value = {};
  groupSelectedSizeMap.value = {};
}

function clearLoadedSelectionFlags() {
  for (const file of fileList.value) {
    if (file) file.isSelected = false;
  }
}

function resetGroupingState() {
  groupedModeActive.value = false;
  groupedRows.value = [];
  totalRowCount.value = 0;
  groupFileIdsCache.clear();
  groupedTimelineGroups.value = [];
  groupSelectedCountMap.value = {};
  groupSelectedSizeMap.value = {};
  groupSelectionLoading.value = {};
  fileIdToGroupId.clear();
  groupMetaMap.clear();
}

function clearContentRows() {
  resetGroupingState();
  fileList.value = [];
}

function createViewBackup() {
  return {
    fileList: [...fileList.value],
    groupedModeActive: groupedModeActive.value,
    groupedRows: [...groupedRows.value],
    totalRowCount: totalRowCount.value,
    groupedTimelineGroups: [...groupedTimelineGroups.value],
    folderGroupRoots: [...folderGroupRoots.value],
    groupMetaEntries: Array.from(groupMetaMap.entries()),
    fileIdToGroupIdEntries: Array.from(fileIdToGroupId.entries()),
    totalFileCount: totalFileCount.value,
    totalFileSize: totalFileSize.value,
    contentTitle: contentTitle.value,
    selectedItemIndex: selectedItemIndex.value,
    scrollPosition: scrollPosition.value,
    timelineData: [...timelineData.value],
    currentQueryParams: { ...currentQueryParams.value },
    currentQuerySource: currentQuerySource.value,
    currentSmartQueryParams: currentSmartQueryParams.value ? { ...currentSmartQueryParams.value } : null,
    currentCollectionId: currentCollectionId.value,
    currentSearchFileIds: [...currentSearchFileIds.value],
    thumbCount: thumbCount.value,
    showProgressBar: showProgressBar.value,
    scrollTop: gridViewRef.value ? gridViewRef.value.getScrollTop() : 0,
  };
}

function getActiveCustomSmartAlbum() {
  if (libConfig.smartAlbum.type !== 'custom' || !libConfig.smartAlbum.id) return null;
  return (libConfig.smartAlbums || []).find(
    (item: any) => item.id === libConfig.smartAlbum.id
  ) || null;
}

function getAutoGroupByForCurrentView() {
  if (libConfig.activePane === 'collection') return GROUP.NONE;
  switch (Number(config.main.sidebarIndex)) {
    case SIDEBAR.LIBRARY:
      if (libConfig.library.item === LIB_ITEM.TODAY) return GROUP.YEAR;
      return libConfig.library.item === LIB_ITEM.RATINGS && libConfig.rating.item === RATE.ALL ? GROUP.RATING : GROUP.NONE;
    case SIDEBAR.ALBUM:
      return libConfig.album.selected || !config.settings.showSubfolderFiles || !selectedFolderHasChildren.value
        ? GROUP.NONE
        : GROUP.FOLDER;
    case SIDEBAR.SMART_ALBUM:
      return Number(getActiveCustomSmartAlbum()?.group?.type ?? GROUP.NONE);
    case SIDEBAR.CALENDAR:
      if (config.calendar.isMonthly) {
        return libConfig.calendar.year !== null && libConfig.calendar.month === -1 ? GROUP.MONTH : GROUP.NONE;
      }
      return libConfig.calendar.year !== null && libConfig.calendar.month !== -1 && libConfig.calendar.date === -1 ? GROUP.DAY : GROUP.NONE;
    case SIDEBAR.TAG:
      return GROUP.NONE; // Tag and smart tag: disabled
    case SIDEBAR.LOCATION:
      return libConfig.location.admin1 && !libConfig.location.name ? GROUP.LOCATION : GROUP.NONE;
    case SIDEBAR.CAMERA:
      if (!config.camera.isCamera) {
        return (libConfig.camera as any).lensMake && !(libConfig.camera as any).lensModel ? GROUP.LENS : GROUP.NONE;
      }
      return libConfig.camera.make && !libConfig.camera.model ? GROUP.CAMERA : GROUP.NONE;
    default:
      return GROUP.NONE;
  }
}

const activeGroupBy = computed(() => getAutoGroupByForCurrentView());

function isGroupingSupportedForCurrentView() {
  return (
    activeGroupBy.value > 0 &&
    !config.settings.grid.showFilmStrip &&
    !isScanStreamingMode.value &&
    tempViewMode.value === 'none' &&
    config.main.sidebarIndex !== SIDEBAR.SEARCH
  );
}

function formatFolderGroupLabel(label: string) {
  const folderPath = String(label || '').trim();
  if (!folderPath || folderPath === 'Unknown folder') return folderPath;

  const root = [...folderGroupRoots.value]
    .filter(root => root.path && isWithinRootPath(folderPath, root.path))
    .sort((a, b) => b.path.length - a.path.length)[0];
  if (!root) return folderPath;

  return formatFolderBreadcrumb(folderPath, root.path, root.name || '');
}

function formatDateGroupLabel(groupBy: number, label: string) {
  const rawLabel = String(label || '').trim();

  const timestamp = Number(rawLabel);
  if (!Number.isFinite(timestamp) || timestamp <= 0) return rawLabel;

  const value = new Date(timestamp * 1000);
  const year = value.getFullYear();
  const month = value.getMonth() + 1;
  const date = value.getDate();
  if (!year || !month || !date) return rawLabel;

  switch (groupBy) {
    case GROUP.DAY:
      return formatDate(year, month, date, localeMsg.value.format.date_long);
    case GROUP.MONTH:
      return formatDate(year, month, 1, localeMsg.value.format.month);
    case GROUP.YEAR: {
      if (config.main.sidebarIndex === SIDEBAR.LIBRARY && libConfig.library.item === LIB_ITEM.TODAY) {
        const today = new Date();
        return formatDate(year, today.getMonth() + 1, today.getDate(), localeMsg.value.format.date_long);
      }
      return formatDate(year, 1, 1, localeMsg.value.format.year);
    }
    default:
      return rawLabel;
  }
}

function ratingLabelKey(rating: number) {
  const keyMap: Record<number, string> = {
    5: 'five_stars',
    4: 'four_stars',
    3: 'three_stars',
    2: 'two_stars',
    1: 'one_star',
  };
  return keyMap[rating] || '';
}

function formatRatingGroupLabel(label: string) {
  const rating = Number(label || 0);
  if (rating === RATE.UNRATED) return localeMsg.value.rating.unrated;
  const key = ratingLabelKey(rating);
  return key ? localeMsg.value.rating[key] : label || '';
}

function formatGroupLabel(label: string) {
  const groupBy = Number(activeGroupBy.value || 0);
  if (groupBy === GROUP.DAY || groupBy === GROUP.MONTH || groupBy === GROUP.YEAR) return formatDateGroupLabel(groupBy, label);
  if (groupBy === GROUP.FOLDER) return formatFolderGroupLabel(label);
  if (groupBy === GROUP.RATING) return formatRatingGroupLabel(label);
  return label || '';
}

function getGroupingQueryParams() {
  const baseParams = currentQuerySource.value === 'smart' && currentSmartQueryParams.value
    ? currentSmartQueryParams.value
    : currentQueryParams.value;
  const calendarSort = config.main.sidebarIndex === SIDEBAR.LIBRARY && libConfig.library.item === LIB_ITEM.TODAY
    ? DATE_SORT.TAKEN_DESC
    : Number(config.settings.calendarSort || 0);
  return {
    ...baseParams,
    groupBy: activeGroupBy.value,
    folderSort: Number(config.settings.folderSort || 0),
    calendarSort,
    categorySort: Number(config.settings.categorySort || 0),
  };
}

async function ensureFolderGroupRoots() {
  if (folderGroupRoots.value.length > 0) return;
  try {
    const albums = await getAllAlbums();
    folderGroupRoots.value = Array.isArray(albums)
      ? albums
          .map((album: any) => ({
            path: String(album?.path || ''),
            name: String(album?.name || ''),
          }))
          .filter(album => album.path)
      : [];
  } catch (error) {
    console.error('ensureFolderGroupRoots error:', error);
    folderGroupRoots.value = [];
  }
}

function updateGroupSelectedCount(groupId: string, count: number) {
  groupSelectedCountMap.value = {
    ...groupSelectedCountMap.value,
    [groupId]: Math.max(0, count),
  };
}

function updateGroupSelectedSize(groupId: string, size: number) {
  groupSelectedSizeMap.value = {
    ...groupSelectedSizeMap.value,
    [groupId]: Math.max(0, size),
  };
}

function syncSelectionVersions() {
  selectionVersion.value++;
  selectedFilesVersion.value++;
}

function applySelectionDelta(file: any, delta: number) {
  if (!isRealFileItem(file) || delta === 0) return;
  selectedCount.value = Math.max(0, selectedCount.value + delta);
  selectedSize.value = Math.max(0, selectedSize.value + delta * Number(file.size || 0));
  updateGroupSelectedCountForFile(file, delta);
  syncSelectionVersions();
}

function updateGroupSelectedCountForFile(file: any, delta: number) {
  const fileId = Number(file?.id || 0);
  if (!fileId) return;
  const groupId = fileIdToGroupId.get(fileId);
  if (!groupId) return;
  const meta = groupMetaMap.get(groupId);
  const current = Number(groupSelectedCountMap.value[groupId] || 0);
  const maxCount = Number(meta?.count || Number.MAX_SAFE_INTEGER);
  updateGroupSelectedCount(groupId, Math.min(maxCount, Math.max(0, current + delta)));

  const currentSize = Number(groupSelectedSizeMap.value[groupId] || 0);
  const fileSize = Number(file?.size || 0);
  const maxSize = Number(meta?.size || Number.MAX_SAFE_INTEGER);
  updateGroupSelectedSize(groupId, Math.min(maxSize, Math.max(0, currentSize + delta * fileSize)));
}

function resetSelectionSummary() {
  selectedCount.value = 0;
  selectedSize.value = 0;
  selectedFileIds.clear();
  syncSelectionVersions();
}

function recomputeLoadedGroupSelectedCounts() {
  const next: Record<string, number> = {};
  const nextSize: Record<string, number> = {};
  for (const groupId of groupMetaMap.keys()) {
    next[groupId] = 0;
    nextSize[groupId] = 0;
  }

  for (const file of fileList.value) {
    if (!isRealFileItem(file)) continue;
    const fileId = Number(file.id || 0);
    const groupId = fileIdToGroupId.get(fileId);
    if (!groupId) continue;
    if (file.isSelected || selectedFileIds.has(fileId)) {
      const maxCount = Number(groupMetaMap.get(groupId)?.count || Number.MAX_SAFE_INTEGER);
      next[groupId] = Math.min(maxCount, Number(next[groupId] || 0) + 1);
      const maxSize = Number(groupMetaMap.get(groupId)?.size || Number.MAX_SAFE_INTEGER);
      nextSize[groupId] = Math.min(maxSize, Number(nextSize[groupId] || 0) + Number(file.size || 0));
    }
  }

  groupSelectedCountMap.value = next;
  groupSelectedSizeMap.value = nextSize;
}

function normalizeGroupedRowsResult(result: any) {
  const rows = Array.isArray(result) ? result : (Array.isArray(result?.rows) ? result.rows : []);
  const groups = Array.isArray(result?.groups)
    ? result.groups.map((group: any) => ({
        groupId: String(group.group_id ?? group.groupId ?? ''),
        label: formatGroupLabel(group.label ?? ''),
        count: Number(group.count || 0),
        size: Number(group.size || 0),
        rowIndex: Number(group.row_index ?? group.rowIndex ?? 0),
      }))
    : [];
  return {
    rows,
    groups,
    totalItemCount: Number(result?.totalItemCount ?? result?.total_item_count ?? result?.totalFileCount ?? result?.total_count ?? totalFileCount.value ?? 0),
    totalRowCount: Number(result?.total_row_count ?? result?.totalRowCount ?? rows.length),
    totalSize: Number(result?.total_size ?? result?.totalSize ?? totalFileSize.value ?? 0),
  };
}

function ensureGroupedFileAtIndex(index: number) {
  if (!groupedModeActive.value || isRealFileItem(fileList.value[index])) return true;
  const row = groupedRows.value.find((item: any) =>
    isItemRow(item) && Number(item.file_index) === Number(index)
  );
  const file = row?.file;
  if (!file) return false;

  const fileId = Number(file?.id || 0);
  const existingItem = fileList.value[index];
  fileList.value[index] = {
    ...file,
    isSelected: Boolean(existingItem?.isSelected) || (fileId > 0 && selectedFileIds.has(fileId)),
    rotate: existingItem?.rotate || file.rotate || 0,
    thumbnail: existingItem?.thumbnail || file.thumbnail,
  };
  const groupId = String(row.group_id || '');
  if (groupId && fileId) fileIdToGroupId.set(fileId, groupId);
  if (fileList.value[index]?.isSelected && fileId > 0 && !selectedFileIds.has(fileId)) {
    selectedFileIds.add(fileId);
    applySelectionDelta(fileList.value[index], 1);
  }
  return true;
}

function createGroupHeaderRow(group: any) {
  const groupId = String(group.groupId || '');
  return {
    type: 'group',
    id: `group-row-${groupId}`,
    group_id: groupId,
    label: formatGroupLabel(group.label || ''),
    count: Number(group.count || 0),
    size: Number(group.size || 0),
  };
}

function getGroupedRowIndexForFileIndex(fileIndex: number) {
  if (!groupedModeActive.value || fileIndex < 0) return fileIndex;
  let fileCursor = 0;
  for (const group of groupedTimelineGroups.value) {
    const count = Number(group.count || 0);
    if (fileIndex < fileCursor + count) {
      return Number(group.rowIndex || 0) + 1 + (fileIndex - fileCursor);
    }
    fileCursor += count;
  }
  const row = groupedRows.value.findIndex((item: any) =>
    isItemRow(item) && Number(item.file_index) === Number(fileIndex)
  );
  return row >= 0 ? row : -1;
}

function getGroupedFileIndexForRowIndex(rowIndex: number) {
  if (!groupedModeActive.value || rowIndex < 0) return rowIndex;
  let fileCursor = 0;
  for (const group of groupedTimelineGroups.value) {
    const count = Number(group.count || 0);
    const groupRowIndex = Number(group.rowIndex || 0);
    const firstItemRowIndex = groupRowIndex + 1;
    const nextGroupRowIndex = firstItemRowIndex + count;
    if (rowIndex >= groupRowIndex && rowIndex < nextGroupRowIndex) {
      return count > 0
        ? fileCursor + Math.max(0, rowIndex - firstItemRowIndex)
        : -1;
    }
    fileCursor += count;
  }

  const row = groupedRows.value[rowIndex];
  if (isItemRow(row)) return Number(row.file_index);
  if (isGroupRow(row)) {
    const nextRow = groupedRows.value[rowIndex + 1];
    return isItemRow(nextRow) ? Number(nextRow.file_index) : -1;
  }
  return -1;
}

class CopyIndexError extends Error {}

async function confirmLargeBatch(count: number) {
  if (count <= LARGE_BATCH_CONFIRM_THRESHOLD) return true;
  return ask(
    t('info_panel.large_batch.content', { count: count.toLocaleString() }),
    {
      title: t('info_panel.large_batch.title'),
      kind: 'warning',
      okLabel: t('info_panel.large_batch.ok'),
      cancelLabel: t('msgbox.cancel'),
    },
  );
}

async function runWithConcurrency<T, R>(
  items: T[],
  worker: (item: T) => Promise<R>,
  concurrency = FILE_OPERATION_CONCURRENCY,
): Promise<PromiseSettledResult<R>[]> {
  const results: PromiseSettledResult<R>[] = new Array(items.length);
  let nextIndex = 0;
  const workerCount = Math.min(Math.max(1, concurrency), items.length);

  await Promise.all(Array.from({ length: workerCount }, async () => {
    while (nextIndex < items.length) {
      const index = nextIndex++;
      try {
        results[index] = { status: 'fulfilled', value: await worker(items[index]) };
      } catch (reason) {
        results[index] = { status: 'rejected', reason };
      }
    }
  }));
  return results;
}

async function runWithKeyedConcurrency<T, R>(
  items: T[],
  getKey: (item: T) => string,
  worker: (item: T) => Promise<R>,
  concurrency = FILE_OPERATION_CONCURRENCY,
): Promise<PromiseSettledResult<R>[]> {
  const groups = new Map<string, Array<{ item: T; index: number }>>();
  items.forEach((item, index) => {
    const key = getKey(item);
    const group = groups.get(key);
    if (group) {
      group.push({ item, index });
    } else {
      groups.set(key, [{ item, index }]);
    }
  });

  const results: PromiseSettledResult<R>[] = new Array(items.length);
  await runWithConcurrency(Array.from(groups.values()), async group => {
    for (const { item, index } of group) {
      try {
        results[index] = { status: 'fulfilled', value: await worker(item) };
      } catch (reason) {
        results[index] = { status: 'rejected', reason };
      }
    }
  }, concurrency);
  return results;
}

// quick view
const showQuickView = ref(false);
const quickViewMediaRef = ref<any>(null);
const quickViewZoomFit = ref(true);
const quickViewHoverLeft = ref(false);
const quickViewHoverRight = ref(false);

function handleQuickViewMouseMove(event: MouseEvent) {
  const target = event.currentTarget as HTMLElement | null;
  if (!target) return;
  const rect = target.getBoundingClientRect();
  if (rect.width <= 0 || rect.height <= 0) return;
  const x = event.clientX - rect.left;
  quickViewHoverLeft.value = x >= 0 && x < rect.width * 0.1;
  quickViewHoverRight.value = x <= rect.width && x > rect.width * 0.9;
}

function handleQuickViewMouseLeave() {
  quickViewHoverLeft.value = false;
  quickViewHoverRight.value = false;
}

function getActivePreviewMode(): 'quick-view' | 'filmstrip' | 'none' {
  if (showQuickView.value) return 'quick-view';
  if (config.settings.grid.showFilmStrip) return 'filmstrip';
  return 'none';
}

function getActivePreviewMediaRef() {
  if (showQuickView.value) return quickViewMediaRef.value;
  if (config.settings.grid.showFilmStrip) return filmStripMediaRef.value;
  return null;
}

function getCurrentPreviewImageSrc() {
  const viewer = getActivePreviewMediaRef();
  return viewer?.getCurrentImageSrc?.() || '';
}

function clearPreviewPreloadCache(filePath?: string) {
  const viewer = getActivePreviewMediaRef();
  viewer?.clearPreloadCache?.(filePath);
}

// film strip view
const filmStripMediaRef = ref<any>(null);
const filmStripZoomFit = ref(true);

function closeQuickPreview() {
  showQuickView.value = false;
  stopSlideShow();
}

// toolbar state for MediaViewer
const imageScale = ref(1);
const imageDisplayScale = ref(1);
const imageMinScale = ref(0);
const imageMaxScale = ref(10);
const isSlideShow = ref(false);

function getNextImagePath(index: number): string {
  const target = fileList.value[index + 1];
  return target && !target.isPlaceholder && target.file_type === 1 ? target.file_path : '';
}

// Request ID tracking to prevent race conditions during async content updates
let currentContentRequestId = 0;

const onScale = (event: any) => {
  imageScale.value = event.scale;
  imageDisplayScale.value = event.displayScale ?? event.scale;
  imageMinScale.value = event.minScale;
  imageMaxScale.value = event.maxScale;
};

const videoRef = ref<HTMLVideoElement | null>(null);             // preview video reference

// info panel splitter
const isDraggingInfoPanel = ref(false);
const rightPanelDragStartX = ref(0);
const rightPanelDragStartWidth = ref(0);

// message box
const showRenameMsgbox = ref(false);  // show rename message box
const renamingFileName = ref<{name?: string, ext?: string}>({}); // extract the file name to {name, ext}

const showMoveTo = ref(false);
type FileConflictPolicy = 'skip' | 'keep_both' | 'replace';
const fileConflictDialog = ref({
  show: false,
  name: '',
  destination: '',
  showApplyAll: false,
  allowReplace: true,
});
let fileConflictResolver: ((result: { policy: FileConflictPolicy; applyAll: boolean }) => void) | null = null;
const showTrashMsgbox = ref(false);
const showExternalOpenWarningMsgbox = ref(false);
const pendingExternalOpen = ref<{ paths: string[]; appPath: string } | null>(null);
const permanentDeleteChecked = ref(false);
const deletePermanently = ref(false);
const dedupReclaimBytes = ref(0);
const dedupTrashGroupKey = ref('');
const dedupDeleteFileIds = ref<number[]>([]);
const dedupPaneRef = ref<InstanceType<typeof DedupPane> | null>(null);
const dedupStatuses = ref<Record<number, 'keep' | 'dup'>>({});
const showCommentMsgbox = ref(false);
const commentInputText = computed(() => {
  if (selectMode.value) {
    const selectedItems = getActionableSelectedItems();
    if (selectedItems.length === 0) return '';
    const firstComment = selectedItems[0]?.comments ?? '';
    return selectedItems.every(item => (item.comments ?? '') === firstComment) ? firstComment : '';
  }
  return fileList.value[selectedItemIndex.value]?.comments ?? '';
});
const errorMessage = ref('');

// Unsaved changes confirmation
const showUnsavedChangesMsgbox = ref(false);
const pendingAction = ref<(() => void) | null>(null);
const fileInfoRef = ref<any>(null);
const isDedupPanelOpen = computed(() => config.rightPanel.show && config.rightPanel.mode === 'dedup');
const isInfoPanelOpen = computed(() => config.rightPanel.show && config.rightPanel.mode === 'info');
const RIGHT_PANEL_MIN_WIDTH = 160; // Keep aligned with left panel minimum width.
const RIGHT_PANEL_ANIMATION_MS = 200;
const activeRightPanelWidth = computed(() => Number(config.rightPanel.width || 360));
const shouldShowRightPanel = computed(() => config.rightPanel.show || selectMode.value);
const rightPanelMounted = ref(shouldShowRightPanel.value);
const rightPanelVisualVisible = ref(shouldShowRightPanel.value);
const rightPanelLayoutVisible = ref(shouldShowRightPanel.value);
let rightPanelAnimationTimer: ReturnType<typeof setTimeout> | null = null;
let rightPanelAnimationVersion = 0;

async function refreshCenteredGridLayout() {
  gridViewRef.value?.refreshLayout?.();
  await nextTick();
  gridViewRef.value?.centerItem?.(selectedItemIndex.value);
}

async function commitRightPanelLayout(visible: boolean) {
  rightPanelLayoutVisible.value = visible;
  await nextTick();
  await refreshCenteredGridLayout();
}

function clearRightPanelAnimationTimer() {
  if (rightPanelAnimationTimer) {
    clearTimeout(rightPanelAnimationTimer);
    rightPanelAnimationTimer = null;
  }
}

watch(shouldShowRightPanel, async (visible) => {
  clearRightPanelAnimationTimer();
  const animationVersion = ++rightPanelAnimationVersion;

  if (visible) {
    rightPanelMounted.value = true;
    await nextTick();
    if (animationVersion !== rightPanelAnimationVersion) return;
    rightPanelVisualVisible.value = true;
    rightPanelAnimationTimer = setTimeout(() => {
      if (animationVersion !== rightPanelAnimationVersion) return;
      rightPanelAnimationTimer = null;
      void commitRightPanelLayout(true);
    }, RIGHT_PANEL_ANIMATION_MS);
    return;
  }

  rightPanelVisualVisible.value = false;
  rightPanelAnimationTimer = setTimeout(() => {
    if (animationVersion !== rightPanelAnimationVersion) return;
    rightPanelAnimationTimer = null;
    void commitRightPanelLayout(false).then(() => {
      if (animationVersion !== rightPanelAnimationVersion) return;
      rightPanelMounted.value = false;
    });
  }, RIGHT_PANEL_ANIMATION_MS);
});

function getRightPanelMaxWidth() {
  const mainWindowWidth = window.innerWidth;
  return Math.max(RIGHT_PANEL_MIN_WIDTH, Math.floor(mainWindowWidth / 3));
}

function clampRightPanelWidth(width: number) {
  return Math.round(Math.min(Math.max(width, RIGHT_PANEL_MIN_WIDTH), getRightPanelMaxWidth()));
}

function migrateRightPanelWidthToPixels() {
  if (!contentViewDiv.value) return;
  const layoutWidth = contentViewDiv.value.clientWidth;
  if (layoutWidth <= 0) return;

  const rawWidth = Number(config.rightPanel.width || 0);
  if (rawWidth <= 0) {
    config.rightPanel.width = clampRightPanelWidth(layoutWidth * 0.3);
    return;
  }

  // Backward compatibility: old config persisted width in percent (20-80).
  if (rawWidth <= 100) {
    config.rightPanel.width = clampRightPanelWidth((rawWidth / 100) * layoutWidth);
    return;
  }

  config.rightPanel.width = clampRightPanelWidth(rawWidth);
}

function handleWindowResize() {
  config.rightPanel.width = clampRightPanelWidth(Number(config.rightPanel.width || 360));
}

const confirmSave = async () => {
  showUnsavedChangesMsgbox.value = false;
  if (fileInfoRef.value) {
    const success = await fileInfoRef.value.quickSave();
    if (success && pendingAction.value) {
      pendingAction.value();
      pendingAction.value = null;
    }
  }
};

const confirmDiscard = () => {
  showUnsavedChangesMsgbox.value = false;
  // Clear active adjustments in store to "discard" the changes
  uiStore.clearActiveAdjustments();
  if (pendingAction.value) {
    pendingAction.value();
    pendingAction.value = null;
  }
};

const cancelDiscard = () => {
  showUnsavedChangesMsgbox.value = false;
  pendingAction.value = null;
};

// Check if current file has unsaved changes
const hasUnsavedChanges = computed(() => {
  if (!isInfoPanelOpen.value) return false;
  const currentFile = fileList.value[selectedItemIndex.value];
  return uiStore.hasActiveChanges(currentFile);
});

const checkUnsavedChanges = (action: () => void) => {
  if (hasUnsavedChanges.value) {
    pendingAction.value = action;
    showUnsavedChangesMsgbox.value = true;
  } else {
    action();
  }
};

async function removeFileFromCurrentCollection(index: number) {
  const file = fileList.value[index];
  if (!file || !currentCollectionId.value) return;
  await removeFilesFromCollection(currentCollectionId.value, [file.id]);
  fileList.value.splice(index, 1);
  totalFileCount.value = fileList.value.length;
  totalFileSize.value -= file.size || 0;
  if (selectedItemIndex.value >= fileList.value.length) {
    selectedItemIndex.value = fileList.value.length - 1;
  }
  void tauriEmit('collection-files-dropped', { collectionId: currentCollectionId.value });
  toast.success(t('collection.removed_toast', { count: 1 }));
}

async function removeSelectedFromCollection() {
  if (!selectMode.value || selectedCount.value === 0 || !currentCollectionId.value) return;
  const selectedItems = getActionableSelectedItems();
  const fileIds = selectedItems.map((item: any) => Number(item.id)).filter((id: number) => id > 0);
  if (fileIds.length === 0) return;
  await removeFilesFromCollection(currentCollectionId.value, fileIds);
  const removedIdSet = new Set(fileIds);
  fileList.value = fileList.value.filter(f => !removedIdSet.has(f.id));
  totalFileCount.value = fileList.value.length;
  totalFileSize.value = fileList.value.reduce((total, file) => total + file.size, 0);
  selectedItemIndex.value = fileList.value.length > 0 ? Math.min(selectedItemIndex.value, fileList.value.length - 1) : -1;
  selectedCount.value = Math.max(0, selectedCount.value - fileIds.length);
  selectedSize.value = Math.max(0, selectedSize.value - selectedItems.reduce((total, item) => total + Number(item.size || 0), 0));
  void tauriEmit('collection-files-dropped', { collectionId: currentCollectionId.value });
  toast.success(t('collection.removed_toast', { count: fileIds.length }));
}

const openTrashMsgbox = (reclaimBytes = 0, groupKey = '', fileIds: number[] = []) => {
  dedupReclaimBytes.value = Math.max(0, reclaimBytes);
  dedupTrashGroupKey.value = groupKey || '';
  dedupDeleteFileIds.value = Array.isArray(fileIds) ? [...new Set(fileIds)] : [];
  deletePermanently.value = permanentDeleteChecked.value;
  showTrashMsgbox.value = true;
};

const closeTrashMsgbox = () => {
  showTrashMsgbox.value = false;
  dedupReclaimBytes.value = 0;
  dedupTrashGroupKey.value = '';
  dedupDeleteFileIds.value = [];
};

const isDedupTrash = computed(() => dedupDeleteFileIds.value.length > 0);

const trashMsgboxTitle = computed(() => {
  if (deletePermanently.value) {
    return localeMsg.value.msgbox.permanent_delete.title;
  }
  return localeMsg.value.msgbox.move_to_trash.title;
});

const trashMsgboxOkText = computed(() => {
  if (deletePermanently.value) {
    return localeMsg.value.msgbox.permanent_delete.ok;
  }
  return localeMsg.value.msgbox.move_to_trash.ok;
});

const trashMsgboxMessage = computed(() => {
  const deleteCount = isDedupTrash.value ? dedupDeleteFileIds.value.length : selectedCount.value;
  const base = deletePermanently.value
    ? ((isDedupTrash.value || selectMode.value)
        ? localeMsg.value.msgbox.permanent_delete.files_content.replace('{count}', deleteCount.toLocaleString())
        : localeMsg.value.msgbox.permanent_delete.file_content.replace('{file}', fileList.value[selectedItemIndex.value]?.name || ''))
    : ((isDedupTrash.value || selectMode.value)
        ? localeMsg.value.msgbox.move_to_trash.files_content.replace('{count}', deleteCount.toLocaleString())
        : localeMsg.value.msgbox.move_to_trash.file_content.replace('{file}', fileList.value[selectedItemIndex.value]?.name || ''));
  if (dedupReclaimBytes.value <= 0 || !(isDedupTrash.value || selectMode.value)) return base;
  return `${base}\n${localeMsg.value.info_panel.dedup.reclaimable_size}: ${formatFileSize(dedupReclaimBytes.value)}`;
});

// tagging dialog
const showTaggingDialog = ref(false);
const fileIdsToTag = ref<number[]>([]);

// grid view
const gridScrollContainerRef = ref<HTMLElement | null>(null);
const gridViewRef = ref<any>(null);

const scrollPosition = ref(0);    // scrollbar position (item index)

const containerHeight = ref(0);   // container height
const containerWidth = ref(0);    // container width
const layoutContentHeight = ref(0); // reported content height from GridView
const layoutVersion = ref(0);     // version to force layout update
let layoutRefreshTimer: ReturnType<typeof setTimeout> | null = null;
const isGeometryGridStyle = computed(() => config.settings.grid.style === 2 || config.settings.grid.style === 3);
const usesGeometryNavigation = computed(() =>
  config.settings.grid.style === 2 ||
  (!config.settings.grid.showFilmStrip && config.settings.grid.style === 3)
);

function scheduleLayoutRefresh() {
  if (!isGeometryGridStyle.value) return;
  if (layoutRefreshTimer) return;
  layoutRefreshTimer = setTimeout(() => {
    layoutVersion.value++;
    layoutRefreshTimer = null;
  }, 120);
}
const gap = 8;                    // Gap between items (must match GridView)

const itemWidth = computed(() => {
  if (config.settings.grid.style === 0) {
    return config.settings.grid.size + 20; // size + padding/border/gap(20)
  } else if (config.settings.grid.style === 1) {
    return config.settings.grid.size;
  } else if (isGeometryGridStyle.value) {
    return config.settings.grid.size; // Approximation for geometry layouts
  }
  return 0;
});

const itemSize = computed(() => {
  if (config.settings.grid.style === 0) {
    let labelHeight = 0
    if (config.settings.grid.labelPrimary > 0 ) labelHeight += 16;      // height of text-sm
    if (config.settings.grid.labelSecondary > 0 ) labelHeight += 16;    // height of text-xs
    return config.settings.grid.size + 20 + labelHeight; // size + padding/border/gap(20) + labels
  } else if (config.settings.grid.style === 1) {
    return itemWidth.value + gap / 2;
  } else if (isGeometryGridStyle.value) {
    return config.settings.grid.size;
  }
  return 0;
});

const columnCount = computed(() => {
  if (containerWidth.value <= 0 || itemWidth.value <= 0) return 4;
  return Math.max(1, Math.floor(containerWidth.value / itemWidth.value));
});

const visibleItemCount = computed(() => {
  if (containerHeight.value <= 0 || itemSize.value <= 0) return 20;
  const rows = Math.floor(containerHeight.value / itemSize.value);
  return rows * columnCount.value;
});

const timelineData = ref<any[]>([]);  // timeline markers for scrollbar
const groupTimelineData = computed(() => {
  if (!groupedModeActive.value) return [];
  if (groupedTimelineGroups.value.length > 0) {
    return groupedTimelineGroups.value.map((group: any) => ({
      label: group.label || '',
      position: Number(group.rowIndex || 0),
    }));
  }
  return groupedRows.value
    .map((row: any, index: number) => isGroupRow(row) && !row.isPlaceholder
      ? {
          label: row.label || '',
          position: index,
        }
      : null)
    .filter(Boolean);
});
const scrollbarTotal = computed(() => groupedModeActive.value ? totalRowCount.value : totalFileCount.value);
const scrollbarPageSize = computed(() => Math.max(1, Math.min(visibleItemCount.value, scrollbarTotal.value || visibleItemCount.value)));
const scrollbarMarkers = computed(() => {
  if (groupedModeActive.value) return groupTimelineData.value;
  if (isSearchLikeView.value) return [];
  return timelineData.value;
});
const scrollbarSelectedIndex = computed(() => (
  groupedModeActive.value
    ? getGroupedRowIndexForFileIndex(selectedItemIndex.value)
    : selectedItemIndex.value
));

const toast = useToast();
const shortcutPlatform: ShortcutPlatform = isMac ? 'mac' : (isLinux ? 'linux' : 'windows');
const pendingFolderSyncs = new Map<number, Promise<any>>();
let lastSyncedAlbumFolder = '';
const shortcut = (actionId: ShortcutActionId) => getShortcutLabel(actionId, shortcutPlatform);
const ratingActions: Array<{ actionId: ShortcutActionId; rating: number }> = [
  { actionId: 'meta.rating.clear', rating: 0 },
  { actionId: 'meta.rating.one', rating: 1 },
  { actionId: 'meta.rating.two', rating: 2 },
  { actionId: 'meta.rating.three', rating: 3 },
  { actionId: 'meta.rating.four', rating: 4 },
  { actionId: 'meta.rating.five', rating: 5 },
];

function getMatchedRating(event: KeyboardEvent) {
  const match = ratingActions.find(({ actionId }) => matchesShortcut(actionId, event, shortcutPlatform));
  return match ? match.rating : null;
}

// Drag-drop file import
const isDragOver = ref(false);
const dragOverCount = ref(0);
const showDropWarning = ref(false);
const isContentInternalDrag = ref(false);
const isPastingClipboard = ref(false);
const acceptDrops = computed(() =>
  tempViewMode.value === 'none'
  && config.main.sidebarIndex === SIDEBAR.ALBUM
  && libConfig.album.id > 0
);

// DOM drop handler references (Windows/Linux — for cleanup in onBeforeUnmount)
let domDragEnter: ((e: DragEvent) => void) | null = null;
let domDragLeave: ((e: DragEvent) => void) | null = null;
let domDragOver: ((e: DragEvent) => void) | null = null;
let domDrop: ((e: DragEvent) => void) | null = null;
let dragGhost: HTMLElement | null = null;
let dragGhostAction: HTMLElement | null = null;
let pointerDropTarget: HTMLElement | null = null;
let pointerDragUsesSelection = false;
let pointerDragFiles: Array<{
  id: number;
  file_path: string;
  folder_id: number;
  album_id: number;
}> | null = null;
let dragGhostHotspotX = 0;
let dragGhostHotspotY = 0;

function getExternalDropUris(dt: DataTransfer | null) {
  const value = dt?.getData('text/uri-list')
    || dt?.getData('text/x-moz-url')
    || dt?.getData('text/plain')
    || '';
  return value
    .split(/\r?\n/)
    .map(uri => uri.trim())
    .filter(uri => uri.length > 0 && !uri.startsWith('#'));
}

function getExternalHttpDropUrls(uris: string[]) {
  return uris
    .filter(uri => uri.startsWith('http://') || uri.startsWith('https://'));
}

function fileUrlToPath(uri: string) {
  if (!uri.startsWith('file://')) return null;
  try {
    const url = new URL(uri);
    let path = decodeURIComponent(url.pathname);
    if (/^\/[A-Za-z]:\//.test(path)) {
      path = path.slice(1);
    } else if (url.hostname && url.hostname !== 'localhost') {
      path = `//${url.hostname}${path}`;
    }
    return path || null;
  } catch {
    return null;
  }
}

function getExternalFileDropPaths(uris: string[]) {
  return uris
    .map(fileUrlToPath)
    .filter((path): path is string => !!path);
}

function hasExternalDomDrop(event: DragEvent) {
  return hasExternalDragIntent(event);
}

function hasExternalDragIntent(event: DragEvent) {
  if (isContentInternalDrag.value) return false;
  const dt = event.dataTransfer;
  if (!dt) return false;
  const types = Array.from(dt.types || []);
  return dt.files.length > 0
    || types.includes('Files')
    || types.includes('text/uri-list')
    || types.includes('text/x-moz-url')
    || types.includes('text/plain');
}

function isInternalReorderActive() {
  return uiStore.isInputActive('ManageLibraries')
    || uiStore.isInputActive('AlbumListDrag');
}

async function resolveAlbumImportDestination(albumId: number, folderPath?: string) {
  if (albumId <= 0) return null;
  const album = await getAlbum(albumId);
  const destinationPath = folderPath || album?.path;
  if (!destinationPath) return null;

  const folder = await selectFolder(albumId, destinationPath);
  return folder?.id
    ? { albumId, folderId: Number(folder.id), folderPath: String(folder.path) }
    : null;
}

async function resolveCurrentAlbumImportDestination() {
  const albumId = Number(libConfig.album.id || 0);
  if (config.main.sidebarIndex !== SIDEBAR.ALBUM || albumId <= 0) return null;

  const folderPath = !libConfig.album.selected && libConfig.album.folderPath
    ? libConfig.album.folderPath
    : undefined;
  return resolveAlbumImportDestination(albumId, folderPath);
}

async function refreshImportedFiles(albumId: number) {
  await refreshAffectedAlbums([albumId]);
  await refreshLibraryTotalCount();
  await updateContent();
}

async function pasteClipboardImage(target?: { albumId: number; folderPath?: string }) {
  if (isPastingClipboard.value) return;
  if (!target && !acceptDrops.value) {
    showDropWarning.value = true;
    return;
  }

  const destination = target
    ? await resolveAlbumImportDestination(Number(target.albumId || 0), target.folderPath)
    : await resolveCurrentAlbumImportDestination();
  if (!destination) {
    showDropWarning.value = true;
    return;
  }

  isPastingClipboard.value = true;
  try {
    const files = await importClipboard(destination.folderId, destination.folderPath);
    if (!Array.isArray(files) || files.length === 0) {
      toast.warning(t('msgbox.drop_import.no_clipboard_image'));
      return;
    }
    await refreshImportedFiles(destination.albumId);
    toast.success(t('msgbox.drop_import.success', { count: files.length }));
  } catch (error) {
    console.error('Failed to import clipboard image:', error);
    toast.warning(t('msgbox.drop_import.no_clipboard_image'));
  } finally {
    isPastingClipboard.value = false;
  }
}

function removeDragGhost() {
  if (dragGhostAction?.firstElementChild) {
    render(null, dragGhostAction.firstElementChild as HTMLElement);
  }
  dragGhost?.remove();
  dragGhost = null;
  dragGhostAction = null;
  setPointerDropTarget(null);
  document.removeEventListener('pointermove', updateContentDragPosition);
  document.removeEventListener('keydown', updateDragGhostModifier);
  document.removeEventListener('keyup', updateDragGhostModifier);
}

function setPointerDropTarget(target: HTMLElement | null) {
  if (pointerDropTarget === target) return;
  pointerDropTarget?.classList.remove('!bg-primary/10', '!border-primary/60');
  pointerDropTarget = target;
  pointerDropTarget?.classList.add('!bg-primary/10', '!border-primary/60');
}

function isCopyDragModifier(event: Pick<MouseEvent, 'altKey' | 'ctrlKey'>) {
  return isMac ? event.altKey : event.ctrlKey;
}

function updateDragGhostAction(event: Pick<MouseEvent, 'altKey' | 'ctrlKey'>) {
  if (!dragGhostAction) return;
  const copy = isCopyDragModifier(event);
  const showLabel = !!pointerDropTarget;
  const isCollectionDropTarget = !!pointerDropTarget?.dataset.collectionDropId || pointerDropTarget?.dataset.collectionDropNew === 'true';
  const icon = dragGhostAction.firstElementChild as HTMLElement | null;
  const label = dragGhostAction.lastElementChild as HTMLElement | null;
  if (icon) {
    render(createVNode(isCollectionDropTarget || (showLabel && copy) ? IconAdd : IconPrev, {
      class: 'w-4 h-4',
    }), icon);
  }
  if (label) {
    label.textContent = showLabel
      ? (isCollectionDropTarget
        ? t('collection.drop_action')
        : copy
        ? t('info_panel.drag_action.copy')
        : t('info_panel.drag_action.move'))
      : '';
    label.style.display = showLabel ? 'inline' : 'none';
  }
  dragGhostAction.style.padding = showLabel ? '0 9px 0 6px' : '0 5px';
  dragGhostAction.style.gap = showLabel ? '5px' : '0';
  dragGhostAction.style.background = showLabel && (copy || isCollectionDropTarget)
    ? 'var(--color-success)'
    : 'color-mix(in oklab, var(--color-base-300) 94%, transparent)';
  dragGhostAction.style.color = showLabel && (copy || isCollectionDropTarget)
    ? 'var(--color-success-content)'
    : 'var(--color-base-content)';
}

function updateDragGhostModifier(event: KeyboardEvent) {
  updateDragGhostAction(event);
}

function sanitizeDragGhostClone(element: HTMLElement) {
  const clone = element.cloneNode(true) as HTMLElement;
  clone.removeAttribute('id');
  clone.querySelectorAll('[id]').forEach(child => child.removeAttribute('id'));
  clone.querySelectorAll('button, input, [role="button"]').forEach(child => child.remove());
  clone.querySelectorAll('video').forEach(video => {
    video.muted = true;
    video.defaultMuted = true;
    video.volume = 0;
    video.pause();
    video.removeAttribute('autoplay');
  });
  clone.style.pointerEvents = 'none';
  clone.style.margin = '0';
  clone.style.width = `${element.getBoundingClientRect().width}px`;
  clone.style.height = `${element.getBoundingClientRect().height}px`;
  return clone;
}

function createDragGhost(
  draggedElement: HTMLElement,
  draggedFile: any,
  files: any[],
  fileCount = files.length,
  hotspot = { xRatio: 0.5, yRatio: 0.5 },
) {
  removeDragGhost();

  const firstFile = files[0] || draggedFile;
  const firstIndex = fileList.value.findIndex(file => Number(file?.id) === Number(firstFile?.id));
  const renderedFirst = firstIndex >= 0
    ? document.getElementById(`item-${firstIndex}`)
    : null;
  const sourceElement = renderedFirst || draggedElement;
  const front = sanitizeDragGhostClone(sourceElement);

  if (!renderedFirst && firstFile?.thumbnail) {
    const image = front.querySelector('img');
    if (image) image.src = firstFile.thumbnail;
  }

  const sourceRect = sourceElement.getBoundingClientRect();
  const scale = Math.min(1, 200 / Math.max(sourceRect.width, sourceRect.height, 1));
  const width = Math.max(1, Math.round(sourceRect.width * scale));
  const height = Math.max(1, Math.round(sourceRect.height * scale));
  dragGhostHotspotX = width * hotspot.xRatio;
  dragGhostHotspotY = height * hotspot.yRatio;
  const thumbnailElement = sourceElement.querySelector('.rounded-box') as HTMLElement | null;
  const computedRadius = Number.parseFloat(
    getComputedStyle(thumbnailElement || sourceElement).borderTopLeftRadius,
  );
  const radius = Math.max(6, Math.round((Number.isFinite(computedRadius) ? computedRadius : 8) * scale));
  const clipPath = `inset(0 round ${radius}px)`;
  front.style.width = `${sourceRect.width}px`;
  front.style.height = `${sourceRect.height}px`;
  front.style.transform = `scale(${scale})`;
  front.style.transformOrigin = 'top left';
  front.style.borderRadius = `${radius / scale}px`;
  front.style.overflow = 'hidden';
  front.querySelectorAll('img, video').forEach(media => {
    (media as HTMLElement).style.borderRadius = `${radius / scale}px`;
  });

  const ghost = document.createElement('div');
  ghost.style.position = 'fixed';
  ghost.style.left = '0';
  ghost.style.top = '0';
  ghost.style.width = `${width + (fileCount > 1 ? 12 : 0)}px`;
  ghost.style.height = `${height + (fileCount > 1 ? 12 : 0)}px`;
  ghost.style.pointerEvents = 'none';
  ghost.style.zIndex = '2147483647';
  ghost.style.willChange = 'transform';

  if (fileCount > 1) {
    for (const offset of [12, 6]) {
      const layer = document.createElement('div');
      layer.style.position = 'absolute';
      layer.style.inset = `${offset}px 0 0 ${offset}px`;
      layer.style.width = `${width}px`;
      layer.style.height = `${height}px`;
      layer.style.borderRadius = `${radius}px`;
      layer.style.clipPath = clipPath;
      layer.style.background = 'color-mix(in oklab, var(--color-base-200) 92%, transparent)';
      layer.style.border = '1px solid color-mix(in oklab, var(--color-base-content) 25%, transparent)';
      layer.style.boxShadow = '0 4px 12px rgb(0 0 0 / 0.2)';
      ghost.appendChild(layer);
    }
  }

  const frontWrapper = document.createElement('div');
  frontWrapper.style.position = 'absolute';
  frontWrapper.style.left = '0';
  frontWrapper.style.top = '0';
  frontWrapper.style.width = `${width}px`;
  frontWrapper.style.height = `${height}px`;
  frontWrapper.style.overflow = 'hidden';
  frontWrapper.style.borderRadius = `${radius}px`;
  frontWrapper.style.clipPath = clipPath;
  frontWrapper.style.boxShadow = '0 8px 20px rgb(0 0 0 / 0.3)';
  frontWrapper.appendChild(front);
  ghost.appendChild(frontWrapper);

  if (fileCount > 1) {
    const badge = document.createElement('div');
    badge.textContent = fileCount.toLocaleString();
    badge.style.position = 'absolute';
    badge.style.right = '0';
    badge.style.bottom = '0';
    badge.style.minWidth = '24px';
    badge.style.height = '24px';
    badge.style.padding = '0 7px';
    badge.style.display = 'flex';
    badge.style.alignItems = 'center';
    badge.style.justifyContent = 'center';
    badge.style.borderRadius = '9999px';
    badge.style.background = 'var(--color-primary)';
    badge.style.color = 'var(--color-primary-content)';
    badge.style.border = '2px solid var(--color-base-100)';
    badge.style.fontSize = '12px';
    badge.style.fontWeight = '700';
    badge.style.lineHeight = '1';
    badge.style.boxShadow = '0 3px 10px rgb(0 0 0 / 0.35)';
    ghost.appendChild(badge);
  }

  const action = document.createElement('div');
  action.style.position = 'absolute';
  action.style.left = `${width / 2}px`;
  action.style.top = `${height / 2}px`;
  action.style.transform = 'translate(-50%, -50%)';
  action.style.height = '24px';
  action.style.padding = '0 9px 0 6px';
  action.style.display = 'inline-flex';
  action.style.alignItems = 'center';
  action.style.gap = '5px';
  action.style.borderRadius = '9999px';
  action.style.border = '1px solid color-mix(in oklab, var(--color-base-content) 18%, transparent)';
  action.style.fontSize = '12px';
  action.style.fontWeight = '650';
  action.style.lineHeight = '1';
  action.style.whiteSpace = 'nowrap';
  action.style.boxShadow = '0 3px 12px rgb(0 0 0 / 0.3)';
  action.style.backdropFilter = 'blur(8px)';

  const actionIcon = document.createElement('span');
  actionIcon.style.width = '14px';
  actionIcon.style.height = '14px';
  actionIcon.style.display = 'inline-flex';
  actionIcon.style.alignItems = 'center';
  actionIcon.style.justifyContent = 'center';
  actionIcon.style.fontSize = '15px';
  actionIcon.style.fontWeight = '800';

  const actionLabel = document.createElement('span');
  action.append(actionIcon, actionLabel);
  ghost.appendChild(action);

  document.body.appendChild(ghost);
  dragGhost = ghost;
  dragGhostAction = action;
}

function updateContentDragPosition(event: PointerEvent) {
  if (!dragGhost || (event.clientX === 0 && event.clientY === 0)) return;
  dragGhost.style.transform = `translate3d(${Math.round(event.clientX - dragGhostHotspotX)}px, ${Math.round(event.clientY - dragGhostHotspotY)}px, 0)`;
  const elementAtPointer = document.elementFromPoint(event.clientX, event.clientY);
  if (elementAtPointer?.closest('[data-collection-tray-root]') && !config.collectionTray.expanded) {
    config.collectionTray.expanded = true;
    requestAnimationFrame(() => {
      if (dragGhost) {
        setPointerDropTarget(null);
        updateContentDragPosition(event);
      }
    });
  }
  const target = elementAtPointer
    ?.closest('[data-file-drop-path][data-file-drop-album-id], [data-collection-drop-id], [data-collection-drop-new]') as HTMLElement | null;
  setPointerDropTarget(target);
  updateDragGhostAction(event);
}

function markContentInternalDrag({
  event,
  index,
  hotspotXRatio,
  hotspotYRatio,
}: {
  event: PointerEvent;
  index: number;
  hotspotXRatio: number;
  hotspotYRatio: number;
}) {
  const draggedFile = fileList.value[index];
  const fileItem = document.getElementById(`item-${index}`);
  if (!fileItem || !isRealFileItem(draggedFile)) return;
  isContentInternalDrag.value = true;
  const selected = getActionableSelectedItems();
  pointerDragUsesSelection = Boolean(draggedFile.isSelected && selectedCount.value > 0);
  const files = pointerDragUsesSelection && selected.length > 0 ? selected : [draggedFile];

  pointerDragFiles = files.map((f: any) => ({
    id: f.id,
    file_path: f.file_path,
    folder_id: f.folder_id,
    album_id: f.album_id,
  }));
  createDragGhost(
    fileItem,
    draggedFile,
    files,
    pointerDragUsesSelection ? selectedCount.value : files.length,
    { xRatio: hotspotXRatio, yRatio: hotspotYRatio },
  );
  void tauriEmit('content-items-drag-state', { dragging: true });
  updateContentDragPosition(event);
  document.addEventListener('pointermove', updateContentDragPosition);
  document.addEventListener('keydown', updateDragGhostModifier);
  document.addEventListener('keyup', updateDragGhostModifier);
}

async function clearContentInternalDrag(event?: PointerEvent) {
  const target = pointerDropTarget;
  let files = pointerDragFiles;
  const usesSelection = pointerDragUsesSelection;
  const copy = event ? isCopyDragModifier(event) : false;
  const shouldDrop = event?.type !== 'pointercancel';
  isContentInternalDrag.value = false;
  pointerDragUsesSelection = false;
  pointerDragFiles = null;
  removeDragGhost();
  void tauriEmit('content-items-drag-state', { dragging: false });

  if (shouldDrop && event && target && files?.length) {
    if (usesSelection) {
      const selectedItems = await getActionableSelectedItemsForAction();
      if (!selectedItems) return;
      files = selectedItems.map((file: any) => ({
        id: file.id,
        file_path: file.file_path,
        folder_id: file.folder_id,
        album_id: file.album_id,
      }));
    }

    const collectionId = target.dataset.collectionDropId;
    const collectionDropNew = target.dataset.collectionDropNew === 'true';
    if (collectionId || collectionDropNew) {
      let targetCollectionId = Number(collectionId || 0);
      if (collectionDropNew) {
        const collection = await createCollection(t('collection.default_name', { index: 1 }));
        targetCollectionId = Number(collection?.id || 0);
      }
      if (targetCollectionId <= 0) return;
      const result = await addFilesToCollection(targetCollectionId, files.map((file: any) => Number(file.id)).filter((id: number) => id > 0));
      if (result) {
        const added = Number(result.added || 0);
        const skipped = Number(result.skipped || 0);
        if (added > 0) {
          toast.success(t('collection.added_toast', { count: added }));
        }
        if (skipped > 0) {
          toast.info(t('collection.already_exists_toast', { count: skipped }));
        }
      }
      await tauriEmit('collection-files-dropped', {
        collectionId: targetCollectionId,
        count: Number(result?.added || 0),
        fileIds: files.map((file: any) => file.id),
      });
      if (libConfig.activePane === 'collection' && Number(libConfig.collection.selectedId || 0) === targetCollectionId) {
        await updateContent(true);
      }
      return;
    }

    if (!await confirmLargeBatch(files.length)) return;

    const destPath = String(target.dataset.fileDropPath || '');
    const albumId = Number(target.dataset.fileDropAlbumId || 0);
    const selected = destPath && albumId > 0 ? await selectFolder(albumId, destPath) : null;
    if (selected?.id) {
      let done = 0;
      let indexFailureCount = 0;
      const affectedAlbumIds = new Set<number>([albumId]);
      for (const file of files) {
        if (!copy && file.folder_id === selected.id) continue;
        if (copy) {
          const copiedPath = await copyFile(file.file_path, destPath);
          if (copiedPath) {
            if (await addFileToDb(selected.id, copiedPath)) {
              done++;
            } else {
              indexFailureCount++;
            }
          }
        } else {
          const movedPath = await moveFile(file.id, file.file_path, selected.id, destPath);
          if (movedPath) {
            done++;
            affectedAlbumIds.add(Number(file.album_id || 0));
          }
        }
      }
      if (done > 0) {
        await refreshAffectedAlbums(Array.from(affectedAlbumIds));
        await refreshLibraryTotalCount();
        await tauriEmit('refresh-content');
      }
      if (indexFailureCount > 0) {
        toast.error(t('msgbox.copy_to_folder.index_error', {
          count: indexFailureCount.toLocaleString(),
        }));
      }
    }
  }
}

const isProcessing = ref(false);  // show processing status
const isLoading = ref(false);     // show loading status in GridView (for empty file list)
const hasLoadedInitialResult = ref(false); // avoid showing "No files found" before first real result returns
const contentReady = ref(false);  // true after current view's content has loaded (empty or not), reset on navigation
const dedupSourceVersion = ref(0);

// Store current query params for virtual scrolling
const currentQueryParams = ref({
  searchFileType: 0,
  sortType: 0,
  sortOrder: 0,
  searchFileName: "",
  searchAllSubfolders: "",
  searchFolder: "",
  startDate: 0,
  endDate: 0,
  calendarSort: 0,
  folderSort: 0,
  categorySort: 0,
  make: "",
  model: "",
  lensMake: "",
  lensModel: "",
  locationAdmin1: "",
  locationName: "",
  isFavorite: false,
  rating: -1,
  tagId: 0,
  personId: 0,
});
const currentQuerySource = ref<'query' | 'smart' | 'collection' | 'search'>('query');
const currentSmartQueryParams = ref<any | null>(null);
const currentCollectionId = ref<number | null>(null);
const currentSearchFileIds = ref<number[]>([]);

const scanStreamRequestInFlight = ref(false);
const scanStreamPullPending = ref(false);
const scanStreamAlbumId = ref<number | null>(null);
let scanStreamFlushTimer: ReturnType<typeof setTimeout> | null = null;
const scanStreamQueuedAlbumId = ref<number | null>(null);
const scanStreamQueuedIndexed = ref(0);
let scanVisiblePrefetchTimer: ReturnType<typeof setTimeout> | null = null;
const scanVisiblePrefetchStart = ref(0);
const scanVisiblePrefetchEnd = ref(0);
const pendingRestoreScrollTop = ref<number | null>(null);

const isFilmstripVertical = computed(() => config.settings.grid.showFilmStrip && config.settings.grid.previewPosition >= 2);

const libraryChecked = ref(false);

watch(() => props.libraryEmpty, () => {
  libraryChecked.value = true;
}, { immediate: true });

watch(contentReady, (ready) => {
  if (
    !ready ||
    tempViewMode.value !== 'none' ||
    config.main.sidebarIndex !== SIDEBAR.LIBRARY ||
    libConfig.activePane !== 'main'
  ) return;
  void tauriEmit('library-item-count-updated', {
    item: libConfig.library.item,
    rating: libConfig.rating.item,
    smartId: libConfig.library.smartId,
    count: totalFileCount.value,
  });
});

const showWelcomeContent = computed(() => props.libraryEmpty && libraryChecked.value);

const gridViewLayoutClass = computed(() => {
  const pos = config.settings.grid.previewPosition || 0;
  if (pos === 0) return 'flex-col';
  if (pos === 1) return 'flex-col-reverse';
  if (pos === 2) return 'flex-row';
  if (pos === 3) return 'flex-row-reverse';
  return 'flex-col';
});

// ai image search params
const currentImageSearchParams = ref({
  searchText: "",
  fileId: 0,
  threshold: 0,
  limit: 0,
});

function showEmptyContent(requestId: number) {
  if (requestId !== currentContentRequestId) return;
  clearSelectionForFileListUpdate();
  clearContentRows();
  totalFileCount.value = 0;
  totalFileSize.value = 0;
  timelineData.value = [];
  lastVisibleRange = { start: -1, end: -1 };
  visibleRangeSeqId++;
  markDedupSourceUpdated(requestId);
  openImageViewer(0, false, true);
  isLoading.value = false;
  hasLoadedInitialResult.value = true;
  contentReady.value = true;
}

function showLoadingContent(requestId: number) {
  if (requestId !== currentContentRequestId) return;
  clearSelectionForFileListUpdate();
  clearContentRows();
  totalFileCount.value = 0;
  totalFileSize.value = 0;
  timelineData.value = [];
  isLoading.value = true;
  contentReady.value = false;
}

// Similar Search Mode State
const tempViewMode = ref<'none' | 'similar' | 'album' | 'person'>('none');
const dedupQueryParams = computed(() => {
  return { ...currentQueryParams.value };
});
const dedupCollectionId = computed(() =>
  currentQuerySource.value === 'collection' ? currentCollectionId.value : null
);
const dedupFileIds = computed(() =>
  currentQuerySource.value === 'search' ? [...currentSearchFileIds.value] : null
);

const dedupScanKey = computed(() => {
  if (dedupSourceVersion.value <= 0) return '';
  return `query:${JSON.stringify(dedupQueryParams.value)}|collection:${dedupCollectionId.value ?? ''}|files:${JSON.stringify(dedupFileIds.value)}|version:${dedupSourceVersion.value}`;
});

const currentTitleIcon = computed(() => {
  if (libConfig.activePane === 'collection') return IconBookmark;
  switch (tempViewMode.value) {
    case 'none':
      if (contentTitle.value) {
        switch (config.main.sidebarIndex) {
          case SIDEBAR.LIBRARY:
            switch (libConfig.library.item) {
              case LIB_ITEM.ALL: return IconPhotoAll;
              case LIB_ITEM.FAV: return IconHeartFilled;
              case LIB_ITEM.RATINGS: return libConfig.rating.item > 0 || libConfig.rating.item === RATE.ALL ? IconStarFilled : IconStar;
              case LIB_ITEM.SUBJECTS: return IconBox;
              case LIB_ITEM.TODAY: return IconHistory;
              default: return IconPhotoAll;
            }
          case SIDEBAR.ALBUM:
              return libConfig.album.selected || config.settings.showSubfolderFiles ? IconFolders : IconFolder;
          case SIDEBAR.SMART_ALBUM: return IconBolt;
          case SIDEBAR.SEARCH: return IconSearch;
          case SIDEBAR.CALENDAR: return config.calendar.isMonthly ? IconCalendarMonth : IconCalendarDay;
          case SIDEBAR.TAG: return IconTag;
          case SIDEBAR.PERSON: return IconPersonSearch;
          case SIDEBAR.LOCATION: return IconLocation;
          case SIDEBAR.CAMERA: return config.camera.isCamera ? IconCamera : IconCameraAperture;
          default: return IconFiles;
        }
      }
      return null;
    case 'similar': return IconPhotoSearch;
    case 'album': return IconFolderSearch;
    case 'person': return IconPersonSearch;
    default: return null;
  }
});

const backupState = ref<any>(null);

let unlistenKeydown: () => void;
let unlistenImageViewer: () => void;
let unlistenImageEditor: (() => void) | null = null;
let unlistenFaceIndexProgress: (() => void) | null = null;
let unlistenLibraryTotalRefreshed: (() => void) | null = null;
let unlistenPasteClipboard: (() => void) | null = null;

let resizeObserver: ResizeObserver | null = null;
let contentUpdateTimer: ReturnType<typeof setTimeout> | null = null;

function scheduleContentRefresh(task: () => void) {
  if (contentUpdateTimer) {
    clearTimeout(contentUpdateTimer);
  }
  contentUpdateTimer = setTimeout(() => {
    contentUpdateTimer = null;
    task();
  }, 0);
}

function waitForContentLoadingPaint() {
  return new Promise<void>(resolve => {
    requestAnimationFrame(() => setTimeout(resolve, 0));
  });
}

function resetContentViewportState() {
  scrollPosition.value = 0;
  selectedItemIndex.value = 0;
  if (gridViewRef.value) {
    gridViewRef.value.scrollToPosition(0);
  }
}

function refreshContentFromSelectionChange() {
  resetContentViewportState();
  updateContent();
  // Reset ImageViewer context if open (without focusing/showing it)
  openImageViewer(selectedItemIndex.value, false, true);
}

onMounted(() => {
  if (gridScrollContainerRef.value) {
    resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        containerHeight.value = entry.contentRect.height;
        containerWidth.value = entry.contentRect.width;
      }
    });
    resizeObserver.observe(gridScrollContainerRef.value);
    
    // Initial values
    containerHeight.value = gridScrollContainerRef.value.clientHeight;
    containerWidth.value = gridScrollContainerRef.value.clientWidth;
  }

  migrateRightPanelWidthToPixels();
  window.addEventListener('resize', handleWindowResize);
});

onBeforeUnmount(() => {
  stopSlideShow();
  clearRightPanelAnimationTimer();
  if (contentUpdateTimer) {
    clearTimeout(contentUpdateTimer);
    contentUpdateTimer = null;
  }
  window.removeEventListener('resize', handleWindowResize);
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
  if (unlistenKeydown) unlistenKeydown();
  if (unlistenImageViewer) unlistenImageViewer();
  if (unlistenImageEditor) unlistenImageEditor();
  if (unlistenLibraryTotalRefreshed) unlistenLibraryTotalRefreshed();
});

// New event handlers for GridView
function handleItemClicked(
  index: number,
  modifiers: { shiftKey?: boolean; metaKey?: boolean; ctrlKey?: boolean } = {}
) {
  if (!ensureGroupedFileAtIndex(index)) return;
  const shiftKey = !!modifiers.shiftKey;
  const toggleSelection = !!(modifiers.metaKey || modifiers.ctrlKey);

  if (!selectMode.value && shiftKey && selectedItemIndex.value >= 0 && selectedItemIndex.value !== index) {
    checkUnsavedChanges(() => {
      void selectRangeFromSingleSelection(selectedItemIndex.value, index);
    });
    return;
  }

  if (!selectMode.value && toggleSelection && selectedItemIndex.value >= 0) {
    checkUnsavedChanges(() => {
      const anchorIndex = selectedItemIndex.value;
      handleSelectMode(true);
      setItemSelected(anchorIndex, true);

      selectedItemIndex.value = index;
      if (index !== anchorIndex) {
        void handleItemSelectToggled(index);
      }
    });
    return;
  }

  if (index === selectedItemIndex.value) {
    if (selectMode.value) {
      void handleItemSelectToggled(index, shiftKey);
    }
    return;
  }
  
  checkUnsavedChanges(() => {
    selectedItemIndex.value = index;
    if (selectMode.value) {
      void handleItemSelectToggled(index, shiftKey);
    } else {
      lastSelectedIndex.value = index;
    }
  });
}

// Double click grid view item
function handleItemDblClicked(
  index: number,
  modifiers: { shiftKey?: boolean; metaKey?: boolean; ctrlKey?: boolean } = {}
) {
  if (!ensureGroupedFileAtIndex(index)) return;
  const openInNewWindow = !!(modifiers.shiftKey || modifiers.metaKey || modifiers.ctrlKey);
  if (openInNewWindow) {
    checkUnsavedChanges(() => {
      selectedItemIndex.value = index;
      openImageViewer(index, true);
    });
    return;
  }

  if (index === selectedItemIndex.value) {
    if (!config.settings.grid.showFilmStrip) {
      // quick view
      showQuickView.value = true;
      quickViewZoomFit.value = true;
    }
    return;
  }
  
  checkUnsavedChanges(() => {
    selectedItemIndex.value = index;

    if (!config.settings.grid.showFilmStrip) {
      // quick view
      showQuickView.value = true;
      quickViewZoomFit.value = true;
    }
  });
}

// Track last selected index for shift-click range selection
const lastSelectedIndex = ref(-1);
const keyboardSelectionAnchorIndex = ref(-1);

async function hydrateSelectionRange(startIndex: number, endIndex: number) {
  const start = Math.max(0, Math.min(startIndex, endIndex));
  const end = Math.min(fileList.value.length - 1, Math.max(startIndex, endIndex));
  if (start > end) return false;

  if (groupedModeActive.value) {
    const rowStart = getGroupedRowIndexForFileIndex(start);
    const rowEnd = getGroupedRowIndexForFileIndex(end);
    if (rowStart < 0 || rowEnd < 0) return false;
    await fetchGroupedRowsRange(Math.min(rowStart, rowEnd), Math.max(rowStart, rowEnd) + 1);
    for (let i = start; i <= end; i++) {
      if (!isRealFileItem(fileList.value[i]) && !ensureGroupedFileAtIndex(i)) {
        return false;
      }
    }
    return true;
  }

  const hasPlaceholders = fileList.value
    .slice(start, end + 1)
    .some(isPendingFileItem);
  return !hasPlaceholders || await hydrateRangeForSelection(start, end + 1);
}

async function selectRangeFromSingleSelection(anchorIndex: number, targetIndex: number) {
  const start = Math.min(anchorIndex, targetIndex);
  const end = Math.max(anchorIndex, targetIndex);
  if (!await hydrateSelectionRange(start, end)) {
    toast.error(t('info_panel.selection_load_failed'));
    return;
  }

  clearLoadedSelectionFlags();
  resetSelectionSummary();
  groupSelectedCountMap.value = {};
  groupSelectedSizeMap.value = {};
  selectMode.value = true;
  showQuickView.value = false;
  stopSlideShow();
  config.rightPanel.show = false;

  for (let i = start; i <= end; i++) {
    if (isRealFileItem(fileList.value[i])) {
      setItemSelected(i, true);
    }
  }

  selectedItemIndex.value = targetIndex;
  lastSelectedIndex.value = targetIndex;
}

async function handleItemSelectToggled(index: number, shiftKey: boolean = false) {
  if (!ensureGroupedFileAtIndex(index)) return;
  const targetItem = fileList.value[index];
  if (!targetItem) return;
  if (shiftKey && lastSelectedIndex.value !== -1 && lastSelectedIndex.value !== index) {
    // Range selection: select all items between lastSelectedIndex and index
    const start = Math.min(lastSelectedIndex.value, index);
    const end = Math.max(lastSelectedIndex.value, index);
    
    // Set all items in range to the same selection state as the target item
    const targetState = !targetItem.isSelected;
    if (!await hydrateSelectionRange(start, end)) {
      toast.error(t('info_panel.selection_load_failed'));
      return;
    }
    for (let i = start; i <= end; i++) {
      if (isRealFileItem(fileList.value[i])) {
        setItemSelected(i, targetState);
      }
    }
  } else {
    // Single toggle
    setItemSelected(index, !targetItem.isSelected);
  }
  
  // Update last selected index
  lastSelectedIndex.value = index;
}

function toggleKeyboardSelection(direction: 'prev' | 'next') {
  if (getActivePreviewMode() !== 'none') return false;

  const currentIndex = selectedItemIndex.value;
  const nextIndex = direction === 'next' ? currentIndex + 1 : currentIndex - 1;
  if (currentIndex < 0 || nextIndex < 0 || nextIndex >= fileList.value.length) return false;
  if (!ensureGroupedFileAtIndex(nextIndex)) return false;

  checkUnsavedChanges(() => {
    void (async () => {
      if (keyboardSelectionAnchorIndex.value < 0) {
        keyboardSelectionAnchorIndex.value = currentIndex;
      }

      if (!selectMode.value) {
        handleSelectMode(true);
      }

      const start = Math.min(keyboardSelectionAnchorIndex.value, nextIndex);
      const end = Math.max(keyboardSelectionAnchorIndex.value, nextIndex);
      const nextItem = fileList.value[nextIndex];
      if (!nextItem) return;
      const targetState = !nextItem.isSelected;
      if (!await hydrateSelectionRange(start, end)) {
        toast.error(t('info_panel.selection_load_failed'));
        return;
      }
      for (let i = start; i <= end; i++) {
        if (isRealFileItem(fileList.value[i])) {
          setItemSelected(i, targetState);
        }
      }

      selectedItemIndex.value = nextIndex;
      lastSelectedIndex.value = nextIndex;
    })();
  });
  return true;
}

async function handleDateGroupSelect({ startIndex, endIndex, selected }: { startIndex: number; endIndex: number; selected: boolean }) {
  if (!selectMode.value) return;
  const start = Math.max(0, Math.min(Number(startIndex || 0), fileList.value.length));
  const end = Math.max(start, Math.min(Number(endIndex || start), fileList.value.length));
  if (start >= end) return;

  const needsLoad = fileList.value.slice(start, end).some(isPendingFileItem);
  if (needsLoad) {
    await fetchDataRange(start, end);
  }

  for (let i = start; i < end; i++) {
    if (isRealFileItem(fileList.value[i])) {
      setItemSelected(i, selected);
    }
  }

  lastSelectedIndex.value = selected ? start : -1;
}

async function handleGroupSelectToggled(groupRow: any, selected: boolean) {
  const groupId = String(groupRow?.group_id || '');
  if (!groupId || groupSelectionLoading.value[groupId]) return;

  groupSelectionLoading.value = {
    ...groupSelectionLoading.value,
    [groupId]: true,
  };

  try {
    const ids = await getCachedGroupFileIds(groupId);
    if (!ids || ids.length === 0) return;

    selectMode.value = true;
    const idSet = new Set(ids.map(id => Number(id)).filter(id => Number.isFinite(id) && id > 0));
    const loadedById = new Map<number, number>();
    fileList.value.forEach((file, index) => {
      const fileId = Number(file?.id || 0);
      if (fileId) loadedById.set(fileId, index);
    });

    for (const fileId of idSet) {
      if (selected) selectedFileIds.add(fileId);
      else selectedFileIds.delete(fileId);

      const index = loadedById.get(fileId);
      if (index === undefined) continue;
      const file = fileList.value[index];
      if (isRealFileItem(file)) {
        file.isSelected = selected;
      }
    }

    const currentGroupCount = Number(groupSelectedCountMap.value[groupId] || 0);
    const currentGroupSize = Number(groupSelectedSizeMap.value[groupId] || 0);
    const nextGroupCount = selected ? Number(groupRow.count || idSet.size) : 0;
    const nextGroupSize = selected ? Number(groupRow.size || groupMetaMap.get(groupId)?.size || 0) : 0;
    updateGroupSelectedCount(groupId, nextGroupCount);
    updateGroupSelectedSize(groupId, nextGroupSize);
    selectedCount.value = Math.max(0, selectedCount.value + (nextGroupCount - currentGroupCount));
    selectedSize.value = Math.max(0, selectedSize.value + (nextGroupSize - currentGroupSize));

    if (selected) {
      let firstFileIndex = 0;
      for (const group of groupedTimelineGroups.value) {
        if (String(group.groupId) === groupId) {
          selectedItemIndex.value = firstFileIndex;
          lastSelectedIndex.value = firstFileIndex;
          break;
        }
        firstFileIndex += Number(group.count || 0);
      }
    }

    syncSelectionVersions();
  } catch (error) {
    console.error('handleGroupSelectToggled error:', error);
    toast.error(t('info_panel.selection_load_failed'));
  } finally {
    groupSelectionLoading.value = {
      ...groupSelectionLoading.value,
      [groupId]: false,
    };
  }
}

async function getCachedGroupFileIds(groupId: string) {
  const cached = groupFileIdsCache.get(groupId);
  if (cached) return cached;

  const result = currentQuerySource.value === 'smart' && currentSmartQueryParams.value
    ? await getSmartGroupFileIds(getGroupingQueryParams(), groupId)
    : await getGroupFileIds(getGroupingQueryParams(), groupId);
  const ids = Array.isArray(result)
    ? result
    : Array.isArray(result?.file_ids)
      ? result.file_ids
      : Array.isArray(result?.fileIds)
        ? result.fileIds
        : [];
  const normalized = ids.map((id: any) => Number(id)).filter((id: number) => Number.isFinite(id) && id > 0);
  groupFileIdsCache.set(groupId, normalized);
  return normalized;
}

function unselectFileFromSelection(fileId: number) {
  const targetId = Number(fileId);
  const file = fileList.value.find(item => Number(item?.id || 0) === targetId);
  if (!file) return;
  const index = fileList.value.indexOf(file);
  setItemSelected(index, false);
}

async function handleTimelineSelectItem(index: number) {
  if (groupedModeActive.value) {
    if (index < 0 || index >= totalRowCount.value) return;
    const fileIndex = getGroupedFileIndexForRowIndex(index);
    if (fileIndex < 0 || fileIndex >= fileList.value.length) return;
    const rowIndex = getGroupedRowIndexForFileIndex(fileIndex);
    if (rowIndex >= 0) {
      await fetchGroupedRowsRange(rowIndex, rowIndex + 1);
    }
    if (!ensureGroupedFileAtIndex(fileIndex)) return;
    if (fileIndex === selectedItemIndex.value) return;

    checkUnsavedChanges(() => {
      selectedItemIndex.value = fileIndex;
      if (!selectMode.value) {
        lastSelectedIndex.value = fileIndex;
      }
    });
    return;
  }

  if (index < 0 || index >= fileList.value.length) return;
  if (index === selectedItemIndex.value) return;

  checkUnsavedChanges(() => {
    selectedItemIndex.value = index;
  });
}

function clickRename() {
  renamingFileName.value = extractFileName(fileList.value[selectedItemIndex.value].name);
  showRenameMsgbox.value = true;
}

async function clickSetAlbumCover() {
  const file = fileList.value[selectedItemIndex.value];
  const albumId = libConfig.album.id || file?.album_id;
  
  if (file && albumId) {
    try {
      const result = await setAlbumCover(albumId, file.id);
      if (result === null) throw new Error('Failed to update album cover');
      await tauriEmit('album-cover-changed', { albumId: albumId, fileId: file.id });
      toast.success(localeMsg.value.tooltip.set_album_cover.success);
    } catch (error) {
      toast.error(localeMsg.value.tooltip.set_album_cover.failed);
    }
  }
}

function handleItemAction(payload: { action: string, index: number }) {
  if (isSlideShow.value) return;

  const { action, index } = payload;
  selectedItemIndex.value = index; // Ensure the item for the action is selected

  if (action.startsWith('rating-')) {
    const rating = Number.parseInt(action.slice('rating-'.length), 10);
    if (!Number.isNaN(rating)) {
      void setSelectedFileRating(rating);
    }
    return;
  }

  const actionMap = {
    'open': () => openImageViewer(selectedItemIndex.value, true),
    'print': () => void printImage(selectedItemIndex.value),
    'edit': () => void openImageEditor(selectedItemIndex.value),
    'open-external-app': () => {
      void openInExternalApp();
    },
    'copy': () => void clickCopyImages(fileList.value[selectedItemIndex.value].file_path),
    'rename': clickRename,
    'move-within-library': () => showMoveTo.value = true,
    'move-to-folder': () => void onMoveToFolder(),
    'copy-to-folder': () => void onCopyToFolder(),
    'trash': () => openTrashMsgbox(),
    'remove-from-collection': () => {
      if (currentQuerySource.value === 'collection' && currentCollectionId.value) {
        void removeFileFromCurrentCollection(selectedItemIndex.value);
      }
    },
    'reveal': () => revealPath(fileList.value[selectedItemIndex.value].file_path),
    'refresh-file-info': () => void updateFile(fileList.value[selectedItemIndex.value], true),
    'favorite': toggleFavorite,
    'rotate': clickRotate,
    'info': toggleInfoPanel,
    'tag': clickTag,
    'comment': () => showCommentMsgbox.value = true,
    'search-similar': () => enterSimilarSearchMode(fileList.value[selectedItemIndex.value]),
    'find-person': () => {
      if (!config.settings.face.enabled) return;
      enterPersonSearchMode(fileList.value[selectedItemIndex.value]);
    },
    'set-album-cover': clickSetAlbumCover,
  };

  if ((actionMap as any)[action]) {
    // Check for unsaved changes before performing action, especially if it might change the context
    // Most actions here operate on `index` which becomes the selected index. 
    // If index is different from current selectedItemIndex, we should check.
    
    if (index !== selectedItemIndex.value) {
        checkUnsavedChanges(() => {
             (actionMap as any)[action]();
        });
    } else {
         (actionMap as any)[action]();
    }
  }
}

function requestNavigate(direction: 'prev' | 'next') {
  checkUnsavedChanges(() => {
    const viewer = showQuickView.value ? quickViewMediaRef.value : (config.settings.grid.showFilmStrip ? filmStripMediaRef.value : null);
    
    if (direction === 'next') {
      if (viewer) {
        viewer.triggerNext();
      } else {
        performNavigate('next');
      }
    } else {
      if (viewer) {
        viewer.triggerPrev();
      } else {
        performNavigate('prev');
      }
    }
  });
}

function performNavigate(direction: 'prev' | 'next') {
  checkUnsavedChanges(() => {
    if (direction === 'next') {
      if (selectedItemIndex.value < fileList.value.length - 1) {
        selectedItemIndex.value += 1;
      }
    } else {
      if (selectedItemIndex.value > 0) {
        selectedItemIndex.value -= 1;
      }
    }
  });
}

function updateScrollPosition(currentScrollTop: number, currentScrollHeight: number) {
    if (!config.settings.grid.showFilmStrip) {
      // Calculate max scroll top
      const totalRows = Math.ceil(scrollbarTotal.value / columnCount.value);
      const topPadding = 48;
      const bottomPadding = config.settings.showStatusBar ? 32 : 4;
      
      // Determine effective scroll height
      // If provided (from event), use it. Otherwise calculate based on layout.
      let scrollHeight = currentScrollHeight;
      if (!scrollHeight) {
         const contentHeight = layoutContentHeight.value > 0
            ? layoutContentHeight.value 
            : (totalRows * itemSize.value);
         scrollHeight = contentHeight + topPadding + bottomPadding;
      }

      const maxScrollTop = Math.max(0, scrollHeight - containerHeight.value);
      
      if (maxScrollTop <= 0) {
        scrollPosition.value = 0;
      } else {
        const ratio = Math.min(1, Math.max(0, currentScrollTop / maxScrollTop));
        const maxIndex = Math.max(1, scrollbarTotal.value - scrollbarPageSize.value);
        scrollPosition.value = Math.round(ratio * maxIndex);
      }
    } else if (config.settings.grid.showFilmStrip) {
      // Fallback for filmstrip or other layouts (horizontal)
      const rowIndex = Math.floor(currentScrollTop / itemSize.value);
      scrollPosition.value = rowIndex * columnCount.value;
    }
}

function handleGridScroll(event: any) {
  if (event && event.target) {
    updateScrollPosition(
        event.target.scrollTop, 
        event.target.scrollHeight
    );
  }
}

function handleLayoutUpdate({ height }: { height: number }) {
  layoutContentHeight.value = height;
  if (gridViewRef.value) {
    updateScrollPosition(gridViewRef.value.getScrollTop(), 0);
  }
}

function markDedupSourceUpdated(requestId?: number) {
  if (requestId !== undefined && requestId !== currentContentRequestId) {
    return;
  }
  dedupSourceVersion.value += 1;
}

function handleScrollUpdate(newIndex: number) {
  scrollPosition.value = newIndex;
  
  if (!config.settings.grid.showFilmStrip && gridViewRef.value) {
    // Calculate ratio (0 to 1)
    const maxIndex = Math.max(1, scrollbarTotal.value - scrollbarPageSize.value);
    const ratio = Math.min(1, Math.max(0, newIndex / maxIndex));
    
    // Calculate max scroll top
    const totalRows = Math.ceil(scrollbarTotal.value / columnCount.value);
    const topPadding = 48;
    const bottomPadding = config.settings.showStatusBar ? 32 : 4;
    
    // Use reported layout height when available; date headers also affect normal grid height.
    const contentHeight = layoutContentHeight.value > 0
      ? layoutContentHeight.value 
      : (totalRows * itemSize.value);
      
    const totalHeight = contentHeight + topPadding + bottomPadding;
    const maxScrollTop = Math.max(0, totalHeight - containerHeight.value);
    
    const newScrollTop = ratio * maxScrollTop;
    
    gridViewRef.value.scrollToPosition(newScrollTop);
  } else if (gridScrollContainerRef.value) {
    // For filmstrip or other layouts, fallback to simple calculation or existing logic
    // But filmstrip uses horizontal scroll, handled differently.
    // This block might be for fallback.
    const rowIndex = Math.floor(newIndex / columnCount.value);
    const newScrollTop = rowIndex * itemSize.value;
    gridScrollContainerRef.value.scrollTop = newScrollTop;
  }
}

// Keyboard navigation actions
const keyActions = {
  ArrowDown: () => {
    if (getActivePreviewMode() !== 'none') return;
    checkUnsavedChanges(() => {
      if (gridViewRef.value) {
        if (usesGeometryNavigation.value) {
          // Use geometry-aware navigation for variable-sized layouts.
          const nextIndex = gridViewRef.value.getNextItemIndex(selectedItemIndex.value, 'down');
          selectedItemIndex.value = nextIndex !== -1 ? nextIndex : selectedItemIndex.value;
        } else {
          selectedItemIndex.value = Math.min(selectedItemIndex.value + gridViewRef.value.getColumnCount(), fileList.value.length - 1);
        }
      }
    });
  },
  ArrowUp: () => {
    if (getActivePreviewMode() !== 'none') return;
    checkUnsavedChanges(() => {
      if (gridViewRef.value) {
        if (usesGeometryNavigation.value) {
          // Use geometry-aware navigation for variable-sized layouts.
          const nextIndex = gridViewRef.value.getNextItemIndex(selectedItemIndex.value, 'up');
          selectedItemIndex.value = nextIndex !== -1 ? nextIndex : selectedItemIndex.value;
        } else {
          selectedItemIndex.value = Math.max(selectedItemIndex.value - gridViewRef.value.getColumnCount(), 0);
        }
      }
    });
  },
  Home: () => {
    checkUnsavedChanges(() => {
      selectedItemIndex.value = 0;
    });
  },
  End: () => {
    checkUnsavedChanges(() => {
      selectedItemIndex.value = fileList.value.length - 1;
    });
  },
};

// Local keydown handler for navigation (prevents default browser behavior)
function handleLocalKeyDown(event: KeyboardEvent) {
  if (event.defaultPrevented) return;

  // Do not intercept editing shortcuts while typing.
  const target = event.target as HTMLElement;
  if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
    return;
  }

  // Check if there are modal dialogs
  if (uiStore.inputStack.length > 0) {
    return;
  }

  if (matchesShortcut('file.paste', event, shortcutPlatform)) {
    event.preventDefault();
    void pasteClipboardImage();
    return;
  }

  if (uiStore.activePane === 'left-sidebar') {
    return;
  }

  if (event.key === 'Shift') {
    if (selectedItemIndex.value >= 0) {
      keyboardSelectionAnchorIndex.value = selectedItemIndex.value;
    }
    return;
  }

  if (selectMode.value && matchesShortcut('file.selectNone', event, shortcutPlatform)) {
    event.preventDefault();
    selectNoneInCurrentList();
    return;
  }

  if (selectMode.value && matchesShortcut('file.invertSelection', event, shortcutPlatform)) {
    event.preventDefault();
    void invertSelectionInCurrentList();
    return;
  }

  if (matchesShortcut('meta.info', event, shortcutPlatform)) {
    event.preventDefault();
    toggleInfoPanel();
    return;
  }

  if (matchesShortcut('view.close', event, shortcutPlatform)) {
    if (selectMode.value && showQuickView.value) {
      closeQuickPreview();
      event.preventDefault();
      return;
    }
    if (selectMode.value) {
      if (selectedCount.value > 0) {
        selectNoneInCurrentList();
      } else {
        handleSelectMode(false);
      }
      event.preventDefault();
      return;
    }
    if (showQuickView.value) {
      closeQuickPreview();
      event.preventDefault();
      return;
    }
    if (tempViewMode.value !== 'none') {
      exitTempViewMode();
      event.preventDefault();
      return;
    } else if (config.rightPanel.show) {
      config.rightPanel.show = false;
      event.preventDefault();
      return;
    }
  }

  if (matchesShortcut('file.selectAll', event, shortcutPlatform)) {
    event.preventDefault();
    void selectAllInCurrentList();
    return;
  }

  if (selectedItemIndex.value < 0 || fileList.value.length === 0) {
    return;
  }

  // Disable keyboard events during slideshow except close and toggle slideshow.
  if (
    isSlideShow.value &&
    !matchesShortcut('view.close', event, shortcutPlatform) &&
    !matchesShortcut('view.cycleBackground', event, shortcutPlatform) &&
    !matchesShortcut('slideshow.toggle', event, shortcutPlatform)
  ) {
    return;
  }

  const ratingShortcut = getMatchedRating(event);
  if (ratingShortcut !== null) {
    event.preventDefault();
    if (selectMode.value) {
      void selectModeSetRatings(ratingShortcut);
    } else {
      void setSelectedFileRating(ratingShortcut);
    }
    return;
  }

  if (matchesShortcut('file.searchSimilar', event, shortcutPlatform)) {
    event.preventDefault();
    enterSimilarSearchMode(fileList.value[selectedItemIndex.value]);
    return;
  }

  if (matchesShortcut('file.openExternalApp', event, shortcutPlatform)) {
    event.preventDefault();
    void openInExternalApp();
    return;
  }

  if (matchesShortcut('file.reveal', event, shortcutPlatform)) {
    event.preventDefault();
    revealPath(fileList.value[selectedItemIndex.value].file_path);
    return;
  }

  if (getActivePreviewMode() !== 'none' && matchesShortcut('view.cycleBackground', event, shortcutPlatform)) {
    event.preventDefault();
    config.cycleViewBackground();
    void tauriEmit('settings-viewBackground-changed', config.settings.viewBackground);
    return;
  }

  if ((showQuickView.value || config.settings.grid.showFilmStrip) && matchesShortcut('slideshow.toggle', event, shortcutPlatform)) {
    event.preventDefault();
    toggleSlideShow();
    return;
  }

  if (getActivePreviewMode() !== 'none' && matchesShortcut('view.zoomIn', event, shortcutPlatform) && event.key === '=') {
    event.preventDefault();
    getActivePreviewMediaRef()?.zoomIn?.();
    return;
  }

  if (getActivePreviewMode() !== 'none' && matchesShortcut('view.zoomOut', event, shortcutPlatform) && event.key === '-') {
    event.preventDefault();
    getActivePreviewMediaRef()?.zoomOut?.();
    return;
  }

  if (getActivePreviewMode() === 'none' && matchesShortcut('view.zoomIn', event, shortcutPlatform) && event.key === '=') {
    if (!isContentInteractionActive()) return;
    event.preventDefault();
    config.settings.grid.size = Math.min(360, Number(config.settings.grid.size || 160) + 10);
    return;
  }

  if (getActivePreviewMode() === 'none' && matchesShortcut('view.zoomOut', event, shortcutPlatform) && event.key === '-') {
    if (!isContentInteractionActive()) return;
    event.preventDefault();
    config.settings.grid.size = Math.max(120, Number(config.settings.grid.size || 160) - 10);
    return;
  }

  if (matchesShortcut('meta.favorite', event, shortcutPlatform)) {
    event.preventDefault();
    if (selectMode.value) {
      void toggleSelectModeFavorite();
    } else {
      void toggleFavorite();
    }
    return;
  }

  if (matchesShortcut('meta.tag', event, shortcutPlatform)) {
    event.preventDefault();
    void clickTag();
    return;
  }

  if (matchesShortcut('meta.comment', event, shortcutPlatform)) {
    event.preventDefault();
    showCommentMsgbox.value = true;
    return;
  }

  if (matchesShortcut('meta.rotate', event, shortcutPlatform)) {
    event.preventDefault();
    void clickRotate();
    return;
  }

  if (matchesShortcut('file.moveTo', event, shortcutPlatform)) {
    event.preventDefault();
    showMoveTo.value = true;
    return;
  }

  if (matchesShortcut('file.moveToFolder', event, shortcutPlatform)) {
    event.preventDefault();
    void onMoveToFolder();
    return;
  }

  if (matchesShortcut('file.rename', event, shortcutPlatform)) {
    event.preventDefault();
    clickRename();
    return;
  }

  if (!isMac && matchesShortcut('view.first', event, shortcutPlatform)) {
    event.preventDefault();
    checkUnsavedChanges(() => {
      selectedItemIndex.value = 0;
    });
    return;
  }

  if (!isMac && matchesShortcut('view.last', event, shortcutPlatform)) {
    event.preventDefault();
    checkUnsavedChanges(() => {
      selectedItemIndex.value = fileList.value.length - 1;
    });
    return;
  }

  const handledKeys = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Home', 'End', 'Enter', 'Space', ' '];

  if (!isMac) {
    handleQuickPreviewShortcut(event);
  }

  if (handledKeys.includes(event.key)) {
    event.preventDefault();
  }

  const isFilmstrip = config.settings.grid.showFilmStrip;
  const selectionDirection =
    event.key === 'ArrowRight' || (isFilmstrip && event.key === 'ArrowDown')
      ? 'next'
      : event.key === 'ArrowLeft' || (isFilmstrip && event.key === 'ArrowUp')
        ? 'prev'
        : null;

  if (event.shiftKey && selectionDirection && toggleKeyboardSelection(selectionDirection)) {
    return;
  }

  if (event.key === 'ArrowRight' || (isFilmstrip && event.key === 'ArrowDown')) {
    requestNavigate('next');
  } else if (event.key === 'ArrowLeft' || (isFilmstrip && event.key === 'ArrowUp')) {
    requestNavigate('prev');
  }
}

function handleQuickPreviewShortcut(event: { key: string; code?: string }): boolean {
  if (!matchesShortcut('view.quickPreview', event, shortcutPlatform)) {
    return false;
  }

  if (event.key === 'Enter') {
    if (!showQuickView.value && !config.settings.grid.showFilmStrip) {
      showQuickView.value = true;
      quickViewZoomFit.value = true;
    }
    return true;
  }

  if (event.key !== 'Space' && event.key !== ' ') {
    return false;
  }

  if (getActivePreviewMode() === 'quick-view') {
    if (fileList.value[selectedItemIndex.value]?.file_type === 2) {
      getActivePreviewMediaRef()?.togglePlay?.();
    } else {
      quickViewZoomFit.value = !quickViewZoomFit.value;
    }
  } else if (getActivePreviewMode() === 'filmstrip') {
    filmStripZoomFit.value = !filmStripZoomFit.value;
  } else if (!config.settings.grid.showFilmStrip) {
    showQuickView.value = true;
    quickViewZoomFit.value = true;
  }

  return true;
}

function handleLocalKeyUp(event: KeyboardEvent) {
  if (event.key === 'Shift') {
    keyboardSelectionAnchorIndex.value = -1;
  }
}

function isContentInteractionActive() {
  return isContentHovered.value && !uiStore.mapActive;
}

function activateContentPane() {
  uiStore.setActivePane('content');
  contentRootRef.value?.focus({ preventScroll: true });
}

function handleContentWheel(event: WheelEvent) {
  if (!event.ctrlKey) return;
  if (getActivePreviewMode() !== 'none') return;
  if (!isContentInteractionActive()) return;

  event.preventDefault();
  event.stopPropagation();

  const currentSize = Number(config.settings.grid.size || 160);
  const delta = event.deltaY < 0 ? 10 : -10;
  config.settings.grid.size = Math.max(120, Math.min(360, currentSize + delta));
}

function isTextInputFocused() {
  const active = document.activeElement as HTMLElement | null;
  if (!active) return false;
  return active.tagName === 'INPUT' || active.tagName === 'TEXTAREA' || active.isContentEditable;
}

// Global keydown handler (from Tauri)
const handleKeyDown = (e: any) => {
  if (uiStore.activePane === 'left-sidebar') {
    return;
  }

  if (isTextInputFocused()) {
    return;
  }

  if (uiStore.inputStack.length > 0) {
    return;
  }

  const event = e.payload;
  const { key } = event;

  // Disable global shortcuts during slideshow except close for safety.
  if (isSlideShow.value && !matchesShortcut('view.close', event, shortcutPlatform)) {
    return;
  }

  // GridView prevents Space's default behavior before Content's window listener on macOS,
  // so Quick Preview uses the App capture/global channel there.
  if (isMac && handleQuickPreviewShortcut(event)) {
    return;
  }

  if (matchesShortcut('file.openNewWindow', event, shortcutPlatform)) {
    openImageViewer(selectedItemIndex.value, true);
  } else if (matchesShortcut('file.copy', event, shortcutPlatform)) {
    void clickCopyImages(fileList.value[selectedItemIndex.value].file_path);
  // macOS handles Cmd+Arrow in App's capture listener before the content DOM handler.
  } else if (isMac && matchesShortcut('view.first', event, shortcutPlatform)) {
    keyActions.Home();
  } else if (isMac && matchesShortcut('view.last', event, shortcutPlatform)) {
    keyActions.End();
  } else if (matchesShortcut('file.editImage', event, shortcutPlatform)) {
    const file = fileList.value[selectedItemIndex.value];
    if (file && (file.file_type === 1 || file.file_type === 3)) {
      void openImageEditor(selectedItemIndex.value);
    }
  } else if (matchesShortcut('file.trash', event, shortcutPlatform)) {
    if (currentQuerySource.value === 'collection' && currentCollectionId.value) {
      void removeFileFromCurrentCollection(selectedItemIndex.value);
    } else {
      openTrashMsgbox();
    }
  } else if (key === 'ArrowUp' || key === 'ArrowDown') {
    (keyActions as any)[key]();
  }
};

// --- Indexing Logic ---
let unlistenIndexProgress: (() => void) | undefined;
let unlistenIndexFinished: (() => void) | undefined;
let unlistenThumbnailReady: (() => void) | undefined;
let unlistenTriggerNextAlbum: (() => void) | undefined;
let unlistenRefreshContent: (() => void) | undefined;
let unlistenFilesDeleted: (() => void) | undefined;
let unlistenAlbumUpdated: (() => void) | undefined;
const showIndexRecoveryMsgbox = ref(false);
const recoverySkipFilePath = ref('');  // local: file path from crash trace
const indexRecoveryTitle = computed(() => localeMsg.value.search.index.recovery.title);
const indexRecoveryOkText = computed(() => localeMsg.value.search.index.recovery.continue);
const indexRecoverySkipLabel = computed(() => localeMsg.value.search.index.recovery.skip_once);
const indexRecoveryFileLabel = computed(() => localeMsg.value.search.index.recovery.file_label);
const indexRecoveryMessage = computed(() => {
  return localeMsg.value.search.index.recovery.message;
});

async function processNextAlbum(skipFilePath: string | null = null, skipRecoveryCheck = false) {
  if (libConfig.index.albumQueue.length > 0) {
    const albumId = libConfig.index.albumQueue[0];
    const album = await getAlbum(albumId);
    if (album) {
      // Check for crash recovery: if trace file exists and matches this album
      if (!skipFilePath && !skipRecoveryCheck) {
        const recoveryInfo = await getIndexRecoveryInfo();
        if (recoveryInfo && Number(recoveryInfo.album_id) === Number(albumId)) {
          recoverySkipFilePath.value = String(recoveryInfo.file_path || '');
          libConfig.index.status = 2;
          showIndexRecoveryMsgbox.value = true;
          return;
        }
      }
      libConfig.index.status = 1;
      libConfig.index.pausedAlbumIds = (libConfig.index.pausedAlbumIds as any[]).filter(
        id => Number(id) !== Number(albumId)
      );
      libConfig.index.albumName = album.name;
      libConfig.index.phase = 'discovering';
      libConfig.index.discovered = 0;
      libConfig.index.processed = 0;
      libConfig.index.searchReady = 0;
      libConfig.index.indexed = 0;
      libConfig.index.total = 0;
      libConfig.index.searchTotal = 0;
      libConfig.index.failed = 0;
      await indexAlbum(albumId, skipFilePath || null);
    } else {
      // album not found (maybe deleted), remove from queue and process next
      libConfig.index.albumQueue.shift();
      processNextAlbum();
    }
  } else {
    if (libConfig.index.status !== 2) {
      libConfig.index.status = 0;
    }
  }
}

// Check if current album is being indexed
const isIndexing = computed(() => {
  return config.main.sidebarIndex === SIDEBAR.ALBUM &&
         !!libConfig.album.id && libConfig.album.id > 0 && // Valid album
         libConfig.index.albumQueue.includes(libConfig.album.id) &&
         Number(libConfig.index.status || 0) !== 2;
});

const isAnyIndexing = computed(() =>
  libConfig.index.albumQueue.length > 0 && Number(libConfig.index.status || 0) !== 2
);

const isScanStreamingMode = computed(() =>
  isIndexing.value &&
  config.main.sidebarIndex === SIDEBAR.ALBUM &&
  Boolean(libConfig.album.selected)
);

// When scanning starts, close panels and multi-select which rely on a stable file list.
watch(isScanStreamingMode, (streaming) => {
  if (streaming) {
    config.rightPanel.show = false;
    if (selectMode.value) selectMode.value = false;
  }
  if (groupedModeActive.value || activeGroupBy.value > 0) {
    updateContent();
  }
});

const thumbProgressPercent = computed(() => {
  if (fileList.value.length <= 0) return 0;
  return Number(((thumbCount.value / fileList.value.length) * 100).toFixed(0));
});

const isAlbumPaused = (albumId: number | null | undefined) =>
  (libConfig.index.pausedAlbumIds as any[]).some(id => Number(id) === Number(albumId || 0));
const syncIndexStatus = () => {
  if ((libConfig.index.albumQueue as any[]).length > 0) {
    libConfig.index.status = 1;
  } else if ((libConfig.index.pausedAlbumIds as any[]).length > 0) {
    libConfig.index.status = 2;
  } else {
    libConfig.index.status = 0;
  }
};

const isRecoveryInProgress = ref(false);

const confirmIndexRecoveryContinue = async (shouldSkip = false) => {
  showIndexRecoveryMsgbox.value = false;
  const skipFilePath = shouldSkip ? recoverySkipFilePath.value : null;
  recoverySkipFilePath.value = '';
  await clearIndexRecoveryInfo();  // delete trace file first to prevent re-detection
  isRecoveryInProgress.value = true;
  await processNextAlbum(skipFilePath, true);
  isRecoveryInProgress.value = false;
};

const cancelIndexRecovery = () => {
  showIndexRecoveryMsgbox.value = false;
  libConfig.index.status = 2;
};
const activeScanningAlbumId = computed(() => Number(libConfig.index.albumQueue[0] || 0));
const suppressNextIndexingIdleRefresh = ref(false);
const selectedAlbumIdForStatusBar = computed(() => Number(libConfig.album.id || 0));
const selectedAlbumScanState = computed(() => getAlbumScanState({
  albumId: selectedAlbumIdForStatusBar.value,
  albumQueue: libConfig.index.albumQueue as any[],
  pausedAlbumIds: libConfig.index.pausedAlbumIds as any[],
  status: Number(libConfig.index.status || 0),
}));
const selectedAlbumScanIcon = computed(() => getAlbumScanIcon(selectedAlbumScanState.value));
const selectedAlbumScanAnimating = computed(() => shouldAnimateAlbumScanIcon(selectedAlbumScanState.value));
const isOtherTabScanning = computed(() =>
  config.main.sidebarIndex !== SIDEBAR.ALBUM &&
  (libConfig.index.albumQueue as any[]).length > 0 &&
  Number(libConfig.index.status || 0) !== 2
);
const showBackgroundScanningIcon = computed(() =>
  selectedAlbumScanState.value === 'complete' &&
  (libConfig.index.albumQueue as any[]).length > 0 &&
  Number(libConfig.index.status || 0) !== 2
);

const statusBarScanMode = computed<'none' | 'current' | 'waiting' | 'background' | 'paused'>(() => {
  if (config.main.sidebarIndex === SIDEBAR.ALBUM) {
    if (selectedAlbumScanState.value === 'scanning') return 'current';
    if (selectedAlbumScanState.value === 'queued') return 'waiting';
    if (selectedAlbumScanState.value === 'paused') return 'paused';
    if (showBackgroundScanningIcon.value) return 'background';
    return 'none';
  }
  if (isOtherTabScanning.value) return 'background';
  return 'none';
});

const statusBarShowUpdateIcon = computed(() =>
  selectedAlbumScanIcon.value !== 'none' || statusBarScanMode.value === 'background'
);
const statusBarUpdateIcon = computed<'update' | 'dot'>(() =>
  statusBarScanMode.value === 'background' ? 'update' : selectedAlbumScanIcon.value === 'dot' ? 'dot' : 'update'
);
const statusBarIsUpdateAnimating = computed(() =>
  statusBarScanMode.value === 'background' || selectedAlbumScanAnimating.value
);

const statusBarScanText = computed(() => {
  const discovered = Number(libConfig.index.discovered || 0).toLocaleString();
  const processed = Number(libConfig.index.processed || libConfig.index.indexed || 0).toLocaleString();
  const searchReady = Number(libConfig.index.searchReady || 0).toLocaleString();
  const total = Number(libConfig.index.total || 0).toLocaleString();
  const searchTotal = Number(libConfig.index.searchTotal || 0).toLocaleString();
  const phase = String(libConfig.index.phase || 'discovering');

  if (statusBarScanMode.value === 'waiting') {
    return localeMsg.value.search.index.wait_index || 'Wait for scan...';
  }
  if (statusBarScanMode.value === 'paused') {
    return localeMsg.value.search.index.paused || 'Scan paused';
  }
  if (statusBarScanMode.value !== 'current') return '';

  if (phase === 'preparing_previews') {
    return (localeMsg.value.search.index.preparing_previews || 'Generating previews... {count} / {total}')
      .replace('{count}', processed)
      .replace('{total}', total);
  }
  if (phase === 'preparing_search') {
    return (localeMsg.value.search.index.preparing_search || 'Preparing search... {count} / {total}')
      .replace('{count}', searchReady)
      .replace('{total}', searchTotal);
  }
  if (phase === 'complete') {
    if (Number(libConfig.index.failed || 0) > 0) {
      return (localeMsg.value.search.index.complete_with_issues || 'Scan complete with {count} issues')
        .replace('{count}', Number(libConfig.index.failed || 0).toLocaleString());
    }
    return localeMsg.value.search.index.complete || 'Scan complete';
  }

  return (localeMsg.value.search.index.discovering || 'Scanning library... {count} / {total}')
    .replace('{count}', discovered)
    .replace('{total}', total);
});

const showTopProgressBar = computed(() =>
  fileList.value.length > 0 && showProgressBar.value
);

const topProgressPercent = computed(() => thumbProgressPercent.value);

function buildScanStreamQueryParams() {
  return {
    searchFileType: 0,
    sortType: 9, // internal sort: by id asc (insert/scan order)
    sortOrder: 0,
    searchFileName: "",
    searchAllSubfolders: libConfig.album.folderPath || "",
    searchFolder: "",
    startDate: 0,
    endDate: 0,
    calendarSort: 0,
    folderSort: 0,
    categorySort: 0,
    make: "",
    model: "",
    lensMake: "",
    lensModel: "",
    locationAdmin1: "",
    locationName: "",
    isFavorite: false,
    rating: -1,
    tagId: 0,
    personId: 0,
  };
}

function enterScanStreamingMode(albumId: number) {
  scanStreamAlbumId.value = albumId;
  clearSelectionForFileListUpdate();
  clearContentRows();
  totalFileCount.value = 0;
  totalFileSize.value = 0;
  selectedItemIndex.value = -1;
  thumbCount.value = 0;
  showProgressBar.value = false;
  isLoading.value = false;
  hasLoadedInitialResult.value = true;
  contentReady.value = true;
  currentQueryParams.value = buildScanStreamQueryParams();
  timelineData.value = [];
  lastVisibleRange = { start: -1, end: -1 };
  visibleRangeSeqId++;
  markDedupSourceUpdated();
}

async function pullScanStreamingDelta(albumId: number, current: number) {
  if (!isScanStreamingMode.value || Number(libConfig.album.id) !== Number(albumId)) {
    return;
  }

  if (scanStreamAlbumId.value !== albumId) {
    enterScanStreamingMode(albumId);
  }

  const targetCount = Math.max(0, Number(current || 0));
  const offset = fileList.value.length;
  const gap = targetCount - offset;
  if (gap <= 0) return;

  for (let i = 0; i < gap; i++) {
    const idx = offset + i;
    fileList.value.push({
      id: `scan-ph-${albumId}-${idx}`,
      isPlaceholder: true,
      name: '',
      size: 0,
      isSelected: false,
      rotate: 0,
    });
  }

  totalFileCount.value = fileList.value.length;
  const startIndex = lastVisibleRange.start >= 0 ? lastVisibleRange.start : 0;
  const endIndex =
    lastVisibleRange.end > startIndex
      ? lastVisibleRange.end
      : Math.min(fileList.value.length, selectionChunkSize.value);
  queueScanVisiblePrefetch(startIndex, endIndex);
  markDedupSourceUpdated();
}

function queueScanVisiblePrefetch(startIndex: number, endIndex: number) {
  const buffer = 200;
  scanVisiblePrefetchStart.value = startIndex - buffer;
  scanVisiblePrefetchEnd.value = endIndex + buffer;

  if (scanVisiblePrefetchTimer) return;
  scanVisiblePrefetchTimer = setTimeout(() => {
    scanVisiblePrefetchTimer = null;
    fetchDataRange(scanVisiblePrefetchStart.value, scanVisiblePrefetchEnd.value);
  }, 80);
}

async function scheduleScanStreamingPull(albumId: number, current: number) {
  if (scanStreamRequestInFlight.value) {
    scanStreamPullPending.value = true;
    return;
  }

  scanStreamRequestInFlight.value = true;
  try {
    await pullScanStreamingDelta(albumId, current);
  } finally {
    scanStreamRequestInFlight.value = false;
    if (scanStreamPullPending.value) {
      scanStreamPullPending.value = false;
      await scheduleScanStreamingPull(albumId, libConfig.index.discovered);
    }
  }
}

function queueScanStreamingPull(albumId: number, current: number) {
  scanStreamQueuedAlbumId.value = albumId;
  scanStreamQueuedIndexed.value = Math.max(scanStreamQueuedIndexed.value, Number(current || 0));

  if (scanStreamFlushTimer) return;
  scanStreamFlushTimer = setTimeout(async () => {
    scanStreamFlushTimer = null;
    const queuedAlbumId = scanStreamQueuedAlbumId.value;
    const queuedIndexed = scanStreamQueuedIndexed.value;
    scanStreamQueuedAlbumId.value = null;
    scanStreamQueuedIndexed.value = 0;

    if (!queuedAlbumId || queuedIndexed < 0) return;
    await scheduleScanStreamingPull(queuedAlbumId, queuedIndexed);

    const currentAlbumId = Number(libConfig.album.id || 0);
    if (
      isScanStreamingMode.value &&
      currentAlbumId > 0 &&
      libConfig.index.albumQueue.includes(currentAlbumId) &&
      Number(libConfig.index.discovered || 0) > fileList.value.length
    ) {
      queueScanStreamingPull(currentAlbumId, Number(libConfig.index.discovered || 0));
    }
  }, 300);
}

function restoreScrollAfterRefresh() {
  const scrollTop = pendingRestoreScrollTop.value;
  if (scrollTop === null) return;
  nextTick(() => {
    if (gridViewRef.value) {
      gridViewRef.value.scrollToPosition(scrollTop);
    }
    pendingRestoreScrollTop.value = null;
  });
}

watch(isIndexing, (val) => {
  if (!val) {
    if (suppressNextIndexingIdleRefresh.value) {
      suppressNextIndexingIdleRefresh.value = false;
      return;
    }
    updateContent();
  }
});

watch(
  () => [config.main.sidebarIndex, libConfig.album.id, isAnyIndexing.value],
  () => {
    if (!isScanStreamingMode.value) {
      scanStreamAlbumId.value = null;
      scanStreamPullPending.value = false;
      scanStreamQueuedAlbumId.value = null;
      scanStreamQueuedIndexed.value = 0;
      if (scanStreamFlushTimer) {
        clearTimeout(scanStreamFlushTimer);
        scanStreamFlushTimer = null;
      }
      if (scanVisiblePrefetchTimer) {
        clearTimeout(scanVisiblePrefetchTimer);
        scanVisiblePrefetchTimer = null;
      }
    }
  }
);

watch(
  () => [config.main.sidebarIndex, libConfig.album.id, activeScanningAlbumId.value],
  ([sidebarIndex, albumId, activeId]) => {
    const targetAlbumId = Number(activeId || 0);
    if (
      sidebarIndex === SIDEBAR.ALBUM &&
      Number(albumId || 0) > 0 &&
      Number(albumId || 0) === targetAlbumId &&
      targetAlbumId > 0
    ) {
      // Re-entering album tab should immediately sync placeholder length
      // to current indexed count, instead of waiting for next progress tick.
      if (scanStreamAlbumId.value !== targetAlbumId) {
        enterScanStreamingMode(targetAlbumId);
      }
      queueScanStreamingPull(targetAlbumId, Number(libConfig.index.discovered || 0));
    }
  }
);

watch(
  () => [libConfig.index.discovered, libConfig.album.id, config.main.sidebarIndex, libConfig.album.selected],
  ([discovered, albumId, sidebarIndex, selected]) => {
    if (
      sidebarIndex === SIDEBAR.ALBUM &&
      Number(albumId) > 0 &&
      libConfig.index.albumQueue.includes(Number(albumId)) &&
      Number(discovered || 0) >= 0
    ) {
      queueScanStreamingPull(Number(albumId), Number(discovered || 0));
    }
  },
  { immediate: true }
);

// Cancel indexing for current album
async function cancelIndexing() {
  const albumId = libConfig.album.id;
  const index = normalizedAlbumQueue.value.findIndex(id => id === Number(albumId || 0));
  if (index === -1) return;

  showIndexRecoveryMsgbox.value = false;

  if (index === 0) {
      libConfig.index.albumQueue.shift();
      await cancelIndexingApi(albumId);
      if (!isAlbumPaused(albumId)) {
        (libConfig.index.pausedAlbumIds as any[]).push(Number(albumId || 0));
      }
      if (libConfig.index.albumQueue.length > 0) {
        syncIndexStatus();
        setTimeout(() => {
          processNextAlbum();
        }, 1000);
      } else {
        syncIndexStatus();
      }
  } else {
    libConfig.index.albumQueue.splice(index, 1);
    if (!isAlbumPaused(albumId)) {
      (libConfig.index.pausedAlbumIds as any[]).push(Number(albumId || 0));
    }
    syncIndexStatus();
  }
}

onMounted( async() => {
  const appConfig = await getAppConfig();
  pendingInitialSelectedIndex = Number(appConfig?.last_selected_item_index ?? -1);
  hasRestoredInitialSelection = false;

  window.addEventListener('keydown', handleLocalKeyDown);
  window.addEventListener('keyup', handleLocalKeyUp);
  unlistenKeydown = await listen('global-keydown', handleKeyDown);

  unlistenLibraryTotalRefreshed = await listen('library-total-refreshed', (event: any) => {
    if (event?.payload?.source === 'content') return;
    if (isScanStreamingMode.value) return;
    if (config.main.sidebarIndex === SIDEBAR.ALBUM) {
      pendingInitialSelectedIndex = selectedItemIndex.value;
      hasRestoredInitialSelection = false;
      updateContent(true);
    }
  });
  unlistenPasteClipboard = await listen('paste-clipboard-to-folder', (event: any) => {
    const albumId = Number(event.payload?.albumId || 0);
    const folderPath = String(event.payload?.folderPath || '');
    if (albumId > 0 && folderPath) {
      void pasteClipboardImage({ albumId, folderPath });
    }
  });

  // Drag-drop file import. Tauri native drag/drop is disabled so internal
  // HTML5 drag interactions (e.g. sortable lists) keep their drop events.
  domDragEnter = (e: DragEvent) => {
    if (isInternalReorderActive()) return;
    if (!hasExternalDragIntent(e)) return;
    e.preventDefault();
    if (acceptDrops.value) {
      dragOverCount.value++;
      isDragOver.value = true;
    }
  };
  domDragLeave = (e: DragEvent) => {
    if (isInternalReorderActive()) return;
    if (!hasExternalDragIntent(e)) return;
    e.preventDefault();
    dragOverCount.value = Math.max(0, dragOverCount.value - 1);
    if (dragOverCount.value === 0) isDragOver.value = false;
  };
  domDragOver = (e: DragEvent) => {
    if (isInternalReorderActive()) return;
    if (!hasExternalDragIntent(e)) return;
    e.preventDefault();
  };
  domDrop = async (e: DragEvent) => {
    if (isInternalReorderActive() || isContentInternalDrag.value) {
      clearContentInternalDrag();
      return;
    }
    if (!hasExternalDomDrop(e)) return;
    e.preventDefault();
    dragOverCount.value = 0;
    isDragOver.value = false;
    if (!acceptDrops.value) {
      showDropWarning.value = true;
      return;
    }
    const dt = e.dataTransfer;
    if (!dt) return;
    const droppedFiles = Array.from(dt.files);
    const droppedUris = getExternalDropUris(dt);
    const nativeDragPayloadPromise = isMac
      ? getDragPayload() as Promise<{ filePaths?: string[]; url?: string | null }>
      : Promise.resolve(null);
    const destination = await resolveCurrentAlbumImportDestination();
    if (!destination) return;
    const nativeDragPayload = await nativeDragPayloadPromise;
    const { albumId, folderId, folderPath } = destination;

    let imported = 0;
    const filePaths = [
      ...getExternalFileDropPaths(droppedUris),
      ...(nativeDragPayload?.filePaths || []),
    ];
    for (const filePath of [...new Set(filePaths)]) {
      const file = await importFile(filePath, folderId, folderPath);
      if (file) imported++;
    }
    if (imported > 0) {
      await refreshImportedFiles(albumId);
      toast.success(t('msgbox.drop_import.success', { count: imported }));
      return;
    }

    if (droppedFiles.length > 0) {
      const MAX_FILE_SIZE = 200 * 1024 * 1024; // 200 MB
      const extRE = /\.(\w+)$/i;
      imported = 0;
      for (const file of droppedFiles) {
        if (!extRE.test(file.name) || file.size <= 0 || file.size > MAX_FILE_SIZE) continue;
        try {
          const buf = await file.arrayBuffer();
          if (buf.byteLength === 0) continue;
          const bytes = Array.from(new Uint8Array(buf));
          const result = await importFileBytes(bytes, file.name, folderId, folderPath);
          if (result) imported++;
        } catch (err) {
          console.error('Failed to import file via bytes:', file.name, err);
        }
      }
      if (imported > 0) {
        await refreshImportedFiles(albumId);
        toast.success(t('msgbox.drop_import.success', { count: imported }));
        return;
      }
      // Fall through to URL handling — some browsers provide both
      // file-like items and text/uri-list.
    }

    const urls = [
      ...getExternalHttpDropUrls(droppedUris),
      ...(nativeDragPayload?.url ? [nativeDragPayload.url] : []),
    ];
    if (urls.length > 0) {
      void handleDropUrls([...new Set(urls)], destination);
      return;
    }
    toast.warning(t('msgbox.drop_import.no_files'));
  };
  document.addEventListener('dragenter', domDragEnter);
  document.addEventListener('dragleave', domDragLeave);
  document.addEventListener('dragover', domDragOver);
  document.addEventListener('drop', domDrop);

  unlistenImageViewer = await listen('message-from-image-viewer', async (event) => {
    const { message } = event.payload as any;
    switch (message) {
      case 'request-file-at-index':
        const requestIndex = (event.payload as any).index;
        const requestedPane = String((event.payload as any).pane || 'left');
        const pane = ['left', 'right', 'bottomLeft', 'bottomRight'].includes(requestedPane)
          ? requestedPane
          : 'left';
        if (!isRealFileItem(fileList.value[requestIndex])) {
          ensureGroupedFileAtIndex(requestIndex);
        }
        if (!isRealFileItem(fileList.value[requestIndex])) {
          await fetchDataRange(requestIndex, requestIndex + 2);
        }
        const file = fileList.value[requestIndex];
        if (isRealFileItem(file)) {
           const imageWindow = await WebviewWindow.getByLabel('imageviewer');
           if (imageWindow) {
             imageWindow.emit('update-img', {
               fileId: file.id,
               fileIndex: requestIndex,
               fileCount: fileList.value.length,
               nextFilePath: getNextImagePath(requestIndex),
               pane,
             });
           }
        }
        break;
      case 'update-file-meta':
        const targetFileId = Number((event.payload as any).fileId);
        const changes = (event.payload as any).changes || {};
        if (targetFileId > 0) {
          const index = fileList.value.findIndex(file => file.id === targetFileId);
          if (index !== -1) {
            Object.assign(fileList.value[index], changes);
          }
        }
        break;
      default:
        break;
    }
  });

  unlistenImageEditor = await listen('message-from-image-editor', async (event: any) => {
    const { type, saveAsNew, filePath } = event.payload as any;
    if (type === 'success') {
      try {
        const editorWindow = await WebviewWindow.getByLabel('imageeditor');
        if (editorWindow) {
          try {
            await editorWindow.destroy();
          } catch (error) {
            console.error('Failed to destroy ImageEditor window from parent:', error);
          }
        }

        if (!saveAsNew && filePath) {
          uiStore.updateFileVersion(filePath);
        }
        await onFileSaved(true, { saveAsNew, filePath });
      } catch (error) {
        console.error('Failed handling ImageEditor save success:', error);
      }
    } else if (type === 'failed') {
      await onFileSaved(false);
    }
  });

  // Indexing listeners
  unlistenIndexProgress = await listenIndexProgress(async (event: any) => {
    const { album_id, phase, current, discovered, processed, search_ready, total, search_total, current_size, failed } = event.payload;
    if (libConfig.index.albumQueue.length > 0 && libConfig.index.albumQueue[0] === album_id) {
        libConfig.index.phase = String(phase || libConfig.index.phase || 'discovering');
        libConfig.index.discovered = Number(discovered || 0);
        libConfig.index.processed = Number(processed || current || 0);
        libConfig.index.searchReady = Number(search_ready || 0);
        libConfig.index.indexed = Number(processed || current || 0);
        libConfig.index.total = Number(total || 0);
        libConfig.index.searchTotal = Number(search_total || 0);
        libConfig.index.failed = Number(failed || 0);
        if (
          config.main.sidebarIndex === SIDEBAR.ALBUM &&
          Number(libConfig.album.id || 0) === Number(album_id || 0)
        ) {
          totalFileSize.value = Number(current_size || 0);
        }
    }
  });

  unlistenIndexFinished = await listenIndexFinished(async (event: any) => {
    const { album_id, phase, indexed, processed, search_ready, total, search_total, failed } = event.payload;
    // notify album list to update cover
    await tauriEmit('album-cover-changed', { albumId: album_id, fileId: null });
    const shouldRefreshCurrentView =
      config.main.sidebarIndex === SIDEBAR.ALBUM &&
      Number(libConfig.album.id) > 0 &&
      Number(libConfig.album.id) === Number(album_id);

    if (libConfig.index.albumQueue.length > 0 && libConfig.index.albumQueue[0] === album_id) {
      libConfig.index.phase = String(phase || 'complete');
      libConfig.index.discovered = Number(total || 0);
      libConfig.index.processed = Number(processed || indexed || 0);
      libConfig.index.searchReady = Number(search_ready || 0);
      libConfig.index.indexed = Number(processed || indexed || 0);
      libConfig.index.total = Number(total || 0);
      libConfig.index.searchTotal = Number(search_total || 0);
      libConfig.index.failed = Number(failed || 0);
      libConfig.index.albumQueue.shift();
      if (libConfig.index.albumQueue.length > 0) {
        processNextAlbum();
      } else {
        syncIndexStatus();
      }
    }

    if (shouldRefreshCurrentView) {
      scanStreamAlbumId.value = null;
      scanStreamPullPending.value = false;
      scanStreamQueuedAlbumId.value = null;
      scanStreamQueuedIndexed.value = 0;
      if (scanStreamFlushTimer) {
        clearTimeout(scanStreamFlushTimer);
        scanStreamFlushTimer = null;
      }
      if (scanVisiblePrefetchTimer) {
        clearTimeout(scanVisiblePrefetchTimer);
        scanVisiblePrefetchTimer = null;
      }
      if (gridViewRef.value) {
        pendingRestoreScrollTop.value = gridViewRef.value.getScrollTop();
      }
      // Avoid duplicated refresh: this explicit refresh replaces the
      // watch(isIndexing) idle refresh for this finish cycle.
      suppressNextIndexingIdleRefresh.value = true;
      selectedItemIndex.value = 0;
      setTimeout(() => {
        updateContent(true);
      }, 200);
    }

    refreshLibraryTotalCount();
  });

  unlistenThumbnailReady = await listen('thumbnail_ready', async (event: any) => {
    const { file_ids } = event.payload || {};
    if (!Array.isArray(file_ids) || file_ids.length === 0) return;
    if (fileList.value.length === 0) return;

    const readyIds = new Set(
      file_ids.map((id: any) => Number(id)).filter((id: number) => Number.isFinite(id) && id > 0)
    );
    if (readyIds.size === 0) return;

    const loadedFiles = fileList.value.filter(
      (file: any) => file && !file.isPlaceholder && readyIds.has(Number(file.id || 0))
    );
    if (loadedFiles.length === 0) return;

    // Only fetch thumbnails for files that don't already have one loaded
    const pendingFiles = loadedFiles.filter((f: any) => !f.thumbnail);
    if (pendingFiles.length === 0) return;

    getFileListThumb(pendingFiles, 0, 8, true);
  });

  // listen for external refresh requests (e.g. from folder context menu)
  unlistenRefreshContent = await listen('refresh-content', () => {
    updateContent();
  });

  unlistenTriggerNextAlbum = await listen('trigger-next-album', () => {
    processNextAlbum();
  });

  unlistenFilesDeleted = await listen('files-deleted', async (event: any) => {
    if (event?.payload?.source === 'content') return;
    const deletedIds = Array.isArray(event?.payload?.fileIds)
      ? event.payload.fileIds.map((id: any) => Number(id)).filter((id: number) => id > 0)
      : [];
    if (deletedIds.length === 0 || fileList.value.length === 0) return;

    if (tempViewMode.value === 'similar' || tempViewMode.value === 'album') {
      updateContent();
      return;
    }

    const deleteSet = new Set(deletedIds);
    let removedAny = false;
    for (let i = fileList.value.length - 1; i >= 0; i--) {
      if (deleteSet.has(fileList.value[i].id)) {
        fileList.value.splice(i, 1);
        removedAny = true;
      }
    }
    if (!removedAny) return;

    totalFileCount.value = fileList.value.length;
    totalFileSize.value = fileList.value.reduce((total, file) => total + file.size, 0);
    if (fileList.value.length === 0) {
      selectedItemIndex.value = -1;
    } else {
      selectedItemIndex.value = Math.min(selectedItemIndex.value, fileList.value.length - 1);
      if (selectedItemIndex.value < 0) selectedItemIndex.value = 0;
    }
    await updateSelectedImage(selectedItemIndex.value);
  });

  unlistenAlbumUpdated = await listen('album-updated', (event: any) => {
    const { albumId, name } = event.payload || {};
    const targetId = Number(albumId || 0);
    if (targetId <= 0 || !name) return;
    for (const file of fileList.value) {
      if (Number(file.album_id) === targetId) {
        file.album_name = name;
      }
    }
  });

  if (libConfig.index.albumQueue.length > 0 && libConfig.index.status === 1) {
    processNextAlbum();
  }

  // Face Indexing listeners
  unlistenFaceIndexProgress = await listenFaceIndexProgress((event: any) => {
    if (config.main.sidebarIndex === SIDEBAR.PERSON && fileList.value.length > 0) {
      clearContentRows();
      totalFileCount.value = 0;
      totalFileSize.value = 0;
      scrollPosition.value = 0;
      selectedItemIndex.value = -1;
    }
  });
});

function restoreInitialSelectionIfNeeded() {
  if (hasRestoredInitialSelection || pendingInitialSelectedIndex < 0 || fileList.value.length === 0) {
    return;
  }

  selectedItemIndex.value = Math.min(pendingInitialSelectedIndex, fileList.value.length - 1);
  hasRestoredInitialSelection = true;
  void updateSelectedImage(selectedItemIndex.value);
}

onBeforeUnmount(() => {
  if (scanStreamFlushTimer) {
    clearTimeout(scanStreamFlushTimer);
    scanStreamFlushTimer = null;
  }
  if (scanVisiblePrefetchTimer) {
    clearTimeout(scanVisiblePrefetchTimer);
    scanVisiblePrefetchTimer = null;
  }
  if (layoutRefreshTimer) {
    clearTimeout(layoutRefreshTimer);
    layoutRefreshTimer = null;
  }
  window.removeEventListener('keydown', handleLocalKeyDown);
  window.removeEventListener('keyup', handleLocalKeyUp);
  // unlisten
  unlistenImageViewer();
  if (unlistenImageEditor) unlistenImageEditor();
  if (unlistenKeydown) unlistenKeydown();
  if (unlistenTriggerNextAlbum) unlistenTriggerNextAlbum();
  if (unlistenIndexProgress) unlistenIndexProgress();
  if (unlistenIndexFinished) unlistenIndexFinished();
  if (unlistenThumbnailReady) unlistenThumbnailReady();
  if (unlistenRefreshContent) unlistenRefreshContent();
  if (unlistenFilesDeleted) unlistenFilesDeleted();
  if (unlistenAlbumUpdated) unlistenAlbumUpdated();
  if (unlistenFaceIndexProgress) unlistenFaceIndexProgress();
  if (unlistenPasteClipboard) unlistenPasteClipboard();
  if (domDragEnter) document.removeEventListener('dragenter', domDragEnter);
  if (domDragLeave) document.removeEventListener('dragleave', domDragLeave);
  if (domDragOver) document.removeEventListener('dragover', domDragOver);
  if (domDrop) document.removeEventListener('drop', domDrop);
  removeDragGhost();
});

/// watch appearance
watch(() => config.settings.appearance, (newAppearance) => {
  setTheme(newAppearance, newAppearance === 0 ? config.settings.lightTheme : config.settings.darkTheme);
});

/// watch light theme
watch(() => config.settings.lightTheme, (newLightTheme) => {
  setTheme(config.settings.appearance, newLightTheme);
});

/// watch dark theme
watch(() => config.settings.darkTheme, (newDarkTheme) => {
  setTheme(config.settings.appearance, newDarkTheme);
});

/// watch language
watch(() => config.settings.language, (newLanguage) => {
    locale.value = newLanguage; // update locale based on config.settings.language
    updateContent();
});

// Load tags when info panel is opened
watch(() => isInfoPanelOpen.value, async (newShow) => {
  if (newShow && selectedItemIndex.value >= 0 && selectedItemIndex.value < fileList.value.length) {
    const file = fileList.value[selectedItemIndex.value];
    if (file.has_tags && !file.tags) {
      // Load tags if has_tags is true but tags not yet loaded
      fileList.value[selectedItemIndex.value] = {
        ...file,
        tags: await getTagsForFile(file.id)
      };
    }
  }
});

watch(() => libConfig.index.status, (newStatus) => {
  if (newStatus === 1 && libConfig.index.albumQueue.length > 0 && !isRecoveryInProgress.value) {
    processNextAlbum();
  }
});

watch(() => libConfig.index.albumQueue.length, (newLength) => {
   if (newLength > 0 && libConfig.index.status === 0) {
       libConfig.index.status = 1; 
   }
});

/// watch image search params
watch(
  () => [
    libConfig.search.searchText,
    uiStore.searchCountRequestTick,
  ],
  () => {
    scheduleContentRefresh(() => {
      // Only update content if we are currently in the Image Search view
      if (config.main.sidebarIndex === SIDEBAR.SEARCH) {
        if (isDedupPanelOpen.value) config.rightPanel.show = false;
        refreshContentFromSelectionChange();
      }
    });
  }
);

/// watch for file list changes
watch(
  () => [
    config.main.sidebarIndex,      // toolbar index
    libConfig.activePane,          // main content or collection tray
    libConfig.collection.selectedId, // collection
    (libConfig.library as any).item, // library
    libConfig.library.smartId,
    libConfig.album.id, libConfig.album.folderId, libConfig.album.folderPath, libConfig.album.selected, // album
    libConfig.smartAlbum.type, libConfig.smartAlbum.id, JSON.stringify(libConfig.smartAlbums || []), // smart album
    uiStore.smartAlbumCountRequestTick,
    libConfig.rating.item, // rating
    config.search.fileType, config.search.sortType, config.search.sortOrder, // search and sort 
    config.settings.showSubfolderFiles,                                            // album folder view
    config.settings.folderSort, config.settings.calendarSort, config.settings.categorySort, // group sorting
    libConfig.person.id,                                                              // person
    config.calendar.isMonthly, libConfig.calendar.year, libConfig.calendar.month, libConfig.calendar.date, // calendar
    libConfig.tag.id,             // tag
    libConfig.location.admin1, libConfig.location.name,                               // location
    libConfig.camera.make, libConfig.camera.model,                                    // camera 
    config.camera.isCamera, (libConfig.camera as any).lensMake, (libConfig.camera as any).lensModel, // lens
  ], 
  (_newValues, oldValues) => {
    // Clear active adjustments when the file list changes to avoid unnecessary confirmation dialogs
    uiStore.clearActiveAdjustments();
    if (oldValues && isDedupPanelOpen.value) {
      config.rightPanel.show = false;
    }

    // If temp mode is active and query context changed, exit temp mode and refresh.
    if (tempViewMode.value === 'similar') {
      const params = currentImageSearchParams.value;
      if (params && (params.searchText || params.fileId)) {
        const requestId = ++currentContentRequestId;
        getImageSearchFileList(params.searchText, params.fileId || 0, requestId, false);
      }
      return;
    }
    if (tempViewMode.value === 'album') {
      updateContent();
      return;
    }
    // Skip other temp modes to prevent race conditions
    if (tempViewMode.value !== 'none') return;
    
    scheduleContentRefresh(() => {
      // Double check in case tempViewMode changed during setTimeout
      if (tempViewMode.value !== 'none') return;

      refreshContentFromSelectionChange();
    });
  }, 
  { immediate: true }
);

watch(
  () => Number(libConfig.album.activateTick || 0),
  () => {
    if (!showQuickView.value) return;
    showQuickView.value = false;
    stopSlideShow();
  }
);

// watch for selected item (not in select mode)
watch(() => selectedItemIndex.value, (newIndex, oldIndex) => {
  if(oldIndex >= 0 && oldIndex !== newIndex && fileList.value[oldIndex]?.rotate >= 360) {
    fileList.value[oldIndex].rotate %= 360;
  }
  void setLastSelectedItemIndex(Number(newIndex ?? -1));
  updateSelectedImage(newIndex);
});

// watch for show preview or layout change
watch(() => config.settings.grid.style, () => {
  updateSelectedImage(selectedItemIndex.value);
  stopSlideShow();
});

watch(
  () => Boolean(config.settings.grid.showFilmStrip),
  () => {
    if (!groupedModeActive.value && activeGroupBy.value <= 0) return;
    updateContent();
  },
);

function disablePreviewModes() {
  showQuickView.value = false;
  stopSlideShow();
}

watch(() => config.settings.grid.size, (newSize, oldSize) => {
  if (newSize === oldSize) return;
  if (showQuickView.value || isSlideShow.value) {
    disablePreviewModes();
  }
});

function toggleFilmstripView() {
  showQuickView.value = false;
  config.settings.grid.showFilmStrip = !config.settings.grid.showFilmStrip;
  void tauriEmit('settings-showFilmStrip-changed', config.settings.grid.showFilmStrip);
  if (config.settings.grid.showFilmStrip) {
    filmStripZoomFit.value = true;
  }
}

function cycleGridStyle() {
  showQuickView.value = false;
  // Cycle between card, tile, justified, and masonry.
  config.settings.grid.style = (config.settings.grid.style + 1) % 4;
  void tauriEmit('settings-gridStyle-changed', config.settings.grid.style);
}

// Track pending requests to avoid duplicates
const pendingRequests = new Set();

const getCurrentQueryCountAndSum = () => {
  if (currentQuerySource.value === 'search') {
    return Promise.resolve([
      currentSearchFileIds.value.length,
      fileList.value.reduce((total, file) => total + Number(file?.size || 0), 0),
    ]);
  }
  if (currentQuerySource.value === 'collection' && currentCollectionId.value) {
    return getCollectionCountAndSum(currentCollectionId.value, currentQueryParams.value);
  }
  if (currentQuerySource.value === 'smart' && currentSmartQueryParams.value) {
    return getSmartQueryCountAndSum(currentSmartQueryParams.value);
  }
  return getQueryCountAndSum(currentQueryParams.value);
};

const getCurrentQueryTimeLine = () => {
  if (currentQuerySource.value === 'collection' || currentQuerySource.value === 'search') {
    return Promise.resolve([]);
  }
  if (currentQuerySource.value === 'smart' && currentSmartQueryParams.value) {
    return getSmartQueryTimeLine(currentSmartQueryParams.value);
  }
  return getQueryTimeLine(currentQueryParams.value);
};

const getCurrentQueryFiles = (offset: number, limit: number) => {
  if (currentQuerySource.value === 'search') {
    return Promise.resolve(fileList.value.slice(offset, offset + limit));
  }
  if (currentQuerySource.value === 'collection' && currentCollectionId.value) {
    return getCollectionFiles(currentCollectionId.value, currentQueryParams.value, offset, limit);
  }
  if (currentQuerySource.value === 'smart' && currentSmartQueryParams.value) {
    return getSmartQueryFiles(currentSmartQueryParams.value, offset, limit);
  }
  return getQueryFiles(currentQueryParams.value, offset, limit);
};

const getCurrentGroupedQueryRows = (offset: number, limit: number) => {
  if (currentQuerySource.value === 'smart' && currentSmartQueryParams.value) {
    return getSmartGroupedQueryRows(getGroupingQueryParams(), offset, limit);
  }
  return getGroupedQueryRows(getGroupingQueryParams(), offset, limit);
};

const getCurrentQueryFileIds = () => {
  if (currentQuerySource.value === 'search') {
    return Promise.resolve([...currentSearchFileIds.value]);
  }
  if (currentQuerySource.value === 'collection' && currentCollectionId.value) {
    return getCollectionFileIds(currentCollectionId.value);
  }
  if (currentQuerySource.value === 'smart' && currentSmartQueryParams.value) {
    return getSmartQueryFileIds(getGroupingQueryParams());
  }
  return getQueryFileIds(getGroupingQueryParams());
};

const getCurrentQueryFilePosition = async (fileId: number) => {
  if (currentQuerySource.value === 'search') {
    return currentSearchFileIds.value.findIndex(id => Number(id) === Number(fileId));
  }
  if (currentQuerySource.value === 'collection' && currentCollectionId.value) {
    const ids = await getCollectionFileIds(currentCollectionId.value);
    return Array.isArray(ids) ? ids.findIndex((id: number) => Number(id) === Number(fileId)) : null;
  }
  if (currentQuerySource.value === 'smart' && currentSmartQueryParams.value) {
    return getSmartQueryFilePosition(currentSmartQueryParams.value, fileId);
  }
  return getQueryFilePosition(currentQueryParams.value, fileId);
};

async function fetchDataRange(start: number, end: number, reverse = false) {
  if (groupedModeActive.value) {
    return fetchGroupedRowsRange(start, end, reverse);
  }

  const requestId = currentContentRequestId;

  // Clamp range
  start = Math.max(0, start);
  end = Math.min(totalFileCount.value, end);
  
  if (start >= end || requestId !== currentContentRequestId) return;

  // Fetch in chunks
  // Keep normal browsing requests close to the actual viewport size. Large
  // fixed chunks make the initial "all files" view decode metadata and fetch
  // thumbnails for many items that are not yet visible.
  const chunkSize = Math.max(
    1,
    Math.min(selectionChunkSize.value, Math.max(32, visibleItemCount.value)),
  );
  const startChunk = Math.floor(start / chunkSize);
  const endChunk = Math.floor((end - 1) / chunkSize);
  const chunkPromises: Promise<void>[] = [];

  const chunkStartIdx = reverse ? endChunk : startChunk;
  const chunkEndIdx = reverse ? startChunk : endChunk;
  for (let i = chunkStartIdx; reverse ? i >= chunkEndIdx : i <= chunkEndIdx; reverse ? i-- : i++) {
    const chunkStart = i * chunkSize;
    const chunkEnd = Math.min(totalFileCount.value, chunkStart + chunkSize);
    
    // Check if this chunk still contains any placeholders.
    let chunkNeedsLoad = false;
    for (let idx = chunkStart; idx < chunkEnd; idx++) {
      if (isPendingFileItem(fileList.value[idx])) {
        chunkNeedsLoad = true;
        break;
      }
    }

    if (chunkNeedsLoad) {
      const key = `${requestId}:${chunkStart}-${chunkSize}`;
      if (pendingRequests.has(key)) {
        continue;
      }
      
      pendingRequests.add(key);
      
      const promise = getCurrentQueryFiles(chunkStart, chunkSize)
        .then(async (newFiles) => {
          if (requestId !== currentContentRequestId) return;
          if (newFiles) {
            // Update fileList and collect reactive references
            const filesToFetch = [];
            for (let j = 0; j < newFiles.length; j++) {
              if (requestId !== currentContentRequestId) return;
              if (chunkStart + j >= fileList.value.length) continue;

              const existingItem = fileList.value[chunkStart + j];
              // Avoid replacing already-loaded items; this prevents thumbnail flicker.
              if (existingItem && !existingItem.isPlaceholder) {
                continue;
              }

              // Preserve client-side state when upgrading placeholder -> real item.
              const fileId = Number(newFiles[j]?.id || 0);
              const isSelected = Boolean(existingItem?.isSelected) || (fileId > 0 && selectedFileIds.has(fileId));
              const rotate = existingItem ? (existingItem.rotate || 0) : 0;
              const thumbnail = existingItem?.thumbnail;

              fileList.value[chunkStart + j] = {
                ...newFiles[j],
                isSelected,
                rotate: rotate || newFiles[j].rotate || 0,
                thumbnail,
              };
              if (isSelected && fileId > 0 && !selectedFileIds.has(fileId)) {
                selectedFileIds.add(fileId);
                applySelectionDelta(fileList.value[chunkStart + j], 1);
              }

              // In scan streaming mode, total size starts at 0 and should grow
              // as placeholders are upgraded to real files.
              if (isScanStreamingMode.value) {
                totalFileSize.value += Number(newFiles[j]?.size || 0);
              }
              filesToFetch.push(fileList.value[chunkStart + j]);

              // Update ImageViewer if the selected file is loaded
              if (chunkStart + j === selectedItemIndex.value) {
                if (selectedItemIndex.value === 0) {
                  openImageViewer(selectedItemIndex.value, false, true);
                } else {
                  openImageViewer(selectedItemIndex.value, false);
                }
                updateSelectedImage(selectedItemIndex.value);
              }
            }
            // Trigger layout update as soon as metadata is available
            scheduleLayoutRefresh();
            // Fetch thumbnails for these files; await so the phase completes only when images are ready
            if (filesToFetch.length > 0) {
              if (reverse) filesToFetch.reverse();
              await getFileListThumb(filesToFetch);
            }
          }
        })
        .catch(err => {
            console.error(`Error fetching chunk ${key}:`, err);
        })
        .finally(() => {
          pendingRequests.delete(key);
        });

      chunkPromises.push(promise);
    } else {
        // console.log(`Chunk already loaded or invalid: ${chunkStart}`);
    }
  }

  if (chunkPromises.length > 0) {
    await Promise.all(chunkPromises);
  }
}

async function fetchGroupedRowsRange(start: number, end: number, reverse = false) {
  const requestId = currentContentRequestId;

  start = Math.max(0, start);
  end = Math.min(totalRowCount.value, end);
  if (start >= end || requestId !== currentContentRequestId) return;

  const chunkSize = selectionChunkSize.value;
  const startChunk = Math.floor(start / chunkSize);
  const endChunk = Math.floor((end - 1) / chunkSize);
  const chunkPromises: Promise<void>[] = [];

  const chunkStartIdx = reverse ? endChunk : startChunk;
  const chunkEndIdx = reverse ? startChunk : endChunk;
  for (let i = chunkStartIdx; reverse ? i >= chunkEndIdx : i <= chunkEndIdx; reverse ? i-- : i++) {
    const chunkStart = i * chunkSize;
    const chunkEnd = Math.min(totalRowCount.value, chunkStart + chunkSize);
    let chunkNeedsLoad = false;
    for (let idx = chunkStart; idx < chunkEnd; idx++) {
      if (groupedRows.value[idx]?.isPlaceholder) {
        chunkNeedsLoad = true;
        break;
      }
    }
    if (!chunkNeedsLoad) continue;

    const key = `${requestId}:grouped:${chunkStart}-${chunkSize}`;
    if (pendingRequests.has(key)) continue;
    pendingRequests.add(key);

    const promise = getCurrentGroupedQueryRows(chunkStart, chunkEnd - chunkStart)
      .then(async (result) => {
        if (requestId !== currentContentRequestId) return;
        const normalized = normalizeGroupedRowsResult(result);
        const filesToFetch = applyGroupedRows(normalized.rows, chunkStart);

        scheduleLayoutRefresh();
        if (filesToFetch.length > 0) {
          if (reverse) filesToFetch.reverse();
          await getFileListThumb(filesToFetch);
        }
      })
      .catch(err => {
        console.error(`Error fetching grouped chunk ${key}:`, err);
      })
      .finally(() => {
        pendingRequests.delete(key);
      });

    chunkPromises.push(promise);
  }

  if (chunkPromises.length > 0) {
    await Promise.all(chunkPromises);
  }
}

function normalizeGridRow(row: any, rowIndex: number) {
  if (!row) {
    return {
      id: `grouped-missing-${rowIndex}`,
      isPlaceholder: true,
    };
  }
  if (row?.type === 'group') {
    const groupId = String(row.group_id ?? row.groupId ?? row.id ?? `group-${rowIndex}`);
    return {
      ...row,
      type: 'group',
      id: row.row_id ?? row.rowId ?? `group-row-${groupId}`,
      group_id: groupId,
      label: formatGroupLabel(row.label ?? ''),
      count: Number(row.count || 0),
    };
  }

  const file = row?.file ?? row;
  const fileIndex = Number(row?.file_index ?? row?.fileIndex ?? rowIndex);
  const groupId = row?.group_id ?? row?.groupId;
  return {
    ...row,
    type: 'item',
    id: row?.row_id ?? row?.rowId ?? `item-row-${file?.id ?? rowIndex}`,
    group_id: groupId ? String(groupId) : '',
    file_index: fileIndex,
    file,
  };
}

function applyGroupedRows(rows: any[], rowOffset: number) {
  const filesToFetch: any[] = [];
  for (let j = 0; j < rows.length; j++) {
    const rowIndex = rowOffset + j;
    if (rowIndex >= groupedRows.value.length) continue;
    const row = normalizeGridRow(rows[j], rowIndex);

    if (isGroupRow(row)) {
      groupedRows.value[rowIndex] = row;
      const groupId = String(row.group_id || '');
      if (groupId) {
        groupMetaMap.set(groupId, {
          count: Number(row.count || 0),
          size: Number(row.size || 0),
        });
        if (groupSelectedCountMap.value[groupId] === undefined) {
          updateGroupSelectedCount(groupId, 0);
        }
        if (groupSelectedSizeMap.value[groupId] === undefined) {
          updateGroupSelectedSize(groupId, 0);
        }
      }
      continue;
    }

    if (isItemRow(row)) {
      const fileIndex = Number(row.file_index);
      const file = row.file;
      if (!Number.isFinite(fileIndex) || fileIndex < 0 || fileIndex >= fileList.value.length || !file) {
        groupedRows.value[rowIndex] = row;
        continue;
      }

      const existingItem = fileList.value[fileIndex];
      const fileId = Number(file?.id || 0);
      const isSelected = Boolean(existingItem?.isSelected) || (fileId > 0 && selectedFileIds.has(fileId));
      const rotate = existingItem ? (existingItem.rotate || 0) : 0;
      const thumbnail = existingItem?.thumbnail;
      const nextFile = {
        ...file,
        isSelected,
        rotate: rotate || file.rotate || 0,
        thumbnail,
      };
      fileList.value[fileIndex] = nextFile;
      row.file = fileList.value[fileIndex];
      groupedRows.value[rowIndex] = row;
      if (isSelected && fileId > 0 && !selectedFileIds.has(fileId)) {
        selectedFileIds.add(fileId);
        applySelectionDelta(fileList.value[fileIndex], 1);
      }

      const groupId = String(row.group_id || '');
      if (groupId && fileId) fileIdToGroupId.set(fileId, groupId);
      filesToFetch.push(fileList.value[fileIndex]);

      if (fileIndex === selectedItemIndex.value) {
        openImageViewer(selectedItemIndex.value, false, selectedItemIndex.value === 0);
        updateSelectedImage(selectedItemIndex.value);
      }
    }
  }
  return filesToFetch;
}

async function initializeGroupedFileList(requestId: number) {
  if (!isGroupingSupportedForCurrentView()) {
    resetGroupingState();
    return false;
  }

  if (activeGroupBy.value === GROUP.FOLDER) {
    await ensureFolderGroupRoots();
  }

  const groupedResult = await getCurrentGroupedQueryRows(0, selectionChunkSize.value);
  if (requestId !== currentContentRequestId) return true;
  if (!groupedResult) {
    resetGroupingState();
    return false;
  }

  const normalized = normalizeGroupedRowsResult(groupedResult);
  clearSelectionForFileListUpdate();
  groupedModeActive.value = true;
  groupedTimelineGroups.value = normalized.groups;
  totalFileCount.value = normalized.totalItemCount;
  totalRowCount.value = normalized.totalRowCount;
  totalFileSize.value = normalized.totalSize;
  scrollPosition.value = 0;
  timelineData.value = [];
  groupedRows.value = Array.from({ length: totalRowCount.value }).map((_, i) => ({
    id: `grouped-ph-${i}`,
    isPlaceholder: true,
  }));

  for (const group of normalized.groups) {
    const rowIndex = Number(group.rowIndex || 0);
    if (rowIndex < 0 || rowIndex >= groupedRows.value.length) continue;
    groupedRows.value[rowIndex] = createGroupHeaderRow(group);
    if (group.groupId) {
      groupMetaMap.set(group.groupId, {
        count: Number(group.count || 0),
        size: Number(group.size || 0),
      });
      updateGroupSelectedCount(group.groupId, 0);
      updateGroupSelectedSize(group.groupId, 0);
    }
  }

  fileList.value = Array.from({ length: totalFileCount.value }).map((_, i) => ({
    id: `ph-${i}`,
    isPlaceholder: true,
    name: '',
    size: 0,
  }));

  const initialFiles = applyGroupedRows(normalized.rows, 0);
  if (initialFiles.length > 0) {
    await getFileListThumb(initialFiles);
  }
  await nextTick();
  gridViewRef.value?.refreshLayout?.();
  markDedupSourceUpdated(requestId);
  restoreInitialSelectionIfNeeded();
  restoreScrollAfterRefresh();
  if (totalFileCount.value === 0) {
    openImageViewer(0, false, true);
  }
  lastVisibleRange = { start: -1, end: -1 };
  visibleRangeSeqId++;
  return true;
}

async function hydrateRangeForSelection(startIndex: number, endIndex: number) {
  if (fileList.value.length === 0 || endIndex <= startIndex) return true;
  const requestId = currentContentRequestId;
  const chunkSize = selectionChunkSize.value;
  const cappedStart = Math.max(0, Math.min(startIndex, fileList.value.length));
  const cappedEnd = Math.max(cappedStart, Math.min(endIndex, fileList.value.length));
  let hydratedPlaceholders = false;
  isProcessing.value = true;

  try {
    const firstChunkStart = Math.floor(cappedStart / chunkSize) * chunkSize;
    for (let chunkStart = firstChunkStart; chunkStart < cappedEnd; chunkStart += chunkSize) {
      if (requestId !== currentContentRequestId) return false;
      const chunkEnd = Math.min(cappedEnd, chunkStart + chunkSize);
      const selectionStart = Math.max(cappedStart, chunkStart);
      const needsLoad = fileList.value
        .slice(selectionStart, chunkEnd)
        .some(isPendingFileItem);
      if (!needsLoad) continue;

      const loadedFiles = await getCurrentQueryFiles(chunkStart, chunkSize);
      if (!loadedFiles || loadedFiles.length === 0) return false;
      hydratedPlaceholders = true;

      for (let i = 0; i < loadedFiles.length; i++) {
        const targetIndex = chunkStart + i;
        if (targetIndex >= fileList.value.length || targetIndex >= cappedEnd) break;
        if (targetIndex < cappedStart) continue;
        const existingItem = fileList.value[targetIndex];
        const fileId = Number(loadedFiles[i]?.id || 0);
        const isSelected = Boolean(existingItem?.isSelected) || (fileId > 0 && selectedFileIds.has(fileId));
        fileList.value[targetIndex] = {
          ...loadedFiles[i],
          isSelected,
          rotate: existingItem?.rotate || loadedFiles[i].rotate || 0,
        };
        if (isSelected && fileId > 0 && !selectedFileIds.has(fileId)) {
          selectedFileIds.add(fileId);
          applySelectionDelta(fileList.value[targetIndex], 1);
        }
      }
      if (fileList.value.slice(selectionStart, chunkEnd).some(isPendingFileItem)) {
        return false;
      }
    }
    return requestId === currentContentRequestId;
  } catch (err) {
    console.error('hydrateRangeForSelection error:', err);
    return false;
  } finally {
    isProcessing.value = false;
    if (hydratedPlaceholders) {
      scheduleLayoutRefresh();
    }
  }
}

async function hydrateSelectedFilesByIds() {
  if (selectedFileIds.size === 0) return getActionableSelectedItems();

  const loadedSelectedItems = getActionableSelectedItems();
  const loadedIds = new Set(
    loadedSelectedItems
      .map(item => Number(item.id || 0))
      .filter(id => id > 0),
  );
  const missingIds = Array.from(selectedFileIds).filter(id => !loadedIds.has(id));
  if (missingIds.length === 0) return loadedSelectedItems;

  const hydratedItems: any[] = [...loadedSelectedItems];
  const missingIdSet = new Set(missingIds);
  const chunkSize = Math.max(1, selectionChunkSize.value);
  isProcessing.value = true;
  try {
    for (let start = 0; start < totalFileCount.value && missingIdSet.size > 0; start += chunkSize) {
      const files = await getCurrentQueryFiles(start, chunkSize);
      if (!Array.isArray(files)) return null;

      for (const file of files) {
        const fileId = Number(file?.id || 0);
        if (fileId <= 0 || !missingIdSet.has(fileId)) continue;
        hydratedItems.push({
          ...file,
          isSelected: true,
          rotate: file.rotate || 0,
        });
        missingIdSet.delete(fileId);
      }
    }
    return missingIdSet.size === 0 ? hydratedItems : null;
  } catch (err) {
    console.error('hydrateSelectedFilesByIds error:', err);
    return null;
  } finally {
    isProcessing.value = false;
  }
}

async function getActionableSelectedItemsForAction() {
  if (selectedFileIds.size > getActionableSelectedItems().length) {
    const hydratedItems = await hydrateSelectedFilesByIds();
    if (!hydratedItems) {
      toast.error(t('info_panel.selection_load_failed'));
      return null;
    }
    return hydratedItems;
  }

  const hasSelectedPlaceholders = fileList.value.some(
    item => item?.isSelected && item?.isPlaceholder,
  );
  if (hasSelectedPlaceholders && !await hydrateRangeForSelection(0, fileList.value.length)) {
    toast.error(t('info_panel.selection_load_failed'));
    return null;
  }
  return getActionableSelectedItems();
}

async function getSelectedItemsForClipboard(limit = 10) {
  if (selectedFileIds.size > 0 && getActionableSelectedItems().length < Math.min(selectedFileIds.size, limit)) {
    const hydratedItems = await hydrateSelectedFilesByIds();
    if (!hydratedItems) {
      toast.error(t('info_panel.selection_load_failed'));
      return null;
    }
    return hydratedItems.slice(0, limit);
  }

  const selectedItems: any[] = [];
  const chunkSize = Math.max(1, selectionChunkSize.value);

  for (let chunkStart = 0; chunkStart < fileList.value.length && selectedItems.length < limit; chunkStart += chunkSize) {
    const chunkEnd = Math.min(fileList.value.length, chunkStart + chunkSize);
    const chunk = fileList.value.slice(chunkStart, chunkEnd);
    if (chunk.some(item => item?.isSelected && item?.isPlaceholder)) {
      if (!await hydrateRangeForSelection(chunkStart, chunkEnd)) {
        toast.error(t('info_panel.selection_load_failed'));
        return null;
      }
    }
    for (let index = chunkStart; index < chunkEnd && selectedItems.length < limit; index++) {
      const item = fileList.value[index];
      if (item?.isSelected && isRealFileItem(item)) {
        selectedItems.push(item);
      }
    }
  }

  return selectedItems;
}

// Track last visible range to avoid redundant fetches
let lastVisibleRange = { start: -1, end: -1 };
let visibleRangeSeqId = 0;

async function fetchMissingVisibleThumbnails(startIndex: number, endIndex: number) {
  const rows = groupedModeActive.value
    ? gridRows.value.slice(startIndex, endIndex + 1)
    : fileList.value.slice(startIndex, endIndex + 1);
  const seen = new Set<number>();
  const files = rows
    .map((row: any) => isItemRow(row) ? row.file : row)
    .filter((file: any) => {
      if (!isRealFileItem(file) || file.thumbnail) return false;
      const fileId = Number(file.id || 0);
      if (fileId <= 0 || seen.has(fileId)) return false;
      seen.add(fileId);
      return true;
    });

  if (files.length > 0) {
    await getFileListThumb(files);
  }
}

function handleVisibleRangeUpdate({ startIndex, endIndex }: { startIndex: number, endIndex: number }) {
  // Skip if the range hasn't changed significantly
  if (lastVisibleRange.start === startIndex && lastVisibleRange.end === endIndex) {
    void fetchMissingVisibleThumbnails(startIndex, endIndex);
    return;
  }

  lastVisibleRange = { start: startIndex, end: endIndex };

  const buffer = Math.max(40, Math.min(visibleItemCount.value, 120));
  const seqId = ++visibleRangeSeqId;

  // Phase 1: viewport thumbnails first (immediately visible)
  fetchDataRange(startIndex, endIndex + 1).then(() => {
    if (seqId !== visibleRangeSeqId) return;
    fetchMissingVisibleThumbnails(startIndex, endIndex).then(() => {
      if (seqId !== visibleRangeSeqId) return;

      // Phase 2: below viewport (most likely scroll direction)
      fetchDataRange(endIndex + 1, endIndex + 1 + buffer).then(() => {
        if (seqId !== visibleRangeSeqId) return;

        // Phase 3: above viewport (least likely, reverse: load closest to viewport first)
        fetchDataRange(Math.max(0, startIndex - buffer), startIndex, true);
      });
    });
  });
}

// get file list 
async function getFileList(
  {
    searchFileType = config.search.fileType, 
    sortType = config.search.sortType, 
    sortOrder = config.search.sortOrder, 
    searchFileName = '', 
    searchAllSubfolders = '',
    searchFolder = '', 
    startDate = 0,
    endDate = 0,
    calendarSort = config.settings.calendarSort,
    folderSort = config.settings.folderSort,
    categorySort = config.settings.categorySort,
    make = '',
    model = '', 
    lensMake = '',
    lensModel = '',
    locationAdmin1 = '', 
    locationName = '', 
    isFavorite = false, 
    rating = -1,
    tagId = 0,
    personId = 0
  } = {},
  requestId: number, 
) { 
  currentQuerySource.value = 'query';
  currentSmartQueryParams.value = null;
  currentCollectionId.value = null;
  currentSearchFileIds.value = [];

  // Update current query params with all fields
  currentQueryParams.value = {
    searchFileType,
    sortType,
    sortOrder,
    searchFileName,
    searchAllSubfolders,
    searchFolder,
    startDate,
    endDate,
    calendarSort,
    folderSort,
    categorySort,
    make,
    model,
    lensMake,
    lensModel,
    locationAdmin1,
    locationName,
    isFavorite,
    rating,
    tagId,
    personId,
  };

  // Set loading state
  isLoading.value = true;

  try {
    if (await initializeGroupedFileList(requestId)) return;

    // Get count and sum first
    const result = await getCurrentQueryCountAndSum();
    
    // Check if the request is still valid. 
    if (requestId !== currentContentRequestId) {
      return;
    }

    if (result) {
      clearSelectionForFileListUpdate();
      totalFileCount.value = result[0];
      totalFileSize.value = result[1];
      
      // Get timeline data for date-based sorts
      getCurrentQueryTimeLine().then(data => {
        if (requestId === currentContentRequestId) {
          timelineData.value = data;
        }
      });
      
      // Initialize fileList with placeholders
      fileList.value = createVirtualFileSlots(totalFileCount.value);
      markDedupSourceUpdated(requestId);
      restoreInitialSelectionIfNeeded();
      restoreScrollAfterRefresh();
      if (totalFileCount.value === 0) {
        openImageViewer(0, false, true);
      }
      
      // Reset visible range tracking when changing views
      lastVisibleRange = { start: -1, end: -1 };
      visibleRangeSeqId++;
    } else {
      clearSelectionForFileListUpdate();
      clearContentRows();
      markDedupSourceUpdated(requestId);
      openImageViewer(0, false, true);
    }
  } catch (err) {
    console.error('getFileList error:', err);
    if (requestId === currentContentRequestId) {
      clearSelectionForFileListUpdate();
      clearContentRows();
      markDedupSourceUpdated(requestId);
      openImageViewer(0, false, true);
    }
  } finally {
    // Only clear loading state if this is still the active request
    if (requestId === currentContentRequestId) {
      isLoading.value = false;
      hasLoadedInitialResult.value = true;
      contentReady.value = true;
    }
  }
}

async function getCollectionFileList(collectionId: number, requestId: number) {
  currentQuerySource.value = 'collection';
  currentCollectionId.value = collectionId;
  currentSmartQueryParams.value = null;
  currentSearchFileIds.value = [];
  currentQueryParams.value = {
    ...currentQueryParams.value,
    searchFileType: config.search.fileType,
    sortType: config.search.sortType,
    sortOrder: config.search.sortOrder,
    searchFileName: '',
    searchAllSubfolders: '',
    searchFolder: '',
    startDate: 0,
    endDate: 0,
    calendarSort: config.settings.calendarSort,
    folderSort: config.settings.folderSort,
    categorySort: config.settings.categorySort,
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
  };

  isLoading.value = true;

  try {
    const result = await getCurrentQueryCountAndSum();
    if (requestId !== currentContentRequestId) return;

    if (result) {
      clearSelectionForFileListUpdate();
      resetGroupingState();
      totalFileCount.value = result[0];
      totalFileSize.value = result[1];
      timelineData.value = [];

      fileList.value = createVirtualFileSlots(totalFileCount.value);
      markDedupSourceUpdated(requestId);
      restoreInitialSelectionIfNeeded();
      restoreScrollAfterRefresh();
      if (totalFileCount.value === 0) {
        openImageViewer(0, false, true);
      }

      lastVisibleRange = { start: -1, end: -1 };
      visibleRangeSeqId++;
    } else {
      clearSelectionForFileListUpdate();
      clearContentRows();
      markDedupSourceUpdated(requestId);
      openImageViewer(0, false, true);
    }
  } catch (err) {
    console.error('getCollectionFileList error:', err);
    if (requestId === currentContentRequestId) {
      clearSelectionForFileListUpdate();
      clearContentRows();
      markDedupSourceUpdated(requestId);
      openImageViewer(0, false, true);
    }
  } finally {
    if (requestId === currentContentRequestId) {
      isLoading.value = false;
      hasLoadedInitialResult.value = true;
      contentReady.value = true;
    }
  }
}

async function getSmartFileList(smartAlbum: any, requestId: number) {
  const query = smartAlbum?.query;
  const rules = Array.isArray(query?.rules) ? query.rules : [];
  if (!query || rules.length === 0) {
    if (uiStore.smartAlbumCountRequestedFor === String(smartAlbum?.id)) {
      updateSmartAlbumCount(smartAlbum.id, 0);
      uiStore.smartAlbumCountRequestedFor = null;
    }
    showEmptyContent(requestId);
    return;
  }

  currentQuerySource.value = 'smart';
  currentCollectionId.value = null;
  currentSearchFileIds.value = [];
  currentSmartQueryParams.value = {
    version: Number(query.version || 1),
    match: query.match === 'any' ? 'any' : 'all',
    rules,
    sortType: Number(smartAlbum?.sort?.type ?? 0),
    sortOrder: Number(smartAlbum?.sort?.order ?? 1),
    folderSort: Number(config.settings.folderSort || 0),
    calendarSort: Number(config.settings.calendarSort || 0),
    categorySort: Number(config.settings.categorySort || 0),
    groupBy: Number(smartAlbum?.group?.type ?? GROUP.NONE),
  };
  void updateSmartAlbumCover(smartAlbum, requestId);

  isLoading.value = true;

  try {
    if (await initializeGroupedFileList(requestId)) {
      if (
        requestId === currentContentRequestId &&
        uiStore.smartAlbumCountRequestedFor === String(smartAlbum?.id)
      ) {
        updateSmartAlbumCount(smartAlbum.id, totalFileCount.value);
        uiStore.smartAlbumCountRequestedFor = null;
      }
      return;
    }

    const result = await getCurrentQueryCountAndSum();
    if (requestId !== currentContentRequestId) return;

    if (result) {
      if (uiStore.smartAlbumCountRequestedFor === String(smartAlbum?.id)) {
        updateSmartAlbumCount(smartAlbum.id, Number(result[0] || 0));
        uiStore.smartAlbumCountRequestedFor = null;
      }
      clearSelectionForFileListUpdate();
      totalFileCount.value = result[0];
      totalFileSize.value = result[1];

      getCurrentQueryTimeLine().then(data => {
        if (requestId === currentContentRequestId) {
          timelineData.value = data;
        }
      });

      fileList.value = createVirtualFileSlots(totalFileCount.value);
      markDedupSourceUpdated(requestId);
      restoreInitialSelectionIfNeeded();
      restoreScrollAfterRefresh();
      if (totalFileCount.value === 0) {
        openImageViewer(0, false, true);
      }

      lastVisibleRange = { start: -1, end: -1 };
      visibleRangeSeqId++;
    } else {
      clearSelectionForFileListUpdate();
      clearContentRows();
      markDedupSourceUpdated(requestId);
      openImageViewer(0, false, true);
    }
  } catch (err) {
    console.error('getSmartFileList error:', err);
    if (requestId === currentContentRequestId) {
      clearSelectionForFileListUpdate();
      clearContentRows();
      markDedupSourceUpdated(requestId);
      openImageViewer(0, false, true);
    }
  } finally {
    if (requestId === currentContentRequestId) {
      isLoading.value = false;
      hasLoadedInitialResult.value = true;
      contentReady.value = true;
    }
  }
}

async function updateSmartAlbumCover(smartAlbum: any, requestId: number) {
  const result = await getSmartQueryFiles(currentSmartQueryParams.value, 0, 1);
  if (
    requestId !== currentContentRequestId ||
    libConfig.smartAlbum.type !== 'custom' ||
    String(libConfig.smartAlbum.id) !== String(smartAlbum?.id)
  ) return;

  const firstFile = Array.isArray(result) ? result[0] : result?.files?.[0];
  const coverFileId = Number(firstFile?.id || 0) || null;
  const albums = [...(libConfig.smartAlbums || [])];
  const albumIndex = albums.findIndex((album: any) => String(album.id) === String(smartAlbum?.id));
  if (albumIndex < 0 || Number(albums[albumIndex].coverFileId || 0) === Number(coverFileId || 0)) return;

  albums[albumIndex] = {
    ...albums[albumIndex],
    coverFileId,
  };
  libConfig.smartAlbums = albums;
}

function updateSmartAlbumCount(smartAlbumId: string | number, count: number) {
  const albums = [...(libConfig.smartAlbums || [])];
  const albumIndex = albums.findIndex((album: any) => String(album.id) === String(smartAlbumId));
  const currentCount = albums[albumIndex]?.count;
  if (albumIndex < 0 || (currentCount !== null && currentCount !== undefined && Number(currentCount) === count)) return;

  albums[albumIndex] = {
    ...albums[albumIndex],
    count,
  };
  libConfig.smartAlbums = albums;
}

function updateSearchHistoryCount(searchText: string, count: number) {
  const history = libConfig.search.searchHistory as any[];
  const historyIndex = history.findIndex((item: any) =>
    (typeof item === 'string' ? item : item?.text) === searchText
  );
  if (historyIndex < 0) return;

  const item = history[historyIndex];
  if (typeof item !== 'string' && item.count !== null && item.count !== undefined && Number(item.count) === count) return;
  history[historyIndex] = {
    text: typeof item === 'string' ? item : item.text,
    fileId: typeof item === 'string' ? null : item.fileId ?? null,
    count,
  };
}

async function getImageSearchFileList(
  searchText: string,
  fileId: number,
  requestId: number,
  updateHistory = true,
  thresholdOverride?: number,
) {
  currentQuerySource.value = 'search';
  currentSmartQueryParams.value = null;
  currentImageSearchParams.value = {
    searchText,
    fileId,
    threshold: thresholdOverride ?? config.imageSearchThresholds[config.settings.imageSearch.thresholdIndex],
    limit: config.settings.imageSearch.limit,
    fileType: Number(config.search.fileType || 0),
  };
  currentCollectionId.value = null;
  currentSearchFileIds.value = [];

  // set loading state
  isLoading.value = true;
  timelineData.value = []; // reset timeline data

  try {
    // Check if the request is still valid. 
    if (requestId !== currentContentRequestId) {
      return;
    }

    const result = await searchSimilarImages(currentImageSearchParams.value);
    if (requestId !== currentContentRequestId) return;

    if (result) {
      clearSelectionForFileListUpdate();
      resetGroupingState();
      fileList.value = preserveLoadedThumbnails(result);
      currentSearchFileIds.value = fileList.value
        .map(file => Number(file.id))
        .filter(id => Number.isFinite(id) && id > 0);
      totalFileCount.value = fileList.value.length;
      totalFileSize.value = fileList.value.reduce((total, file) => total + file.size, 0);
      markDedupSourceUpdated(requestId);
      restoreInitialSelectionIfNeeded();
      openImageViewer(0, false, true);

      // Reset visible range tracking when changing views
      lastVisibleRange = { start: -1, end: -1 };
      visibleRangeSeqId++;

      // Update search history with the first result's file_id
      if (updateHistory && searchText && result.length > 0) {
        const history = libConfig.search.searchHistory as any[];
        const index = history.findIndex((item: any) => {
            const text = typeof item === 'string' ? item : item.text;
            return text === searchText;
        });

        if (index !== -1) {
            const item = history[index];
            const firstId = result[0].id;

            // Always update the history item's fileId to the latest first result
            if (typeof item === 'string') {
              history[index] = { text: item, fileId: firstId };
            } else {
              item.fileId = firstId;
            }
        }
      }

      // Thumbnail loading is driven by visible-range updates. Avoid requesting
      // every semantic-search result before the user scrolls to it.
    } else {
      clearSelectionForFileListUpdate();
      clearContentRows();
      totalFileCount.value = 0;
      totalFileSize.value = 0;
      markDedupSourceUpdated(requestId);
    }
  } catch (err) {
    console.error('getImageSearchFileList error:', err);
    if (requestId === currentContentRequestId) {
      clearSelectionForFileListUpdate();
      clearContentRows();
      totalFileCount.value = 0;
      totalFileSize.value = 0;
      markDedupSourceUpdated(requestId);
    }
  } finally {
    // Only clear loading state if this is still the active request
    if (requestId === currentContentRequestId) {
      isLoading.value = false;
      hasLoadedInitialResult.value = true;
      contentReady.value = true;
    }
  }
}

async function getUnifiedSearchFileList(searchText: string, requestId: number) {
  const filenameParams = {
    searchFileType: config.search.fileType,
    sortType: 3,
    sortOrder: 0,
    searchFileName: searchText,
    searchAllSubfolders: '',
    searchFolder: '',
    startDate: 0,
    endDate: 0,
    calendarSort: config.settings.calendarSort,
    folderSort: config.settings.folderSort,
    categorySort: config.settings.categorySort,
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
    groupBy: 0,
  };
  currentQueryParams.value = filenameParams;
  currentQuerySource.value = 'search';
  currentSmartQueryParams.value = null;
  currentCollectionId.value = null;
  currentSearchFileIds.value = [];
  currentImageSearchParams.value = {
    searchText,
    fileId: 0,
    threshold: config.imageSearchThresholds[config.settings.imageSearch.thresholdIndex],
    limit: config.settings.imageSearch.limit,
    fileType: Number(config.search.fileType || 0),
  };
  isLoading.value = true;
  timelineData.value = [];

  try {
    const [filenameResult, filenameIdResult, visualResult] = await Promise.all([
      getQueryFiles(filenameParams, 0, 10).catch(error => {
        console.error('Filename search failed:', error);
        return [];
      }),
      getQueryFileIds(filenameParams).catch(error => {
        console.error('Filename ID search failed:', error);
        return [];
      }),
      searchSimilarImages(currentImageSearchParams.value).catch(error => {
        console.error('Visual search failed:', error);
        return [];
      }),
    ]);
    if (requestId !== currentContentRequestId) return;

    const filenameMatches = Array.isArray(filenameResult) ? filenameResult : [];
    const allFilenameIds = Array.isArray(filenameIdResult) ? filenameIdResult : [];
    const filenameHasMore = allFilenameIds.length > 10;
    const filenameIds = new Set(allFilenameIds.map((id: any) => Number(id)));
    const visualMatches = (Array.isArray(visualResult) ? visualResult : [])
      .filter((file: any) => !filenameIds.has(Number(file.id)));
    const files = preserveLoadedThumbnails([...filenameMatches, ...visualMatches]);

    if (uiStore.searchCountRequestedFor === searchText) {
      updateSearchHistoryCount(searchText, files.length);
      uiStore.searchCountRequestedFor = null;
    }

    clearSelectionForFileListUpdate();
    resetGroupingState();
    fileList.value = files;
    currentSearchFileIds.value = fileList.value
      .map(file => Number(file.id))
      .filter(id => Number.isFinite(id) && id > 0);
    totalFileCount.value = fileList.value.length;
    totalFileSize.value = fileList.value.reduce((total, file) => total + Number(file.size || 0), 0);

    const groups = [
      filenameMatches.length > 0
        ? {
            id: 'search-filename',
            label: localeMsg.value.search.filename_matches,
            icon: IconFile,
            files: filenameMatches,
            countLabel: filenameHasMore ? '10+' : String(filenameMatches.length),
          }
        : null,
      visualMatches.length > 0
        ? {
            id: 'search-visual',
            label: localeMsg.value.search.semantic_matches,
            icon: IconSparkles,
            files: visualMatches,
            countLabel: String(visualMatches.length),
          }
        : null,
    ].filter(Boolean) as Array<{ id: string; label: string; icon: any; files: any[]; countLabel: string }>;

    let fileIndex = 0;
    for (const group of groups) {
      const rowIndex = groupedRows.value.length;
      const size = group.files.reduce((total, file) => total + Number(file.size || 0), 0);
      groupedRows.value.push({
        type: 'group',
        id: `group-row-${group.id}`,
        group_id: group.id,
        label: group.label,
        icon: group.icon,
        count: group.files.length,
        countLabel: group.countLabel,
        size,
      });
      groupedTimelineGroups.value.push({
        groupId: group.id,
        label: group.label,
        icon: group.icon,
        count: group.files.length,
        size,
        rowIndex,
      });
      groupMetaMap.set(group.id, { count: group.files.length, size });
      groupFileIdsCache.set(group.id, group.files.map(file => Number(file.id)));

      for (let groupFileIndex = 0; groupFileIndex < group.files.length; groupFileIndex++) {
        // Always reference the reactive object stored in fileList. Thumbnail
        // updates made through the raw search result do not trigger a rerender.
        const loadedFile = fileList.value[fileIndex];
        const id = Number(loadedFile?.id || 0);
        if (id > 0) fileIdToGroupId.set(id, group.id);
        groupedRows.value.push({
          type: 'item',
          id: `search-item-${group.id}-${id}`,
          group_id: group.id,
          file_index: fileIndex,
          file: loadedFile,
        });
        fileIndex++;
      }
    }

    groupedModeActive.value = groups.length > 0;
    totalRowCount.value = groupedRows.value.length;
    selectedItemIndex.value = fileList.value.length > 0 ? 0 : -1;
    scrollPosition.value = 0;
    markDedupSourceUpdated(requestId);
    openImageViewer(0, false, true);

    const firstHistoryFile = visualMatches[0] || filenameMatches[0];
    if (firstHistoryFile) {
      const history = libConfig.search.searchHistory as any[];
      const historyIndex = history.findIndex((item: any) =>
        (typeof item === 'string' ? item : item.text) === searchText
      );
      if (historyIndex >= 0) {
        const item = history[historyIndex];
        history[historyIndex] = {
          text: typeof item === 'string' ? item : item.text,
          fileId: Number(firstHistoryFile.id),
          count: typeof item === 'string' ? null : item.count ?? null,
        };
      }
    }

    if (fileList.value.length > 0) {
      getFileListThumb(fileList.value);
    }
  } finally {
    if (requestId === currentContentRequestId) {
      isLoading.value = false;
      hasLoadedInitialResult.value = true;
      contentReady.value = true;
    }
  }
}

let contentUpdateSeq = 0;

async function updateContent(force = false) {
  const updateSeq = ++contentUpdateSeq;
  const newIndex = config.main.sidebarIndex;
  const isCurrentAlbumIndexing =
    newIndex === SIDEBAR.ALBUM &&
    !!libConfig.album.id &&
    libConfig.album.id > 0 &&
    libConfig.index.albumQueue.includes(libConfig.album.id);

  if (
    !force &&
    isScanStreamingMode.value &&
    isCurrentAlbumIndexing &&
    nextAlbumId > 0
  ) {
    enterScanStreamingMode(nextAlbumId);
    queueScanStreamingPull(nextAlbumId, Number(libConfig.index.discovered || 0));
    return;
  }

  // Reset temp view mode on any content update
  tempViewMode.value = 'none';
  showQuickView.value = false;
  isSlideShow.value = false;
  stopSlideShow();

  backupState.value = null;
  
  // Increment request ID to cancel any previous thumbnail generation and reset queue
  currentThumbRequestId++;
  thumbCount.value = 0;
  showProgressBar.value = false;

  const requestId = ++currentContentRequestId;

  contentReady.value = false;
  isCurrentFolderExcluded.value = false;

  // Reset file list immediately to reflect UI change
  clearSelectionForFileListUpdate();
  clearContentRows();
  isLoading.value = true;

  await nextTick();
  if (updateSeq !== contentUpdateSeq) return; // superseded by a later updateContent call
  await waitForContentLoadingPaint();
  if (requestId !== currentContentRequestId) return;

  if (libConfig.activePane === 'collection') {
    const collectionId = Number(libConfig.collection.selectedId || 0);
    if (collectionId <= 0) {
      contentTitle.value = localeMsg.value.collection.title;
      showEmptyContent(requestId);
      return;
    }

    const collections = await listCollections();
    const collection = Array.isArray(collections)
      ? collections.find((item: any) => Number(item.id) === collectionId)
      : null;
    if (!collection) {
      libConfig.collection.selectedId = null;
      contentTitle.value = localeMsg.value.collection.title;
      showEmptyContent(requestId);
      return;
    }

    contentTitle.value = collection.name || localeMsg.value.collection.title;
    getCollectionFileList(collectionId, requestId);
    return;
  }

  if(newIndex === SIDEBAR.LIBRARY) {
    switch ((libConfig.library as any).item) {
      case LIB_ITEM.FAV:
        contentTitle.value = localeMsg.value.favorite.files;
        getFileList({ isFavorite: true }, requestId);
        break;
      case LIB_ITEM.TODAY:
        contentTitle.value = localeMsg.value.library.on_this_day;
        getFileList({ startDate: -1, endDate: -1, calendarSort: DATE_SORT.TAKEN_DESC }, requestId);
        break;
      case LIB_ITEM.RATINGS:
        if (libConfig.rating.item === RATE.ALL) {
          contentTitle.value = localeMsg.value.rating.rated;
          getFileList({ rating: RATE.ALL }, requestId);
        } else if (libConfig.rating.item === RATE.UNRATED) {
          contentTitle.value = localeMsg.value.rating.unrated;
          getFileList({ rating: RATE.UNRATED }, requestId);
        } else if ((libConfig.rating.item || 0) > 0) {
          const rating = Number(libConfig.rating.item || 0);
          const key = ratingLabelKey(rating);
          contentTitle.value = key ? localeMsg.value.rating[key] : `${rating}★`;
          getFileList({ rating }, requestId);
        } else {
          contentTitle.value = localeMsg.value.rating.title;
          showEmptyContent(requestId);
        }
        break;
      case LIB_ITEM.SUBJECTS: {
        const smartId = libConfig.library.smartId;
        if (!smartId) {
          contentTitle.value = localeMsg.value.subject.title;
          showEmptyContent(requestId);
          break;
        }
        const smartTag = getSmartTagById(smartId);
        if (!smartTag) {
          contentTitle.value = localeMsg.value.subject.title;
          showEmptyContent(requestId);
          break;
        }
        const smartTagLabel = localeMsg.value.subject.items?.[smartTag.id] || smartTag.id;
        contentTitle.value = `${localeMsg.value.subject.title} > ${smartTagLabel}`;
        getImageSearchFileList(smartTag.prompt, 0, requestId, false, SMART_TAG_SEARCH_THRESHOLD);
        break;
      }
      default:
        contentTitle.value = localeMsg.value.library.all_files;
        getFileList({}, requestId);
        break;
    }
  }
  else if(newIndex === SIDEBAR.ALBUM) {
    if(libConfig.album.id === null) {
      contentTitle.value = "";
    } else if(libConfig.album.id === 0) {   // all files
      contentTitle.value = localeMsg.value.library.all_files;
      getFileList({}, requestId);
    } else {
      getAlbum(libConfig.album.id).then(async album => {
        if (requestId !== currentContentRequestId) return;
        if(album) {
          if(libConfig.album.selected) {     
            // album is selected, show all files including subfolders
            contentTitle.value = album.name;
            getFileList({ searchAllSubfolders: libConfig.album.folderPath }, requestId);
          } else {                        
            // folder is selected, show files in the folder
            const folderPath = libConfig.album.folderPath || "";
            getFolderSearchExcluded(folderPath).then(excluded => {
              if (requestId === currentContentRequestId) isCurrentFolderExcluded.value = !!excluded;
            });
            contentTitle.value = formatFolderBreadcrumb(folderPath, album.path);
            const folderId = Number(libConfig.album.folderId || 0);
            const folderNode = await fetchFolder(folderPath, false, config.settings.folderSort);
            if (requestId !== currentContentRequestId) return;
            selectedFolderHasChildren.value = Boolean(folderNode?.children?.length);
            const folderQueryParams = () => config.settings.showSubfolderFiles
              ? { searchAllSubfolders: folderPath }
              : { searchFolder: folderPath };
            const syncKey = `${album.id}:${folderId}:${folderPath}`;
            if (folderId > 0 && folderPath && lastSyncedAlbumFolder !== syncKey) {
              lastSyncedAlbumFolder = syncKey;
              // Debounce: if a sync is already in-flight for this folder, reuse it.
              const existing = pendingFolderSyncs.get(folderId);
              const syncPromise = existing ?? syncAlbumFolderMtimes(album.id, folderId, folderPath);
              if (!existing) pendingFolderSyncs.set(folderId, syncPromise);
              syncPromise.then(syncResult => {
                if (pendingFolderSyncs.get(folderId) === syncPromise) {
                  pendingFolderSyncs.delete(folderId);
                }
                if (
                  requestId !== currentContentRequestId ||
                  config.main.sidebarIndex !== SIDEBAR.ALBUM ||
                  libConfig.album.folderId !== folderId ||
                  libConfig.album.folderPath !== folderPath
                ) return;
                if (syncResult?.current_folder_synced) {
                  console.log(
                    `folder sync: ${syncResult.new_file_count} new, ${syncResult.updated_file_count} updated, ${syncResult.deleted_file_count} deleted, ${syncResult.rename_count || 0} renamed`
                  );
                  const visibleRange = { ...lastVisibleRange };
                  const refreshRequestId = ++currentContentRequestId;
                  if (pendingInitialSelectedIndex < 0) {
                    pendingInitialSelectedIndex = selectedItemIndex.value;
                    hasRestoredInitialSelection = false;
                  }
                  getFileList(folderQueryParams(), refreshRequestId).then(() => {
                    if (
                      refreshRequestId !== currentContentRequestId ||
                      visibleRange.start < 0 ||
                      visibleRange.end <= visibleRange.start
                    ) return;
                    fetchDataRange(visibleRange.start, visibleRange.end + 1);
                  });
                }
              }).catch(error => {
                if (pendingFolderSyncs.get(folderId) === syncPromise) {
                  pendingFolderSyncs.delete(folderId);
                }
                console.error('folder sync failed:', error);
              });
            }
            getFileList(folderQueryParams(), requestId);
          }
        } else {
          contentTitle.value = "";
        }
      });
    }
  }
  else if(newIndex === SIDEBAR.SMART_ALBUM) {
    const smartAlbumType = libConfig.smartAlbum.type;
    const smartAlbumId = libConfig.smartAlbum.id;
    if (smartAlbumType === 'custom' && smartAlbumId) {
      const smartAlbum = (libConfig.smartAlbums || []).find((item: any) => item.id === smartAlbumId);
      if (smartAlbum) {
        contentTitle.value = smartAlbum.name || '';
        getSmartFileList(smartAlbum, requestId);
      } else {
        contentTitle.value = "";
        showEmptyContent(requestId);
      }
    } else {
      contentTitle.value = "";
      showEmptyContent(requestId);
    }
  }
  else if(newIndex === SIDEBAR.SEARCH) {
    if (libConfig.search.searchText) {
      contentTitle.value = localeMsg.value.search.title + ' - ' + libConfig.search.searchText;
      getUnifiedSearchFileList(libConfig.search.searchText, requestId);
    } else {
      contentTitle.value = localeMsg.value.search.title;
      showEmptyContent(requestId);
    }
  }
  else if(newIndex === SIDEBAR.CALENDAR) {
    if(libConfig.calendar.year === null) {
      contentTitle.value = "";
      showEmptyContent(requestId);
    } else {
      if (libConfig.calendar.month === -1) {          // yearly
        contentTitle.value = formatDate(libConfig.calendar.year!, 1, 1, localeMsg.value.format.year);
      } else if (libConfig.calendar.date === -1) {    // monthly
        contentTitle.value = formatDate(libConfig.calendar.year!, libConfig.calendar.month!, 1, localeMsg.value.format.month);
      } else {                                    // daily
        contentTitle.value = formatDate(libConfig.calendar.year!, libConfig.calendar.month!, libConfig.calendar.date!, localeMsg.value.format.date_long);
      }
      const [startDate, endDate] = getCalendarDateRange(libConfig.calendar.year!, libConfig.calendar.month!, libConfig.calendar.date!);
      getFileList({ startDate, endDate }, requestId);
    }
  } 
  else if(newIndex === SIDEBAR.TAG) {
    if (libConfig.tag.id === null) {
      contentTitle.value = "";
      showEmptyContent(requestId);
    } else {
      getTagName(libConfig.tag.id).then(tagName => {
        if (requestId !== currentContentRequestId) return;
        if (tagName) {
          contentTitle.value = tagName;
          getFileList({ tagId: libConfig.tag.id || 0 }, requestId);
        } else {
          contentTitle.value = "";
          showEmptyContent(requestId);
        }
      });
    }
  }
  else if(newIndex === SIDEBAR.PERSON) {
    if (libConfig.person.id === null) {
      contentTitle.value = "";
      showEmptyContent(requestId);
    } else {
      contentTitle.value = libConfig.person.name || `${localeMsg.value.sidebar.people}`;
      getFileList({ personId: libConfig.person.id }, requestId);
    }
  }
  else if(newIndex === SIDEBAR.LOCATION) {
    if(libConfig.location.admin1 === null) {
      contentTitle.value = "";
      showEmptyContent(requestId);
    } else {
      if(libConfig.location.name) {
        contentTitle.value = `${libConfig.location.admin1} > ${libConfig.location.name}`;
        getFileList({ locationAdmin1: libConfig.location.admin1, locationName: libConfig.location.name }, requestId);
      } else {
        contentTitle.value = `${libConfig.location.admin1}`;
        getFileList({ locationAdmin1: libConfig.location.admin1 }, requestId);
      } 
    }
  }
  else if(newIndex === SIDEBAR.CAMERA) {
    if (!config.camera.isCamera) {
      const lensMake = (libConfig.camera as any).lensMake;
      const lensModel = (libConfig.camera as any).lensModel;
      if (lensMake === null) {
        contentTitle.value = "";
        showEmptyContent(requestId);
      } else if (lensModel) {
        contentTitle.value = `${lensMake} > ${lensModel}`;
        getFileList({ lensMake, lensModel }, requestId);
      } else {
        contentTitle.value = `${lensMake}`;
        getFileList({ lensMake }, requestId);
      }
    } else if(libConfig.camera.make === null) {
      contentTitle.value = "";
      showEmptyContent(requestId);
    } else {
      if(libConfig.camera.model) {
        contentTitle.value = `${libConfig.camera.make} > ${libConfig.camera.model}`;
        getFileList({ make: libConfig.camera.make, model: libConfig.camera.model }, requestId);
      } else {
        contentTitle.value = `${libConfig.camera.make}`;
        getFileList({ make: libConfig.camera.make }, requestId);
      } 
    }
  } 

  if(fileList.value.length === 0) {
    isLoading.value = false;
  }
}

// --- Similar Search Mode Logic ---
function enterSimilarSearchMode(file: any) {
  if (file.file_type !== 1 && file.file_type !== 3) {
    return;
  }

  // Increment request ID to cancel any previous thumbnail generation and reset queue
  currentThumbRequestId++;
  thumbCount.value = 0;

  // 1. Backup current state
  if (tempViewMode.value === 'none') {
    backupState.value = createViewBackup();
  }

  // 2. Set mode
  tempViewMode.value = 'similar';
  showQuickView.value = false;
  
  // 3. Update Title to indicate context
  contentTitle.value = localeMsg.value.search.similar_images + ' - ' + file.name;

  // 4. Perform Search (reusing existing API call logic)
  const requestId = ++currentContentRequestId;
  showLoadingContent(requestId);
  
  // Reset scroll and selection
  scrollPosition.value = 0;
  selectedItemIndex.value = 0;
  if (gridViewRef.value) {
    gridViewRef.value.scrollToPosition(0);
  }
  
  getImageSearchFileList("", file.id, requestId);
}

// --- Person Search Mode Logic ---
async function enterPersonSearchMode(file: any) {
  if (!config.settings.face.enabled) {
    return;
  }
  if (!file || !file.id) {
    return;
  }

  // fetch faces
  const faces = await getFacesForFile(file.id);
  if (!faces || faces.length === 0) {
     toast.info(localeMsg.value.tooltip.not_found.person || "No person found");
     return;
  }

  // Find first face with person_id
  const face = faces.find((f: any) => f.person_id && f.person_id > 0);
  if (!face) {
     toast.info(localeMsg.value.tooltip.not_found.person || "No person found");
     return;
  }

  // Increment request ID to cancel any previous thumbnail generation and reset queue
  currentThumbRequestId++;
  thumbCount.value = 0;

  // 1. Backup current state
  if (tempViewMode.value === 'none') {
    backupState.value = createViewBackup();
  }

  // 2. Set mode
  tempViewMode.value = 'person';
  showQuickView.value = false;

  // 3. Update libConfig.person to reflect the found person
  libConfig.person.id = face.person_id;
  libConfig.person.name = face.person_name || null;

  // 4. Update Title to indicate context
  contentTitle.value = face.person_name || localeMsg.value.sidebar.people;

  // 5. Perform Search
  const requestId = ++currentContentRequestId;
  showLoadingContent(requestId);
  
  // Reset scroll and selection
  scrollPosition.value = 0;
  selectedItemIndex.value = 0;
  if (gridViewRef.value) {
    gridViewRef.value.scrollToPosition(0);
  }
  
  getFileList({ personId: face.person_id }, requestId);
}

function enterAlbumPreviewMode(file: any, targetFolderPath?: string) {
  if (!file.album_id) return;
  const folderPath = targetFolderPath || getFolderPath(file.file_path);
  if (!folderPath) return;
  if (tempViewMode.value === 'album' && currentQueryParams.value.searchFolder === folderPath) return;

  // 1. Backup current state
  if (tempViewMode.value === 'none') {
    backupState.value = createViewBackup();
  }
  
  // Increment request ID to cancel any previous thumbnail generation and reset queue
  currentThumbRequestId++;
  thumbCount.value = 0;

  // 2. Set mode
  tempViewMode.value = 'album';
  showQuickView.value = false;
  
  // 3. Update Title and Fetch Files
  getAlbum(file.album_id).then((album: any) => {
    if (album) {
      if(folderPath === album.path) { // current folder is root
        contentTitle.value = album.name;
      } else {
        contentTitle.value = formatFolderBreadcrumb(folderPath || "", album.path);
      };
    }
  });

  // 4. Fetch Files
  const requestId = ++currentContentRequestId;
  
  // Reset list for loading state
  clearContentRows();
  totalFileCount.value = 0;
  totalFileSize.value = 0;
  
  // Reset scroll and selection
  scrollPosition.value = 0;
  selectedItemIndex.value = 0;
  if (gridViewRef.value) {
    gridViewRef.value.scrollToPosition(0);
  }
  
  getFileList({ searchFolder: folderPath }, requestId);
}

function exitTempViewMode() {
  if (!backupState.value) return;

  const state = backupState.value;
  
  // 1. Restore State
  clearSelectionForFileListUpdate();
  for (const file of state.fileList) {
    if (file) file.isSelected = false;
  }
  fileList.value = state.fileList;
  groupedModeActive.value = Boolean(state.groupedModeActive);
  groupedRows.value = state.groupedRows || [];
  totalRowCount.value = Number(state.totalRowCount || 0);
  groupedTimelineGroups.value = state.groupedTimelineGroups || [];
  folderGroupRoots.value = state.folderGroupRoots || [];
  groupMetaMap.clear();
  for (const [groupId, meta] of state.groupMetaEntries || []) {
    groupMetaMap.set(groupId, meta);
  }
  fileIdToGroupId.clear();
  for (const [fileId, groupId] of state.fileIdToGroupIdEntries || []) {
    fileIdToGroupId.set(fileId, groupId);
  }
  groupSelectedCountMap.value = {};
  groupSelectedSizeMap.value = {};
  totalFileCount.value = state.totalFileCount;
  totalFileSize.value = state.totalFileSize;
  contentTitle.value = state.contentTitle;
  selectedItemIndex.value = state.selectedItemIndex;
  scrollPosition.value = state.scrollPosition;
  timelineData.value = state.timelineData;
  currentQueryParams.value = state.currentQueryParams;
  currentQuerySource.value = state.currentQuerySource || 'query';
  currentSmartQueryParams.value = state.currentSmartQueryParams || null;
  currentCollectionId.value = state.currentCollectionId || null;
  currentSearchFileIds.value = [...(state.currentSearchFileIds || [])];
  thumbCount.value = state.thumbCount;
  showProgressBar.value = state.showProgressBar;

  // Increment request ID to cancel any previous thumbnail generation (from temp view)
  currentThumbRequestId++;

  // 2. Reset Mode
  tempViewMode.value = 'none';
  showQuickView.value = false;
  backupState.value = null;

  // 3. Restore Scroll Position (need to wait for Vue to re-render the list)
  // Using nextTick or a small timeout
  setTimeout(() => {
    if (gridViewRef.value) {
      gridViewRef.value.scrollToPosition(state.scrollTop);
    }
  }, 0);
  
  // 4. Ensure the originally selected item is highlighted/visible logic is handled by the restored index
  updateSelectedImage(selectedItemIndex.value);
}

function handleTitleClick() {
  switch (tempViewMode.value) {
    case 'similar':
      exitTempViewMode();
      break;
    case 'person':
      config.main.sidebarIndex = SIDEBAR.PERSON;
      break;
    case 'album':
      config.main.sidebarIndex = SIDEBAR.ALBUM;

      // Get first file to extract album info
      const file = fileList.value[0];
      if (!file || !file.album_id || !file.folder_id) return;
      
      const folderPath = getFolderPath(file.file_path);
      
      // Set libConfig.album to select this folder in Album tab
      libConfig.album.id = file.album_id;
      libConfig.album.folderId = file.folder_id;
      libConfig.album.folderPath = folderPath;
      libConfig.album.selected = false;
      
      // Emit event to trigger album expansion in AlbumList
      tauriEmit('expand-album-folder', { albumId: file.album_id, folderPath: folderPath });
      break;
    default:
      break;
  }

  exitTempViewMode();
}

// update file state after a save from either FileInfo or EditImage
const onFileSaved = async (success: boolean, payload: SavedFilePayload = {}) => {
  if (success) {
    if (payload.saveAsNew && payload.filePath) {
      uiStore.updateFileVersion(payload.filePath);
      clearPreviewPreloadCache(payload.filePath);
      const inserted = await indexAndInsertSavedFile(payload.filePath);
      if (!inserted) {
        updateContent();
      } else {
        getCurrentQueryTimeLine().then(data => {
          timelineData.value = data;
        });
      }
      toast.success(localeMsg.value.tooltip.save_image.save_as_success || localeMsg.value.tooltip.save_image.success);
    } else {
      const savedFile = fileList.value[selectedItemIndex.value];
      if (savedFile?.file_path) {
        uiStore.updateFileVersion(savedFile.file_path);
        clearPreviewPreloadCache(savedFile.file_path);
      }
      if (savedFile && Number(savedFile.rotate || 0) !== 0) {
        savedFile.rotate = 0;
        await setFileRotate(savedFile.id, 0);
        await syncFileMetaToImageViewer(savedFile.id, { rotate: 0 });
      }
      await updateFile(fileList.value[selectedItemIndex.value]);
      toast.success(localeMsg.value.tooltip.save_image.success);
    }
  } else {
    toast.error(localeMsg.value.tooltip.save_image.failed);
  }
}

const clickCopyImages = async (fallbackPath?: string) => {
  if (isProcessing.value) return;

  let copiedCount = 0;
  let requestedCount = 0;
  let cancelled = false;
  let filePaths: string[] = [];
  try {
    if (selectMode.value && selectedCount.value > 0) {
      requestedCount = selectedCount.value;
      const selectedItems = await getSelectedItemsForClipboard(10);
      if (!selectedItems) {
        cancelled = true;
        return;
      }
      filePaths = selectedItems
        .map((file: any) => String(file.file_path || ''))
        .filter(Boolean);
    } else if (fallbackPath) {
      filePaths = [fallbackPath];
      requestedCount = 1;
    }
    if (filePaths.length === 0) {
      cancelled = true;
      return;
    }
    isProcessing.value = true;
    copiedCount = Number(await copyImages(filePaths)) || 0;
  } finally {
    isProcessing.value = false;
    if (cancelled) {
      return;
    } else if (copiedCount > 0) {
      toast.success(t('tooltip.copy_image.success', { count: copiedCount.toLocaleString() }));
      if (requestedCount > 10) {
        toast.warning(t('tooltip.copy_image.limit', { count: 10 }));
      }
    } else {
      toast.error(localeMsg.value.tooltip.copy_image.failed);
    }
  }
}

const appPathForMediaKind = (kind: 'image' | 'video') =>
  kind === 'image'
    ? String(config.settings.externalImageAppPath || '')
    : String(config.settings.externalVideoAppPath || '');

const EXTERNAL_OPEN_WARNING_THRESHOLD = 100;

const launchInExternalApp = async (paths: string[], appPath: string) => {
  try {
    await openFilesWithApp(paths, appPath);
  } catch (error) {
    console.error('Failed to open external app:', error);
    toast.error(t('tooltip.open_external.failed'));
  }
};

const confirmExternalOpen = () => {
  const pending = pendingExternalOpen.value;
  showExternalOpenWarningMsgbox.value = false;
  pendingExternalOpen.value = null;
  if (pending) void launchInExternalApp(pending.paths, pending.appPath);
};

const cancelExternalOpen = () => {
  showExternalOpenWarningMsgbox.value = false;
  pendingExternalOpen.value = null;
};

const openInExternalApp = async () => {
  const items = selectMode.value
    ? (selectedCount.value > 0 ? await getActionableSelectedItemsForAction() : [])
    : [fileList.value[selectedItemIndex.value]].filter(Boolean);
  if (!items || items.length === 0) return;

  const kind = getMediaKind(items);
  if (kind === 'empty') return;
  if (kind === 'mixed') {
    toast.warning(t('tooltip.open_external.mixed_selection'));
    return;
  }

  const appPath = appPathForMediaKind(kind);
  if (!appPath) {
    toast.warning(t('tooltip.open_external.no_app'));
    return;
  }

  const paths = items.map((item: any) => item.file_path).filter(Boolean);
  if (paths.length === 0) return;

  if (paths.length > EXTERNAL_OPEN_WARNING_THRESHOLD) {
    pendingExternalOpen.value = { paths, appPath };
    showExternalOpenWarningMsgbox.value = true;
    return;
  }

  await launchInExternalApp(paths, appPath);
}

const onRenameFile = async (newName: string) => {
  if(selectedItemIndex.value >= 0) {
    const file = fileList.value[selectedItemIndex.value];
    const fileName = combineFileName(newName, renamingFileName.value.ext ?? '');
    const newFilePath = await renameFile(file.id, file.file_path, fileName );
    if(newFilePath) {
      console.log('onRenameFile:', newFilePath);
      file.name = fileName;
      file.file_path = newFilePath;
      showRenameMsgbox.value = false;
      errorMessage.value = '';
    } else {
      errorMessage.value = localeMsg.value.msgbox.rename_file.error;
    }
  }
}

const refreshAffectedAlbums = async (albumIds: Array<number | null | undefined>) => {
  const uniqueAlbumIds = Array.from(
    new Set(
      albumIds
        .map((id) => Number(id || 0))
        .filter((id) => id > 0),
    ),
  );

  if (uniqueAlbumIds.length === 0) return;

  const albums = (
    await Promise.all(uniqueAlbumIds.map((albumId) => recountAlbum(albumId)))
  ).filter(Boolean);

  if (albums.length > 0) {
    await tauriEmit('albums-refreshed', { albums, refreshFolders: false });
  }
};

const refreshLibraryTotalCount = async () => {
  await tauriEmit('library-total-refreshed', { source: 'content' });
};

function rebuildSelectionAfterListMutation(selectedIds: Set<number>) {
  let remainingCount = 0;
  let remainingSize = 0;
  selectedFileIds.clear();
  for (const file of fileList.value) {
    const selected = isRealFileItem(file) && selectedIds.has(Number(file.id));
    file.isSelected = selected;
    if (selected) {
      selectedFileIds.add(Number(file.id));
      remainingCount++;
      remainingSize += Number(file.size || 0);
    }
  }
  recomputeLoadedGroupSelectedCounts();
  selectedCount.value = remainingCount;
  selectedSize.value = remainingCount === totalFileCount.value ? totalFileSize.value : remainingSize;
  syncSelectionVersions();

  if (remainingCount === 0) {
    selectMode.value = false;
    lastSelectedIndex.value = -1;
    keyboardSelectionAnchorIndex.value = -1;
  }
}

const onMoveTo = async () => {
  const affectedAlbumIds = new Set<number>();
  const destPath = String(libConfig.destFolder.folderPath || '');
  const destAlbumId = Number(libConfig.destFolder.albumId || 0);
  const destLabel = getFolderName(destPath) || destPath;
  let successCount = 0;

  // Resolve destination folder ID: when the user selects an album root (not a
  // subfolder), destFolder.folderId is null. Ensure the root folder record
  // exists in afolders so that moved files keep a valid folder_id.
  let destFolderId = Number(libConfig.destFolder.folderId || 0);
  if (destFolderId <= 0 && destPath) {
    if (destAlbumId > 0) {
      const resolved = await selectFolder(destAlbumId, destPath);
      if (resolved?.id) {
        destFolderId = resolved.id;
      }
    }
  }

  if (selectMode.value && selectedCount.value > 0) {    // multi-select mode
    const sourceLabel = t('toolbar.filter.select_count', { count: selectedCount.value.toLocaleString() });
    const items = await getActionableSelectedItemsForAction();
    if (!items) return;
    if (!await confirmLargeBatch(items.length)) return;
    const moves = await runWithKeyedConcurrency(
      items,
      item => getTransferDestinationKey(item),
      async item => {
        const movedFile = await moveFile(item.id, item.file_path, destFolderId, destPath);
        return { item, movedFile };
      },
    );
    const successfulMoves = moves.flatMap(result =>
      result.status === 'fulfilled' && result.value.movedFile ? [result.value] : [],
    );
    const successIds = successfulMoves.map(({ item }) => item.id);
    successfulMoves.forEach(({ item, movedFile }) => {
      console.log('onMoveTo:', movedFile);
      affectedAlbumIds.add(Number(item.album_id || 0));
    });
    const failedCount = moves.filter(
      result => result.status === 'rejected' || !result.value.movedFile,
    ).length;
    if (successIds.length > 0) {
      successCount = successIds.length;
      const successIdSet = new Set(successIds);
      const remainingSelectedIds = new Set(
        items
          .filter(item => !successIdSet.has(item.id))
          .map(item => Number(item.id)),
      );
      fileList.value = fileList.value.filter((f) => !successIdSet.has(f.id));
      totalFileCount.value = fileList.value.length;
      totalFileSize.value = fileList.value.reduce((total, file) => total + file.size, 0);
      selectedItemIndex.value = fileList.value.length > 0 ? Math.min(selectedItemIndex.value, fileList.value.length - 1) : -1;
      rebuildSelectionAfterListMutation(remainingSelectedIds);
      toast.success(t('msgbox.move_to.success', { source: sourceLabel, dest: destLabel }));
    }
    if (failedCount > 0 && successCount > 0) {
      toast.error(t('msgbox.move_to.error', { source: sourceLabel, dest: destLabel }));
    }
  } 
  else if(selectedItemIndex.value >= 0) {               // single select mode
    const file = fileList.value[selectedItemIndex.value];
    const sourceLabel = file.name;
    const movedFile = await moveFile(file.id, file.file_path, destFolderId, destPath);
    if(movedFile) {
      console.log('onMoveTo:', movedFile);
      affectedAlbumIds.add(Number(file.album_id || 0));
      removeFromFileList(selectedItemIndex.value);
      successCount = 1;
      toast.success(t('msgbox.move_to.success', { source: sourceLabel, dest: destLabel }));
    }
  }

  if (destAlbumId > 0) affectedAlbumIds.add(destAlbumId);
  await refreshAffectedAlbums(Array.from(affectedAlbumIds));
  await refreshLibraryTotalCount();
  if (successCount > 0 && (groupedModeActive.value || activeGroupBy.value > 0)) {
    await updateContent(true);
  }

  if (successCount === 0) {
    const sourceLabel = selectMode.value
      ? t('toolbar.filter.select_count', { count: selectedCount.value.toLocaleString() })
      : (fileList.value[selectedItemIndex.value]?.name || '');
    toast.error(t('msgbox.move_to.error', { source: sourceLabel, dest: destLabel }));
  }

  showMoveTo.value = false;
}

const selectSystemDestination = async (title: string) => {
  const destination = await openDialog({
    title,
    multiple: false,
    directory: true,
  });
  return !destination || Array.isArray(destination) ? null : String(destination);
}

const requestFileConflict = (
  name: string,
  destination: string,
  showApplyAll: boolean,
  allowReplace = true,
): Promise<{ policy: FileConflictPolicy; applyAll: boolean }> => {
  fileConflictDialog.value = { show: true, name, destination, showApplyAll, allowReplace };
  return new Promise(resolve => {
    fileConflictResolver = resolve;
  });
}

const resolveFileConflict = (result: { policy: FileConflictPolicy; applyAll: boolean }) => {
  fileConflictDialog.value.show = false;
  fileConflictResolver?.(result);
  fileConflictResolver = null;
}

const resolveConflictPolicy = async (
  sourcePath: string,
  name: string,
  destPath: string,
  applyAllPolicy: FileConflictPolicy | null,
  showApplyAll: boolean,
  sameDestinationMeansSkip: boolean,
) => {
  const destinationPath = getFullPath(destPath, name);
  const isSamePath =
    normalizePathForCompare(sourcePath) === normalizePathForCompare(destinationPath);
  if (isSamePath && sameDestinationMeansSkip) {
    toast.info(t('msgbox.file_conflict.same_folder'));
    return { policy: 'skip' as FileConflictPolicy, applyAll: false };
  }
  if (!await checkFileExists(destinationPath)) {
    return { policy: 'keep_both' as FileConflictPolicy, applyAll: false };
  }
  if (applyAllPolicy && !(isSamePath && applyAllPolicy === 'replace')) {
    return { policy: applyAllPolicy, applyAll: true };
  }
  return requestFileConflict(name, destPath, showApplyAll, !isSamePath);
}

const getFilesForFolderAction = async () => {
  return selectMode.value && selectedCount.value > 0
    ? await getActionableSelectedItemsForAction()
    : (selectedItemIndex.value >= 0 ? [fileList.value[selectedItemIndex.value]] : []);
}

const getTransferDestinationKey = (file: any) =>
  String(file?.name || getFolderName(file?.file_path || '')).normalize('NFC').toLowerCase();

const resolveLibraryDestination = async (destPath: string) => {
  const albums = await getAllAlbums();
  const album = Array.isArray(albums)
    ? albums.find(item => isWithinRootPath(destPath, item.path))
    : null;
  if (!album) return null;

  const folder = await selectFolder(album.id, destPath);
  return folder?.id ? { albumId: Number(album.id), folderId: Number(folder.id) } : null;
}

const onMoveToFolder = async () => {
  const files = await getFilesForFolderAction();
  if (!files || files.length === 0) return;

  const destPath = await selectSystemDestination(t('msgbox.move_to_folder.title'));
  if (!destPath) return;
  if (!await confirmLargeBatch(files.length)) return;

  const sourceLabel = selectMode.value
    ? t('toolbar.filter.select_count', { count: files.length.toLocaleString() })
    : (files[0]?.name || '');
  const libraryDestination = await resolveLibraryDestination(destPath);
  if (!libraryDestination) {
    const confirmed = await ask(
      t('msgbox.move_to_folder.warning', { source: sourceLabel, dest: destPath }),
      {
        title: t('msgbox.move_to_folder.confirm_title'),
        kind: 'warning',
        okLabel: t('msgbox.move_to_folder.ok'),
        cancelLabel: t('msgbox.cancel'),
      },
    );
    if (!confirmed) return;
  }

  const affectedAlbumIds = new Set<number>();
  let applyAllPolicy: FileConflictPolicy | null = null;
  const operations: Array<{ file: any; policy: FileConflictPolicy }> = [];
  for (const file of files) {
    if (!file?.file_path) continue;
    const conflict = await resolveConflictPolicy(
      file.file_path,
      file.name,
      destPath,
      applyAllPolicy,
      files.length > 1,
      true,
    );
    if (conflict.applyAll) applyAllPolicy = conflict.policy;
    if (conflict.policy === 'skip') continue;
    operations.push({ file, policy: conflict.policy });
  }
  const moveResults = await runWithKeyedConcurrency(
    operations,
    ({ file }) => getTransferDestinationKey(file),
    async ({ file, policy }) => {
      const movedFile = libraryDestination
        ? await moveFile(file.id, file.file_path, libraryDestination.folderId, destPath, policy)
        : await moveFileOutsideLibrary(file.id, file.file_path, destPath, policy);
      if (!movedFile) throw new Error(`Failed to move ${file.file_path}`);
      return { file, movedFile };
    },
  );
  const successfulMoves = moveResults.flatMap(result =>
    result.status === 'fulfilled' ? [result.value] : [],
  );
  const successIds = successfulMoves.map(({ file }) => file.id);
  successfulMoves.forEach(({ file }) => {
    affectedAlbumIds.add(Number(file.album_id || 0));
  });
  if (libraryDestination && successfulMoves.length > 0) {
    affectedAlbumIds.add(libraryDestination.albumId);
  }

  if (successIds.length > 0) {
    const successIdSet = new Set(successIds);
    const remainingSelectedIds = new Set(
      files
        .filter(file => !successIdSet.has(file.id))
        .map(file => Number(file.id)),
    );
    fileList.value = fileList.value.filter(file => !successIdSet.has(file.id));
    totalFileCount.value = fileList.value.length;
    totalFileSize.value = fileList.value.reduce((total, file) => total + file.size, 0);
    selectedItemIndex.value = fileList.value.length > 0
      ? Math.min(selectedItemIndex.value, fileList.value.length - 1)
      : -1;
    rebuildSelectionAfterListMutation(remainingSelectedIds);
    toast.success(t('msgbox.move_to_folder.success', { source: sourceLabel, dest: getFolderName(destPath) || destPath }));
    await refreshAffectedAlbums(Array.from(affectedAlbumIds));
    await refreshLibraryTotalCount();
    // Moving a file changes its grouping key (for example, folder grouping).
    // The flat list was updated optimistically above, but grouped rows and
    // group counts must be rebuilt from the current query result.
    if (groupedModeActive.value || activeGroupBy.value > 0) {
      await updateContent(true);
    }
  }
  if (moveResults.some(result => result.status === 'rejected')) {
    toast.error(t('msgbox.move_to_folder.error', { source: sourceLabel, dest: getFolderName(destPath) || destPath }));
  }
}

const onCopyToFolder = async () => {
  const destPath = await selectSystemDestination(t('msgbox.copy_to_folder.title'));
  if (!destPath) return;

  const files = await getFilesForFolderAction();
  if (!files || files.length === 0) return;
  if (!await confirmLargeBatch(files.length)) return;

  const libraryDestination = await resolveLibraryDestination(destPath);
  const destLabel = getFolderName(destPath) || destPath;
  const sourceLabel = selectMode.value
    ? t('toolbar.filter.select_count', { count: files.length.toLocaleString() })
    : (files[0]?.name || '');
  let applyAllPolicy: FileConflictPolicy | null = null;
  const operations: Array<{ file: any; policy: FileConflictPolicy }> = [];

  for (const file of files) {
    if (!file?.file_path) continue;
    const conflict = await resolveConflictPolicy(
      file.file_path,
      file.name,
      destPath,
      applyAllPolicy,
      files.length > 1,
      false,
    );
    if (conflict.applyAll) applyAllPolicy = conflict.policy;
    if (conflict.policy === 'skip') continue;
    operations.push({ file, policy: conflict.policy });
  }
  const copyResults = await runWithKeyedConcurrency(
    operations,
    ({ file }) => getTransferDestinationKey(file),
    async ({ file, policy }) => {
      const copiedFile = await copyFile(file.file_path, destPath, policy);
      if (!copiedFile) throw new Error(`Failed to copy ${file.file_path}`);
      if (libraryDestination) {
        const indexedFile = await addFileToDb(libraryDestination.folderId, copiedFile);
        if (!indexedFile) {
          throw new CopyIndexError(`Copied but failed to index ${copiedFile}`);
        }
      }
      return { file, copiedFile };
    },
  );
  const successCount = copyResults.filter(result => result.status === 'fulfilled').length;
  const indexFailureCount = copyResults.filter(
    result => result.status === 'rejected' && result.reason instanceof CopyIndexError,
  ).length;
  const copyFailureCount = copyResults.filter(
    result => result.status === 'rejected' && !(result.reason instanceof CopyIndexError),
  ).length;

  if (successCount > 0) {
    toast.success(t('msgbox.copy_to_folder.success', { source: sourceLabel, dest: destLabel }));
    if (libraryDestination) {
      await refreshAffectedAlbums([libraryDestination.albumId]);
      await refreshLibraryTotalCount();
      await tauriEmit('refresh-content');
    }
  }
  if (copyFailureCount > 0) {
    toast.error(t('msgbox.copy_to_folder.error', { source: sourceLabel, dest: destLabel }));
  }
  if (indexFailureCount > 0) {
    toast.error(t('msgbox.copy_to_folder.index_error', {
      count: indexFailureCount.toLocaleString(),
    }));
  }
}

async function handleDropUrls(
  urls: string[],
  resolvedDestination?: { albumId: number; folderId: number; folderPath: string },
) {
  const destination = resolvedDestination || await resolveCurrentAlbumImportDestination();
  if (!destination) return;
  const { albumId, folderId, folderPath } = destination;
  let imported = 0;
  for (const url of urls) {
    try {
      const file = await importUrl(url, folderId, folderPath);
      if (file) imported++;
    } catch (e) {
      console.error('Failed to import URL:', url, e);
    }
  }
  if (imported > 0) {
    await refreshImportedFiles(albumId);
    toast.success(t('msgbox.drop_import.success', { count: imported }));
  } else {
    toast.warning(t('msgbox.drop_import.no_files'));
  }
}

const getDeleteFileSuccessMessage = (fileName: string, permanently: boolean) =>
  permanently
    ? localeMsg.value.msgbox.permanent_delete.file_success.replace('{file}', fileName)
    : localeMsg.value.msgbox.move_to_trash.file_success.replace('{file}', fileName);

const getDeleteFilesSuccessMessage = (count: number, permanently: boolean) =>
  permanently
    ? localeMsg.value.msgbox.permanent_delete.files_success.replace('{count}', count.toLocaleString())
    : localeMsg.value.msgbox.move_to_trash.files_success.replace('{count}', count.toLocaleString());

const getDeleteFileErrorMessage = (permanently: boolean) =>
  permanently
    ? localeMsg.value.msgbox.permanent_delete.file_error
    : localeMsg.value.msgbox.move_to_trash.file_error;

const getDeleteFilesErrorMessage = (permanently: boolean) =>
  permanently
    ? localeMsg.value.msgbox.permanent_delete.files_error
    : localeMsg.value.msgbox.move_to_trash.files_error;

const onTrashFile = async () => {
  permanentDeleteChecked.value = deletePermanently.value;
  const permanently = deletePermanently.value;
  const deletedFileIds: number[] = [];
  const affectedAlbumIds = new Set<number>();
  let failedDeleteCount = 0;
  const shouldUpdateDedup =
    isDedupPanelOpen.value &&
    !!dedupTrashGroupKey.value;
  const currentDedupGroupId = Number(dedupTrashGroupKey.value);
  try {
    if (dedupDeleteFileIds.value.length > 0) {
      const ids = [...dedupDeleteFileIds.value];
      if (permanently) {
        const selectedItems = ids
          .map(id => fileList.value.find(file => Number(file.id) === id))
          .filter((file): file is any => !!file);
        const result = await batchDeleteFiles(
          selectedItems.map(item => ({ fileId: item.id, filePath: item.file_path })),
          true,
        );
        if (!result) throw new Error('Failed to permanently delete dedup files');
        const deletedIdSet = new Set(result.deletedFileIds.map((id: any) => Number(id)));
        const deletedItems = selectedItems.filter(item => deletedIdSet.has(Number(item.id)));
        failedDeleteCount = Number(result.failedCount || 0) + (ids.length - selectedItems.length);

        if (failedDeleteCount > 0 && deletedItems.length === 0) {
          throw new Error('Failed to permanently delete dedup files');
        }

        deletedItems.forEach(item => affectedAlbumIds.add(Number(item.album_id || 0)));
        deletedFileIds.push(...deletedItems.map(item => item.id));
      } else {
        const result = await dedupDeleteSelected(null, ids);
        if (result !== undefined) {
          const resultDeletedIds = Array.isArray(result?.deletedFileIds)
            ? result.deletedFileIds.map((id: any) => Number(id)).filter((id: number) => id > 0)
            : [];
          deletedFileIds.push(...resultDeletedIds);
          const deletedIdSet = new Set(deletedFileIds);
          fileList.value
            .filter(file => deletedIdSet.has(file.id))
            .forEach(file => affectedAlbumIds.add(Number(file.album_id || 0)));
          failedDeleteCount = Number(result?.failedCount || 0);
          if (deletedFileIds.length === 0) {
            failedDeleteCount = Math.max(1, failedDeleteCount);
          }
        } else {
          throw new Error('Failed to trash dedup files');
        }
      }

      if (failedDeleteCount > 0 && deletedFileIds.length === 0) {
        throw new Error(`Failed to ${permanently ? 'permanently delete' : 'trash'} dedup files`);
      }

      const deletedIdSet = new Set(deletedFileIds);
      fileList.value = fileList.value.filter((f) => !deletedIdSet.has(f.id));
      totalFileCount.value = fileList.value.length;
      totalFileSize.value = fileList.value.reduce((total, file) => total + file.size, 0);
      selectedItemIndex.value = fileList.value.length > 0 ? Math.min(selectedItemIndex.value, fileList.value.length - 1) : -1;
    }
    else if (selectMode.value && selectedCount.value > 0) {     // multi-select mode
      const selectedItems = await getActionableSelectedItemsForAction();
      if (!selectedItems) return;
      const result = await batchDeleteFiles(
        selectedItems.map(item => ({ fileId: item.id, filePath: item.file_path })),
        permanently,
      );
      if (!result) {
        throw new Error(`Failed to ${permanently ? 'permanently delete' : 'trash'} selected files`);
      }
      const deletedIdSet = new Set(result.deletedFileIds.map((id: any) => Number(id)));
      const deletedItems = selectedItems.filter(item => deletedIdSet.has(Number(item.id)));
      failedDeleteCount = Number(result.failedCount || 0);

      if (failedDeleteCount > 0 && deletedItems.length === 0) {
        throw new Error(`Failed to ${permanently ? 'permanently delete' : 'trash'} selected files`);
      }

      deletedItems.forEach(item => affectedAlbumIds.add(Number(item.album_id || 0)));
      deletedFileIds.push(...deletedItems.map(item => item.id));
      const remainingSelectedIds = new Set(
        selectedItems
          .filter(item => !deletedIdSet.has(Number(item.id)))
          .map(item => Number(item.id)),
      );
      fileList.value = fileList.value.filter((f) => !deletedIdSet.has(f.id));
      totalFileCount.value = fileList.value.length;
      totalFileSize.value = fileList.value.reduce((total, file) => total + file.size, 0);
      selectedItemIndex.value = fileList.value.length > 0 ? Math.min(selectedItemIndex.value, fileList.value.length - 1) : -1;
      rebuildSelectionAfterListMutation(remainingSelectedIds);
    } 
    else if(selectedItemIndex.value >= 0) {               // single select mode
      const selectedFile = fileList.value[selectedItemIndex.value];
      const deletedFileName = selectedFile?.name || '';
      const result = await deleteFileAlways(selectedFile, permanently);
      const deletedIdSet = new Set(result.deletedFileIds.map((id: any) => Number(id)));
      if (!deletedIdSet.has(Number(selectedFile.id))) {
        throw new Error(`Failed to ${permanently ? 'permanently delete' : 'trash'} file: ${selectedFile.file_path}`);
      }
      affectedAlbumIds.add(Number(selectedFile.album_id || 0));
      deletedFileIds.push(selectedFile.id);
      failedDeleteCount = Number(result.failedCount || 0);
      removeFromFileList(selectedItemIndex.value);
      if (failedDeleteCount === 0) {
        toast.success(getDeleteFileSuccessMessage(deletedFileName, permanently));
      }
    }

    await refreshGroupedRowsAfterDelete(deletedFileIds);
    await refreshAffectedAlbums(Array.from(affectedAlbumIds));
    await refreshLibraryTotalCount();

    // Search results keep their relevance order and never use date groups.
    if (isSearchLikeView.value || tempViewMode.value === 'similar') {
      timelineData.value = [];
    } else {
      getCurrentQueryTimeLine().then(data => {
        timelineData.value = data;
      });
    }

    if (deletedFileIds.length > 0) {
      tauriEmit('files-deleted', {
        source: 'content',
        fileIds: deletedFileIds,
        fileCount: fileList.value.length,
        selectedIndex: selectedItemIndex.value,
      });
    }

    if ((selectMode.value || isDedupTrash.value) && deletedFileIds.length > 0) {
      toast.success(
        getDeleteFilesSuccessMessage(deletedFileIds.length, permanently)
      );
    }

    if (failedDeleteCount > 0) {
      toast.error(
        (dedupDeleteFileIds.value.length > 0 || selectMode.value)
          ? getDeleteFilesErrorMessage(permanently)
          : getDeleteFileErrorMessage(permanently)
      );
    }

    closeTrashMsgbox();
    updateSelectedImage(selectedItemIndex.value);

    if (shouldUpdateDedup && deletedFileIds.length > 0) {
      dedupPaneRef.value?.applyDeletedFiles(currentDedupGroupId, deletedFileIds);
    }
  } catch (error) {
    console.error(`Failed to ${permanently ? 'permanently delete' : 'trash'} file(s):`, error);
    toast.error(
      (dedupDeleteFileIds.value.length > 0 || selectMode.value)
        ? getDeleteFilesErrorMessage(permanently)
        : getDeleteFileErrorMessage(permanently)
    );
  }
}

// Remove a file from disk first. Keep the DB row if the filesystem operation fails.
async function deleteFileAlways(file: any, permanently = false) {
  const deletedFile = permanently
    ? await deleteFilePermanently(file.id, file.file_path)
    : await deleteFile(file.id, file.file_path);
  if (deletedFile && Array.isArray(deletedFile.deletedFileIds)) {
    console.log(`clickDeleteFile - ${permanently ? 'permanently deleted' : 'trashed'} file:`, file.file_path);
    return deletedFile;
  } else {
    throw new Error(`Failed to ${permanently ? 'permanently delete' : 'trash'} file: ${file.file_path}`);
  }
}

// remove an file item from the list and update the selected item index
function removeFromFileList(index: number = 0) {
  // remove file from list
  fileList.value.splice(index, 1);
  
  // update total file count and size
  totalFileCount.value = fileList.value.length;
  totalFileSize.value = fileList.value.reduce((total, file) => total + file.size, 0);
  
  // update selected item index (ensure it's always a valid number)
  if (fileList.value.length > 0) {
    selectedItemIndex.value = Math.min(index, fileList.value.length - 1);
  } else {
    selectedItemIndex.value = -1;
  }
}

async function refreshGroupedRowsAfterDelete(fileIds: number[]) {
  if (!groupedModeActive.value || fileIds.length === 0) return;
  pendingRestoreScrollTop.value = gridViewRef.value?.getScrollTop() ?? null;
  await initializeGroupedFileList(currentContentRequestId);
}

// update the file info from the file
const updateFile = async (file: any, showToast = false) => {
  try {
    const updatedFile = await updateFileInfo(file.id, file.file_path);
    if (updatedFile) {
      Object.assign(file, updatedFile);
      await updateThumbForFile(file);
      await updateSelectedImage(selectedItemIndex.value);

      // Force Image.vue to reload the saved image by briefly nullifying file_path
      // to trigger its filePath watcher (since the path itself hasn't changed, only the version)
      const savedPath = file.file_path;
      file.file_path = '';
      await nextTick();
      file.file_path = savedPath;

      // Clear CSS filter adjustments after image reload is triggered
      uiStore.clearActiveAdjustments();

      if (showToast) {
        toast.success(localeMsg.value.tooltip.update_image.success);
      }
    } else if (showToast) {
      toast.error(localeMsg.value.tooltip.update_image.failed);
    }
  } catch (err) {
    console.error('Failed to update file info:', err);
    if (showToast) {
      toast.error(localeMsg.value.tooltip.update_image.failed);
    }
  }
}

// force-update the thumbnail for the file
const VIDEO_THUMB_REFRESH_PERCENTS = [50, 90, 10];
const videoThumbRefreshIndex = new Map<number, number>();
const VIDEO_THUMB_REFRESH_MAP_MAX = 1000;

function getNextVideoThumbnailRefreshPercent(file: any) {
  if (file?.file_type !== 2 || !file?.id) return null;
  const fileId = Number(file.id);
  const currentIndex = videoThumbRefreshIndex.get(fileId) ?? 0;
  const percent = VIDEO_THUMB_REFRESH_PERCENTS[currentIndex % VIDEO_THUMB_REFRESH_PERCENTS.length];
  videoThumbRefreshIndex.set(fileId, (currentIndex + 1) % VIDEO_THUMB_REFRESH_PERCENTS.length);
  // Evict oldest entries when the map grows too large
  if (videoThumbRefreshIndex.size > VIDEO_THUMB_REFRESH_MAP_MAX) {
    const first = videoThumbRefreshIndex.keys().next().value;
    if (first !== undefined) videoThumbRefreshIndex.delete(first);
  }
  return percent;
}

const updateThumbForFile = async (file: any) => {
  file.thumbnail = '';
  clearCachedThumbnailDataUrl(file.id, config.settings.thumbnailSize);
  const thumb = await getFileThumb(
    file.id,
    file.file_path,
    file.file_type,
    file.e_orientation || 0,
    config.settings.thumbnailSize,
    true,
    getNextVideoThumbnailRefreshPercent(file)
  );
  if (thumb) {
    if (thumb.error_code === 0 || thumb.error_code === 2) {
      file.thumbnail = getThumbnailDataUrl(thumb, thumbnailPlaceholder, true, config.settings.thumbnailSize, file.file_path);
    } else if (thumb.error_code === 1) {
      file.thumbnail = thumbnailPlaceholder;
    }
  }
}

const syncFileMetaToImageViewer = async (fileId: number, changes: Record<string, any>) => {
  if (!fileId || !changes || Object.keys(changes).length === 0) return;
  tauriEmit('message-from-content', {
    message: 'update-file-meta',
    fileId,
    changes,
  });
};

type SavedFilePayload = {
  saveAsNew?: boolean;
  filePath?: string;
};

const insertIndexedFileIntoList = async (indexedFile: any) => {
  const position = await getCurrentQueryFilePosition(indexedFile.id);
  if (position === null || position < 0) {
    return false;
  }

  const nextFile = {
    ...indexedFile,
    isSelected: false,
    rotate: indexedFile.rotate || 0,
  };

  const existingIndex = fileList.value.findIndex((file: any) => {
    const sameId = Number(file?.id || 0) === Number(indexedFile.id || 0);
    const samePath = file?.file_path && indexedFile.file_path && file.file_path === indexedFile.file_path;
    return sameId || samePath;
  });

  if (existingIndex >= 0) {
    const previousSize = Number(fileList.value[existingIndex]?.size || 0);
    fileList.value.splice(existingIndex, 1, nextFile);
    totalFileSize.value += Number(nextFile.size || 0) - previousSize;
    selectedItemIndex.value = existingIndex;
    markDedupSourceUpdated(currentContentRequestId);

    await nextTick();
    const updatedFile = fileList.value[existingIndex];
    if (!updatedFile) {
      return false;
    }
    await updateThumbForFile(updatedFile);
    updateSelectedImage(existingIndex);
    openImageViewer(existingIndex, false, false);
    return true;
  }

  totalFileCount.value += 1;
  totalFileSize.value += nextFile.size || 0;
  fileList.value.splice(position, 0, nextFile);
  selectedItemIndex.value = position;
  markDedupSourceUpdated(currentContentRequestId);

  await nextTick();
  const insertedFile = fileList.value[position];
  if (!insertedFile) {
    return false;
  }
  await updateThumbForFile(insertedFile);
  updateSelectedImage(position);
  openImageViewer(position, false, false);
  return true;
};

const indexAndInsertSavedFile = async (filePath: string) => {
  const currentFile = fileList.value[selectedItemIndex.value];
  if (!currentFile?.folder_id) return false;

  const indexedFile = await addFileToDb(currentFile.folder_id, filePath);
  if (!indexedFile) return false;

  return insertIndexedFileIntoList(indexedFile);
};

// toggle the selected file's favorite status (selectMode = false)
const toggleFavorite = async () => {
  if (selectedItemIndex.value >= 0) {
    const item = fileList.value[selectedItemIndex.value];
    item.is_favorite = !item.is_favorite;
    // update the favorite status in the database
    await setFileFavorite(item.id, item.is_favorite);
    syncFileMetaToImageViewer(item.id, { is_favorite: item.is_favorite });
  }
};

// set selected files' favorite status (selectMode = true)
const selectModeSetFavorites = async (isFavorite: boolean) => {
  if (selectMode.value && selectedCount.value > 0) {
    const items = await getActionableSelectedItemsForAction();
    if (!items) return;
    if (!await confirmLargeBatch(items.length)) return;
    const result = await batchUpdateFileMetadata({
      fileIds: items.map(item => item.id),
      isFavorite,
    });
    if (result === null) return;
    items.forEach(item => {
      item.is_favorite = isFavorite;
    });
    const activeItem = fileList.value[selectedItemIndex.value];
    if (activeItem?.isSelected) {
      syncFileMetaToImageViewer(activeItem.id, { is_favorite: isFavorite });
    }
  }
}

const toggleSelectModeFavorite = async () => {
  if (!selectMode.value || selectedCount.value === 0) return;
  const selectedItems = await getActionableSelectedItemsForAction();
  if (!selectedItems || selectedItems.length === 0) return;
  const shouldFavorite = !selectedItems.every(item => Boolean(item.is_favorite));
  await selectModeSetFavorites(shouldFavorite);
};

const setSelectedFileRating = async (rating: number) => {
  if (selectedItemIndex.value >= 0) {
    const item = fileList.value[selectedItemIndex.value];
    const normalized = item.rating === rating ? 0 : rating;
    item.rating = normalized;
    await setFileRating(item.id, normalized);
    syncFileMetaToImageViewer(item.id, { rating: normalized });
  }
};

const selectModeSetRatings = async (rating: number) => {
  if (selectMode.value && selectedCount.value > 0) {
    const normalized = Math.max(0, Math.min(5, rating));
    const items = await getActionableSelectedItemsForAction();
    if (!items) return;
    if (!await confirmLargeBatch(items.length)) return;
    const result = await batchUpdateFileMetadata({
      fileIds: items.map(item => item.id),
      rating: normalized,
    });
    if (result === null) return;
    items.forEach(item => {
      item.rating = normalized;
    });
    const activeItem = fileList.value[selectedItemIndex.value];
    if (activeItem?.isSelected) {
      syncFileMetaToImageViewer(activeItem.id, { rating: normalized });
    }
  }
}

// slide show
let slideShowIntervalId: NodeJS.Timeout | null = null;

function toggleSlideShow() {
  isSlideShow.value = !isSlideShow.value;
  if (isSlideShow.value) {
    startSlideShow();
  } else {
    stopSlideShow();
  }
}

function clearSlideShowTimer() {
  if (slideShowIntervalId) {
    clearInterval(slideShowIntervalId);
    slideShowIntervalId = null;
  }
}

// Check if current file is a video
function isCurrentFileVideo() {
  const currentFile = fileList.value[selectedItemIndex.value];
  return currentFile?.file_type === 2;
}

// Advance to next slide (handles looping)
function advanceSlideShow() {
  if (fileList.value.length === 0) return;
  
  if (selectedItemIndex.value >= fileList.value.length - 1) {
    selectedItemIndex.value = 0; // Loop
  } else {
    selectedItemIndex.value++;
  }
  
  // Schedule next advance based on file type
  scheduleNextSlide();
}

// Schedule the next slide transition
function scheduleNextSlide() {
  clearSlideShowTimer();
  
  if (!isSlideShow.value) return;
  
  // If current file is video, don't set timer - video's ended event will trigger next
  if (isCurrentFileVideo()) {
    return;
  }
  
  // For images, use the configured interval
  const interval = getSlideShowInterval(config.settings.slideShowInterval) * 1000;
  slideShowIntervalId = setTimeout(() => {
    advanceSlideShow();
  }, interval);
}

function startSlideShow() {
  scheduleNextSlide();
}

function stopSlideShow() {
  isSlideShow.value = false;
  clearSlideShowTimer();
}

// Called when video ends in slideshow mode
function handleSlideshowNext() {
  if (isSlideShow.value) {
    advanceSlideShow();
  }
}

watch(() => config.settings.slideShowInterval, () => {
  if (isSlideShow.value && !isCurrentFileVideo()) {
    scheduleNextSlide();
  }
});

// set file rotate
const clickRotate = async () => {
  if (selectMode.value && selectedCount.value > 0) {
    const items = await getActionableSelectedItemsForAction();
    if (!items) return;
    if (!await confirmLargeBatch(items.length)) return;
    const result = await batchUpdateFileMetadata({
      fileIds: items.map(item => item.id),
      rotateDelta: 90,
    });
    if (result === null) return;
    items.forEach(item => {
      item.rotate = ((Number(item.rotate) || 0) + 90) % 360;
    });
    const activeItem = fileList.value[selectedItemIndex.value];
    if (activeItem?.isSelected) {
      tauriEmit('message-from-content', { message: 'rotate', fileId: activeItem.id });
      syncFileMetaToImageViewer(activeItem.id, { rotate: activeItem.rotate });
    }
    return;
  }

  if (selectedItemIndex.value >= 0) {
    fileList.value[selectedItemIndex.value].rotate += 90;

    // notify the image viewer
    tauriEmit('message-from-content', { message: 'rotate', fileId: fileList.value[selectedItemIndex.value].id });

    // update the rotate status in the database
    setFileRotate(fileList.value[selectedItemIndex.value].id, fileList.value[selectedItemIndex.value].rotate);
    syncFileMetaToImageViewer(fileList.value[selectedItemIndex.value].id, {
      rotate: fileList.value[selectedItemIndex.value].rotate,
    });
  }
};

// set file tag
const clickTag = async () => {
  console.log('clickTag');
  if (selectMode.value) {
    const items = await getActionableSelectedItemsForAction();
    if (!items) return;
    const fileIds = items.map(file => file.id);
    if (!await confirmLargeBatch(fileIds.length)) return;
    fileIdsToTag.value = fileIds;
  } else if (selectedItemIndex.value >= 0) {
    fileIdsToTag.value = [fileList.value[selectedItemIndex.value].id];
  } else {
    fileIdsToTag.value = [];
  }
  showTaggingDialog.value = true;
}

const onEditComment = async (newComment: any) => {
  if (selectMode.value && selectedCount.value > 0) {
    const items = await getActionableSelectedItemsForAction();
    if (!items) return;
    if (!await confirmLargeBatch(items.length)) return;
    const result = await batchUpdateFileMetadata({
      fileIds: items.map(item => item.id),
      comment: newComment,
    });
    if (result === null) return;
    items.forEach(item => {
      item.comments = newComment;
    });
    const activeItem = fileList.value[selectedItemIndex.value];
    if (activeItem?.isSelected) {
      syncFileMetaToImageViewer(activeItem.id, { comments: newComment });
    }
    showCommentMsgbox.value = false;
    return;
  }

  if (selectedItemIndex.value >= 0) {
    const file = fileList.value[selectedItemIndex.value];
    const result = await editFileComment(file.id, newComment);
    if(result) {
      console.log('onEditComment:', newComment);
      file.comments = newComment;
      showCommentMsgbox.value = false;
      syncFileMetaToImageViewer(file.id, { comments: newComment });
    }
  }
}

const openCommentEditor = () => {
  if ((selectMode.value && selectedCount.value > 0) || selectedItemIndex.value >= 0) {
    showCommentMsgbox.value = true;
  }
}

const selectAllInCurrentList = async () => {
  if (groupedModeActive.value) {
    const nextCounts: Record<string, number> = {};
    const nextSizes: Record<string, number> = {};
    const ids = await getCurrentQueryFileIds();
    if (!Array.isArray(ids)) {
      toast.error(t('info_panel.selection_load_failed'));
      return;
    }

    selectedFileIds.clear();
    for (const id of Array.isArray(ids) ? ids : []) {
      const fileId = Number(id);
      if (Number.isFinite(fileId) && fileId > 0) selectedFileIds.add(fileId);
    }

    for (const groupId of groupMetaMap.keys()) {
      const meta = groupMetaMap.get(groupId);
      nextCounts[groupId] = Number(meta?.count || 0);
      nextSizes[groupId] = Number(meta?.size || 0);
    }

    for (const file of fileList.value) {
      if (isRealFileItem(file)) file.isSelected = true;
    }
    groupSelectedCountMap.value = nextCounts;
    groupSelectedSizeMap.value = nextSizes;
    selectedCount.value = selectedFileIds.size;
    selectedSize.value = selectedFileIds.size === totalFileCount.value ? totalFileSize.value : Object.values(nextSizes)
      .reduce((total, size) => total + Number(size || 0), 0);
    syncSelectionVersions();
    selectMode.value = true;
    return;
  }

  const hasPlaceholders = fileList.value.some(isPendingFileItem);
  selectedFileIds.clear();
  if (hasPlaceholders) {
    const ids = await getCurrentQueryFileIds();
    if (!Array.isArray(ids)) {
      toast.error(t('info_panel.selection_load_failed'));
      return;
    }
    for (const id of ids) {
      const fileId = Number(id);
      if (Number.isFinite(fileId) && fileId > 0) selectedFileIds.add(fileId);
    }
  }

  let nextSize = 0;
  for (const file of fileList.value) {
    if (!file) continue;
    if (isRealFileItem(file)) {
      file.isSelected = true;
      selectedFileIds.add(Number(file.id));
      nextSize += Number(file.size || 0);
    } else {
      file.isSelected = false;
    }
  }
  recomputeLoadedGroupSelectedCounts();
  selectedCount.value = selectedFileIds.size;
  selectedSize.value = selectedFileIds.size === totalFileCount.value ? totalFileSize.value : nextSize;
  syncSelectionVersions();
  selectMode.value = true;
};

const selectNoneInCurrentList = () => {
  clearLoadedSelectionFlags();
  resetSelectionSummary();
  groupSelectedCountMap.value = {};
  groupSelectedSizeMap.value = {};
  selectMode.value = true;
};

const invertSelectionInCurrentList = async () => {
  selectedFileIds.clear();
  let nextCount = 0;
  let nextSize = 0;
  for (let i = 0; i < fileList.value.length; i++) {
    const file = fileList.value[i];
    if (!file) continue;
    if (!isRealFileItem(file)) {
      file.isSelected = false;
      continue;
    }
    file.isSelected = !file.isSelected;
    if (file.isSelected) {
      selectedFileIds.add(Number(file.id));
      nextCount++;
      nextSize += Number(file.size || 0);
    }
  }
  recomputeLoadedGroupSelectedCounts();
  selectedCount.value = nextCount;
  selectedSize.value = nextCount === totalFileCount.value ? totalFileSize.value : nextSize;
  syncSelectionVersions();
  selectMode.value = true;
};

const handleSelectMode = (value: any) => {
  if (isScanStreamingMode.value) return;
  selectMode.value = value;
  if(!selectMode.value) {
    clearLoadedSelectionFlags();
    resetSelectionSummary();
    groupSelectedCountMap.value = {};
    groupSelectedSizeMap.value = {};
  } else {
    if (fileList.value.length > 0) {
      if (selectedItemIndex.value >= 0 && selectedItemIndex.value < fileList.value.length) {
        ensureGroupedFileAtIndex(selectedItemIndex.value);
      }
      const fallbackIndex = fileList.value.findIndex(item => isRealFileItem(item));
      const targetIndex =
        selectedItemIndex.value >= 0 &&
        selectedItemIndex.value < fileList.value.length &&
        isRealFileItem(fileList.value[selectedItemIndex.value])
          ? selectedItemIndex.value
          : fallbackIndex;

      if (targetIndex >= 0) {
        selectedItemIndex.value = targetIndex;
        setItemSelected(targetIndex, true);
      }
    }
    showQuickView.value = false;
    stopSlideShow();
    config.rightPanel.show = false;
  }
};

const handleInfoNavigateFolder = (folderPath: string) => {
  const targetFile = fileList.value[selectedItemIndex.value];
  if (!folderPath || !targetFile?.album_id) return;
  enterAlbumPreviewMode(targetFile, folderPath);
};

const openAlbumEdit = (albumId: number) => {
  if (Number(albumId || 0) > 0) {
    tauriEmit('edit-album-requested', { albumId });
  }
};

const FILE_TYPE_IMAGE = 1;
const FILE_TYPE_VIDEO = 2;
const FILE_TYPE_RAW = 4;
const FILE_TYPE_ALL_MASK = FILE_TYPE_IMAGE | FILE_TYPE_VIDEO | FILE_TYPE_RAW;

function normalizeFileTypeMask(mask: number): number {
  if (!Number.isFinite(mask) || mask <= 0) return 0;
  const normalized = mask & FILE_TYPE_ALL_MASK;
  return normalized === 0 || normalized === FILE_TYPE_ALL_MASK ? 0 : normalized;
}

const emptyFilesMessage = computed(() => {
  // if (currentQuerySource.value === 'collection') {
  //   return `${localeMsg.value.collection.empty_content}`;
  // }
  const notFound = localeMsg.value.tooltip.not_found;
  const mask = normalizeFileTypeMask(Number(config.search.fileType || 0));
  const messageKey = {
    [FILE_TYPE_IMAGE]: 'image_files',
    [FILE_TYPE_VIDEO]: 'video_files',
    [FILE_TYPE_RAW]: 'raw_files',
    [FILE_TYPE_IMAGE | FILE_TYPE_VIDEO]: 'image_video_files',
    [FILE_TYPE_IMAGE | FILE_TYPE_RAW]: 'image_raw_files',
    [FILE_TYPE_VIDEO | FILE_TYPE_RAW]: 'raw_video_files',
  }[mask];

  return (messageKey && notFound[messageKey]) || notFound.files;
});

const emptyFilesHint = computed(() => {
  if (!showFolderFiles.value) return '';
  const notFound = localeMsg.value.tooltip.not_found;
  if (isCurrentFolderExcluded.value) return notFound.folder_excluded_hint || '';
  if (!config.settings.showSubfolderFiles) return notFound.folder_files_hint || '';
  return '';
});

const smartAlbumFileTypeMask = computed(() => {
  if (!isSmartAlbumView.value) return 0;
  const album = getActiveCustomSmartAlbum();
  if (!album) return 0;
  const ftRules = (album.query?.rules || []).filter((r: any) => r.field === 'file_type' && r.operator === 'is');
  return ftRules.reduce((mask: number, r: any) => mask | Number(r.value || 0), 0);
});

const activeFileTypeMask = computed(() =>
  isSmartAlbumView.value
    ? smartAlbumFileTypeMask.value
    : Number(config.search.fileType || 0)
);

const fileTypeSelectedValues = computed(() => {
  const mask = normalizeFileTypeMask(activeFileTypeMask.value);
  if (mask === 0) return [0];
  return [FILE_TYPE_IMAGE, FILE_TYPE_RAW, FILE_TYPE_VIDEO].filter(value => (mask & value) === value);
});

const fileTypeSummaryLabel = computed(() => {
  const options = fileTypeOptions.value;
  const mask = normalizeFileTypeMask(activeFileTypeMask.value);
  if (mask === 0) return options[0]?.label || '';

  const labels = [FILE_TYPE_IMAGE, FILE_TYPE_RAW, FILE_TYPE_VIDEO]
    .filter(value => (mask & value) === value)
    .map(value => options.find(option => option.value === value)?.label)
    .filter(Boolean);

  return labels.length > 0 ? labels.join(' + ') : (options[0]?.label || '');
});

const handleFileTypeSelect = (values: any[]) => {
  if (isScanStreamingMode.value) return;
  selectMode.value = false;   // exit multi-select mode
  const nextValues = (Array.isArray(values) ? values : []).map(value => Number(value));
  const hasAll = nextValues.includes(0);
  const mask = hasAll ? 0 : nextValues.reduce((acc, value) => acc | value, 0);
  if (isSmartAlbumView.value) {
    const album = getActiveCustomSmartAlbum();
    if (!album) return;
    album.query.rules = (album.query.rules || []).filter((r: any) => r.field !== 'file_type');
    if (mask !== 0) {
      album.query.rules.push({
        id: crypto.randomUUID?.() || `${Date.now()}-${Math.random()}`,
        field: 'file_type',
        operator: 'is',
        value: mask,
      });
    }
    return;
  }
  config.search.fileType = normalizeFileTypeMask(mask);
};

const handleSortTypeSelect = (option: any, extendOption: any) => {
  if (isScanStreamingMode.value) return;
  selectMode.value = false;   // exit multi-select mode
  if (isSmartAlbumView.value) {
    const album = getActiveCustomSmartAlbum();
    if (album) {
      album.sort.type = option;
      album.sort.order = extendOption;
    }
  } else {
    config.search.sortType = option;
    config.search.sortOrder = extendOption;
  }
};

const toggleInfoPanel = () => {
  checkUnsavedChanges(() => {
    if (isInfoPanelOpen.value) {
      config.rightPanel.show = false;
      return;
    }
    handleSelectMode(false);
    config.rightPanel.mode = 'info';
    config.rightPanel.show = true;
  });
};

const toggleDedupPanel = () => {
  if (isScanStreamingMode.value) return;
  checkUnsavedChanges(() => {
    if (isDedupPanelOpen.value) {
      config.rightPanel.show = false;
      return;
    }
    handleSelectMode(false);
    disablePreviewModes();
    config.rightPanel.mode = 'dedup';
    config.rightPanel.show = true;
  });
};

let dedupNavigationRequestId = 0;

async function resolveGroupedFileIndexForDedup(fileId: number): Promise<number> {
  let fileIndexCursor = 0;

  for (const group of groupedTimelineGroups.value) {
    const groupId = String(group.groupId || '');
    const groupCount = Number(group.count || 0);
    if (!groupId || groupCount <= 0) {
      fileIndexCursor += Math.max(0, groupCount);
      continue;
    }

    const ids = await getCachedGroupFileIds(groupId);
    const groupFileIndex = Array.isArray(ids)
      ? ids.findIndex((id: number) => Number(id) === Number(fileId))
      : -1;
    if (groupFileIndex >= 0) {
      return fileIndexCursor + groupFileIndex;
    }

    fileIndexCursor += groupCount;
  }

  return -1;
}

async function resolveFileIndexForDedup(fileId: number): Promise<number> {
  const loadedIndex = fileList.value.findIndex(file => file.id === fileId);
  if (loadedIndex !== -1) return loadedIndex;

  const position = groupedModeActive.value
    ? await resolveGroupedFileIndexForDedup(fileId)
    : await getCurrentQueryFilePosition(fileId);
  if (position === null || position < 0 || position >= totalFileCount.value) {
    return -1;
  }

  const buffer = 200;
  if (groupedModeActive.value) {
    const rowIndex = getGroupedRowIndexForFileIndex(position);
    if (rowIndex < 0) return -1;
    await fetchGroupedRowsRange(rowIndex - buffer, rowIndex + buffer);
    if (!ensureGroupedFileAtIndex(position)) return -1;
  } else {
    await fetchDataRange(position - buffer, position + buffer);
  }

  return isRealFileItem(fileList.value[position]) ? position : -1;
}

const handleDedupSelectFile = (fileId: number) => {
  const requestId = ++dedupNavigationRequestId;
  checkUnsavedChanges(async () => {
    const index = await resolveFileIndexForDedup(fileId);
    if (requestId !== dedupNavigationRequestId) return;
    if (index === -1) return;
    selectedItemIndex.value = index;
    updateSelectedImage(index);
  });
};

const handleDedupPreviewFile = (fileId: number) => {
  const requestId = ++dedupNavigationRequestId;
  checkUnsavedChanges(async () => {
    const index = await resolveFileIndexForDedup(fileId);
    if (requestId !== dedupNavigationRequestId) return;
    if (index === -1) return;
    selectedItemIndex.value = index;
    handleItemDblClicked(index);
  });
};

const handleDedupTrashSelectedDuplicates = (groupKey: string, fileIds: number[], reclaimableBytes: number) => {
  if (!groupKey || !fileIds || fileIds.length === 0) return;
  openTrashMsgbox(reclaimableBytes, groupKey, fileIds);
};

// file type options
const fileTypeOptions = computed(() => {
  const options = localeMsg.value.toolbar.filter?.file_type_options || [];
  return [
    { label: options[0] || 'All', value: 0 },
    { label: options[1] || 'Image', value: FILE_TYPE_IMAGE },
    { label: options[2] || 'RAW', value: FILE_TYPE_RAW },
    { label: options[3] || 'Video', value: FILE_TYPE_VIDEO },
  ];
});

// sort type options
const sortOptions = computed(() => {
  return getSelectOptions(localeMsg.value.toolbar.filter?.sort_type_options);
});

// sort extend options
const sortExtendOptions = computed(() => {
  return getSelectOptions(localeMsg.value.toolbar.filter?.sort_order_options);
});

const toolbarSortType = computed(() => (
  config.main.sidebarIndex === SIDEBAR.SMART_ALBUM
    ? Number(getActiveCustomSmartAlbum()?.sort?.type ?? 0)
    : Number(config.search.sortType || 0)
));

const toolbarSortOrder = computed(() => (
  config.main.sidebarIndex === SIDEBAR.SMART_ALBUM
    ? Number(getActiveCustomSmartAlbum()?.sort?.order ?? 1)
    : Number(config.search.sortOrder || 0)
));

const isSearchLikeView = computed(() => {
  return config.main.sidebarIndex === SIDEBAR.SMART_ALBUM ||
    config.main.sidebarIndex === SIDEBAR.SEARCH ||
    (config.main.sidebarIndex === SIDEBAR.LIBRARY && libConfig.library.item === LIB_ITEM.SUBJECTS);
});

const isAiSearchMode = computed(() =>
  config.main.sidebarIndex === SIDEBAR.SEARCH ||
  (config.main.sidebarIndex === SIDEBAR.LIBRARY && libConfig.library.item === LIB_ITEM.SUBJECTS)
);

const isSmartAlbumView = computed(() =>
  config.main.sidebarIndex === SIDEBAR.SMART_ALBUM
);

// update image when the select file is changed
async function updateSelectedImage(index: number) {
  if(index < 0 || index >= fileList.value.length) return;

  // update the tags for the selected file
  if(isInfoPanelOpen.value && fileList.value[index].has_tags) {
    fileList.value[index].tags = await getTagsForFile(fileList.value[index].id);
  }
}

// click ok in tagging dialog
async function updateFileHasTags(fileStates: Array<{ file_id: number; has_tags: boolean }>) {
  showTaggingDialog.value = false;
  const filesById = new Map(
    fileList.value
      .filter(file => isRealFileItem(file))
      .map(file => [Number(file.id), file]),
  );
  for (const state of fileStates) {
    const file = filesById.get(Number(state.file_id));
    if (!file) continue;
    file.has_tags = Boolean(state.has_tags);
    file.tags = undefined;
  }

  const activeFile = fileList.value[selectedItemIndex.value];
  const activeState = fileStates.find(state => Number(state.file_id) === Number(activeFile?.id));
  if (activeFile && activeState) {
    activeFile.tags = activeState.has_tags ? ((await getTagsForFile(activeFile.id)) || []) : [];
    syncFileMetaToImageViewer(activeFile.id, {
      has_tags: activeFile.has_tags,
      tags: activeFile.tags,
    });
  }
}

// Helper to yield to main thread
const yieldToMain = () => new Promise(resolve => setTimeout(resolve, 0));

// Track current thumbnail request to enable cancellation when switching folders
let currentThumbRequestId = 0;

function preserveLoadedThumbnails(files: any[]) {
  const thumbnailsById = new Map<number, string>();
  for (const file of fileList.value || []) {
    const fileId = Number(file?.id || 0);
    if (fileId > 0 && file?.thumbnail) {
      thumbnailsById.set(fileId, file.thumbnail);
    }
  }

  return (files || []).map((file: any) => {
    const fileId = Number(file?.id || 0);
    if (!file?.thumbnail && fileId > 0) {
      const thumbnail = thumbnailsById.get(fileId);
      if (thumbnail) return { ...file, thumbnail };
    }
    return file;
  });
}

// Get the thumbnail for the files (non-blocking, runs in background)
// Automatically cancels when a new request starts (e.g., switching folders)
async function getFileListThumb(files: any[], offset = 0, concurrencyLimit = 4, bustCache = false) {
  // Use current request ID to check for cancellation
  const requestId = currentThumbRequestId;
  const thumbnailSize = config.settings.thumbnailSize;
  const batchSize = Math.max(1, concurrencyLimit);

  const applyThumbToFile = (file: any, thumb: any) => {
    if (!thumb) return;

    if (thumb.error_code === 0 || thumb.error_code === 2) {
      file.thumbnail = getThumbnailDataUrl(thumb, thumbnailPlaceholder, bustCache, thumbnailSize, file.file_path);
    } else if (thumb.error_code === 1) {
      file.thumbnail = thumbnailPlaceholder;
    }
    thumbCount.value++;
  };

  const processBatch = async (startIndex: number) => {
    if (requestId !== currentThumbRequestId) return;

    const endIndex = Math.min(startIndex + batchSize, files.length);
    const batchFiles: any[] = [];

    for (let i = startIndex; i < endIndex; i++) {
      const file = files[i];
      if (!file || file.thumbnail) continue;

      const cached = getCachedThumbnailDataUrl(file.id, thumbnailSize);
      if (cached) {
        file.thumbnail = cached;
        thumbCount.value++;
        continue;
      }

      batchFiles.push(file);
    }

    if (batchFiles.length === 0) return;

    const requests = batchFiles.map((file: any) => ({
      fileId: file.id,
      filePath: file.file_path,
      fileType: file.file_type,
      orientation: file.e_orientation || 0,
      albumId: file.album_id || 0,
    }));

    try {
      const thumbs = await getFileThumbs(requests, thumbnailSize, false);

      if (requestId !== currentThumbRequestId) return;

      for (let j = 0; j < batchFiles.length; j++) {
        applyThumbToFile(batchFiles[j], Array.isArray(thumbs) ? thumbs[j] : null);
      }
    } catch (error) {
      console.log('Error fetching thumbnails:', error);
    }
  };

  const runWithConcurrencyLimit = async (files: any[]) => {
    const queue: Promise<void>[] = [];
    let activeRequests = 0;

    for (let i = offset; i < files.length; i += batchSize) {
      if (requestId !== currentThumbRequestId) {
        console.log(`Thumbnail generation cancelled (request ${requestId} superseded by ${currentThumbRequestId})`);
        return;
      }

      // Wait for a slot to free up before starting a new batch
      while (activeRequests >= concurrencyLimit) {
        await Promise.race(queue);
      }

      const batchPromise = processBatch(i)
        .then(() => {
          queue.splice(queue.indexOf(batchPromise), 1);
          activeRequests--;
        })
        .catch(() => {
          queue.splice(queue.indexOf(batchPromise), 1);
          activeRequests--;
        });

      queue.push(batchPromise);
      activeRequests++;

      // Yield to main thread periodically to keep UI responsive
      if (i > 0 && i % (batchSize * 10) === 0) {
        await yieldToMain();
      }
    }

    // Wait for remaining batches (only if not cancelled)
    if (requestId === currentThumbRequestId && queue.length > 0) {
      await Promise.all(queue);
    }
  };

  return runWithConcurrencyLimit(files);
}

// Open the image viewer window
async function openImageViewer(
  index: number,
  newViewer = false,
  syncFromFileListChange = false,
  options: { rightIndex?: number; forceSplit?: boolean } = {}
) {

  const webViewLabel = 'imageviewer';

  const fileCount = fileList.value.length;
  const isRealFile = (item: any) => !!item && !item.isPlaceholder && typeof item.id === 'number';
  const getRealFileAt = (targetIndex: number) => {
    if (targetIndex < 0 || targetIndex >= fileCount) return null;
    const file = fileList.value[targetIndex];
    return isRealFile(file) ? file : null;
  };
  const getNextImageFilePath = (targetIndex: number) => {
    const file = getRealFileAt(targetIndex + 1);
    return file?.file_type === 1 ? file.file_path : '';
  };

  let leftIndex = index;
  let rightIndex = -1;

  if (syncFromFileListChange) {
    if (fileCount === 0) {
      leftIndex = -1;
      rightIndex = -1;
    } else if (fileCount === 1) {
      leftIndex = 0;
      rightIndex = 0;
    } else {
      leftIndex = 0;
      rightIndex = 1;
    }
  }
  if (typeof options.rightIndex === 'number') {
    rightIndex = options.rightIndex;
  }
  const compareMode = options.forceSplit === true;
  if (options.forceSplit) {
    if (rightIndex < 0 && fileCount > 0) {
      rightIndex = Math.min(leftIndex + 1, fileCount - 1);
    }
  }

  const leftFile = getRealFileAt(leftIndex);
  const rightFile = getRealFileAt(rightIndex);
  const leftFileId = leftFile ? leftFile.id : 0;
  const rightFileId = rightFile ? rightFile.id : 0;
  const leftNextFilePath = getNextImageFilePath(leftIndex);
  const rightNextFilePath = getNextImageFilePath(rightIndex);
  
  // create a new window if it doesn't exist
  let imageWindow = await WebviewWindow.getByLabel(webViewLabel);
  if (!imageWindow) {
    if (newViewer) {
      const forceSplitParam = options.forceSplit ? 1 : 0;
      const compareModeParam = compareMode ? 1 : 0;
      imageWindow = new WebviewWindow(webViewLabel, {
        url: `/image-viewer?fileId=${leftFileId}&fileIndex=${leftIndex}&fileCount=${fileCount}&rightFileId=${rightFileId}&rightFileIndex=${rightIndex}&forceSplit=${forceSplitParam}&compareMode=${compareModeParam}&nextFilePath=${encodeURIComponent(leftNextFilePath)}&rightNextFilePath=${encodeURIComponent(rightNextFilePath)}`,
        title: 'Image Viewer',
        width: 1200,
        height: 800,
        minWidth: 800,
        minHeight: 600,
        resizable: true,
        visible: false, // Start hidden, will show after mount
        transparent: true, // Prevent white flash on show (Tauri 2.x workaround)
        decorations: isMac,
        zoomHotkeysEnabled: true, // Windows WebView2: needed for touchpad pinch-to-zoom
        ...(isMac && {
          titleBarStyle: 'overlay',
          hiddenTitle: true,
          minimizable: true,
        }),
      });

      imageWindow.once('tauri://created', () => {
        console.log('ImageViewer window created');
        videoRef.value?.pause();  // pause video playing in preview pane
      });

      imageWindow.once('tauri://close-requested', () => {
        imageWindow?.close();
      });

      imageWindow.once('tauri://error', (e) => {
        console.error('Error creating ImageViewer window:', e);
      });
    }
  } else {    // update the existing window
    await imageWindow.emit('update-img', { 
      fileId: leftFileId, 
      fileIndex: leftIndex,   // selected file index
      fileCount: fileCount, // total files length
      nextFilePath: leftNextFilePath,
      pane: 'left',
      resetSplit: newViewer,
      compareMode: newViewer ? compareMode : undefined,
      forceSyncViewport: compareMode ? true : undefined,
      forceSplit: options.forceSplit === true ? true : undefined,
      // filePath: encodedFilePath, 
      // nextFilePath: nextEncodedFilePath,
    });

    if (syncFromFileListChange || rightIndex >= 0) {
      await imageWindow.emit('update-img', {
        fileId: rightFileId,
        fileIndex: rightIndex,
        fileCount: fileCount,
        nextFilePath: rightNextFilePath,
        pane: 'right',
      });
    }

    if(newViewer) {
      imageWindow.show();
    }
    videoRef.value?.pause();  // pause video playing in preview pane
  }
}

async function openImageEditor(index: number) {
  const file = fileList.value[index];
  if (!file) return;
  const fileId = Number(file.id || 0);
  if (fileId <= 0) return;

  const webViewLabel = 'imageeditor';
  const imageWindow = await WebviewWindow.getByLabel(webViewLabel);
  if (imageWindow) {
    await imageWindow.emit('update-file', { fileId });
    imageWindow.show();
    imageWindow.setFocus();
    return;
  }

  const newWindow = new WebviewWindow(webViewLabel, {
    url: `/image-editor?fileId=${fileId}`,
    title: 'Image Editor',
    width: 1100,
    height: 700,
    minWidth: 800,
    minHeight: 500,
    resizable: true,
    maximizable: false,
    visible: false,
    transparent: true,
    decorations: isMac,
    ...(isMac && {
      titleBarStyle: 'overlay',
      hiddenTitle: true,
      minimizable: false,
    }),
  });

  newWindow.once('tauri://created', () => {
    newWindow?.show();
  });

}

const printImageSrc = ref('');
const printImageRef = ref<HTMLImageElement | null>(null);

async function waitForPrintImage() {
  await nextTick();
  const image = printImageRef.value;
  if (!image) throw new Error('Print image element was not rendered');
  if (image.complete) {
    if (image.naturalWidth > 0) return;
    throw new Error('Print image failed to load');
  }

  await new Promise<void>((resolve, reject) => {
    image.addEventListener('load', () => resolve(), { once: true });
    image.addEventListener('error', () => reject(new Error('Print image failed to load')), { once: true });
  });
}

async function printImage(index: number) {
  const selectedFile = fileList.value[index];
  if (!selectedFile?.file_path) return;

  const fileId = Number(selectedFile.id || 0);
  const fileType = Number(selectedFile.file_type || 1);

  try {
    printImageSrc.value = shouldUseBackendPreview(selectedFile.file_path, fileType)
      ? getPreviewUrl(fileId, selectedFile.file_path)
      : getAssetSrc(selectedFile.file_path);
    await waitForPrintImage();
    // Defer to let the context menu / UI close before the synchronous print dialog opens
    setTimeout(() => {
      window.addEventListener('afterprint', () => {
        printImageSrc.value = '';
      }, { once: true });
      window.print();
    }, 100);
  } catch (error) {
    printImageSrc.value = '';
    console.error('Failed to prepare image for printing:', error);
  }
}

/// Dragging the film strip view splitter
// function startDraggingfilmStripView(event: MouseEvent) {
//   isDraggingFilmStripView.value = true;
//   document.addEventListener('mousemove', handleMouseMove);
//   document.addEventListener('mouseup', stopDragging);
// }

/// Dragging the info panel splitter
function startDraggingInfoPanelSplitter(event: MouseEvent) {
  isDraggingInfoPanel.value = true;
  rightPanelDragStartX.value = event.clientX;
  rightPanelDragStartWidth.value = Number(config.rightPanel.width || 360);
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', stopDragging);
}

/// handle mouse move event
function handleMouseMove(event: MouseEvent) {
  if (isDraggingInfoPanel.value) {
    const deltaX = rightPanelDragStartX.value - event.clientX;
    const newWidth = rightPanelDragStartWidth.value + deltaX;
    config.rightPanel.width = clampRightPanelWidth(newWidth);
  }
}

function stopDragging() {
  // isDraggingFilmStripView.value = false;
  isDraggingInfoPanel.value = false;
  rightPanelDragStartX.value = 0;
  rightPanelDragStartWidth.value = 0;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', stopDragging);
}

defineExpose({
  focusContent: activateContentPane,
  refreshCenteredGridLayout,
});
</script>

<style scoped>
.print-only {
  display: none;
}

@media print {
  @page {
    margin: 0;
  }

  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  :global(body > *:not(.print-only)) {
    display: none !important;
  }

  .print-only {
    position: absolute;
    inset: 0;
    box-sizing: border-box;
    display: grid !important;
    place-items: center;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: #fff;
    break-inside: avoid;
  }

  .print-only img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: contain;
    object-position: center;
  }
}

.drop-overlay {
  position: absolute;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(59, 130, 246, 0.12);
  border: 3px dashed rgba(59, 130, 246, 0.5);
  border-radius: 8px;
  pointer-events: none;
  backdrop-filter: blur(2px);
}

.drop-overlay-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: rgba(59, 130, 246, 0.9);
  font-size: 1.1rem;
  font-weight: 500;
}
</style>
