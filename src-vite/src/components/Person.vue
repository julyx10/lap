<template>
  <div class="w-full h-full flex flex-col select-none relative overflow-hidden">
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
        <div class="p-3 bg-warning/10 text-warning rounded-box flex flex-row items-center gap-2">
          <IconUpdate class="w-5 h-5 shrink-0" />
          <span class="text-xs flex-1">
            {{ $t('face_index.incomplete', { count: incompleteCount.toLocaleString() }) }}
          </span>
          <button class="btn btn-xs btn-warning btn-outline bg-base-100" @click="clickIndexFaces">
            {{ $t('face_index.resume') }}
          </button>
        </div>
    </div>

    <!-- Person List -->
    <div v-if="allPersons.length > 0" class="flex-grow overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="person in sortedPersons" :key="person.id" :id="'person-' + person.id">
          <div
            :class="[
              'mx-1 p-1 h-12 flex items-center gap-2 rounded-box whitespace-nowrap cursor-pointer group transition-all duration-200 ease-in-out', 
              selectedPerson && selectedPerson.id === person.id && !isRenamingPerson ? 'text-primary bg-base-100 hover:bg-base-100' : 'hover:text-base-content hover:bg-base-100/30',
            ]"
            @click="selectPerson(person)"
          >
            <!-- Face thumbnail -->
            <div class="w-10 h-10 rounded-box overflow-hidden bg-base-300 shrink-0 flex items-center justify-center">
              <img 
                v-if="person.thumbnail" 
                :src="'data:image/jpeg;base64,' + person.thumbnail" 
                class="w-full h-full object-cover"
              />
              <IconPerson v-else class="w-6 h-6 text-base-content/30" />
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
              <span class="flex-1 overflow-hidden whitespace-pre text-ellipsis">
                {{ person.name || `Person ${person.id}` }}
              </span>
              <span v-if="person.count" class="text-xs tabular-nums text-base-content/30 ml-1">
                {{ person.count.toLocaleString() }}
              </span>
              
              <div :class="[
                  'ml-auto flex flex-row items-center text-base-content/30',
                  selectedPerson && selectedPerson.id === person.id ? '' : 'hidden group-hover:block'
                ]"
              >
                <ContextMenu 
                  :iconMenu="IconMore"
                  :menuItems="getMoreMenuItems()"
                  :smallIcon="true"
                />
              </div>
            </template>
          </div>
        </li>
      </ul>
    </div>

    <!-- No Persons Found Message -->
    <div v-else-if="!isIndexing" class="mt-8 px-2 flex flex-col items-center justify-center text-base-content/30">
      <IconPerson class="w-8 h-8 mb-2" />
      <template v-if="incompleteCount > 0">
         <span class="text-sm text-center text-warning">{{ $t('face_index.incomplete', { count: incompleteCount.toLocaleString() }) }}</span>
         <button class="btn btn-warning btn-sm mt-4" @click="clickIndexFaces">
          <IconUpdate class="w-4 h-4" />
          {{ $t('face_index.resume') }}
        </button>
      </template>
      <template v-else>
        <span class="text-sm text-center">{{ $t('tooltip.not_found.person') }}</span>
        <span class="text-xs text-center mt-1">{{ $t('tooltip.not_found.person_hint') }}</span>
        <button class="btn btn-primary btn-sm mt-4" @click="clickIndexFaces">
          <IconUpdate class="w-4 h-4" />
          {{ $t('face_index.start') }}
        </button>
      </template>
    </div>

  </div>

  <!-- Delete person confirmation -->
  <MessageBox
    v-if="showDeletePersonMsgbox"
    :title="$t('msgbox.delete_person.title')"
    :message="`${$t('msgbox.delete_person.content', { person: selectedPerson?.name || 'Person ' + selectedPerson?.id })}`"
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
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { config, libConfig } from '@/common/config';
import { getPersons, renamePerson, deletePerson, indexFaces, cancelFaceIndex, isFaceIndexing, listenFaceIndexProgress, listenFaceIndexFinished, listenClusterProgress, resetFaces, getFaceStats } from '@/common/api';
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

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

// persons
const allPersons = ref<any[]>([]);
const selectedPerson = ref<any>(null);
const isRenamingPerson = ref(false);
const originalPersonName = ref('');
const personInputRef = ref([]);
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

// Event listener unsubscribe functions
let unlistenProgress: (() => void) | null = null;
let unlistenFinished: (() => void) | null = null;
let unlistenCluster: (() => void) | null = null;

const sortedPersons = computed(() => {
  if (config.leftPanel.sortCount) {
    return [...allPersons.value].sort((a, b) => (b.count || 0) - (a.count || 0));
  } else {
    return [...allPersons.value].sort((a, b) => (a.name || '').localeCompare(b.name || '', undefined, { numeric: true, sensitivity: 'base' }));
  }
});

// Computed property to format cluster progress text using i18n
const { t } = useI18n();
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

onUnmounted(() => {
  if (unlistenProgress) unlistenProgress();
  if (unlistenFinished) unlistenFinished();
  if (unlistenCluster) unlistenCluster();
});

async function loadPersons() {
  const persons = await getPersons();
  if (persons) {
    allPersons.value = persons;
    if (allPersons.value.length > 0 && !selectedPerson.value) {
      const index = allPersons.value.findIndex(p => p.id === libConfig.person?.id);
      selectPerson(allPersons.value[index >= 0 ? index : 0]);
    }
  } else {
    if (libConfig.person) {
      libConfig.person.id = null;
    }
  }
}

function selectPerson(person: any) {
  if (isRenamingPerson.value) return;
  selectedPerson.value = person;
  if (!libConfig.person) {
    libConfig.person = { id: null };
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
  } finally {
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

defineExpose({
  clickIndexFaces,
  clickCancelIndex,
  loadPersons,
  clickResetFaces,
});

</script>