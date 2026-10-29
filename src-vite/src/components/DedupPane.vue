<template>
  <div class="w-full h-full rounded-box bg-base-200 flex flex-col overflow-hidden">
    <div class="my-2 px-2 flex items-center justify-between w-full shrink-0">
      <div class="flex items-center text-sm font-semibold gap-1">
        <button
          class="px-1 cursor-pointer"
          :class="activeTab === 'duplicates' ? 'text-primary/70' : 'hover:text-base-content'"
          @click="selectDuplicatesTab"
        >
          {{ $t('info_panel.dedup.tabs.exact') }}
        </button>
        <button
          class="px-1 cursor-pointer"
          :class="activeTab === 'similar' ? 'text-primary/70' : 'hover:text-base-content'"
          @click="openSimilarTab()"
        >
          {{ $t('info_panel.dedup.tabs.similar') }}
        </button>
      </div>
      <div class="flex items-center gap-1">
        <TButton
          v-if="activeTab === 'duplicates'"
          :icon="IconRefresh"
          :tooltip="$t('info_panel.dedup.rescan')"
          :buttonSize="'small'"
          :disabled="isDedupLoading"
          @click="triggerBackendDedup(true)"
        />
        <TButton
          v-else
          :icon="IconRefresh"
          :tooltip="$t('info_panel.dedup.similar.reanalyze')"
          :buttonSize="'small'"
          :disabled="isDedupLoading || similarLoading || !similarHasScanned"
          @click="reanalyzeSimilar"
        />
        <TButton
          :icon="IconClose"
          :tooltip="$t('msgbox.close')"
          :buttonSize="'small'"
          @click.stop="$emit('close')"
        />
      </div>
    </div>

    <div
      class="mb-2 px-2 flex-1 min-h-0 overflow-hidden flex flex-col"
    >
      <template v-if="activeTab === 'similar'">
        <div v-if="similarLoading" class="p-4 flex-1 flex items-center justify-center">
          <div class="text-center text-base-content/30 space-y-3 max-w-65">
            <span class="loading loading-spinner text-primary w-8 h-8 mx-auto"></span>
            <p class="text-xs font-medium">{{ similarProgressLabel }}</p>
            <progress class="progress progress-primary w-full" :value="similarStatus.current" :max="Math.max(1, similarStatus.total)"></progress>
            <button class="btn btn-sm" @click="cancelSimilar">{{ $t('info_panel.dedup.similar.cancel') }}</button>
          </div>
        </div>
        <div v-else-if="similarError" class="p-4 flex-1 flex items-center justify-center">
          <div class="text-center text-base-content/30 space-y-3 max-w-65">
            <p class="text-xs font-medium">{{ $t('info_panel.dedup.similar.error_title') }}</p>
            <PanelActionButton
              class="mx-auto"
              :icon="IconRefresh"
              primary
              @click="openSimilarTab(true)"
            >
              {{ $t('info_panel.dedup.rescan') }}
            </PanelActionButton>
          </div>
        </div>
        <div v-else-if="similarGroups.length === 0" class="p-4 flex-1 flex items-center justify-center">
          <div class="text-center text-base-content/30 space-y-3 max-w-65">
            <p v-if="similarHasScanned || (!similarEligibleCountLoading && similarEligibleCount === 0)" class="text-sm">{{ $t('info_panel.dedup.similar.empty_title') }}</p>
            <template v-else>
              <p class="text-sm">{{ $t('info_panel.dedup.similar.description') }}</p>
              <PanelActionButton primary :disabled="isDedupLoading || similarEligibleCountLoading || similarEligibleCount === 0 || similarLoading" @click="startSimilar">
                {{ similarEligibleCountLoading ? $t('tooltip.loading') : $t('info_panel.dedup.similar.analyze', { count: similarEligibleCount.toLocaleString() }) }}
              </PanelActionButton>
            </template>
          </div>
        </div>
        <div v-else class="flex min-h-0 flex-1 flex-col">
          <section class="mb-2 flex shrink-0 flex-wrap items-center gap-x-3 gap-y-2 rounded-box border border-base-content/10 bg-base-100/40 px-3 py-2" aria-label="Similar photo summary">
            <div class="min-w-0">
              <div class="text-[10px] font-bold uppercase tracking-widest text-base-content/30">
                {{ $t('info_panel.dedup.similar.results') }}
              </div>
              <div class="text-xs text-base-content/60">
                {{ $t('info_panel.dedup.similar_photos_summary', {
                  count: similarPhotoCount.toLocaleString(),
                  size: formatFileSize(similarPhotoBytes),
                }) }}
              </div>
            </div>
          </section>
          <div ref="similarSplitPaneRef" class="flex min-h-0 flex-1 flex-col">
          <div
            class="min-h-0 shrink-0 flex flex-col border-t border-base-content/5 px-1 py-3 space-y-3"
            :style="{ height: `${config.dedup.duplicateSetsHeight}%` }"
          >
              <div class="flex items-center gap-2">
                <span class="text-[10px] uppercase tracking-widest font-bold text-base-content/30">
                  {{ $t('info_panel.dedup.similar.groups_title') }}
                </span>
                <span class="ml-auto sidebar-item-count">
                  {{ similarTotalGroups.toLocaleString() }}
                </span>
              </div>
            <div ref="similarGroupsScrollRef" class="min-h-0 flex-1 overflow-y-auto py-1" @scroll="loadMoreSimilarGroups">
            <div class="mx-2 grid grid-cols-[repeat(auto-fill,minmax(5rem,1fr))] gap-2">
              <button
                v-for="group in visibleSimilarGroups"
                :key="group.id"
                class="group/thumb relative h-20 min-w-0 overflow-hidden rounded-box cursor-pointer"
                :class="selectedSimilarGroupId === group.id ? 'ring-2 ring-primary' : ''"
                :data-similar-group-id="group.id"
                @click="selectSimilarGroup(group)"
              >
                <img v-if="group.representative?.thumbnail" :src="group.representative.thumbnail" class="h-full w-full object-cover" loading="lazy" />
                <div v-else class="h-full w-full skeleton"></div>
                <div class="absolute left-0.5 top-0.5 thumb-badge">
                  {{ group.file_count }}
                </div>
                <div v-if="formatLabel(group.representative)" class="absolute bottom-0.5 left-0.5 thumb-badge text-[9px]">{{ formatLabel(group.representative) }}</div>
              </button>
            </div>
            </div>
          </div>
          <div
            v-if="activeSimilarGroup"
            class="-mx-2 z-10 flex h-1 border-b border-base-content/5 shrink-0 touch-none cursor-row-resize items-center select-none"
            @pointerdown.prevent="startDraggingDuplicateSplitter"
          >
            <div class="h-1 w-full splitter-indicator splitter-horizontal" :class="{ 'splitter-dragging': isDraggingDuplicateSplitter }"></div>
          </div>
          <div v-if="activeSimilarGroup" class="min-h-0 flex-1 overflow-y-auto px-1 py-3 space-y-3">
            <div class="flex items-center gap-2">
              <span class="text-[10px] uppercase tracking-widest font-bold text-base-content/30">
                {{ $t('info_panel.dedup.actions_title') }}
              </span>
              <span v-if="selectedSimilarCount > 0" class="ml-auto text-xs text-base-content/70">
                {{ $t('toolbar.filter.select_count', { count: selectedSimilarCount.toLocaleString() }) }} · {{ formatFileSize(selectedSimilarBytes) }}
              </span>
            </div>
            <div class="flex flex-wrap gap-1">
              <PanelActionButton
                :icon="isAllSimilarItemsSelected(activeSimilarGroup.id) ? IconUnChecked : IconChecked"
                @click="selectAllSimilarItems(activeSimilarGroup)"
              >
                {{ isAllSimilarItemsSelected(activeSimilarGroup.id) ? $t('menu.select.none') : $t('menu.select.all') }}
              </PanelActionButton>
              <PanelActionButton :icon="selectedSimilarCount >= 2 ? IconSplitOn4 : IconSplitOn" :disabled="selectedSimilarCount === 0 || !hasSimilarKeep" @click="compareSelectedSimilarPhotos">
                {{ $t('info_panel.dedup.compare') }}
              </PanelActionButton>
              <PanelActionButton :icon="IconTrash" :disabled="selectedSimilarCount === 0" danger @click="trashSelectedSimilar(activeSimilarGroup.id, selectedSimilarBytes)">
                {{ $t('info_panel.dedup.delete_selected') }}
              </PanelActionButton>
            </div>
            <TransitionGroup
              :key="activeSimilarGroup.id"
              tag="div"
              name="dedup-item"
              move-class="transition-transform duration-200 ease-out"
              class="space-y-2.5"
            >
              <div
                v-for="item in activeSimilarGroup.items"
                :key="item.file_id"
                role="button"
                tabindex="0"
                class="w-full rounded-box p-2.5 border text-left transition-colors cursor-pointer"
                :class="getDedupItemClass(item.file_id, item.is_keep !== 1 && isSimilarSelected(activeSimilarGroup.id, item.file_id))"
                @click="handleSimilarSelection(item.file_id)"
                @dblclick="handleSimilarSelection(item.file_id, true)"
                @keydown.enter.self="handleSimilarSelection(item.file_id)"
                @keydown.space.self.prevent="handleSimilarSelection(item.file_id)"
              >
                <div class="flex items-center gap-2">
                  <label v-if="item.is_keep !== 1" class="flex items-center shrink-0" :class="isOriginalUnavailable(item.file) ? 'cursor-not-allowed' : 'cursor-pointer'" @click.stop @dblclick.stop>
                    <input
                      type="checkbox"
                      class="checkbox checkbox-xs disabled:cursor-not-allowed"
                      :class="isOriginalUnavailable(item.file) ? 'opacity-30' : 'checkbox-primary opacity-70'"
                      :checked="isSimilarSelected(activeSimilarGroup.id, item.file_id)"
                      :disabled="isOriginalUnavailable(item.file)"
                      @change="toggleSimilarSelected(activeSimilarGroup.id, item.file_id)"
                    />
                  </label>
                  <div v-else class="w-4 shrink-0"></div>
                  <div class="w-10 h-10 rounded-box overflow-hidden shrink-0">
                    <img v-if="item.file?.thumbnail" :src="item.file.thumbnail" class="w-full h-full object-cover" />
                    <div v-else class="w-full h-full skeleton"></div>
                  </div>
                  <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-1 text-xs font-semibold text-base-content/70">
                      <span class="truncate">{{ item.file?.name }}</span>
                      <span v-if="formatLabel(item.file)" class="shrink-0 rounded bg-base-content/10 px-1 py-0.5 text-[9px] font-semibold text-base-content/70">{{ formatLabel(item.file) }}</span>
                      <IconExclamation v-if="isOriginalUnavailable(item.file)" class="w-3.5 h-3.5 shrink-0 text-warning" />
                    </div>
                    <div class="text-[11px] text-base-content/30 truncate">
                      <template v-if="item.file?.width && item.file?.height">
                        {{ item.file.width }} × {{ item.file.height }}
                      </template>
                      <template v-else>—</template>
                      · {{ formatFileSize(Number(item.file?.size || 0)) }}
                    </div>
                    <div v-if="item.file?.modified_at" class="text-[11px] text-base-content/30">
                      {{ $t('file_info.modified_at') }}: {{ formatTimestamp(item.file.modified_at, 'date_time') }}
                    </div>
                  </div>
                  <div class="shrink-0 w-16 min-h-10 flex flex-col items-center justify-center gap-0.5">
                    <button
                      class="btn btn-ghost btn-xs min-h-0 h-5 w-5 p-0"
                      :class="item.is_keep === 1 ? 'text-primary' : 'text-base-content/30 hover:text-primary/70'"
                      :title="$t(item.is_keep === 1 ? 'info_panel.dedup.keep_label' : 'info_panel.dedup.unkeep_label')"
                      :aria-label="$t(item.is_keep === 1 ? 'info_panel.dedup.keep_label' : 'info_panel.dedup.unkeep_label')"
                      :aria-current="item.is_keep === 1 ? 'true' : undefined"
                      @click.stop="item.is_keep !== 1 && setSimilarKeep(activeSimilarGroup.id, item.file_id)"
                    >
                      <component :is="item.is_keep === 1 ? IconLock : IconUnlock" class="w-3.5 h-3.5" />
                    </button>
                    <div class="flex items-center gap-0.5" @click.stop>
                      <button
                        class="btn btn-ghost btn-xs min-h-0 h-5 w-5 p-0"
                        :class="getSimilarCullingIconClass(item.file, 1)"
                        :title="$t('culling.picks')"
                        :aria-label="$t('culling.picks')"
                        @click.stop="setSimilarCullingFlag(item, 1)"
                      >
                        <IconFlagFilled class="w-3.5 h-3.5" />
                      </button>
                      <button
                        class="btn btn-ghost btn-xs min-h-0 h-5 w-5 p-0"
                        :class="getSimilarCullingIconClass(item.file, 2)"
                        :title="$t('culling.rejected')"
                        :aria-label="$t('culling.rejected')"
                        @click.stop="setSimilarCullingFlag(item, 2)"
                      >
                        <IconFlagOff class="w-3.5 h-3.5" />
                      </button>
                      <button
                        class="btn btn-ghost btn-xs min-h-0 h-5 w-5 p-0"
                        :class="getSimilarCullingIconClass(item.file, 0)"
                        :title="$t('culling.unreviewed')"
                        :aria-label="$t('culling.unreviewed')"
                        @click.stop="setSimilarCullingFlag(item, 0)"
                      >
                        <IconFlag class="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </TransitionGroup>
          </div>
          </div>
        </div>
      </template>
      <template v-else>
      <div v-if="isDedupLoading" class="p-4 flex-1 flex items-center justify-center">
        <div class="text-center text-base-content/30 space-y-3 max-w-65">
          <span class="loading loading-spinner text-primary w-8 h-8 mx-auto"></span>
          <p class="text-xs font-medium">{{ $t('info_panel.dedup.scanning') }}</p>
        </div>
      </div>

      <div v-else-if="dedupScanError" class="p-4 flex-1 flex items-center justify-center">
        <div class="text-center text-base-content/30 space-y-3 max-w-65">
          <p class="text-xs font-medium">{{ $t('info_panel.dedup.error_title') }}</p>
          <p class="text-xs text-base-content/30">{{ $t('info_panel.dedup.error_desc') }}</p>
          <PanelActionButton
            class="mx-auto"
            :icon="IconRefresh"
            primary
            @click="triggerBackendDedup(true)"
          >
            {{ $t('info_panel.dedup.rescan') }}
          </PanelActionButton>
        </div>
      </div>

      <div v-else-if="duplicateGroups.length === 0" class="p-4 flex-1 flex items-center justify-center">
        <div class="text-center text-base-content/30 space-y-3 max-w-65">
          <p class="text-sm">{{ $t('info_panel.dedup.empty_title') }}</p>
          <!-- <p class="text-xs">{{ $t('info_panel.dedup.empty_desc') }}</p> -->
        </div>
      </div>

      <div v-else class="flex min-h-0 flex-1 flex-col">
        <section class="mb-2 flex shrink-0 flex-wrap items-center gap-x-3 gap-y-2 rounded-box border border-base-content/10 bg-base-100/40 px-3 py-2" aria-label="Bulk duplicate actions">
          <div class="min-w-0">
            <div class="text-[10px] font-bold uppercase tracking-widest text-base-content/30">
              {{ $t('info_panel.dedup.reclaimable_size') }}
            </div>
            <div class="text-xs text-base-content/60">
              {{ $t('info_panel.dedup.removable_copies_summary', {
                count: totalDuplicateFileCount.toLocaleString(),
                size: formatFileSize(totalReclaimableBytes),
              }) }}
            </div>
          </div>
          <PanelActionButton
            class="ml-auto"
            :icon="IconTrash"
            danger
            :disabled="totalDuplicateFileCount === 0 || duplicateAccessPending || hasUnavailableDuplicates"
            @click="trashAllDuplicates"
          >
            {{ $t('info_panel.dedup.delete_all') }}
          </PanelActionButton>
        </section>
      <div ref="dedupSplitPaneRef" class="flex min-h-0 flex-1 flex-col">
        <div
          class="min-h-0 shrink-0 flex flex-col border-t border-base-content/5 px-1 py-3 space-y-3"
          :style="{ height: `${config.dedup.duplicateSetsHeight}%` }"
        >
          <div class="flex items-center gap-2">
            <span class="text-[10px] uppercase tracking-widest font-bold text-base-content/30">
              {{ $t('info_panel.dedup.groups_title') }}
            </span>
            <span class="ml-auto sidebar-item-count">
              {{ totalGroupCount.toLocaleString() }}
            </span>
          </div>
          <div v-if="skippedCloudOnlyCount > 0" class="px-1 text-xs text-base-content/50">
            {{ $t('info_panel.dedup.skipped_cloud_only', { count: skippedCloudOnlyCount.toLocaleString() }) }}
          </div>
          <div
            ref="duplicateGroupsScrollRef"
            class="min-h-0 flex-1 overflow-y-auto py-1"
            @scroll="loadMoreDuplicateThumbnails"
          >
            <div class="mx-2 grid grid-cols-[repeat(auto-fill,minmax(5rem,1fr))] gap-2">
              <button
              v-for="group in visibleDuplicateGroups"
              :key="group.id"
              :data-duplicate-group-id="group.id"
              class="group/thumb relative h-20 min-w-0 overflow-hidden rounded-box cursor-pointer"
                :class="selectedGroupId === group.id ? 'ring-2 ring-primary' : ''"
                @click="selectDuplicateGroup(group)"
              >
                <img
                  v-if="group.keepItem?.file?.thumbnail"
                  :src="group.keepItem.file.thumbnail"
                  class="h-full w-full object-cover"
                  loading="lazy"
                />
                <div v-else class="h-full w-full skeleton"></div>
                <div class="absolute left-0.5 top-0.5 thumb-badge">
                  {{ group.file_count }}
                </div>
                <div
                  class="absolute inset-x-0 bottom-0 bg-linear-to-t from-black/80 to-transparent px-1.5 pb-1 pt-4 text-left text-[10px] leading-tight text-white/90 opacity-0 transition-opacity group-hover/thumb:opacity-100"
                  :class="{ 'opacity-100': selectedGroupId === group.id }"
                >
                  <div>{{ formatFileSize(group.file_size) }}</div>
                  <div v-if="group.keepItem?.file?.width && group.keepItem?.file?.height" class="text-white/70">
                    {{ group.keepItem.file.width }} x {{ group.keepItem.file.height }}
                  </div>
                </div>
              </button>
            </div>
          </div>
        </div>
        <div
          v-if="activeGroup"
          class="-mx-2 z-10 flex h-1 border-b border-base-content/5 shrink-0 touch-none cursor-row-resize items-center select-none"
          @pointerdown.prevent="startDraggingDuplicateSplitter"
        >
          <div class="h-1 w-full splitter-indicator splitter-horizontal" :class="{ 'splitter-dragging': isDraggingDuplicateSplitter }"></div>
        </div>
        <div v-if="activeGroup" class="min-h-0 flex-1 overflow-y-auto px-1 py-3 space-y-3">
          <div class="flex items-center gap-2">
            <span class="text-[10px] uppercase tracking-widest font-bold text-base-content/30">
              {{ $t('info_panel.dedup.actions_title') }}
            </span>
            <span v-if="selectedDeleteCount > 0" class="ml-auto text-xs text-base-content/70">
              {{ $t('toolbar.filter.select_count', { count: selectedDeleteCount.toLocaleString() }) }} · {{ formatFileSize(selectedDeleteBytes) }}
            </span>
          </div>

          <div class="flex flex-wrap gap-1">
            <PanelActionButton
              :icon="isAllGroupDuplicatesSelected(activeGroup.id) ? IconUnChecked : IconChecked"
              @click="selectGroupDuplicates(activeGroup.id, activeGroup.keepItem?.file_id || 0)"
            >
              {{ isAllGroupDuplicatesSelected(activeGroup.id) ? $t('menu.select.none') : $t('menu.select.all') }}
            </PanelActionButton>
            <PanelActionButton
              :icon="IconTrash"
              :disabled="selectedDeleteCount === 0"
              danger
              @click="trashSelectedDuplicates(activeGroup.id, selectedDeleteBytes)"
            >
              {{ $t('info_panel.dedup.delete_selected') }}
            </PanelActionButton>
          </div>
          <TransitionGroup
            :key="activeGroup.id"
            tag="div"
            name="dedup-item"
            move-class="transition-transform duration-200 ease-out"
            class="space-y-2.5"
          >
            <div
              v-for="item in activeGroup.items"
              :key="item.file_id"
              role="button"
              tabindex="0"
              class="w-full rounded-box p-2.5 border text-left transition-colors cursor-pointer"
              :class="getDedupItemClass(item.file_id, item.is_keep !== 1 && isDupSelected(activeGroup.id, item.file_id))"
              @click="handleDuplicateSelection(item.file_id)"
              @dblclick="handleDuplicateSelection(item.file_id, true)"
              @keydown.enter.self="handleDuplicateSelection(item.file_id)"
              @keydown.space.self.prevent="handleDuplicateSelection(item.file_id)"
            >
              <div class="flex items-center gap-2">
                <label v-if="item.is_keep !== 1" class="flex items-center shrink-0" :class="isOriginalUnavailable(item.file) ? 'cursor-not-allowed' : 'cursor-pointer'" @click.stop @dblclick.stop>
                  <input
                    type="checkbox"
                    class="checkbox checkbox-xs disabled:cursor-not-allowed"
                    :class="isOriginalUnavailable(item.file) ? 'opacity-30' : 'checkbox-primary opacity-70'"
                    :checked="isDupSelected(activeGroup.id, item.file_id)"
                    :disabled="isOriginalUnavailable(item.file)"
                    @change="toggleDupSelected(activeGroup.id, item.file_id)"
                  />
                </label>
                <div v-else class="w-4 shrink-0"></div>
                <div class="w-10 h-10 rounded-box overflow-hidden shrink-0">
                  <img v-if="item.file?.thumbnail" :src="item.file.thumbnail" class="w-full h-full object-cover" />
                  <div v-else class="w-full h-full skeleton"></div>
                </div>
                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-1 text-xs font-semibold text-base-content/70">
                    <span class="truncate">{{ item.file?.name }}</span>
                      <span v-if="formatLabel(item.file)" class="shrink-0 rounded bg-base-content/10 px-1 py-0.5 text-[9px] font-semibold text-base-content/70">{{ formatLabel(item.file) }}</span>
                    <IconExclamation v-if="isOriginalUnavailable(item.file)" class="w-3.5 h-3.5 shrink-0 text-warning" />
                  </div>
                  <div
                    class="text-[11px] text-base-content/30 truncate"
                    :title="formatDedupFolderPath(item.file)"
                  >
                    {{ formatDedupFolderPath(item.file) }}
                  </div>
                  <div v-if="item.file?.modified_at" class="text-[11px] text-base-content/30">
                    {{ $t('file_info.modified_at') }}: {{ formatTimestamp(item.file.modified_at, 'date_time') }}
                  </div>
                </div>
                <div class="shrink-0 w-16 min-h-10 flex items-center justify-center">
                  <button
                    type="button"
                    class="btn btn-ghost btn-xs min-h-0 h-5 w-5 p-0"
                    :class="item.is_keep === 1 ? 'text-primary' : 'text-base-content/30 hover:text-primary/70'"
                    :title="$t(item.is_keep === 1 ? 'info_panel.dedup.keep_label' : 'info_panel.dedup.unkeep_label')"
                    :aria-label="$t(item.is_keep === 1 ? 'info_panel.dedup.keep_label' : 'info_panel.dedup.unkeep_label')"
                    :aria-current="item.is_keep === 1 ? 'true' : undefined"
                    @click.stop="item.is_keep !== 1 && setKeep(activeGroup.id, item.file_id)"
                  >
                    <component :is="item.is_keep === 1 ? IconLock : IconUnlock" class="w-3.5 h-3.5" />
                  </button>
                </div>
              </div>
            </div>
          </TransitionGroup>
        </div>
      </div>
      </div>
      </template>
    </div>
  </div>
  <MessageBox
    v-if="showLargeSimilarScanConfirm"
    :title="$t('info_panel.dedup.tabs.similar')"
    :message="$t('info_panel.dedup.similar.large_scan_confirm', { count: similarEligibleCount.toLocaleString() })"
    :OkText="$t('info_panel.dedup.similar.analyze_confirm')"
    :cancelText="$t('msgbox.cancel')"
    @ok="confirmLargeSimilarScan"
    @cancel="cancelLargeSimilarScan"
  />
</template>

<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted, nextTick, PropType } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import {
  formatFileSize,
  getFolderName,
  getFolderPath,
  formatFolderBreadcrumb,
  getThumbnailDataUrl,
  isMac,
  formatTimestamp,
} from '@/common/utils';
import TButton from '@/components/TButton.vue';
import PanelActionButton from '@/components/PanelActionButton.vue';
import MessageBox from '@/components/MessageBox.vue';
import { IconChecked, IconExclamation, IconUnChecked, IconClose, IconFlag, IconFlagFilled, IconFlagOff, IconLock, IconRefresh, IconSplitOn, IconSplitOn4, IconTrash, IconUnlock } from '@/common/icons';
import {
  dedupStartScan,
  dedupCancelScan,
  dedupGetScanStatus,
  dedupGetOverview,
  listenDedupScanProgress,
  dedupListGroups,
  dedupSetKeep,
  getAlbum,
  getFileThumb,
  checkFileAccessibility,
  similarStartScan,
  similarGetScanStatus,
  similarCancelScan,
  similarGetEligibleCount,
  similarListGroups,
  similarGetOverview,
  similarGetGroup,
  similarSetKeep,
  similarHasScan,
  listenSimilarScanProgress,
  setFileCullingFlag,
} from '@/common/api';
import { config, libConfig } from '@/common/config';
import { dedupFormatLabel } from '@/common/dedupFormat';

const formatLabel = (file: any) => dedupFormatLabel(file, config.settings.groupRawJpegPairs);
import { isOriginalUnavailable, setFileAccessibility } from '@/common/availability';
import { SIMILAR_SCAN, DEDUP_THUMBNAIL_PAGE_SIZE } from '@/common/constants';

const dedupPaneGlobalState = ((globalThis as any).__lapDedupPaneState ||= {
  lastScanKey: '',
});

const thumbnailPlaceholder = new URL('@/assets/images/image-file.png', import.meta.url).href;
const { t } = useI18n();

const props = defineProps({
  selectedFileId: {
    type: [Number, String],
    default: -1,
  },
  dedupScanKey: {
    type: String,
    default: '',
  },
  similarScanKey: {
    type: String,
    default: '',
  },
  similarityThreshold: {
    type: Number,
    required: true,
  },
  dedupQueryParams: {
    type: Object as () => Record<string, any> | null,
    default: null,
  },
  dedupCollectionId: {
    type: Number as PropType<number | null>,
    default: null,
  },
  dedupFileIds: {
    type: Array as PropType<number[] | null>,
    default: null,
  },
});

const emit = defineEmits<{
  close: [];
  'select-file': [fileId: number];
  'preview-file': [fileId: number];
  'trash-selected-duplicates': [groupId: string, fileIds: number[], reclaimableBytes: number];
  'trash-all-duplicates': [fileCount: number, reclaimableBytes: number];
  'trash-selected-similar': [groupId: string, fileIds: number[], reclaimableBytes: number];
  'compare-selected-photos': [files: any[]];
  'culling-status-updated': [fileId: number, cullingFlag: number];
  'dedup-status-updated': [statuses: Record<number, 'keep' | 'dup'>];
}>();

const selectedDupIdsByGroup = ref<Map<number, Set<number>>>(new Map());
const selectedSimilarIdsByGroup = ref<Map<number, Set<number>>>(new Map());
const isDedupLoading = ref(true);
const activeTab = ref<'duplicates' | 'similar'>(
  config.dedup.activeTab === 'similar' ? 'similar' : 'duplicates',
);
const similarLoading = ref(activeTab.value === 'similar');
const similarStatus = ref<any>({ phase: 'idle', current: 0, total: 0 });
const similarGroups = ref<any[]>([]);
const similarTotalGroups = ref(0);
const similarPhotoCount = ref(0);
const similarPhotoBytes = ref(0);
const isLoadingMoreSimilarGroups = ref(false);
const selectedSimilarGroupId = ref<number | null>(null);
const similarEligibleCount = ref(0);
const similarEligibleCountLoading = ref(false);
const similarHasScanned = ref(false);
const similarError = ref(false);
const similarLoadedScope = ref('');
const showLargeSimilarScanConfirm = ref(false);
const pendingSimilarReanalyze = ref(false);
const unlistenSimilarProgress = ref<null | (() => void)>(null);
const unlistenCullingStatus = ref<null | (() => void)>(null);
const dedupScanError = ref(false);
const unlistenDedupProgress = ref<null | (() => void)>(null);
const queuedDedupScan = ref(false);
const scanGeneration = ref(0);
const isStartingDedupScan = ref(false);
const rawGroups = ref<any[]>([]);
const selectedGroupId = ref<number | null>(null);
const totalGroupCount = ref(0);
const totalDuplicateFileCount = ref(0);
const totalReclaimableBytes = ref(0);
const albumRootPaths = ref<Map<number, string>>(new Map());
const dedupStatusPollTimer = ref<ReturnType<typeof setInterval> | null>(null);
// Online-only files the last scan left unhashed (hashing would download them).
const skippedCloudOnlyCount = ref(0);
const isPollingDedupStatus = ref(false);
const loadedDuplicateGroupCount = ref(DEDUP_THUMBNAIL_PAGE_SIZE);
const duplicateGroupsScrollRef = ref<HTMLElement | null>(null);
const similarGroupsScrollRef = ref<HTMLElement | null>(null);
const dedupSplitPaneRef = ref<HTMLElement | null>(null);
const similarSplitPaneRef = ref<HTMLElement | null>(null);
const isDraggingDuplicateSplitter = ref(false);
const isLoadingMoreDuplicateThumbnails = ref(false);
let dedupScanReady = false;

const duplicateGroups = computed(() =>
  rawGroups.value.map((group: any) => {
    const sourceItems = group.items || [];
    const keepItem = sourceItems.find((item: any) => item.is_keep === 1) || null;
    const duplicateItems = sourceItems.filter((item: any) => item.is_keep === 0);
    return {
      ...group,
      items: keepItem ? [keepItem, ...duplicateItems] : sourceItems,
      keepItem,
      duplicateItems,
      reclaimableBytes: Math.max(0, Number(group.total_size || 0) - Number(group.file_size || 0)),
    };
  })
);

const activeGroup = computed(() => {
  if (selectedGroupId.value === null) return null;
  return duplicateGroups.value.find(group => group.id === selectedGroupId.value) || null;
});
const activeSimilarGroup = computed(() => similarGroups.value.find(group => Number(group.id) === selectedSimilarGroupId.value && Array.isArray(group.items)) || null);
const hasSimilarKeep = computed(() => activeSimilarGroup.value?.items?.some((item: any) => item.is_keep === 1) || false);
const similarProgressLabel = computed(() => ({
  preparing: t('info_panel.dedup.similar.preparing'),
  finding_matches: t('info_panel.dedup.similar.finding_matches'),
  building_sets: t('info_panel.dedup.similar.building_sets'),
})[similarStatus.value.phase] || t('info_panel.dedup.similar.analyzing'));
const visibleDuplicateGroups = computed(() => duplicateGroups.value.slice(0, loadedDuplicateGroupCount.value));
const visibleSimilarGroups = computed(() => similarGroups.value);

function emitDedupStatuses() {
  const statuses: Record<number, 'keep' | 'dup'> = {};
  for (const group of rawGroups.value) {
    for (const item of group.items || []) {
      statuses[Number(item.file_id)] = item.is_keep === 1 ? 'keep' : 'dup';
    }
  }
  emit('dedup-status-updated', statuses);
}

const selectedDeleteCount = computed(() => {
  if (!activeGroup.value) return 0;
  return activeGroup.value.duplicateItems.filter((item: any) => isDupSelected(activeGroup.value.id, item.file_id)).length;
});

const selectedDeleteBytes = computed(() => {
  if (!activeGroup.value) return 0;
  return activeGroup.value.duplicateItems.reduce((sum: number, item: any) => {
    return isDupSelected(activeGroup.value.id, item.file_id) ? sum + Number(item.file?.size || 0) : sum;
  }, 0);
});

const selectedSimilarCount = computed(() => {
  if (!activeSimilarGroup.value) return 0;
  return activeSimilarGroup.value.items.filter((item: any) =>
    item.is_keep !== 1 && isSimilarSelected(activeSimilarGroup.value.id, item.file_id)
  ).length;
});

const selectedSimilarBytes = computed(() => {
  if (!activeSimilarGroup.value) return 0;
  return activeSimilarGroup.value.items.reduce((sum: number, item: any) =>
    item.is_keep !== 1 && isSimilarSelected(activeSimilarGroup.value.id, item.file_id)
      ? sum + Number(item.file?.size || 0)
      : sum, 0);
});

function getDupSelectedSet(groupId: number): Set<number> {
  const existing = selectedDupIdsByGroup.value.get(groupId);
  if (existing) return existing;
  const set = new Set<number>();
  selectedDupIdsByGroup.value.set(groupId, set);
  return set;
}

const duplicateFilesById = computed(() => new Map<number, any>(rawGroups.value.flatMap((group: any) =>
  (group.items || []).map((item: any) => [Number(item.file_id), item.file] as [number, any]))));
const similarFilesById = computed(() => new Map<number, any>(similarGroups.value.flatMap((group: any) =>
  (group.items || []).map((item: any) => [Number(item.file_id), item.file] as [number, any]))));

function isDupSelected(groupId: number, fileId: number) {
  return !isOriginalUnavailable(duplicateFilesById.value.get(Number(fileId)))
    && getDupSelectedSet(groupId).has(fileId);
}

function getSimilarSelectedSet(groupId: number): Set<number> {
  const existing = selectedSimilarIdsByGroup.value.get(groupId);
  if (existing) return existing;
  const selected = new Set<number>();
  selectedSimilarIdsByGroup.value.set(groupId, selected);
  return selected;
}

function isSimilarSelected(groupId: number, fileId: number) {
  return !isOriginalUnavailable(similarFilesById.value.get(Number(fileId)))
    && getSimilarSelectedSet(groupId).has(fileId);
}

function selectSimilarDuplicatesByDefault(group: any) {
  const groupId = Number(group?.id || 0);
  if (!groupId || selectedSimilarIdsByGroup.value.has(groupId)) return;
  selectedSimilarIdsByGroup.value.set(
    groupId,
    new Set(
      (group.items || [])
        .filter((item: any) => item.is_keep !== 1 && !isOriginalUnavailable(item.file))
        .map((item: any) => Number(item.file_id)),
    ),
  );
}

function getSimilarCullingIconClass(file: any, cullingFlag: number) {
  const current = Number(file?.culling_flag ?? file?.cullingFlag ?? 0);
  if (current === cullingFlag) return cullingFlag === 2 ? 'text-error' : 'text-primary';
  return cullingFlag === 2
    ? 'text-base-content/30 hover:text-error/70'
    : 'text-base-content/30 hover:text-primary/70';
}

async function setSimilarCullingFlag(item: any, cullingFlag: number) {
  const file = item?.file;
  const fileId = Number(file?.id || item?.file_id || 0);
  if (!file || fileId <= 0) return;
  const normalized = Math.max(0, Math.min(2, cullingFlag));
  const previous = Number(file.culling_flag ?? file.cullingFlag ?? 0);
  file.culling_flag = normalized;
  file.cullingFlag = normalized;
  const result = await setFileCullingFlag(fileId, normalized);
  if (result === null) {
    file.culling_flag = previous;
    file.cullingFlag = previous;
    return;
  }
  emit('culling-status-updated', fileId, normalized);
}

function toggleSimilarSelected(groupId: number, fileId: number) {
  if (isOriginalUnavailable(similarFilesById.value.get(Number(fileId)))) return;
  const selected = getSimilarSelectedSet(groupId);
  if (selected.has(fileId)) selected.delete(fileId);
  else selected.add(fileId);
}

function isAllSimilarItemsSelected(groupId: number) {
  if (!activeSimilarGroup.value?.items?.length || activeSimilarGroup.value.id !== groupId) return false;
  const unkeptItems = activeSimilarGroup.value.items.filter((item: any) => item.is_keep !== 1 && !isOriginalUnavailable(item.file));
  if (unkeptItems.length === 0) return false;
  const selected = getSimilarSelectedSet(groupId);
  return unkeptItems.every((item: any) => selected.has(Number(item.file_id)));
}

function selectAllSimilarItems(group: any) {
  const groupId = Number(group?.id || 0);
  if (!groupId) return;
  const selected = getSimilarSelectedSet(groupId);
  if (isAllSimilarItemsSelected(groupId)) {
    selected.clear();
    return;
  }
  selected.clear();
  for (const item of group.items || []) {
    if (item.is_keep !== 1 && !isOriginalUnavailable(item.file)) selected.add(Number(item.file_id));
  }
}

function compareSelectedSimilarPhotos() {
  if (!activeSimilarGroup.value) return;
  const keepItem = activeSimilarGroup.value.items.find((item: any) => item.is_keep === 1);
  if (!keepItem?.file) return;
  const files = [keepItem.file]
    .concat(activeSimilarGroup.value.items
    .filter((item: any) => item.is_keep !== 1 && isSimilarSelected(activeSimilarGroup.value.id, item.file_id))
      .map((item: any) => item.file)
    )
    .filter(Boolean);
  if (files.length >= 2) emit('compare-selected-photos', files);
}

function trashSelectedSimilar(groupId: number, reclaimableBytes: number) {
  const keptIds = new Set(
    (activeSimilarGroup.value?.items || [])
      .filter((item: any) => item.is_keep === 1)
      .map((item: any) => Number(item.file_id)),
  );
  const fileIds = Array.from(getSimilarSelectedSet(groupId)).filter(fileId => !keptIds.has(fileId) && isSimilarSelected(groupId, fileId));
  if (fileIds.length > 0) emit('trash-selected-similar', String(groupId), fileIds, reclaimableBytes);
}

function getDedupItemClass(fileId: number, isDuplicateSelected = false) {
  const isActive = Number(props.selectedFileId) === Number(fileId);
  // if (isDuplicateSelected) {
  //   return isActive
  //     ? 'border-error/70 bg-error/10'
  //     : 'border-error/30 hover:bg-error/5';
  // }
  return isActive
    ? 'border-primary/70 bg-primary/10'
    : 'border-base-content/10 hover:bg-primary/5';
}

function toggleDupSelected(groupId: number, fileId: number) {
  if (isOriginalUnavailable(duplicateFilesById.value.get(Number(fileId)))) return;
  const set = getDupSelectedSet(groupId);
  if (set.has(fileId)) set.delete(fileId);
  else set.add(fileId);
}

function handleDuplicateSelection(fileId: number, preview = false) {
  emit('select-file', fileId);
  if (preview) {
    emit('preview-file', fileId);
  }
}

function handleSimilarSelection(fileId: number, preview = false) {
  emit('select-file', fileId);
  if (preview) emit('preview-file', fileId);
}

let unlistenAlbumAccessChanged: (() => void) | undefined;
const duplicateAccessPending = ref(false);
const hasUnavailableDuplicates = computed(() => rawGroups.value.some((group: any) =>
  (group.items || []).some((item: any) => item.is_keep !== 1 && isOriginalUnavailable(item.file))));

async function refreshFileAccess(files: any[]) {
  const libraryId = libConfig._libraryId;
  const unique = [...new Map(files.filter(file => file?.file_path && file.album_accessible !== false)
    .map(file => [file.file_path, file])).values()];
  for (let i = 0; i < unique.length; i += 8) {
    await Promise.all(unique.slice(i, i + 8).map(async file => {
      try {
        const available = await checkFileAccessibility(file.file_path);
        if (libraryId === libConfig._libraryId) setFileAccessibility(file.file_path, available);
      } catch { /* Keep the last known state on IPC errors. */ }
    }));
    if (libraryId !== libConfig._libraryId) return;
  }
}

async function hydrateSimilarThumbnails(groups: any[], activeGroupId: number | null) {
  const libraryId = libConfig._libraryId;
  const visibleIds = new Set(
    visibleSimilarGroups.value.map(group => Number(group.id))
  );
  const files = groups.flatMap(group => {
    if (Number(group.id) === activeGroupId) return [group.representative, ...(group.items || []).map((item: any) => item.file)];
    return visibleIds.has(Number(group.id)) ? [group.representative] : [];
  }).filter(Boolean);
  await refreshFileAccess(groups.flatMap(group => Number(group.id) === activeGroupId
    ? (group.items || []).map((item: any) => item.file) : []));
  if (libraryId !== libConfig._libraryId) return;
  await Promise.all(files.map(async (file: any) => {
    if (file.thumbnail || !file.file_path) return;
    const thumb = await getFileThumb(
      file.id,
      file.file_path,
      file.file_type || 1,
      file.e_orientation || 0,
      config.settings.thumbnailSize,
      false,
      null,
      file.album_id || 0,
      libraryId,
    );
    if (libraryId !== libConfig._libraryId) return;
    file.thumbnail = getThumbnailDataUrl(thumb, thumbnailPlaceholder, false, config.settings.thumbnailSize, file.file_path, Number(file.modified_at || 0), !file.cloud_only);
  }));
}

async function fetchSimilarGroups(append = false) {
  const scopeKey = props.similarScanKey;
  const offset = append ? similarGroups.value.length : 0;
  let page;
  try { page = await similarListGroups(scopeKey, SIMILAR_SCAN.PAGE_SIZE, offset); }
  catch (error) {
    console.error('fetchSimilarGroups error:', error);
    if (scopeKey === props.similarScanKey) similarError.value = true;
    return;
  }
  if (scopeKey !== props.similarScanKey) return;
  similarError.value = false;
  const groups = Array.isArray(page?.items) ? page.items : [];
  similarGroups.value = append ? [...similarGroups.value, ...groups] : groups;
  similarTotalGroups.value = Number(page?.total || 0);
  if (!append) await refreshSimilarOverview(scopeKey);
  similarLoadedScope.value = props.similarScanKey;
  if (!append) {
    selectedSimilarGroupId.value = similarGroups.value[0] ? Number(similarGroups.value[0].id) : null;
    const firstGroup = similarGroups.value[0];
    if (firstGroup) {
      let detail;
      try { detail = await similarGetGroup(firstGroup.id, scopeKey); }
      catch (error) { console.error('getSimilarGroup error:', error); similarError.value = true; return; }
      if (scopeKey !== props.similarScanKey) return;
      Object.assign(firstGroup, detail);
      selectSimilarDuplicatesByDefault(firstGroup);
    }
  }
  await hydrateSimilarThumbnails(similarGroups.value, selectedSimilarGroupId.value);
}

async function refreshSimilarOverview(scopeKey = props.similarScanKey) {
  if (!scopeKey) {
    similarPhotoCount.value = 0;
    similarPhotoBytes.value = 0;
    return;
  }
  try {
    const overview = await similarGetOverview(scopeKey);
    if (scopeKey !== props.similarScanKey) return;
    similarPhotoCount.value = Number(overview?.total_files || 0);
    similarPhotoBytes.value = Number(overview?.total_size || 0);
  } catch (error) {
    console.error('refreshSimilarOverview error:', error);
  }
}

async function loadMoreSimilarGroups(event: Event) {
  const target = event.currentTarget as HTMLElement;
  if (isLoadingMoreSimilarGroups.value || similarGroups.value.length >= similarTotalGroups.value) return;
  if (target.scrollTop + target.clientHeight < target.scrollHeight - 24) return;
  isLoadingMoreSimilarGroups.value = true;
  try { await fetchSimilarGroups(true); }
  finally { isLoadingMoreSimilarGroups.value = false; }
}

async function openSimilarTab(forceReload = false) {
  activeTab.value = 'similar';
  config.dedup.activeTab = 'similar';
  if (!props.similarScanKey) {
    similarLoading.value = false;
    return;
  }
  const scopeKey = props.similarScanKey;
  similarLoading.value = true;
  if (!similarHasScanned.value) similarEligibleCountLoading.value = true;
  if (isDedupLoading.value) return;
  if (forceReload) similarLoadedScope.value = '';
  similarError.value = false;
  let status;
  try { status = await similarGetScanStatus(); }
  catch (error) {
    console.error('getSimilarScanStatus error:', error);
    similarLoading.value = false;
    similarEligibleCountLoading.value = false;
    similarError.value = true;
    return;
  }
  if (scopeKey !== props.similarScanKey) return;
  if (status?.scopeKey === props.similarScanKey && (status.state === 'running' || status.isScanning)) {
    similarStatus.value = status;
    similarLoading.value = true;
    similarEligibleCountLoading.value = false;
    return;
  }
  if (similarLoadedScope.value !== props.similarScanKey) await fetchSimilarGroups();
  if (scopeKey !== props.similarScanKey) return;
  if (similarError.value) {
    similarLoading.value = false;
    similarEligibleCountLoading.value = false;
    return;
  }
  const hasCachedGroups = similarGroups.value.length > 0;
  let hasPersistedScan = false;
  try { hasPersistedScan = await similarHasScan(props.similarScanKey); }
  catch (error) {
    console.error('similarHasScan error:', error);
    similarLoading.value = false;
    similarEligibleCountLoading.value = false;
    similarError.value = true;
    return;
  }
  if (scopeKey !== props.similarScanKey) return;
  similarHasScanned.value = hasCachedGroups
    || (status?.scopeKey === props.similarScanKey && status?.state === 'finished')
    || hasPersistedScan;
  if (similarHasScanned.value) {
    similarLoading.value = false;
    similarEligibleCountLoading.value = false;
    return;
  }
  let eligibleCount;
  try {
    eligibleCount = Number(await similarGetEligibleCount(
      props.dedupFileIds === null ? (props.dedupQueryParams || null) : null,
      props.dedupFileIds === null ? props.dedupCollectionId : null,
      props.dedupFileIds,
    ));
  } catch (error) {
    console.error('getSimilarEligibleCount error:', error);
    similarError.value = true;
    return;
  } finally {
    if (scopeKey === props.similarScanKey) {
      similarLoading.value = false;
      similarEligibleCountLoading.value = false;
    }
  }
  if (scopeKey !== props.similarScanKey) return;
  similarEligibleCount.value = eligibleCount;
}

function selectDuplicatesTab() {
  activeTab.value = 'duplicates';
  config.dedup.activeTab = 'duplicates';
  const keepFileId = Number(activeGroup.value?.keepItem?.file_id || 0);
  if (keepFileId > 0) emit('select-file', keepFileId);
}

async function startSimilar() {
  if (isDedupLoading.value || similarLoading.value) return;
  if (similarEligibleCount.value > SIMILAR_SCAN.LARGE_RESULT_THRESHOLD) {
    showLargeSimilarScanConfirm.value = true;
    return;
  }
  await runSimilarScan();
}

function resetSimilarResults() {
  similarGroups.value = [];
  similarTotalGroups.value = 0;
  similarPhotoCount.value = 0;
  similarPhotoBytes.value = 0;
  selectedSimilarIdsByGroup.value.clear();
  selectedSimilarGroupId.value = null;
  similarHasScanned.value = false;
  similarError.value = false;
  similarLoadedScope.value = '';
}

async function reanalyzeSimilar() {
  if (isDedupLoading.value || similarLoading.value) return;
  let eligibleCount;
  try {
    eligibleCount = Number(await similarGetEligibleCount(
      props.dedupFileIds === null ? (props.dedupQueryParams || null) : null,
      props.dedupFileIds === null ? props.dedupCollectionId : null,
      props.dedupFileIds,
    ));
  } catch (error) {
    console.error('getSimilarEligibleCount error:', error);
    similarError.value = true;
    return;
  }
  similarEligibleCount.value = eligibleCount;
  if (eligibleCount > SIMILAR_SCAN.LARGE_RESULT_THRESHOLD) {
    pendingSimilarReanalyze.value = true;
    showLargeSimilarScanConfirm.value = true;
    return;
  }
  resetSimilarResults();
  await runSimilarScan();
}

async function confirmLargeSimilarScan() {
  showLargeSimilarScanConfirm.value = false;
  if (pendingSimilarReanalyze.value) resetSimilarResults();
  pendingSimilarReanalyze.value = false;
  await runSimilarScan();
}

function cancelLargeSimilarScan() {
  showLargeSimilarScanConfirm.value = false;
  pendingSimilarReanalyze.value = false;
}

async function runSimilarScan() {
  if (isDedupLoading.value || similarLoading.value) return;
  similarLoading.value = true;
  try {
    const sourceVersion = Number(props.similarScanKey.match(/\|similar-view:(\d+)$/)?.[1] || 0);
    await similarStartScan(
      props.similarScanKey,
      sourceVersion,
      props.similarityThreshold,
      props.dedupFileIds === null ? (props.dedupQueryParams || null) : null,
      props.dedupFileIds === null ? props.dedupCollectionId : null,
      props.dedupFileIds,
    );
  } catch (error) {
    console.error('startSimilar error:', error);
    similarLoading.value = false;
    similarError.value = true;
  }
}

async function cancelSimilar() {
  try { await similarCancelScan(); }
  catch (error) { console.error('cancelSimilar error:', error); similarLoading.value = false; similarError.value = true; }
}
async function selectSimilarGroup(group: any) {
  selectedSimilarGroupId.value = Number(group.id);
  try { Object.assign(group, await similarGetGroup(group.id, props.similarScanKey)); }
  catch (error) { console.error('getSimilarGroup error:', error); similarError.value = true; return; }
  selectSimilarDuplicatesByDefault(group);
  await hydrateSimilarThumbnails(similarGroups.value, selectedSimilarGroupId.value);
  if (group.representative?.id) emit('select-file', group.representative.id);
}

function selectDuplicateGroup(group: any) {
  selectedGroupId.value = Number(group.id);
  const keepFileId = Number(group.keepItem?.file_id || 0);
  if (keepFileId > 0) {
    emit('select-file', keepFileId);
  }
}

function scrollSelectedDuplicateGroupIntoView(groupId: number) {
  const container = duplicateGroupsScrollRef.value;
  const item = container?.querySelector<HTMLElement>(`[data-duplicate-group-id="${groupId}"]`);
  if (!container || !item) return;

  const containerRect = container.getBoundingClientRect();
  const itemRect = item.getBoundingClientRect();
  const padding = 4;
  if (itemRect.top < containerRect.top + padding) {
    container.scrollBy({ top: itemRect.top - containerRect.top - padding, behavior: 'smooth' });
  } else if (itemRect.bottom > containerRect.bottom - padding) {
    container.scrollBy({ top: itemRect.bottom - containerRect.bottom + padding, behavior: 'smooth' });
  }
}

async function setKeep(groupId: number, fileId: number) {
  await dedupSetKeep(groupId, fileId);
  const groupIndex = rawGroups.value.findIndex((group: any) => Number(group.id) === groupId);
  if (groupIndex < 0) return;

  const group = rawGroups.value[groupIndex];
  const items = (group.items || []).map((item: any) => ({
    ...item,
    is_keep: Number(item.file_id) === fileId ? 1 : 0,
  }));
  rawGroups.value[groupIndex] = { ...group, items };
  emitDedupStatuses();

  const selectedIds = getDupSelectedSet(groupId);
  selectedIds.delete(fileId);
  emit('select-file', fileId);
}

async function setSimilarKeep(groupId: number, fileId: number) {
  await similarSetKeep(groupId, fileId, props.similarScanKey);
  const groupIndex = similarGroups.value.findIndex((group: any) => Number(group.id) === groupId);
  if (groupIndex < 0) return;

  const group = similarGroups.value[groupIndex];
  const items = (group.items || []).map((item: any) => ({
    ...item,
    is_keep: Number(item.file_id) === fileId ? 1 : 0,
  }));
  similarGroups.value[groupIndex] = {
    ...group,
    representative: items.find((item: any) => item.is_keep === 1)?.file || group.representative,
    items: items.sort((a: any, b: any) => Number(b.is_keep) - Number(a.is_keep)),
  };
  getSimilarSelectedSet(groupId).delete(fileId);
  emit('select-file', fileId);
}

function selectGroupDuplicates(groupId: number, keepFileId: number) {
  const group = duplicateGroups.value.find(g => g.id === groupId);
  if (!group) return;

  const set = getDupSelectedSet(groupId);
  const duplicateIds = group.duplicateItems.filter((item: any) => !isOriginalUnavailable(item.file)).map((item: any) => item.file_id);
  const allSelected = duplicateIds.length > 0 && duplicateIds.every((id: number) => set.has(id));

  if (allSelected) {
    set.clear();
    return;
  }

  set.clear();
  for (const id of duplicateIds) {
    if (id !== keepFileId) set.add(id);
  }
}

function isAllGroupDuplicatesSelected(groupId: number) {
  const group = duplicateGroups.value.find(g => g.id === groupId);
  if (!group || group.duplicateItems.length === 0) return false;
  const set = getDupSelectedSet(groupId);
  const available = group.duplicateItems.filter((item: any) => !isOriginalUnavailable(item.file));
  return available.length > 0 && available.every((item: any) => set.has(item.file_id));
}

function trashSelectedDuplicates(groupId: number, reclaimableBytes: number) {
  const ids = Array.from(getDupSelectedSet(groupId).values()).filter(fileId => isDupSelected(groupId, fileId));
  if (ids.length === 0) return;
  emit('trash-selected-duplicates', String(groupId), ids, reclaimableBytes);
}

async function trashAllDuplicates() {
  if (totalDuplicateFileCount.value <= 0 || duplicateAccessPending.value || hasUnavailableDuplicates.value) return;
  const libraryId = libConfig._libraryId;
  duplicateAccessPending.value = true;
  try {
    await refreshFileAccess(rawGroups.value.flatMap((group: any) => (group.items || [])
      .filter((item: any) => item.is_keep !== 1).map((item: any) => item.file)));
    if (libraryId !== libConfig._libraryId || hasUnavailableDuplicates.value) return;
    emit('trash-all-duplicates', totalDuplicateFileCount.value, totalReclaimableBytes.value);
  } finally {
    duplicateAccessPending.value = false;
  }
}

function applyDeletedFiles(groupId: number, deletedFileIds: number[]) {
  const groupIndex = rawGroups.value.findIndex((group: any) => Number(group.id) === groupId);
  if (groupIndex < 0 || deletedFileIds.length === 0) return;

  const group = rawGroups.value[groupIndex];
  const oldItems = Array.isArray(group.items) ? group.items : [];
  const deletedIds = new Set(deletedFileIds);
  const remainingItems = oldItems.filter((item: any) => !deletedIds.has(Number(item.file_id)));
  if (remainingItems.length === oldItems.length) return;

  const fileSize = Number(group.file_size || 0);
  const oldFileCount = oldItems.length;
  const newFileCount = remainingItems.length > 1 ? remainingItems.length : 0;
  const oldDuplicateCount = Math.max(0, oldFileCount - 1);
  const newDuplicateCount = Math.max(0, newFileCount - 1);
  const oldReclaimableBytes = Math.max(0, (oldFileCount - 1) * fileSize);
  const newReclaimableBytes = Math.max(0, (newFileCount - 1) * fileSize);

  totalDuplicateFileCount.value = Math.max(
    0,
    totalDuplicateFileCount.value + newDuplicateCount - oldDuplicateCount
  );
  totalReclaimableBytes.value = Math.max(
    0,
    totalReclaimableBytes.value + newReclaimableBytes - oldReclaimableBytes
  );

  const selectedIds = selectedDupIdsByGroup.value.get(groupId);
  for (const fileId of deletedIds) {
    selectedIds?.delete(fileId);
  }

  if (remainingItems.length <= 1) {
    rawGroups.value.splice(groupIndex, 1);
    selectedDupIdsByGroup.value.delete(groupId);
    totalGroupCount.value = Math.max(0, totalGroupCount.value - 1);

    if (selectedGroupId.value === groupId) {
      const nextGroup = rawGroups.value[groupIndex] || rawGroups.value[groupIndex - 1];
      selectedGroupId.value = nextGroup ? Number(nextGroup.id) : null;
    }
    emitDedupStatuses();
    return;
  }

  rawGroups.value[groupIndex] = {
    ...group,
    items: remainingItems,
    file_count: remainingItems.length,
    total_size: remainingItems.length * fileSize,
  };
  emitDedupStatuses();
}

function applyDeletedSimilarFiles(groupId: number, deletedFileIds: number[]) {
  if (deletedFileIds.length === 0) return;
  const index = similarGroups.value.findIndex((group: any) => Number(group.id) === groupId);
  if (index < 0) return;
  const deleted = new Set(deletedFileIds.map(Number));
  const group = similarGroups.value[index];
  const items = (group.items || []).filter((item: any) => !deleted.has(Number(item.file_id)));
  const selected = selectedSimilarIdsByGroup.value.get(groupId);
  deleted.forEach(fileId => selected?.delete(fileId));
  void refreshSimilarOverview();
  if (items.length < 2) {
    similarGroups.value.splice(index, 1);
    selectedSimilarIdsByGroup.value.delete(groupId);
    if (selectedSimilarGroupId.value === groupId) {
      const next = similarGroups.value[index] || similarGroups.value[index - 1];
      selectedSimilarGroupId.value = next ? Number(next.id) : null;
      if (next) void selectSimilarGroup(next);
    }
    return;
  }
  similarGroups.value[index] = { ...group, items, file_count: items.length };
}

function formatDedupFolderPath(file: any): string {
  const folderPath = getFolderPath(file?.file_path);
  if (!folderPath) return '';

  const albumId = Number(file?.album_id || 0);
  const albumRoot = albumId ? albumRootPaths.value.get(albumId) || '' : '';
  const albumLabel = file?.album_name || (albumRoot ? getFolderName(albumRoot) : '');
  return formatFolderBreadcrumb(folderPath, albumRoot, albumLabel);
}

async function hydrateAlbumRootPaths(groups: any[]) {
  const albumIds = new Set<number>();
  for (const group of groups || []) {
    for (const item of Array.isArray(group?.items) ? group.items : []) {
      const albumId = Number(item?.file?.album_id || 0);
      if (albumId > 0 && !albumRootPaths.value.has(albumId)) {
        albumIds.add(albumId);
      }
    }
  }

  if (albumIds.size === 0) return;

  const results = await Promise.all(
    Array.from(albumIds).map(async (albumId) => ({
      albumId,
      album: await getAlbum(albumId),
    }))
  );

  for (const { albumId, album } of results) {
    if (album?.path) {
      albumRootPaths.value.set(albumId, album.path);
    }
  }
}

async function hydrateGroupThumbnails(groups: any[], activeGroupId: number | null) {
  const libraryId = libConfig._libraryId;
  await refreshFileAccess(groups.flatMap(group => Number(group.id) === activeGroupId
    ? (group.items || []).map((item: any) => item.file) : []));
  if (libraryId !== libConfig._libraryId) return;
  const tasks: Promise<void>[] = [];
  const visibleGroupIds = new Set(
    (groups || []).slice(0, loadedDuplicateGroupCount.value).map((group: any) => Number(group.id))
  );

  for (const group of groups || []) {
    const groupId = Number(group?.id);
    const allItems = Array.isArray(group?.items) ? group.items : [];
    const items = groupId === activeGroupId
      ? allItems
      : visibleGroupIds.has(groupId)
        ? [allItems.find((item: any) => item.is_keep === 1) || allItems[0]].filter(Boolean)
        : [];

    for (const item of items) {
      const file = item?.file;
      if (!file) continue;
      if (file.thumbnail) continue;
      if (!file.file_path) {
        file.thumbnail = thumbnailPlaceholder;
        continue;
      }
      tasks.push((async () => {
        const thumb = await getFileThumb(
          file.id,
          file.file_path,
          file.file_type || 1,
          file.e_orientation || 0,
          config.settings.thumbnailSize,
          false,
          null,
          file.album_id || 0,
          libraryId,
        );
        if (libraryId !== libConfig._libraryId) return;
        file.thumbnail = getThumbnailDataUrl(thumb, thumbnailPlaceholder, false, config.settings.thumbnailSize, file.file_path, Number(file.modified_at || 0), !file.cloud_only);
      })());
    }
  }
  await Promise.all(tasks);
}

async function refreshOverview() {
  try {
    const overview = await dedupGetOverview();
    if (!overview) return;
    totalGroupCount.value = Number(overview.total_groups || 0);
    totalDuplicateFileCount.value = Number(overview.total_files || 0);
    totalReclaimableBytes.value = Number(overview.total_reclaimable_bytes || 0);
  } catch (error) {
    console.error('refreshOverview error:', error);
  }
}

async function fetchGroups(preferredGroupId: number | null = null) {
  duplicateAccessPending.value = true;
  try {
    const groups = await dedupListGroups(1, 0, 'count_desc', 'all');
    const normalized = Array.isArray(groups) ? groups : [];
    const availableGroupIds = new Set(normalized.map((group: any) => Number(group.id)));
    const nextSelectedGroupId =
      preferredGroupId && availableGroupIds.has(preferredGroupId)
        ? preferredGroupId
        : selectedGroupId.value && availableGroupIds.has(selectedGroupId.value)
          ? selectedGroupId.value
          : normalized.length > 0
            ? Number(normalized[0].id)
            : null;

    loadedDuplicateGroupCount.value = DEDUP_THUMBNAIL_PAGE_SIZE;
    await hydrateAlbumRootPaths(normalized);
    await hydrateGroupThumbnails(normalized, nextSelectedGroupId);
    rawGroups.value = normalized;
    emitDedupStatuses();
    await refreshOverview();

    for (const key of Array.from(selectedDupIdsByGroup.value.keys())) {
      if (!availableGroupIds.has(key)) {
        selectedDupIdsByGroup.value.delete(key);
      }
    }

    // Default-select all duplicate (non-keep) items for newly loaded groups
    for (const group of rawGroups.value) {
      const groupId = Number(group.id);
      if (!selectedDupIdsByGroup.value.has(groupId)) {
        const set = new Set<number>();
        for (const item of (group.items || [])) {
          if (item.is_keep !== 1 && !isOriginalUnavailable(item.file)) {
            set.add(Number(item.file_id));
          }
        }
        if (set.size > 0) {
          selectedDupIdsByGroup.value.set(groupId, set);
        }
      }
    }

    selectedGroupId.value = nextSelectedGroupId;
    dedupScanError.value = false;
  } catch (error) {
    console.error('fetchGroups error:', error);
    showDedupScanError();
  } finally {
    duplicateAccessPending.value = false;
  }
}

async function loadMoreDuplicateThumbnails(event: Event) {
  const target = event.currentTarget as HTMLElement;
  const hasMore = loadedDuplicateGroupCount.value < duplicateGroups.value.length;
  const isAtBottom = target.scrollTop + target.clientHeight >= target.scrollHeight - 24;
  if (!hasMore || !isAtBottom || isLoadingMoreDuplicateThumbnails.value) return;

  isLoadingMoreDuplicateThumbnails.value = true;
  loadedDuplicateGroupCount.value = Math.min(
    duplicateGroups.value.length,
    loadedDuplicateGroupCount.value + DEDUP_THUMBNAIL_PAGE_SIZE,
  );
  try {
    await nextTick();
    await hydrateGroupThumbnails(rawGroups.value, selectedGroupId.value);
  } finally {
    isLoadingMoreDuplicateThumbnails.value = false;
  }
}

function startDraggingDuplicateSplitter(event: PointerEvent) {
  (event.currentTarget as HTMLElement).setPointerCapture?.(event.pointerId);
  isDraggingDuplicateSplitter.value = true;
  document.addEventListener('pointermove', handleDuplicateSplitterMouseMove);
  document.addEventListener('pointerup', stopDraggingDuplicateSplitter);
  document.addEventListener('pointercancel', stopDraggingDuplicateSplitter);
}

function handleDuplicateSplitterMouseMove(event: PointerEvent) {
  const container = activeTab.value === 'similar'
    ? similarSplitPaneRef.value
    : dedupSplitPaneRef.value;
  if (!isDraggingDuplicateSplitter.value || !container) return;
  const rect = container.getBoundingClientRect();
  const nextHeight = ((event.clientY - rect.top) / rect.height) * 100;
  config.dedup.duplicateSetsHeight = Math.max(20, Math.min(nextHeight, 80));
}

function stopDraggingDuplicateSplitter() {
  isDraggingDuplicateSplitter.value = false;
  document.removeEventListener('pointermove', handleDuplicateSplitterMouseMove);
  document.removeEventListener('pointerup', stopDraggingDuplicateSplitter);
  document.removeEventListener('pointercancel', stopDraggingDuplicateSplitter);
}

function stopDedupStatusPolling() {
  if (dedupStatusPollTimer.value) {
    clearInterval(dedupStatusPollTimer.value);
    dedupStatusPollTimer.value = null;
  }
}

function showDedupScanError() {
  stopDedupStatusPolling();
  dedupPaneGlobalState.lastScanKey = '';
  rawGroups.value = [];
  emitDedupStatuses();
  selectedGroupId.value = null;
  totalGroupCount.value = 0;
  totalDuplicateFileCount.value = 0;
  totalReclaimableBytes.value = 0;
  dedupScanError.value = true;
  isDedupLoading.value = false;
}

async function handleDedupScanSettled(allowWhileStarting = false) {
  if (isStartingDedupScan.value && !allowWhileStarting) return;

  const gen = scanGeneration.value;
  const status = await dedupGetScanStatus();
  if (!status) {
    ensureDedupStatusPolling();
    return;
  }
  if (status?.state === 'running' || status?.isScanning) {
    ensureDedupStatusPolling();
    return;
  }
  if (gen !== scanGeneration.value) return;

  stopDedupStatusPolling();
  if (queuedDedupScan.value) {
    queuedDedupScan.value = false;
    await triggerBackendDedup(true);
    return;
  }
  if (status.state === 'error') {
    showDedupScanError();
    return;
  }
  skippedCloudOnlyCount.value = Number(status.skipped || 0);
  await fetchGroups();
  if (gen !== scanGeneration.value) return;
  // Only clear the loading flag after results are ready, so the
  // template never shows "no duplicates" before the scan finishes.
  isDedupLoading.value = false;
  if (activeTab.value === 'similar') await openSimilarTab();
}

function ensureDedupStatusPolling() {
  if (dedupStatusPollTimer.value) return;

  dedupStatusPollTimer.value = setInterval(async () => {
    if (isPollingDedupStatus.value) return;

    isPollingDedupStatus.value = true;
    try {
      const status = await dedupGetScanStatus();
      totalGroupCount.value = Math.max(Number(status?.groups || 0), rawGroups.value.length);
      if (status?.state && status.state !== 'running' && !status?.isScanning) {
        await handleDedupScanSettled();
      }
    } catch (error) {
      console.error('ensureDedupStatusPolling error:', error);
    } finally {
      isPollingDedupStatus.value = false;
    }
  }, 1000);
}

async function triggerBackendDedup(force = false) {
  if (!props.dedupScanKey) {
    stopDedupStatusPolling();
    isDedupLoading.value = false;
    return;
  }

  scanGeneration.value++;
  isStartingDedupScan.value = true;
  isDedupLoading.value = true;
  dedupScanError.value = false;

  try {
    const status = await dedupGetScanStatus();
    totalGroupCount.value = Math.max(Number(status?.groups || 0), rawGroups.value.length);

    if (status?.state === 'running' || status?.isScanning) {
      queuedDedupScan.value = true;
      await dedupCancelScan();
      ensureDedupStatusPolling();
      return;
    } else if (!force && dedupPaneGlobalState.lastScanKey === props.dedupScanKey) {
      await fetchGroups();
      isDedupLoading.value = false;
      if (activeTab.value === 'similar') await openSimilarTab();
      return;
    }

    const hasFileIdScope = props.dedupFileIds !== null;
    await dedupStartScan(
      hasFileIdScope ? null : (props.dedupQueryParams || null),
      hasFileIdScope ? null : props.dedupCollectionId,
      props.dedupFileIds,
    );
    dedupPaneGlobalState.lastScanKey = props.dedupScanKey;

    const latest = await dedupGetScanStatus();
    totalGroupCount.value = Math.max(Number(latest?.groups || 0), rawGroups.value.length);
    if (latest?.state === 'running') {
      ensureDedupStatusPolling();
    } else {
      await handleDedupScanSettled(true);
    }
  } catch (error) {
    if (String(error).includes('already running')) {
      queuedDedupScan.value = true;
      await dedupCancelScan();
      ensureDedupStatusPolling();
      return;
    }
    console.error('triggerBackendDedup error:', error);
    showDedupScanError();
  } finally {
    isStartingDedupScan.value = false;
  }
}

watch(
  () => props.dedupScanKey,
  (newKey) => {
    selectedGroupId.value = null;
    // Never expose groups from a previous query/pairing policy while the new scan starts.
    rawGroups.value = [];
    totalGroupCount.value = 0;
    totalDuplicateFileCount.value = 0;
    totalReclaimableBytes.value = 0;
    if (!newKey) {
      scanGeneration.value++;
      stopDedupStatusPolling();
      isDedupLoading.value = true;
      rawGroups.value = [];
      selectedGroupId.value = null;
      queuedDedupScan.value = false;
      dedupScanError.value = false;
      totalGroupCount.value = 0;
      totalDuplicateFileCount.value = 0;
      totalReclaimableBytes.value = 0;
      return;
    }
    if (dedupScanReady) void triggerBackendDedup();
  }
);

watch(
  () => props.similarScanKey,
  (newKey) => {
    similarGroups.value = [];
    similarTotalGroups.value = 0;
    isLoadingMoreSimilarGroups.value = false;
    selectedSimilarIdsByGroup.value.clear();
    selectedSimilarGroupId.value = null;
    similarEligibleCount.value = 0;
    similarEligibleCountLoading.value = false;
    similarHasScanned.value = false;
    similarError.value = false;
    similarLoadedScope.value = '';
    similarLoading.value = Boolean(newKey && activeTab.value === 'similar');
    showLargeSimilarScanConfirm.value = false;
    if (newKey && activeTab.value === 'similar') void openSimilarTab();
  },
);

watch(selectedGroupId, async (groupId, prevGroupId) => {
  if (!groupId || groupId === prevGroupId) return;
  await nextTick();
  scrollSelectedDuplicateGroupIntoView(groupId);
  await hydrateGroupThumbnails(rawGroups.value, groupId);
  if (selectedGroupId.value !== groupId || activeTab.value !== 'duplicates') return;
  const group = duplicateGroups.value.find((item: any) => item.id === groupId);
  const keepId = group?.keepItem?.file_id;
  if (keepId) {
    emit('select-file', keepId);
  }
});

watch(selectedSimilarGroupId, async (groupId, prevGroupId) => {
  if (!groupId || groupId === prevGroupId) return;
  await hydrateSimilarThumbnails(similarGroups.value, groupId);
});

onMounted(async () => {
  unlistenAlbumAccessChanged = await listen('album-accessibility-changed', async (event: any) => {
    const { libraryId, albumId, available } = event.payload || {};
    if (libraryId !== libConfig._libraryId) return;
    for (const group of [...rawGroups.value, ...similarGroups.value]) {
      for (const file of [group.representative, ...(group.items || []).map((item: any) => item.file)]) {
        if (file && Number(file.album_id) === Number(albumId)) file.album_accessible = available;
      }
    }
    if (activeTab.value === 'similar') {
      await hydrateSimilarThumbnails(similarGroups.value, selectedSimilarGroupId.value);
    } else {
      await hydrateGroupThumbnails(rawGroups.value, selectedGroupId.value);
    }
  });
  await nextTick();

  unlistenDedupProgress.value = await listenDedupScanProgress(async (event: any) => {
    const state = event?.payload?.state;
    totalGroupCount.value = Math.max(Number(event?.payload?.groups || 0), totalGroupCount.value);
    if (state === 'running') {
      ensureDedupStatusPolling();
      return;
    }
    if (state === 'finished' || state === 'idle' || state === 'error') {
      await handleDedupScanSettled();
    }
  });
  unlistenSimilarProgress.value = await listenSimilarScanProgress(async (event: any) => {
    const payload = event?.payload;
    if (payload?.scopeKey !== props.similarScanKey) return;
    similarStatus.value = payload || similarStatus.value;
    if (payload?.state === 'running') { similarLoading.value = true; return; }
    if (payload?.state === 'error') {
      similarLoading.value = false;
      similarError.value = true;
      return;
    }
    if (payload?.state === 'finished') {
      similarLoading.value = false;
      await fetchSimilarGroups();
      similarHasScanned.value = true;
      return;
    }
    if (payload?.state === 'idle') {
      similarLoading.value = false;
    }
  });
  unlistenCullingStatus.value = await listen('culling-status-updated', (event) => {
    const payload = event.payload as { fileIds?: number[]; cullingFlag?: number } | null;
    if (!payload?.fileIds || payload.cullingFlag === undefined) return;
    const fileIds = new Set(payload.fileIds.map(Number));
    for (const item of activeSimilarGroup.value?.items || []) {
      if (!fileIds.has(Number(item.file_id)) || !item.file) continue;
      item.file.culling_flag = payload.cullingFlag;
      item.file.cullingFlag = payload.cullingFlag;
    }
  });

  if (!props.dedupScanKey) {
    isDedupLoading.value = false;
    return;
  }

  await new Promise<void>(resolve => {
    requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
  });
  dedupScanReady = true;
  void triggerBackendDedup();
  if (activeTab.value === 'similar') await openSimilarTab();
});

onUnmounted(() => {
  unlistenAlbumAccessChanged?.();
  stopDedupStatusPolling();
  stopDraggingDuplicateSplitter();
  if (unlistenDedupProgress.value) {
    unlistenDedupProgress.value();
    unlistenDedupProgress.value = null;
  }
  if (unlistenSimilarProgress.value) unlistenSimilarProgress.value();
  if (unlistenCullingStatus.value) unlistenCullingStatus.value();
});

defineExpose({
  applyDeletedFiles,
  applyDeletedSimilarFiles,
  refreshGroups: fetchGroups,
});
</script>
