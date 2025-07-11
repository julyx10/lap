<template>

  <div class="w-full flex flex-col select-none">

    <!-- Title Bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between" data-tauri-drag-region>
      <span class="cursor-default">{{ titlebar }}</span>
      <TButton 
        :icon="IconAdd"
        :buttonSize="'medium'"
        @click="showNewTagMsgbox = true"
      />
    </div>

    <!-- Tag List -->
    <div v-if="allTags.length > 0" class="flex-grow overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="tag in allTags" :key="tag.id">
          <div
            :class="[
              'my-1 mr-1 h-8 flex items-center rounded border-l-2 border-base-200 hover:bg-base-content/10 whitespace-nowrap cursor-pointer group',
              {
                'bg-base-content/10 border-primary text-base-content': selectedTag && selectedTag.id === tag.id && !isRenamingTag,
                'border-transparent': !selectedTag || selectedTag.id !== tag.id || isRenamingTag
              }
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
              <div class="overflow-hidden whitespace-pre text-ellipsis">
                {{ tag.name }}
              </div>
              <DropDownMenu 
                :class="[
                  'ml-auto px-1 rounded',
                  selectedTag && selectedTag.id !== tag.id ? 'invisible group-hover:visible' : ''
                ]"
                :iconMenu="IconMore"
                :menuItems="getMoreMenuItems()"
                :smallIcon="true"
              />
            </template>
          </div>
        </li>
      </ul>
    </div>

    <!-- No Tags Found Message -->
    <div v-else class="mt-10 flex items-center justify-center">
      {{ $t('tooltip.not_found.tag') }}
    </div>

  </div>

  <!-- new tag -->
  <MessageBox
    v-if="showNewTagMsgbox"
    :title="$t('msgbox.new_tag.title')"
    :message="$t('msgbox.new_tag.content')"
    :showInput="true"
    :inputText="''"
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
import { config } from '@/common/utils';
import { getAllTags, createTag, renameTag, deleteTag } from '@/common/api';
import { 
  IconTag, 
  IconMore, 
  IconRename, 
  IconTrash,
  IconAdd
} from '@/common/icons';

import TButton from '@/components/TButton.vue';
import DropDownMenu from '@/components/DropDownMenu.vue';
import MessageBox from '@/components/MessageBox.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

// tags
const allTags = ref([]);
const selectedTag = ref(null);
const isRenamingTag = ref(false);
const originalTagName = ref('');
const tagInputRef = ref([]);

// message boxes
const showNewTagMsgbox = ref(false);
const showDeleteTagMsgbox = ref(false);

// more menuitems
const getMoreMenuItems = () => [
  {
    label: localeMsg.value.menu.rename,
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
    label: localeMsg.value.menu.delete,
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
      const index = allTags.value.findIndex(tag => tag.id === config.tagId);
      selectTag(allTags.value[index >= 0 ? index : 0]);
    }
  }
}

function selectTag(tag) {
  if (isRenamingTag.value) return;
  selectedTag.value = tag;
  config.tagId = tag.id;
}

async function clickNewTag(tagName) {
  showNewTagMsgbox.value = false;
  const newTag = await createTag(tagName);
  if (newTag) {
    allTags.value.push(newTag);
    selectTag(newTag);
  }
}

async function handleRenameTag() {
  if (!isRenamingTag.value) return;

  const newName = selectedTag.value.name.trim();

  if (newName.length === 0 || newName === originalTagName.value) {
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
          config.tagId = null;
        }
      } else {
        selectedTag.value = null;
        config.tagId = null;
      }
    }
  }
}

</script>
