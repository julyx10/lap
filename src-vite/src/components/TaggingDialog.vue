<template>
  <ModalDialog :title="$t('tag.edit_tag')" :width="600" @cancel="clickCancel">
    <section class="rounded-box p-3 space-y-3 bg-base-300/30 border border-base-content/5 shadow-sm">
      <div class="flex items-center gap-2">
        <div 
          :class="[
            'grow h-8 flex items-center rounded-box transition-colors bg-base-100',
            isSearchFocused ? 'border-2 border-primary' : 'border border-neutral-content/20 hover:border-neutral-content/50'
          ]"
        >
          <IconSearch class="ml-2 w-4 h-4 text-base-content/50" />
          <input
            ref="tagSearchInputRef"
            type="text"
            v-model="tagSearch"
            :placeholder="$t('tag.search_tags')"
            class="w-full bg-transparent border-none focus:ring-0 px-2 text-sm placeholder-base-content/30 focus:outline-none"
            @focus="isSearchFocused = true"
            @blur="isSearchFocused = false"
          />
        </div>
        <input
          ref="newTagNameInputRef"
          type="text"
          v-model="newTagName"
          :placeholder="$t('tag.enter_new_tag_name')"
          class="input input-sm w-1/2"
          @keydown.enter="addNewTag"
        />
        <TButton 
          :icon="IconAdd"
          :tooltip="$t('tag.add_tag')"
          @click="addNewTag"
        />
      </div>

      <div class="max-h-[180px] overflow-y-auto rounded-box p-2 bg-base-100/40 border border-base-content/5">
        <div v-if="filteredTags.length > 0" class="flex flex-wrap gap-2">
          <div
            v-for="tag in filteredTags"
            :key="tag.id"
            :class="[
              'badge badge-lg overflow-hidden whitespace-pre text-ellipsis cursor-pointer transition-colors duration-200',
              {
                'badge-primary': selectedTags.has(tag.id),
                'badge-outline border-base-content/30 bg-base-content/30': intermediateTags.has(tag.id) && !selectedTags.has(tag.id),
                'badge-outline text-base-content/30 hover:text-base-content hover:bg-base-100': !selectedTags.has(tag.id) && !intermediateTags.has(tag.id),
              }
            ]"
            @click="toggleTag(tag.id)"
          >
            <span>{{ tag.name }}</span>
          </div>
        </div>
        <div v-else class="py-10 text-center text-base-content/35">
          {{ $t('tag.not_found') }}
        </div>
      </div>
    </section>

    <!-- cancel and OK buttons -->
    <div class="mt-4 flex justify-end space-x-4">
      <button 
        class="px-4 py-1 rounded-box hover:bg-base-100 hover:text-base-content cursor-pointer" 
        @click="clickCancel"
      >{{ $t('msgbox.cancel') }}</button>
      
      <button 
        class="px-4 py-1 rounded-box hover:bg-primary hover:text-base-100 cursor-pointer" 
        @click="clickOk"
      >{{ $t('msgbox.ok') }}</button>

    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onBeforeUnmount } from 'vue';
import { 
  getAllTags, 
  getTagsForFile, 
  createTag, 
  addTagToFile, 
  removeTagFromFile 
} from '@/common/api';
import { IconAdd, IconSearch } from '@/common/icons';
import TButton from './TButton.vue';
import { useUIStore } from '@/stores/uiStore';
import ModalDialog from '@/components/ModalDialog.vue';

const props = defineProps({
  fileIds: {
    type: Array as () => number[],
    default: () => [],
  },
});

const emit = defineEmits(['ok', 'cancel']);
const uiStore = useUIStore();

const allTags = ref<any[]>([]);
const tagSearchInputRef = ref<HTMLInputElement | null>(null);
const newTagNameInputRef = ref<HTMLInputElement | null>(null);
const tagSearch = ref('');
const newTagName = ref('');
const isSearchFocused = ref(false);
const focusedTagIndex = ref(-1); // -1 = no tag focused
const isInTagList = ref(false); // true = keyboard focus is in tag list

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

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('TaggingDialog');

  setTimeout(() => {
    tagSearchInputRef.value?.focus();
  }, 50); // 50ms delay

  loadAllTags();
  loadExistingTagsForFiles();
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('TaggingDialog');
});

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
    const tagIdsForFile = new Set<number>(tags.map((tag: any) => tag.id));
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

// Reset tag focus when search results change
watch(filteredTags, () => {
  focusedTagIndex.value = -1;
  isInTagList.value = false;
});

function onSearchFocus() {
  isSearchFocused.value = true;
  isInTagList.value = false;
  focusedTagIndex.value = -1;
}

function onSearchBlur() {
  isSearchFocused.value = false;
}

function enterTagList() {
  if (filteredTags.value.length > 0) {
    isInTagList.value = true;
    focusedTagIndex.value = 0;
    tagSearchInputRef.value?.blur();
    newTagNameInputRef.value?.blur();
  }
}

function exitTagList() {
  isInTagList.value = false;
  focusedTagIndex.value = -1;
  tagSearchInputRef.value?.focus();
}

// Keyboard: ArrowDown→tag list, Space→toggle, Enter→OK, Escape→back/close
const handleKeyDown = (e: KeyboardEvent) => {
  if (!uiStore.isInputActive('TaggingDialog')) return;

  const { key } = e;
  const active = document.activeElement;
  const isInAnyInput = active === tagSearchInputRef.value || active === newTagNameInputRef.value;

  // Escape: tag list → search input → close dialog
  if (key === 'Escape') {
    if (isInTagList.value) {
      exitTagList();
    } else {
      clickCancel();
    }
    return;
  }

  // ArrowDown → enter tag list (from any state, unless already navigating)
  if (key === 'ArrowDown' && !isInTagList.value) {
    e.preventDefault();
    enterTagList();
    return;
  }

  // Tag list keyboard navigation
  if (isInTagList.value && filteredTags.value.length > 0) {
    const lastIndex = filteredTags.value.length - 1;

    if (key === 'ArrowRight' || key === 'ArrowDown') {
      e.preventDefault();
      focusedTagIndex.value = focusedTagIndex.value >= lastIndex ? 0 : focusedTagIndex.value + 1;
    } else if (key === 'ArrowLeft' || key === 'ArrowUp') {
      e.preventDefault();
      if (focusedTagIndex.value <= 0) {
        exitTagList(); // first tag → back to search
      } else {
        focusedTagIndex.value -= 1;
      }
    } else if (key === ' ') {
      e.preventDefault();
      const tag = filteredTags.value[focusedTagIndex.value];
      if (tag) toggleTag(tag.id);
    } else if (key === 'Enter') {
      e.preventDefault();
      clickOk();
    } else if (key === 'Tab') {
      e.preventDefault();
      exitTagList(); // Tab → back to search input (keep focus inside dialog)
    }
    return;
  }

  // Enter → confirm dialog (when not typing in any input)
  if (key === 'Enter' && !isInAnyInput) {
    e.preventDefault();
    clickOk();
  }
};
</script>
