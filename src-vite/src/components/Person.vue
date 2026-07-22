<template>
  <div class="sidebar-panel relative overflow-hidden">
    <!-- Face Indexing Progress Overlay -->
    <div v-if="isIndexing" 
      class="absolute inset-0 z-50 bg-base-200/80 backdrop-blur-md"
    >
      <div class="mt-8 px-2 flex flex-col items-center text-base-content/30">
        <IconUpdate class="w-8 h-8 mb-2 animate-spin" />
        <span class="text-sm text-center">
          {{ indexProgress.phase === 'clustering' 
            ? $t('face_index.clustering') 
            : $t('face_index.indexing', { current: indexProgress.current.toLocaleString(), total: indexProgress.total.toLocaleString() }) 
          }}
        </span>
        <span v-if="indexProgress.phase === 'clustering' && clusterProgressText" class="text-xs text-center mt-1">
          {{ clusterProgressText }}
        </span>
        <span v-else-if="indexProgress.faces_found > 0" class="text-xs text-center mt-1">
          {{ $t('face_index.faces_found', { count: indexProgress.faces_found.toLocaleString() }) }}
        </span>
        <button class="btn btn-primary btn-sm mt-4" @click="clickCancelIndex">
          <IconClose class="w-4 h-4" />
          {{ $t('face_index.cancel') }}
        </button>
      </div>
    </div>

    <!-- Incomplete Indexing Warning Banner -->
    <div v-if="allPersons.length > 0 && incompleteCount > 0 && !isIndexing" class="flex-none px-2 py-2">
        <div class="p-3 rounded-box flex flex-row items-center gap-2">
          <IconUpdate class="w-5 h-5 shrink-0" />
          <span class="text-xs flex-1">
            {{ $t('face_index.incomplete', { count: incompleteCount.toLocaleString() }) }}
          </span>
          <button class="btn btn-xs btn-primary" @click="clickIndexFaces">
            {{ $t('face_index.resume') }}
          </button>
        </div>
    </div>

    <div class="sidebar-panel-header">
      <span class="sidebar-panel-header-title flex-1 min-w-0 overflow-hidden text-ellipsis whitespace-nowrap">
        {{ titlebar }}
      </span>
      <span class="px-1.5 h-5 inline-flex items-center rounded-box text-[10px] font-semibold tracking-[0.08em] text-warning border border-warning/30 bg-warning/10 cursor-default">
        BETA
      </span>

      <ContextMenu :menuItems="personPanelMenuItems" :iconMenu="IconMore" :smallIcon="true" />
    </div>

    <!-- Person List -->
    <div v-if="allPersons.length > 0" class="grow overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="person in sortedPersons" :key="person.id" :id="'person-' + person.id">
          <div
            :class="[
              'sidebar-item gap-2 group',
              selectedPerson && selectedPerson.id === person.id && !isRenamingPerson ? 'sidebar-item-selected' : 'sidebar-item-hover',
            ]"
            @click="selectPerson(person)"
            @contextmenu.prevent.stop="(e: MouseEvent) => handlePersonContextMenu(person, e)"
          >
            <!-- Face thumbnail -->
            <div class="w-8 h-8 rounded-full overflow-hidden bg-base-300/70 ring-1 ring-base-content/5 shrink-0 flex items-center justify-center">
              <img 
                v-if="person.thumbnail" 
                :src="'data:image/jpeg;base64,' + person.thumbnail" 
                class="w-full h-full object-cover"
              />
              <IconPerson v-else class="w-5 h-5 text-base-content/30" />
            </div>
            
            <!-- Name input or display -->
            <input v-if="selectedPerson && selectedPerson.id === person.id && isRenamingPerson"
              ref="personInputRef"
              type="text"
              maxlength="255"
              class="input px-1 flex-1 focus:border text-base"
              v-model="person.name"
              @keydown.enter="handleRenamePerson"
              @keydown.esc="cancelRenamePerson"
              @blur="handleRenamePerson"
            />
            <template v-else>
              <span class="sidebar-item-label">
                {{ getPersonDisplayName(person) }}
              </span>
              <span v-if="person.count" :class="['sidebar-item-count', selectedPerson?.id === person.id ? 'hidden' : 'group-hover:hidden']">
                {{ person.count.toLocaleString() }}
              </span>

              <div :class="[
                  'ml-auto flex flex-row items-center text-base-content/30',
                  selectedPerson?.id === person.id ? '' : 'hidden group-hover:flex'
                ]"
              >
                <ContextMenu
                  :ref="(el: any) => { if (el) personContextMenus[person.id] = el }"
                  :iconMenu="IconMore"
                  :menuItems="getMoreMenuItems()"
                  :smallIcon="true"
                />
              </div>
            </template>
          </div>
        </li>
      </ul>
      <button
        v-if="hasMorePersons"
        type="button"
        class="w-full py-2 text-sm text-base-content/70 hover:text-base-content cursor-pointer disabled:cursor-wait"
        :disabled="isLoadingMorePersons"
        @click="loadMorePersons"
      >
        {{ isLoadingMorePersons
          ? $t('tooltip.loading')
          : $t('menu.person.show_more', {
              loaded: allPersons.length.toLocaleString(),
              total: totalPersons.toLocaleString(),
            })
        }}
      </button>
    </div>

    <div v-else-if="isLoadingPersons" class="mt-2 px-2 flex flex-col items-center justify-center text-base-content/30">
      <span class="text-sm text-center">{{ $t('tooltip.loading') }}</span>
    </div>

    <!-- No Persons Found Message -->
    <div v-else-if="!isIndexing && incompleteCount > 0" class="mt-2 px-2 flex flex-col items-center justify-center text-base-content/30">
      <span class="text-sm text-center">{{ $t('face_index.incomplete', { count: incompleteCount.toLocaleString() }) }}</span>
      <button class="btn btn-primary btn-sm mt-4 rounded-box" @click="clickIndexFaces">
        <IconUpdate class="w-4 h-4" />
        {{ $t('face_index.resume') }}
      </button>
    </div>

    <div v-else-if="!isIndexing" class="mt-2 px-2 flex flex-col items-center justify-center text-base-content/30">
      <span class="text-sm text-center">{{ $t('tooltip.not_found.person') }}</span>
    </div>

  </div>

  <!-- Delete person confirmation -->
  <MessageBox
    v-if="showDeletePersonMsgbox"
    :title="$t('msgbox.delete_person.title')"
    :message="`${$t('msgbox.delete_person.content', { person: getPersonDisplayName(selectedPerson) })}`"
    :OkText="$t('msgbox.delete_person.ok')"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    @ok="clickDeletePerson"
    @cancel="showDeletePersonMsgbox = false"
  />

  <!-- Reset faces confirmation -->
  <MessageBox
    v-if="showResetFacesMsgbox"
    :title="$t('msgbox.reset_faces.title')"
    :message="$t('msgbox.reset_faces.content')"
    :OkText="$t('msgbox.reset_faces.ok')"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    @ok="onResetFacesConfirm"
    @cancel="showResetFacesMsgbox = false"
  />

  <teleport to="body">
    <transition name="fade">
      <div
        v-if="isBetaTooltipVisible && config.settings.showToolTip"
        ref="betaTooltipRef"
        class="fixed z-1000 px-2 py-1 text-xs whitespace-nowrap rounded-box bg-neutral text-neutral-content shadow-lg pointer-events-none"
        :style="betaTooltipStyle"
      >
        {{ $t('tooltip.beta.person') }}
      </div>
    </transition>
  </teleport>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, nextTick, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { config, libConfig } from '@/common/config';
import { getPersonsPage, renamePerson, deletePerson, indexFaces, cancelFaceIndex, isFaceIndexing, listenFaceIndexProgress, listenFaceIndexFinished, listenClusterProgress, resetFaces, getFaceStats } from '@/common/api';
import { 
  IconPerson, 
  IconMore, 
  IconRename, 
  IconTrash,
  IconUpdate,
  IconClose,
} from '@/common/icons';

import ContextMenu from '@/components/ContextMenu.vue';
import MessageBox from '@/components/MessageBox.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const emit = defineEmits(['editDataChanged']);

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

// persons
const allPersons = ref<any[]>([]);
const selectedPerson = ref<any>(null);
const isRenamingPerson = ref(false);
const originalPersonName = ref('');
const personInputRef = ref<HTMLInputElement[]>([]);
const isIndexing = ref(false);
const indexProgress = ref({
  current: 0,
  total: 0,
  faces_found: 0,
  phase: 'indexing'
});
const clusterProgress = ref({
  phase: '',
  current: 0,
  total: 0
});
const incompleteCount = ref(0);
const personContextMenus = ref<Record<number, any>>({});
const isLoadingPersons = ref(true);
const isLoadingMorePersons = ref(false);
const hasMorePersons = ref(false);
const totalPersons = ref(0);
const PERSON_PAGE_SIZE = 100;
let personLoadRequest = 0;

function handlePersonContextMenu(person: any, event: MouseEvent) {
  selectPerson(person);
  personContextMenus.value[person.id]?.open?.(event.clientX, event.clientY);
}
const betaBadgeRef = ref<HTMLElement | null>(null);
const betaTooltipRef = ref<HTMLElement | null>(null);
const isBetaTooltipVisible = ref(false);
const betaTooltipStyle = ref<Record<string, string>>({});

// Event listener unsubscribe functions
let unlistenProgress: (() => void) | null = null;
let unlistenFinished: (() => void) | null = null;
let unlistenCluster: (() => void) | null = null;

const sortedPersons = computed(() => allPersons.value);

// Computed property to format cluster progress text using i18n
const { t } = useI18n();
const getPersonDisplayName = (person: any) => person?.name || t('menu.person.unnamed');
const clusterProgressText = computed(() => {
  const { phase, current, total } = clusterProgress.value;
  switch (phase) {
    case 'graph':
      return t('face_index.cluster_graph', { percent: current });
    case 'iterate':
      return t('face_index.cluster_iterate', { current, total });
    case 'converged':
      return t('face_index.cluster_converged', { current });
    case 'assign':
      return t('face_index.cluster_assign', { current, total });
    case 'thumbnail':
      return t('face_index.cluster_thumbnail');
    default:
      return '';
  }
});

const personPanelMenuItems = computed(() => [
  {
    label: localeMsg.value.menu.person.index_faces,
    icon: IconUpdate,
    action: () => clickIndexFaces(),
    disabled: isIndexing.value,
  },
  {
    label: localeMsg.value.menu.person.reset_index,
    icon: IconTrash,
    action: () => clickResetFaces(),
    disabled: isIndexing.value,
  },
]);

// message boxes
const showDeletePersonMsgbox = ref(false);
const showResetFacesMsgbox = ref(false);

// more menuitems
const getMoreMenuItems = () => [
  {
    label: localeMsg.value.menu?.person?.rename || 'Rename',
    icon: IconRename,
    action: () => {
      isRenamingPerson.value = true;
      originalPersonName.value = selectedPerson.value?.name || '';
      nextTick(() => {
        if (personInputRef.value && personInputRef.value[0]) {
          personInputRef.value[0].focus();
        }
      });
    }
  },
  {
    label: localeMsg.value.menu?.person?.delete || 'Delete',
    icon: IconTrash,
    action: () => {
      showDeletePersonMsgbox.value = true;
    },
  },
];

onMounted(async () => {
  loadPersons();
  checkFaceStats();
  
  // Check if indexing is already running and restore progress
  const [isRunning, progress] = await isFaceIndexing();
  
  if (isRunning) {
    isIndexing.value = true;
    if (progress) {
      indexProgress.value = progress;
    }
  }
  
  // Set up event listeners for face indexing progress
  unlistenProgress = await listenFaceIndexProgress((event: any) => {
    isIndexing.value = true; // Show overlay when receiving progress events
    indexProgress.value = event.payload;
  });
  
  unlistenFinished = await listenFaceIndexFinished((event: any) => {
    isIndexing.value = false;
    indexProgress.value = { current: 0, total: 0, faces_found: 0, phase: 'indexing' };
    clusterProgress.value = { phase: '', current: 0, total: 0 };
    loadPersons(); // Reload persons after indexing completes
    checkFaceStats();
  });
  
  // Listen for detailed clustering progress
  unlistenCluster = await listenClusterProgress((event: any) => {
    clusterProgress.value = event.payload;
  });
});

watch(() => config.settings.categorySort, () => {
  loadPersons();
});

onUnmounted(() => {
  if (unlistenProgress) unlistenProgress();
  if (unlistenFinished) unlistenFinished();
  if (unlistenCluster) unlistenCluster();
});

async function loadPersons(reset = true) {
  if (!reset && (!hasMorePersons.value || isLoadingMorePersons.value || isLoadingPersons.value)) return;

  const requestId = reset ? ++personLoadRequest : personLoadRequest;
  if (reset) {
    isLoadingPersons.value = true;
    allPersons.value = [];
    hasMorePersons.value = false;
    totalPersons.value = 0;
  } else {
    isLoadingMorePersons.value = true;
  }

  try {
    const page = await getPersonsPage(
      config.settings.categorySort,
      reset ? 0 : allPersons.value.length,
      PERSON_PAGE_SIZE,
    );
    if (requestId !== personLoadRequest) return;

    if (page) {
      allPersons.value = reset
        ? page.persons
        : [...allPersons.value, ...page.persons];
      hasMorePersons.value = page.has_more;
      totalPersons.value = page.total;
      if (allPersons.value.length > 0 && !selectedPerson.value) {
        const index = allPersons.value.findIndex(p => p.id === libConfig.person?.id);
        selectPerson(allPersons.value[index >= 0 ? index : 0]);
      }
    } else if (libConfig.person) {
      libConfig.person.id = null;
    }
  } finally {
    if (requestId === personLoadRequest) {
      isLoadingPersons.value = false;
      isLoadingMorePersons.value = false;
    }
  }
}

function loadMorePersons() {
  void loadPersons(false);
}

function selectPerson(person: any) {
  if (isRenamingPerson.value) return;
  selectedPerson.value = person;
  if (!libConfig.person) {
    libConfig.person = { id: null, name: null };
  }
  libConfig.person.id = person.id;
  libConfig.person.name = person.name;
}

async function handleRenamePerson() {
  if (!isRenamingPerson.value) return;

  const newName = selectedPerson.value?.name?.trim() || '';

  if (newName.length === 0 || newName === originalPersonName.value) {
    isRenamingPerson.value = false;
    if (selectedPerson.value) {
      selectedPerson.value.name = originalPersonName.value;
    }
    return;
  }

  const result = await renamePerson(selectedPerson.value.id, newName);
  if (result) {
    isRenamingPerson.value = false;
    await loadPersons();
  }
}

function cancelRenamePerson() {
  if (selectedPerson.value) {
    selectedPerson.value.name = originalPersonName.value;
  }
  isRenamingPerson.value = false;
}

async function clickDeletePerson() {
  if (selectedPerson.value) {
    showDeletePersonMsgbox.value = false;
    const result = await deletePerson(selectedPerson.value.id);
    if (result) {
      const index = allPersons.value.findIndex(p => p.id === selectedPerson.value.id);
      allPersons.value = allPersons.value.filter(p => p.id !== selectedPerson.value.id);
      if (index > 0) {
        selectPerson(allPersons.value[index - 1]);
      } else if (index === 0) {
        if (allPersons.value.length > 0) {
          selectPerson(allPersons.value[0]);
        } else {
          selectedPerson.value = null;
          if (libConfig.person) {
            libConfig.person.id = null;
          }
        }
      } else {
        selectedPerson.value = null;
        if (libConfig.person) {
          libConfig.person.id = null;
        }
      }
    }
  }
}

// Called from title bar context menu
async function clickIndexFaces() {
  if (isIndexing.value) {
    return;
  }
  
  isIndexing.value = true;
  try {
    // Get cluster threshold from array using index
    const face = config.settings.face;
    const thresholdIndex = face?.clusterThresholdIndex ?? 2; // Default: Medium (index 2)
    // Use getter for thresholds to ensure we get the latest values, even if state is old
    const thresholds = config.faceClusterThresholds ?? [0.35, 0.45, 0.55, 0.65];
    const clusterEpsilon = thresholds[thresholdIndex] ?? 0.55;
    console.log('clusterEpsilon', clusterEpsilon);
    await indexFaces(clusterEpsilon);
    await loadPersons();
    await checkFaceStats();
  } catch (e) {
    console.error('indexFaces error:', e);
    isIndexing.value = false;
  }
}

// Cancel face indexing
async function clickCancelIndex() {
  await cancelFaceIndex();
}

// Reset faces
async function clickResetFaces() {
  showResetFacesMsgbox.value = true;
}

async function onResetFacesConfirm() {
  showResetFacesMsgbox.value = false;
  
  // Reset selection and config
  selectedPerson.value = null;
  if (libConfig.person) {
    libConfig.person.id = null;
    libConfig.person.name = null;
  }

  await resetFaces();
  await loadPersons();
  checkFaceStats();
}

async function checkFaceStats() {
  const stats = await getFaceStats();
  if (stats) {
    incompleteCount.value = stats.unprocessed;
  }
}

async function showBetaTooltip() {
  if (!config.settings.showToolTip || !betaBadgeRef.value) return;

  isBetaTooltipVisible.value = true;
  await nextTick();

  if (!betaBadgeRef.value || !betaTooltipRef.value) return;

  const rect = betaBadgeRef.value.getBoundingClientRect();
  const tooltipRect = betaTooltipRef.value.getBoundingClientRect();
  const padding = 4;

  let top = rect.bottom + padding;
  let left = rect.left + (rect.width - tooltipRect.width) / 2;

  if (left + tooltipRect.width > window.innerWidth - padding) {
    left = window.innerWidth - tooltipRect.width - padding;
  }
  if (left < padding) {
    left = padding;
  }
  if (top + tooltipRect.height > window.innerHeight - padding) {
    top = rect.top - tooltipRect.height - padding;
  }

  betaTooltipStyle.value = {
    top: `${top}px`,
    left: `${left}px`,
  };
}

function hideBetaTooltip() {
  isBetaTooltipVisible.value = false;
}

defineExpose({
  clickIndexFaces,
  clickCancelIndex,
  loadPersons,
  clickResetFaces,
  isIndexing,
});

</script>
