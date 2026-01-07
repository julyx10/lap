<template>

  <div class="w-full h-full flex flex-col select-none">

    <!-- Title Bar -->
    <div class="p-1 h-12 flex items-start justify-end whitespace-nowrap" data-tauri-drag-region>
      <TButton
        :icon="IconAdd"
        @click="clickAddTag"
      />
      <TButton
        :icon="config.tag.sortCount ? IconSortingCount : IconSortingName"
        @click="config.tag.sortCount = !config.tag.sortCount"
      />
    </div>

    <!-- Tag List -->
    <div v-if="allTags.length > 0" class="flex-grow overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="tag in sortedTags" :key="tag.id" :id="'tag-' + tag.id">
          <div
            :class="[
              'mx-1 p-1 h-10 flex items-center rounded-box whitespace-nowrap cursor-pointer group transition-all duration-200 ease-in-out', 
              selectedTag && selectedTag.id === tag.id && !isRenamingTag ? 'text-primary bg-base-100 hover:bg-base-100' : 'hover:text-base-content hover:bg-base-100/30',
            ]"
            @click="selectTag(tag)"
          >
            <IconTag 
              class="mx-1 h-5 shrink-0" 
            />
            <input v-if="selectedTag && selectedTag.id === tag.id && isRenamingTag"
              ref="tagInputRef"
              type="text"
              maxlength="255"
              class="input px-1 w-full focus:border text-base"
              v-model="tag.name"
              @keydown.enter="handleRenameTag"
              @keydown.esc="cancelRenameTag"
              @blur="handleRenameTag"
            />
            <template v-else>
              <span class="flex-1 overflow-hidden whitespace-pre text-ellipsis">{{ tag.name }}</span>
              <span v-if="tag.count" class="text-xs tabular-nums text-base-content/30 ml-1">{{ tag.count.toLocaleString() }}</span>
              
              <div :class="[
                  'ml-auto flex flex-row items-center text-base-content/30',
                  selectedTag && selectedTag.id === tag.id ? '' : 'hidden group-hover:block'
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

    <!-- No Tags Found Message -->
    <div v-else class="mt-8 px-2 flex flex-col items-center justify-center text-base-content/30">
      <!-- <IconTag class="w-8 h-8" /> -->
      <span class="text-sm">{{ $t('tooltip.not_found.tag') }}</span>
    </div>
  </div>
  
  <!-- new tag -->
  <MessageBox
    v-if="showNewTagMsgbox"
    :title="$t('msgbox.new_tag.title')"
    :showInput="true"
    :inputText="''"
    :inputPlaceholder="$t('msgbox.new_tag.content')"
    :needValidateInput="true"
    :OkText="$t('msgbox.new_tag.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="clickNewTag"
    @cancel="showNewTagMsgbox = false"
  />

  <!-- delete tag -->
  <MessageBox
    v-if="showDeleteTagMsgbox"
    :title="$t('msgbox.delete_tag.title')"
    :message="`${$t('msgbox.delete_tag.content', { tag: selectedTag.name })}`"
    :OkText="$t('msgbox.delete_tag.ok')"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    @ok="clickDeleteTag"
    @cancel="showDeleteTagMsgbox = false"
  />
</template>

<script setup lang="ts">
import { ref, onMounted, computed, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { getAllTags, renameTag, deleteTag, createTag } from '@/common/api';
import { 
  IconTag, 
  IconMore, 
  IconRename, 
  IconTrash,
  IconAdd,
  IconSortingCount,
  IconSortingName,
} from '@/common/icons';

import TButton from '@/components/TButton.vue';
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

// tags
const allTags = ref<any[]>([]);
const selectedTag = ref(null);
const isRenamingTag = ref(false);
const originalTagName = ref('');
const tagInputRef = ref([]);

const sortedTags = computed(() => {
  if (config.tag.sortCount) {
    return [...allTags.value].sort((a, b) => (b.count || 0) - (a.count || 0));
  }
  return allTags.value;
});

// message boxes
const showDeleteTagMsgbox = ref(false);
const showNewTagMsgbox = ref(false);

// more menuitems
const getMoreMenuItems = () => [
  {
    label: localeMsg.value.menu.tag.rename,
    icon: IconRename,
    action: () => {
      isRenamingTag.value = true;
      originalTagName.value = selectedTag.value.name;
      nextTick(() => {
        if (tagInputRef.value) {
          tagInputRef.value[0].focus();    // array of input elements
        }
      });
    }
  },
  {
    label: localeMsg.value.menu.tag.delete,
    icon: IconTrash,
    action: () => {
      showDeleteTagMsgbox.value = true;
    },
  },
];

onMounted(() => {
  loadTags();
});

async function loadTags() {
  const tags = await getAllTags();
  if (tags) {
    allTags.value = tags;
    if (allTags.value.length > 0 && !selectedTag.value) {
      const index = allTags.value.findIndex(tag => tag.id === config.tag.id);
      selectTag(allTags.value[index >= 0 ? index : 0]);
    }
  } else {
    config.tag.id = null;
  }
}

function selectTag(tag: any) {
  if (isRenamingTag.value) return;
  selectedTag.value = tag;
  config.tag.id = tag.id;
}

async function handleRenameTag() {
  if (!isRenamingTag.value) return;

  const newName = selectedTag.value.name.trim();

  if (newName.length === 0 || newName === originalTagName.value) {
    isRenamingTag.value = false;
    selectedTag.value.name = originalTagName.value;
    return;
  }

  // rename tag
  const result = await renameTag(selectedTag.value.id, newName);
  if (result) {
    isRenamingTag.value = false;
  }
}

function cancelRenameTag() {
  selectedTag.value.name = originalTagName.value; // Revert the name on the selected tag
  isRenamingTag.value = false;
}

function clickAddTag() {
  showNewTagMsgbox.value = true;
}

async function clickNewTag(newTagName: string) {
  if (!newTagName || newTagName.trim().length === 0) {
    return;
  }
  const result = await createTag(newTagName);
  if (result) {
    showNewTagMsgbox.value = false;
    await loadTags();
    
    // select the new tag
    const newTag = allTags.value.find(tag => tag.name === newTagName);
    if (newTag) {
      selectTag(newTag);
      nextTick(() => {
        scrollToTag(newTag.id);
      });
    }
  }
}

function scrollToTag(tagId: number) {
  const tagElement = document.getElementById(`tag-${tagId}`);
  if (tagElement) {
    tagElement.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }
}

async function clickDeleteTag() {
  if (selectedTag.value) {
    showDeleteTagMsgbox.value = false;
    const result = await deleteTag(selectedTag.value.id);
    if (result) {
      // get the index of the selected tag
      const index = allTags.value.findIndex(tag => tag.id === selectedTag.value.id);
      // remove the selected tag
      allTags.value = allTags.value.filter(tag => tag.id !== selectedTag.value.id);
      // select the previous tag if exist
      if (index > 0) {
        selectTag(allTags.value[index - 1]);
      } else if (index === 0) {
        if (allTags.value.length > 0) {
          selectTag(allTags.value[0]);
        } else {
          selectedTag.value = null;
          config.tag.id = null;
        }
      } else {
        selectedTag.value = null;
        config.tag.id = null;
      }
    }
  }
}

</script>
