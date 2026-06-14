<template>
  <ModalDialog :title="$t('tag.edit_tag')" :width="600" @cancel="clickCancel">
    <section class="space-y-3">
      <div class="flex items-center gap-2">
        <div 
          :class="[
            'grow h-8 flex items-center rounded-box transition-colors bg-base-100',
            isSearchFocused ? 'border-2 border-primary' : 'border border-neutral-content/30 hover:border-neutral-content/70'
          ]"
        >
          <IconSearch class="ml-2 w-4 h-4 text-base-content/70" />
          <input
            ref="tagSearchInputRef"
            type="text"
            v-model="tagSearch"
            :placeholder="$t('tag.search_tags')"
            class="w-full bg-transparent border-none focus:ring-0 px-2 text-sm placeholder-base-content/30 focus:outline-none"
            @focus="onSearchFocus"
            @blur="onSearchBlur"
          />
        </div>
        <div
          :class="[
            'w-1/2 h-8 flex items-center rounded-box transition-colors bg-base-100',
            isNewTagFocused ? 'border-2 border-primary' : 'border border-neutral-content/30 hover:border-neutral-content/70'
          ]"
        >
          <input
            ref="newTagNameInputRef"
            type="text"
            v-model="newTagName"
            :placeholder="$t('tag.enter_new_tag_name')"
            class="w-full bg-transparent border-none focus:ring-0 px-2 text-sm placeholder-base-content/30 focus:outline-none"
            @focus="isNewTagFocused = true"
            @blur="isNewTagFocused = false"
            @keydown.enter="addNewTag"
          />
        </div>
        <TButton 
          :icon="IconAdd"
          :tooltip="$t('tag.add_tag')"
          @click="addNewTag"
        />
      </div>

      <div class="max-h-[180px] overflow-y-auto rounded-box p-2 bg-base-100/40 border border-base-content/5">
        <div v-if="filteredTags.length > 0" class="flex flex-wrap gap-2">
          <div
            v-for="(tag, index) in filteredTags"
            :key="tag.id"
            :class="[
              'badge badge-lg overflow-hidden whitespace-pre text-ellipsis cursor-pointer transition-colors duration-200',
              {
                'badge-primary': selectedTags.has(tag.id),
                'badge-outline border-base-content/30 bg-base-content/30': intermediateTags.has(tag.id) && !selectedTags.has(tag.id),
                'badge-outline text-base-content/30 hover:text-base-content/70 hover:bg-base-100': !selectedTags.has(tag.id) && !intermediateTags.has(tag.id),
                'ring-2 ring-primary ring-offset-1 ring-offset-base-100': focusedTagIndex === index,
              }
            ]"
            @click="toggleTag(tag.id)"
          >
            <span>{{ tag.name }}</span>
          </div>
        </div>
        <div v-else class="py-10 text-center text-base-content/30">
          {{ $t('tag.not_found') }}
        </div>
      </div>
      <div v-if="tagLoadFailed" class="text-sm text-error">
        {{ $t('tag.load_failed') }}
      </div>
    </section>

    <!-- cancel and OK buttons -->
    <div class="mt-4 flex justify-end space-x-4">
      <button 
        class="t-button-default" 
        @click="clickCancel"
      >{{ $t('msgbox.cancel') }}</button>
      
      <button 
        class="t-button-primary" 
        :disabled="isLoadingTags || isApplyingTags || tagLoadFailed"
        @click="clickOk"
      >{{ $t('msgbox.ok') }}</button>

    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onBeforeUnmount } from 'vue';
import { 
  getAllTags, 
  createTag, 
  getTagSelectionCounts,
  applyTagsToFiles,
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
const isNewTagFocused = ref(false);
const focusedTagIndex = ref(-1); // -1 = no tag focused
const isInTagList = ref(false); // true = keyboard focus is in tag list
const isLoadingTags = ref(false);
const isApplyingTags = ref(false);
const tagLoadFailed = ref(false);

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

  await nextTick();
  tagSearchInputRef.value?.focus();

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
  tagLoadFailed.value = false;

  if (props.fileIds.length === 0) {
    return;
  }

  isLoadingTags.value = true;
  try {
    const counts = await getTagSelectionCounts(props.fileIds);
    if (counts === null) {
      tagLoadFailed.value = true;
      return;
    }
    for (const entry of counts) {
      const tagId = Number(entry.tag_id);
      const count = Number(entry.count);
      if (count === props.fileIds.length) {
        selectedTags.value.add(tagId);
      } else if (count > 0) {
        intermediateTags.value.add(tagId);
      }
    }
  } finally {
    isLoadingTags.value = false;
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
  if (isLoadingTags.value || isApplyingTags.value || tagLoadFailed.value) return;
  isApplyingTags.value = true;
  const addTagIds = Array.from(selectedTags.value);
  const removeTagIds = allTags.value
    .map((tag: any) => Number(tag.id))
    .filter((tagId: number) => !selectedTags.value.has(tagId) && !intermediateTags.value.has(tagId));
  const result = await applyTagsToFiles(props.fileIds, addTagIds, removeTagIds);
  if (result !== null) {
    emit('ok', result);
  } else {
    isApplyingTags.value = false;
  }
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

  if (key === 'Tab' && active === newTagNameInputRef.value && !e.shiftKey) {
    e.preventDefault();
    if (filteredTags.value.length > 0) {
      enterTagList();
    } else {
      tagSearchInputRef.value?.focus();
    }
    return;
  }

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

    if (key === 'ArrowRight') {
      e.preventDefault();
      focusedTagIndex.value = focusedTagIndex.value >= lastIndex ? 0 : focusedTagIndex.value + 1;
    } else if (key === 'ArrowLeft') {
      e.preventDefault();
      focusedTagIndex.value = focusedTagIndex.value <= 0 ? lastIndex : focusedTagIndex.value - 1;
    } else if (key === 'ArrowUp') {
      e.preventDefault();
      exitTagList();
    } else if (key === ' ') {
      e.preventDefault();
      const tag = filteredTags.value[focusedTagIndex.value];
      if (tag) toggleTag(tag.id);
    } else if (key === 'Enter') {
      e.preventDefault();
      clickOk();
    } else if (key === 'Tab') {
      e.preventDefault();
      if (e.shiftKey) {
        isInTagList.value = false;
        focusedTagIndex.value = -1;
        newTagNameInputRef.value?.focus();
      } else {
        exitTagList(); // Tab → back to search input (keep focus inside dialog)
      }
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
