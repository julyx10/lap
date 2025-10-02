<template>
  <dialog id="taggingDialog" class="modal modal-open">
    <div class="w-[600px] p-4 text-base-content/70 bg-base-100 border border-base-content/30 rounded-box">
      
      <!-- title bar -->
      <div class="mb-2 flex items-center justify-between">
        <span class="text-lg font-bold">{{ $t('tag.edit_tag') }}</span>
        <TButton
          :icon="IconClose"
          :buttonSize="'small'"
          @click="clickCancel"
        />
      </div>

      <!-- Search and Add New Tag -->
      <div class="py-4 flex items-center space-x-2">
        <input
          ref="tagSearchInputRef"
          type="text"
          v-model="tagSearch"
          :placeholder="$t('tag.search_tags')"
          class="input focus:border flex-grow"
        />
        <input
          ref="newTagNameInputRef"
          type="text"
          v-model="newTagName"
          :placeholder="$t('tag.enter_new_tag_name')"
          class="input focus:border w-1/2"
          @keydown.enter="addNewTag"
        />
        <TButton 
          :icon="IconAdd"
          @click="addNewTag"
        />
      </div>

      <!-- Tag List -->
      <div class="max-h-80 overflow-y-auto border border-base-content/10  rounded-md p-2">
        <div v-if="filteredTags.length > 0" class="flex flex-wrap gap-2">
          <div
            v-for="tag in filteredTags"
            :key="tag.id"
            :class="[
              'badge badge-lg overflow-hidden whitespace-pre text-ellipsis cursor-pointer transition-colors duration-100',
              {
                'badge-primary': selectedTags.has(tag.id),
                'badge-outline border-base-content/30 bg-base-content/20': intermediateTags.has(tag.id) && !selectedTags.has(tag.id),
                'badge-outline text-base-content/30': !selectedTags.has(tag.id) && !intermediateTags.has(tag.id),
              }
            ]"
            @click="toggleTag(tag.id)"
          >
            {{ tag.name }}
          </div>
        </div>
        <div v-else class="text-center text-gray-500">
          {{ $t('tag.not_found') }}
        </div>
      </div>

      <!-- cancel and OK buttons -->
      <div class="mt-4 flex justify-end space-x-4">
        <button 
          class="px-4 py-1 rounded-lg hover:bg-base-content/30 cursor-pointer" 
          @click="clickCancel"
        >{{ $t('msgbox.cancel') }}</button>
        
        <button 
          class="px-4 py-1 rounded-lg hover:bg-primary cursor-pointer" 
          @click="clickOk"
        >{{ $t('msgbox.ok') }}</button>

      </div>
    </div>
  </dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue';
import { 
  getAllTags, 
  getTagsForFile, 
  createTag, 
  addTagToFile, 
  removeTagFromFile 
} from '@/common/api';
import { listen } from '@tauri-apps/api/event';
import { IconAdd, IconClose } from '@/common/icons';
import TButton from './TButton.vue';

const props = defineProps({
  fileIds: {
    type: Array as () => number[],
    default: () => [],
  },
});

const emit = defineEmits(['ok', 'cancel']);

const allTags = ref<any[]>([]);
const tagSearchInputRef = ref(null);
const newTagNameInputRef = ref(null);
const tagSearch = ref('');
const newTagName = ref('');

// Sets to track tag states
const selectedTags = ref<Set<number>>(new Set()); // Tags present on ALL selected files
const intermediateTags = ref<Set<number>>(new Set()); // Tags present on SOME selected files

const filteredTags = computed(() => {
  if (!tagSearch.value) {
    return allTags.value;
  }
  return allTags.value.filter(tag =>
    tag.name.toLowerCase().includes(tagSearch.value.toLowerCase())
  );
});

let unlistenKeydown: () => void;

onMounted(async () => {
  unlistenKeydown = await listen('global-keydown', handleKeyDown);

  setTimeout(() => {
    tagSearchInputRef.value?.focus();
  }, 50); // 50ms delay

  loadAllTags();
  loadExistingTagsForFiles();
});

onBeforeUnmount(() => {
  if (unlistenKeydown) {
    unlistenKeydown();
  }
});

// watch(() => taggingDialog.value.show, (newVal) => {
//   if (newVal) {
//     console.log('TaggingDialog show', props.fileIds.length);
//     loadAllTags();
//     loadExistingTagsForFiles();
//   } else {
//     tagSearch.value = '';
//     newTagName.value = '';
//     selectedTags.value.clear();
//     intermediateTags.value.clear();
//   }
// }, { immediate: true });

// load all tags
async function loadAllTags() {
  allTags.value = (await getAllTags()) || [];
}

async function loadExistingTagsForFiles() {
  selectedTags.value.clear();
  intermediateTags.value.clear();

  if (props.fileIds.length === 0) {
    return;
  }

  const tagsPerFile: Map<number, Set<number>> = new Map();
  const allUniqueTagIds = new Set<number>();

  // Fetch tags for each file
  for (const fileId of props.fileIds) {
    const tags = (await getTagsForFile(fileId)) || [];
    const tagIdsForFile = new Set(tags.map((tag: any) => tag.id));
    tagsPerFile.set(fileId, tagIdsForFile);
    tags.forEach((tag: any) => allUniqueTagIds.add(tag.id));
  }

  // Determine selected and intermediate tags
  for (const tagId of allUniqueTagIds) {
    let count = 0;
    for (const fileId of props.fileIds) {
      if (tagsPerFile.get(fileId)?.has(tagId)) {
        count++;
      }
    }

    if (count === props.fileIds.length) {
      selectedTags.value.add(tagId);
    } else if (count > 0) {
      intermediateTags.value.add(tagId);
    }
  }
}

async function addNewTag() {
  const trimmedName = newTagName.value.trim();
  if (trimmedName) {
    const existingTag = allTags.value.find(tag => tag.name.toLowerCase() === trimmedName.toLowerCase());
    if (existingTag) {
      // If tag already exists, just select it
      toggleTag(existingTag.id);
    } else {
      const newTag = await createTag(trimmedName);
      if (newTag) {
        allTags.value.push(newTag);
        toggleTag(newTag.id);
      }
    }
    newTagName.value = ''; // Clear input
  }
}

function toggleTag(tagId: number) {
  if (selectedTags.value.has(tagId)) {
    selectedTags.value.delete(tagId);
    intermediateTags.value.delete(tagId); // No longer intermediate if fully deselected
  } else {
    selectedTags.value.add(tagId);
    intermediateTags.value.delete(tagId); // If it was intermediate, it's now fully selected
  }
}

async function clickOk() {
  // For each file, determine which tags to add/remove
  for (const fileId of props.fileIds) {
    const existingTagsForFile = new Set((await getTagsForFile(fileId))?.map((tag: any) => tag.id) || []);

    for (const tag of allTags.value) {
      const shouldBeSelected = selectedTags.value.has(tag.id);
      const isCurrentlySelected = existingTagsForFile.has(tag.id);

      if (shouldBeSelected && !isCurrentlySelected) {
        // Add tag to file
        await addTagToFile(fileId, tag.id);
      } else if (!shouldBeSelected && isCurrentlySelected && !intermediateTags.value.has(tag.id)) {
        // Remove tag from file (only if not in intermediate state for multi-select)
        await removeTagFromFile(fileId, tag.id);
      }
    }
  }
  emit('ok', props.fileIds);
}

function clickCancel() {
  emit('cancel');
}

// handle escape key
const handleKeyDown = (e: KeyboardEvent) => {
  const { key } = e.payload;
  if(key === 'Escape') {
    clickCancel();
  }
};

</script>

<style scoped>
</style>
