<template>
  <div class="sidebar-panel">
    <div class="sidebar-panel-header">
      <span class="sidebar-panel-header-title flex-1"
        >{{ $t("tag.title")
        }}<template v-if="displayedTagCount">
          ({{ displayedTagCount.toLocaleString() }})</template
        ></span
      >
      <SortMenuButton v-model="config.settings.tagSort" kind="category" />
      <TButton
        :icon="IconAdd"
        buttonSize="small"
        :tooltip="$t('menu.tag.new_group')"
        @click="editGroup()"
      />
    </div>
    <div class="mx-1 mb-2 px-1 shrink-0">
      <div
        :class="[
          'h-8 flex items-center rounded-box transition-colors bg-base-100/40',
          isSearchFocused ? 'border-2 border-primary' : 'border border-base-content/10 hover:border-base-content/30',
        ]"
        @click="searchInputRef?.focus()"
      >
        <IconSearch
          class="ml-2 w-4 h-4 shrink-0"
          :class="isSearchFocused ? 'text-primary/70' : 'text-base-content/30'"
        />
        <input
          ref="searchInputRef"
          type="text"
          v-model="search"
          :placeholder="$t('menu.tag.search')"
          class="w-full min-w-0 bg-transparent border-none focus:ring-0 px-2 text-sm placeholder-base-content/30 focus:outline-none"
          maxlength="255"
          @focus="isSearchFocused = true"
          @blur="isSearchFocused = false"
        />
        <button
          v-if="search"
          type="button"
          :title="$t('menu.tag.clear_search')"
          class="mr-1 p-1 rounded-box text-base-content/30 hover:text-base-content/70"
          @click.stop="search = ''; searchInputRef?.focus()"
        >
          <IconClose class="w-4 h-4" />
        </button>
      </div>
    </div>
    <div class="grow overflow-y-auto overflow-x-hidden">
      <VueDraggable
        v-model="groups"
        tag="ul"
        :animation="200"
        handle=".tag-group-drag-handle"
        :disabled="
          Boolean(search.trim()) ||
          renaming !== null ||
          reorderingGroupId === null ||
          savingOrder
        "
        :onMove="canMoveGroup"
        @start="onReorderStart"
        @end="onReorderEnd"
        @drop.stop
      >
        <li
          v-for="group in visibleGroups"
          :key="group.id"
          :id="`tag-group-${group.id}`"
          :data-default-group="group.is_default ? 'true' : undefined"
          :data-reordering-group="
            reorderingGroupId === group.id ? 'true' : undefined
          "
        >
          <div
            class="sidebar-item group"
            :class="
              libConfig.tag.groupId === group.id && !isRenaming('group', group.id)
                ? 'sidebar-item-selected'
                : 'sidebar-item-hover'
            "
            @click="selectGroup(group)"
            @contextmenu.prevent.stop="
              (event: MouseEvent) =>
                groupMenus[group.id]?.open(event.clientX, event.clientY)
            "
          >
            <IconDragHandle
              v-if="reorderingGroupId === group.id"
              class="tag-group-drag-handle p-1 w-6 h-6 shrink-0 cursor-move text-base-content/70 hover:text-base-content"
              :title="$t('collection.reorder')"
            />
            <span
              v-else-if="reorderingGroupId !== null"
              class="p-1 w-6 h-6 shrink-0"
            />
            <button
              :aria-label="group.name"
              :aria-expanded="expanded(group.id)"
              @click.stop="toggleGroup(group.id)"
            >
              <IconRight
                class="p-1 w-6 h-6 shrink-0 transition-transform"
                :class="expanded(group.id) ? 'rotate-90' : ''"
              />
            </button>
            <input
                  v-if="isRenaming('group', group.id)"
                  :ref="(el: any) => { if (el) renameInput = el; }"
                  v-model="renaming!.name"
                  type="text"
                  maxlength="255"
                  :aria-label="$t('menu.tag.rename')"
                  :placeholder="$t('menu.tag.new_group')"
                  class="input px-1 w-full min-w-0 text-base"
                  :readonly="savingRename"
                  @click.stop
                  @mousedown.stop
                  @keydown.stop
                  @keydown.enter.prevent="!$event.isComposing && saveRename()"
                  @keydown.esc.prevent="cancelRename"
                  @blur="saveRename(true)"
                />
                <span v-else class="sidebar-item-label">{{ group.name }}</span>
            <span
              v-if="!isRenaming('group', group.id) && !expanded(group.id) && selectedTag?.group_id === group.id"
              class="text-primary text-xs"
              :title="$t('menu.tag.active_tag')"
              >•</span
            >
            <div
              v-if="!isRenaming('group', group.id)"
                  class="ml-auto flex flex-row items-center text-base-content/30"
            >
              <span
                v-if="group.count > 0"
                class="sidebar-item-count shrink-0"
                >{{ group.count.toLocaleString() }}</span
              >
              <div
                :class="
                  libConfig.tag.groupId === group.id
                    ? ''
                    : 'hidden group-hover:block'
                "
                @click.stop
              >
                <ContextMenu
                  :ref="
                    (el: any) => {
                      if (el) groupMenus[group.id] = el;
                    }
                  "
                  :iconMenu="IconMore"
                  :menuItems="() => groupMenu(group)"
                  :smallIcon="true"
                />
              </div>
            </div>
          </div>
          <ul v-if="expanded(group.id)">
            <li
              v-for="tag in group.tags"
              :key="tag.id"
              :id="`tag-${tag.id}`"
              class="pl-4"
            >
              <div
                class="sidebar-item sidebar-item-compact ml-2 group"
                :class="
                  libConfig.tag.id === tag.id && !isRenaming('tag', tag.id)
                    ? 'sidebar-item-selected'
                    : 'sidebar-item-hover'
                "
                @click="selectTag(tag)"
                @contextmenu.prevent.stop="
                  (event: MouseEvent) =>
                    tagMenus[tag.id]?.open(event.clientX, event.clientY)
                "
              >
                <IconTag class="mx-1 w-4 h-4 shrink-0" />
                <input
                  v-if="isRenaming('tag', tag.id)"
                  :ref="(el: any) => { if (el) renameInput = el; }"
                  v-model="renaming!.name"
                  type="text"
                  maxlength="255"
                  :aria-label="$t('menu.tag.rename')"
                  :placeholder="$t('msgbox.new_tag.title')"
                  class="input px-1 w-full min-w-0 text-base"
                  :readonly="savingRename"
                  @click.stop
                  @mousedown.stop
                  @keydown.stop
                  @keydown.enter.prevent="!$event.isComposing && saveRename()"
                  @keydown.esc.prevent="cancelRename"
                  @blur="saveRename(true)"
                />
                <span v-else class="sidebar-item-label">{{ tag.name }}</span>
                <div
                  v-if="!isRenaming('tag', tag.id)"
                  class="ml-auto flex flex-row items-center text-base-content/30"
                >
                  <span
                    v-if="getTagCount(tag) > 0"
                    class="sidebar-item-count shrink-0"
                    >{{ getTagCount(tag).toLocaleString() }}</span
                  >
                  <div
                    :class="
                      libConfig.tag.id === tag.id
                        ? ''
                        : 'hidden group-hover:block'
                    "
                    @click.stop
                  >
                    <ContextMenu
                      :ref="
                        (el: any) => {
                          if (el) tagMenus[tag.id] = el;
                        }
                      "
                      :iconMenu="IconMore"
                      :menuItems="() => tagMenu(tag)"
                      :smallIcon="true"
                    />
                  </div>
                </div>
              </div>
            </li>
            <li
              v-if="!group.tags.length"
              class="pl-10 py-2 text-xs text-base-content/40"
            >
              {{ $t("menu.tag.empty") }}
            </li>
          </ul>
        </li>
      </VueDraggable>
      <div
        v-if="!loading && !visibleGroups.length"
        class="sidebar-empty text-sm"
      >
        {{ failed ? $t("tag.load_failed") : $t("tag.not_found") }}
      </div>
    </div>
  </div>
  <MessageBox
    v-if="deleteTarget"
    :title="
      $t(
        deleteTarget.kind === 'group'
          ? 'menu.tag.delete_group'
          : 'msgbox.delete_tag.title',
      )
    "
    :message="
      deleteTarget.kind === 'group'
        ? $t('menu.tag.delete_message', { name: deleteTarget.name })
        : $t('msgbox.delete_tag.content', { tag: deleteTarget.name })
    "
    :OkText="$t('msgbox.delete_tag.ok')"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    @ok="confirmDelete"
    @cancel="deleteTarget = null"
  />
</template>

<script setup lang="ts">
import {
  ref,
  computed,
  nextTick,
  watch,
  onMounted,
  onBeforeUnmount,
} from "vue";
import { VueDraggable } from "vue-draggable-plus";
import { emit as emitEvent, listen } from "@tauri-apps/api/event";
import { useI18n } from "vue-i18n";
import { useToast } from "@/common/toast";
import { config, libConfig } from "@/common/config";
import { useUIStore } from "@/stores/uiStore";
import { SIDEBAR } from "@/common/constants";
import {
  getAllTags,
  getTagCounts,
  getTagGroups,
  createTag,
  renameTag,
  deleteTag,
  saveTagGroup,
  reorderTagGroups,
  deleteTagGroup,
  moveTagsToGroup,
} from "@/common/api";
import { groupTags, type TagGroup, type LibraryTag } from "@/common/tagGroups";
import {
  IconAdd,
  IconClose,
  IconMore,
  IconSearch,
  IconTag,
  IconTagAdd,
  IconRight,
  IconRename,
  IconOrder,
  IconTrash,
  IconDragHandle,
  IconFolderMoveTo,
} from "@/common/icons";
import ContextMenu from "./ContextMenu.vue";
import TButton from "./TButton.vue";
import MessageBox from "./MessageBox.vue";
import SortMenuButton from "./SortMenuButton.vue";

defineProps<{ titlebar: string }>();
defineEmits(["editDataChanged"]);
const { t } = useI18n();
const toast = useToast();
const uiStore = useUIStore();
const tags = ref<LibraryTag[]>([]);
const groups = ref<TagGroup[]>([]);
const search = ref("");
const searchInputRef = ref<HTMLInputElement | null>(null);
const isSearchFocused = ref(false);
const loading = ref(true);
const failed = ref(false);
const saving = ref(false);
const reorderingGroupId = ref<number | null>(null);
const savingOrder = ref(false);
let draggingGroup = false;
const getTagCount = (tag: LibraryTag) =>
  Number(libConfig.tag.counts?.[String(tag.id)] || 0);
const tagMenus = ref<Record<number, any>>({});
const groupMenus = ref<Record<number, any>>({});
// Tracks a placeholder tag/group created by the "+" button that has not been
// renamed yet. If the inline editor is dismissed without a commit, the
// placeholder is deleted so no orphan row is left behind.
const pendingCreate = ref<{ kind: "group" | "tag"; id: number } | null>(null);
const renameInput = ref<HTMLInputElement>();
const renaming = ref<{ kind: "group" | "tag"; id: number; name: string; originalName: string } | null>(null);
const savingRename = ref(false);
const isRenaming = (kind: "group" | "tag", id: number) =>
  renaming.value?.kind === kind && renaming.value.id === id;
async function startRename(kind: "group" | "tag", item: TagGroup | LibraryTag, initialName?: string) {
  if (savingRename.value) return;
  reorderingGroupId.value = null;
  // Callers creating a fresh placeholder pass initialName="" so the inline
  // input starts empty instead of showing the auto-generated default name;
  // originalName matches, so pressing Enter with no input cancels cleanly.
  const name = initialName ?? item.name;
  renaming.value = { kind, id: item.id, name, originalName: name };
  uiStore.removeInputHandler("TagRename");
  uiStore.pushInputHandler("TagRename");
  await nextTick();
  renameInput.value?.focus();
}
async function cancelRename() {
  if (savingRename.value) return;
  // Capture pending placeholder before finishRename clears it, so we can
  // delete the orphan row created by the "+" button.
  const pending = pendingCreate.value;
  const target = renaming.value;
  const wasPendingCreate =
    pending !== null &&
    target !== null &&
    pending.kind === target.kind &&
    pending.id === target.id;
  finishRename();
  if (wasPendingCreate && pending) {
    try {
      if (pending.kind === "group") await deleteTagGroup(pending.id);
      else await deleteTag(pending.id);
      await load();
    } catch {
      // best-effort cleanup; ignore errors
    }
  }
}
function finishRename() {
  renaming.value = null;
  renameInput.value = undefined;
  pendingCreate.value = null;
  uiStore.removeInputHandler("TagRename");
}
async function saveRename(fromBlur = false) {
  const target = renaming.value;
  if (!target || savingRename.value) return;
  const name = target.name.trim();
  if (!name) {
    // Empty input:
    // - Enter: keep the input open so the user can keep typing or press Esc
    //   explicitly. Prevents accidentally discarding a fresh "+" placeholder
    //   with a stray Enter.
    // - Blur: treat as cancel (deletes the pending placeholder, or closes a
    //   regular rename with no changes to save).
    if (fromBlur) cancelRename();
    return;
  }
  if (name === target.originalName) {
    cancelRename();
    return;
  }
  const items = target.kind === "group" ? groups.value : tags.value;
  if (items.some(item => item.id !== target.id && item.name.toLocaleLowerCase() === name.toLocaleLowerCase())) {
    toast.error(t(target.kind === "tag" ? "tag.name_exists" : "menu.tag.name_exists"));
    await nextTick();
    if (renaming.value === target) renameInput.value?.focus();
    return;
  }
  savingRename.value = true;
  try {
    if (target.kind === "group") await saveTagGroup(target.id, name);
    else if (!(await renameTag(target.id, name))) throw new Error("rename failed");
    if (renaming.value === target) {
      finishRename();
    }
    await load();
    await changed();
  } catch {
    toast.error(t("tag.name_save_failed"));
    if (renaming.value === target) {
      await nextTick();
      if (renaming.value === target) renameInput.value?.focus();
    }
  } finally {
    savingRename.value = false;
  }
}
const deleteTarget = ref<{
  kind: "group" | "tag";
  id: number;
  name: string;
} | null>(null);
const selectedTag = computed(() =>
  tags.value.find((tag) => tag.id === libConfig.tag.id),
);
const visibleGroups = computed(() =>
  groupTags(groups.value, tags.value, search.value),
);
// Header count reflects only committed tags. A "+" click immediately creates
// a placeholder row in the DB so the inline input has something to bind to,
// but from the user's perspective the tag isn't real until they type a name
// and press Enter — so hide the placeholder from the count until then.
// Group placeholders don't affect this count (title counts tags only).
const displayedTagCount = computed(() => {
  // While searching, count the tags actually shown (groupTags already filters).
  const q = search.value.trim();
  if (q) return visibleGroups.value.reduce((sum, g) => sum + (g.tags?.length || 0), 0);
  const pending = pendingCreate.value;
  const n = tags.value.length;
  return pending && pending.kind === "tag" ? Math.max(0, n - 1) : n;
});
const expanded = (id: number) =>
  !!search.value.trim() ||
  !(libConfig.tag.collapsedGroupIds || []).includes(id);
watch(
  () => {
    const target = renaming.value;
    if (!target) return true;
    return visibleGroups.value.some((group) =>
      target.kind === "group"
        ? group.id === target.id
        : expanded(group.id) && group.tags.some((tag) => tag.id === target.id),
    );
  },
  (visible) => {
    // A pending save can finish even after its input has disappeared.
    if (!visible) finishRename();
  },
);
function expand(id: number) {
  libConfig.tag.collapsedGroupIds = (
    libConfig.tag.collapsedGroupIds || []
  ).filter((value: number) => value !== id);
}
function toggleGroup(id: number) {
  if (search.value.trim()) return;
  libConfig.tag.collapsedGroupIds = expanded(id)
    ? [...(libConfig.tag.collapsedGroupIds || []), id]
    : (libConfig.tag.collapsedGroupIds || []).filter(
        (value: number) => value !== id,
      );
}
function selectGroup(group: TagGroup) {
  libConfig.tag.id = null;
  libConfig.tag.groupId = group.id;
}
function selectTag(tag: LibraryTag) {
  libConfig.tag.groupId = null;
  libConfig.tag.id = tag.id;
}
async function locate(tag: LibraryTag) {
  search.value = "";
  expand(tag.group_id);
  selectTag(tag);
  await nextTick();
  document
    .getElementById(`tag-${tag.id}`)
    ?.scrollIntoView({ block: "nearest" });
}
async function editGroup() {
  if (saving.value || renaming.value) return;
  saving.value = true;
  try {
    const name = t("menu.tag.new_group");
    const id = await saveTagGroup(null, name);
    if (id == null) {
      toast.error(t("tag.name_save_failed"));
      return;
    }
    expand(id);
    search.value = "";
    await load();
    const created = groups.value.find((g) => g.id === id);
    if (!created) return;
    selectGroup(created);
    pendingCreate.value = { kind: "group", id };
    await startRename("group", created, "");
  } catch {
    toast.error(t("tag.name_save_failed"));
  } finally {
    saving.value = false;
  }
}
async function clickAddTag(groupId?: number) {
  if (saving.value || renaming.value) return;
  const gid = groupId ?? groups.value.find((g) => g.is_default)?.id;
  if (gid == null) return;
  saving.value = true;
  try {
    await load();
    const names = new Set(tags.value.map(tag => tag.name.toLocaleLowerCase()));
    const baseName = t("msgbox.new_tag.title");
    let name = baseName;
    let index = 0;
    while (names.has(name.toLocaleLowerCase())) name = `${baseName} ${++index}`;
    const result = await createTag(name, gid);
    if (!result) {
      toast.error(t("tag.name_save_failed"));
      return;
    }
    await load();
    const created = tags.value.find((tag) => tag.id === result.id);
    if (!created) return;
    // Clear search so the new placeholder is visible; otherwise the
    // renaming-visibility watch would immediately finishRename() and skip
    // the pending-delete cleanup, leaking the placeholder row.
    search.value = "";
    expand(created.group_id);
    pendingCreate.value = { kind: "tag", id: Number(result.id) };
    await startRename("tag", created, "");
  } catch {
    toast.error(t("tag.name_save_failed"));
  } finally {
    saving.value = false;
  }
}
function groupMenu(group: TagGroup) {
  return [
    {
      label: t("msgbox.new_tag.title"),
      icon: IconTagAdd,
      action: () => clickAddTag(group.id),
    },
    { label: "-", action: null },
    {
      label: t("menu.tag.rename"),
      icon: IconRename,
      disabled: group.is_default,
      action: () => {
        if (!group.is_default) startRename("group", group);
      },
    },
    {
      label: t("collection.reorder"),
      icon: IconOrder,
      disabled: group.is_default || savingOrder.value,
      action: () => {
        if (group.is_default || savingOrder.value) return;
        search.value = "";
        reorderingGroupId.value =
          reorderingGroupId.value === group.id ? null : group.id;
      },
    },
    { label: "-", action: null },
    {
      label: t("menu.tag.delete"),
      icon: IconTrash,
      disabled: group.is_default,
      action: () => {
        if (!group.is_default) deleteTarget.value = { kind: "group", ...group };
      },
    },
  ];
}
function canMoveGroup(event: {
  related: HTMLElement;
  willInsertAfter: boolean;
}) {
  return (
    event.related?.dataset.defaultGroup !== "true" || event.willInsertAfter
  );
}
function handleReorderOutside(event: PointerEvent) {
  if (event.button !== 0 || draggingGroup) return;
  if (
    event.target instanceof Element &&
    event.target.closest('[data-reordering-group="true"]')
  )
    return;
  reorderingGroupId.value = null;
}
function onReorderStart() {
  draggingGroup = true;
  request++;
  uiStore.pushInputHandler("TagGroupDrag");
}
async function onReorderEnd() {
  draggingGroup = false;
  uiStore.removeInputHandler("TagGroupDrag");
  savingOrder.value = true;
  try {
    await reorderTagGroups(groups.value.map((group) => group.id));
    await changed();
  } catch {
    toast.error(t("tag.name_save_failed"));
  } finally {
    savingOrder.value = false;
    await load();
  }
}
function tagMenu(tag: LibraryTag) {
  return [
    {
      label: t("menu.tag.rename"),
      icon: IconRename,
      action: () => {
        startRename("tag", tag);
      },
    },
    {
      label: t("menu.tag.move_to"),
      children: groups.value
        .filter((g) => g.id !== tag.group_id)
        .map((group) => ({
          label: group.name,
          action: () => moveTag(tag, group.id),
        })),
    },
    { label: "-", action: null },
    {
      label: t("tag.delete_tag"),
      icon: IconTrash,
      action: () => {
        deleteTarget.value = { kind: "tag", ...tag };
      },
    },
  ];
}
async function changed() {
  await emitEvent("tags-changed");
}
async function moveTag(tag: LibraryTag, groupId: number) {
  try {
    await moveTagsToGroup([tag.id], groupId);
    await load();
    if (libConfig.tag.id === tag.id)
      await locate({ ...tag, group_id: groupId });
    await changed();
  } catch {
    toast.error(t("tag.name_save_failed"));
  }
}
async function confirmDelete() {
  const target = deleteTarget.value;
  if (!target || saving.value) return;
  saving.value = true;
  try {
    if (target.kind === "group") await deleteTagGroup(target.id);
    else if (!(await deleteTag(target.id))) throw new Error("delete failed");
    deleteTarget.value = null;
    await load();
    await changed();
  } catch {
    toast.error(t("tag.name_save_failed"));
  } finally {
    saving.value = false;
  }
}
let request = 0;
let disposed = false;
let unlisten: (() => void) | undefined;
async function load() {
  if (draggingGroup || savingOrder.value) return;
  const current = ++request;
  const library = libConfig._libraryId;
  try {
    const [allTags, counts, allGroups] = await Promise.all([
      getAllTags(config.settings.tagSort),
      getTagCounts(),
      getTagGroups(),
    ]);
    if (disposed || current !== request || library !== libConfig._libraryId)
      return;
    if (!allTags || !counts) throw new Error("load failed");
    const previousGroupId = selectedTag.value?.group_id;
    tags.value = allTags;
    groups.value = allGroups;
    if (
      selectedTag.value &&
      previousGroupId &&
      selectedTag.value.group_id !== previousGroupId
    ) {
      expand(selectedTag.value.group_id);
      const id = selectedTag.value.id;
      void nextTick(() =>
        document
          .getElementById(`tag-${id}`)
          ?.scrollIntoView({ block: "nearest" }),
      );
    }
    libConfig.tag.counts = counts;
    failed.value = false;
    const validTag = tags.value.some((tag) => tag.id === libConfig.tag.id);
    const validGroup = groups.value.some(
      (group) => group.id === libConfig.tag.groupId,
    );
    if (!validTag && !validGroup) {
      const fallback = groups.value.find((g) => g.is_default);
      if (fallback) selectGroup(fallback);
    }
    libConfig.tag.collapsedGroupIds = (
      libConfig.tag.collapsedGroupIds || []
    ).filter((id: number) => groups.value.some((g) => g.id === id));
  } catch {
    if (current === request) failed.value = true;
  } finally {
    if (current === request) loading.value = false;
  }
}
onMounted(async () => {
  document.addEventListener("pointerdown", handleReorderOutside, true);
  const stop = await listen("tags-changed", async () => {
    await load();
    if (
      !disposed &&
      config.main.sidebarIndex === SIDEBAR.TAG &&
      libConfig.activePane === "main"
    )
      libConfig.tag.activateTick++;
  });
  if (disposed) stop();
  else unlisten = stop;
  await load();
  if (selectedTag.value) expand(selectedTag.value.group_id);
});
onBeforeUnmount(() => {
  document.removeEventListener("pointerdown", handleReorderOutside, true);
  uiStore.removeInputHandler("TagGroupDrag");
  uiStore.removeInputHandler("TagRename");
  disposed = true;
  request++;
  unlisten?.();
  // Best-effort cleanup: if a "+" placeholder was in flight when the panel
  // unmounts (e.g. user switched sidebar), delete the orphan row so it does
  // not linger in the library. Fire-and-forget — the component is going away
  // and there is nothing to update on completion.
  const pending = pendingCreate.value;
  if (pending) {
    pendingCreate.value = null;
    void (async () => {
      try {
        if (pending.kind === "group") await deleteTagGroup(pending.id);
        else await deleteTag(pending.id);
      } catch {
        // ignore — the panel is already gone, no UI to notify
      }
    })();
  }
});
watch(() => config.settings.groupRawJpegPairs, () => {
  if (config.main.sidebarIndex === SIDEBAR.TAG && libConfig.activePane === "main") void load();
});
watch(
  () => [config.main.sidebarIndex, libConfig.activePane],
  () => {
    if (
      config.main.sidebarIndex === SIDEBAR.TAG &&
      libConfig.activePane === "main"
    )
      void load();
  },
);
watch(
  () => [
    config.settings.tagSort,
    libConfig._libraryId,
  ],
  () => {
    // Library / sort / filter switched underneath us. Drop any in-flight
    // inline editor state; the placeholder row (if any) stays in the old
    // library — we deliberately do NOT delete here because the API call
    // would run against the new library context and could hit an unrelated
    // row that happens to share the id.
    renaming.value = null;
    pendingCreate.value = null;
    uiStore.removeInputHandler("TagRename");
    reorderingGroupId.value = null;
    deleteTarget.value = null;
    void load();
  },
);
defineExpose({ clickAddTag });
</script>
