<template>

  <div class="sidebar-panel">
    <div class="grow overflow-x-hidden overflow-y-auto">
      <!-- smart tags -->
      <ul>
        <li>
          <div
            :class="[
              'sidebar-item',
              selectedSmartTagId ? 'text-primary' : '',
              'sidebar-item-hover',
            ]"
            @click="isSmartExpanded = !isSmartExpanded"
          >
            <IconBolt class="mx-1 h-5 shrink-0" />
            <span class="sidebar-item-label">{{ localeMsg.tag.smart_group }}</span>
            <IconArrowDown
              class="mx-1 h-4 w-4 shrink-0 transition-transform duration-200"
              :style="{ transform: isSmartExpanded ? 'rotate(180deg)' : 'rotate(0deg)' }"
            />
          </div>
        </li>

        <li v-for="item in smartTagItems" v-show="isSmartExpanded" :key="item.id" :id="'smart-tag-' + item.id">
          <div
            :class="[
              'sidebar-item ml-7',
              selectedSmartTagId === item.id ? 'sidebar-item-selected' : 'sidebar-item-hover',
            ]"
            @click="selectSmartTag(item.id)"
          >
            <IconTag class="mx-1 h-4 shrink-0" />
            <span class="sidebar-item-label">{{ item.label }}</span>
          </div>
        </li>
      </ul>

      <!-- custom tags -->
      <div class="sidebar-panel-header">
        <span class="sidebar-panel-header-title">{{ localeMsg.tag.custom_group }}</span>
        <ContextMenu
          class="sidebar-panel-action"
          :menuItems="panelMenuItems"
          :iconMenu="IconMore"
          :smallIcon="true"
        />
      </div>

      <ul v-if="allTags.length > 0">
        <li v-for="tag in sortedTags" :key="tag.id" :id="'tag-' + tag.id">
          <div
            :class="[
              'sidebar-item group',
              selectedTag && selectedTag.id === tag.id && !isRenamingTag ? 'sidebar-item-selected' : 'sidebar-item-hover',
            ]"
            @click="selectTag(tag)"
          >
            <IconTag class="mx-1 h-5 shrink-0" />
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
            <span v-else class="sidebar-item-label">{{ tag.name }}</span>
            <span v-if="!isRenamingTag && tag.count" class="sidebar-item-count">{{ tag.count.toLocaleString() }}</span>
            <div
              v-if="!isRenamingTag"
              :class="[
                'ml-auto flex flex-row items-center text-base-content/30',
                selectedTag && selectedTag.id === tag.id ? '' : 'hidden'
              ]"
            >
              <ContextMenu
                :iconMenu="IconMore"
                :menuItems="getMoreMenuItems()"
                :smallIcon="true"
              />
            </div>
          </div>
        </li>
      </ul>
    </div>

    <!-- No Tags Found Message -->
    <div v-if="allTags.length === 0" class="sidebar-empty">
        <IconTag class="w-8 h-8 mb-2" />
        <span class="text-sm text-center">{{ $t('tooltip.not_found.tag') }}</span>
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
import { config, libConfig } from '@/common/config';
import { getAllTags, renameTag, deleteTag, createTag } from '@/common/api';
import { 
  IconAdd,
  IconTag,
  IconBolt,
  IconDot,
  IconArrowDown,
  IconMore, 
  IconRename, 
  IconTrash,
} from '@/common/icons';
import { SMART_TAG_CATEGORIES, getSmartTagById } from '@/common/smartTags';

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

const emit = defineEmits(['editDataChanged']);

// tags
const allTags = ref<any[]>([]);
const selectedTag = ref<any>(null);
const selectedSmartTagId = ref<string | null>(libConfig.tag.smartId || null);
const isSmartExpanded = ref(Boolean(libConfig.tag.smartId));
const isRenamingTag = ref(false);
const originalTagName = ref('');
const tagInputRef = ref<HTMLInputElement[]>([]);

const sortedTags = computed(() => {
  if (config.leftPanel.sortCount) {
    return [...allTags.value].sort((a, b) => (b.count || 0) - (a.count || 0));
  }
  return allTags.value;
});

const smartTagItems = computed(() => {
  return SMART_TAG_CATEGORIES.map(category => {
    const item = category.items[0];
    return {
      id: item.id,
      label: localeMsg.value.tag.smart_items?.[item.id] || item.id,
    };
  });
});

const panelMenuItems = computed(() => [
  {
    label: localeMsg.value.menu.tag.add,
    icon: IconAdd,
    action: () => clickAddTag(),
  },
  { label: '-' },
  {
    label: localeMsg.value.menu.sort.sort_by_name,
    icon: config.leftPanel.sortCount ? null : IconDot,
    action: () => { config.leftPanel.sortCount = false; },
  },
  {
    label: localeMsg.value.menu.sort.sort_by_count,
    icon: config.leftPanel.sortCount ? IconDot : null,
    action: () => { config.leftPanel.sortCount = true; },
  },
]);

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
  if (selectedSmartTagId.value && !getSmartTagById(selectedSmartTagId.value)) {
    selectedSmartTagId.value = null;
    libConfig.tag.smartId = null;
  }
  loadTags();
});

async function loadTags() {
  const tags = await getAllTags();
  if (tags) {
    allTags.value = tags;
    if (selectedSmartTagId.value) {
      return;
    }
    if (allTags.value.length > 0 && !selectedTag.value) {
      const index = allTags.value.findIndex(tag => tag.id === libConfig.tag.id);
      selectTag(allTags.value[index >= 0 ? index : 0]);
    }
  } else {
    libConfig.tag.id = null;
  }
}

function selectTag(tag: any) {
  if (isRenamingTag.value) return;
  selectedTag.value = tag;
  selectedSmartTagId.value = null;
  libConfig.tag.smartId = null;
  libConfig.tag.id = tag.id;
}

function selectSmartTag(smartTagId: string) {
  if (isRenamingTag.value) return;
  selectedSmartTagId.value = smartTagId;
  selectedTag.value = null;
  libConfig.tag.id = null;
  libConfig.tag.smartId = smartTagId;
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
          libConfig.tag.id = null;
          libConfig.tag.smartId = null;
        }
      } else {
        selectedTag.value = null;
        libConfig.tag.id = null;
        libConfig.tag.smartId = null;
      }
    }
  }
}

defineExpose({
  clickAddTag,
});

</script>
