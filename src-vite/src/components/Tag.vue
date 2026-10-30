<template>
  <div class="sidebar-panel">
    <div class="sidebar-panel-header">
      <span class="sidebar-panel-header-title flex-1"
        >{{ $t("tag.title")
        }}<template v-if="tags.length">
          ({{ tags.length.toLocaleString() }})</template
        ></span
      >
      <SortMenuButton v-model="config.settings.tagSort" kind="category" />
      <TButton
        :icon="IconAdd"
        buttonSize="small"
        :tooltip="$t('msgbox.new_tag.title')"
        @click="clickAddTag()"
      />
    </div>
    <div v-if="tags.length > 0" class="mx-1 mb-2 px-1 shrink-0">
      <div
        :class="[
          'h-8 flex items-center rounded-box transition-colors bg-base-100/40',
          isSearchFocused
            ? 'border-2 border-primary'
            : 'border border-base-content/10 hover:border-base-content/30',
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
          @click.stop="
            search = '';
            searchInputRef?.focus();
          "
        >
          <IconClose class="w-4 h-4" />
        </button>
      </div>
    </div>

    <div ref="tagList" class="grow min-h-0 overflow-y-auto overflow-x-hidden">
      <ul role="tree">
        <li
          v-for="tag in rows"
          :key="tag.id"
          role="treeitem"
          :aria-level="tag.depth + 1"
          :aria-expanded="tag.hasChildren ? expanded(tag.id) : undefined"
          :aria-selected="libConfig.tag.id === tag.id"
        >
          <div
            :id="`tag-${tag.id}`"
            class="sidebar-item group select-none"
            :class="{
              'text-primary': libConfig.tag.id === tag.id,
              'opacity-50': dragged === tag.id,
              'bg-primary/10 ring-1 ring-inset ring-primary/40':
                dropTarget === tag.id,
            }"
            :style="{ paddingLeft: `${4 + tag.depth * 16}px` }"
            :data-tag-drop-id="tag.id"
            tabindex="0"
            @keydown.enter.stop.prevent="selectTag(tag)"
            @keydown.space.stop.prevent="selectTag(tag)"
            @pointerdown="startDrag($event, tag)"
            @dragstart.prevent
            @click="selectTag(tag)"
            @dblclick.stop="edit(tag)"
          >
            <button
              v-if="tag.hasChildren"
              class="size-6 shrink-0 flex items-center justify-center"
              :aria-label="tag.name"
              @click.stop="toggle(tag.id)"
            >
              <IconRight
                class="size-6 p-1 transition-transform"
                :class="{ 'rotate-90': expanded(tag.id) }"
              />
            </button>
            <span v-else class="size-6 shrink-0" aria-hidden="true" />
            <IconTag class="w-5 h-5 pr-1 shrink-0" />
            <TagNameInput
              v-if="editor && tag.id === (editor.id ?? -1)"
              :key="tag.id"
              :initial-name="editor.name"
              :saving="saving"
              :error="error"
              @save="saveEditor"
              @cancel="cancelEditor"
              @clear-error="error = ''"
            />
            <span v-else class="sidebar-item-label">{{
              search ? tag.path : tag.name
            }}</span>
            <span v-if="Number(tag.count) > 0" class="sidebar-item-count">{{
              Number(tag.count).toLocaleString()
            }}</span>
            <div
              v-if="!editor"
              class="ml-auto hidden group-hover:block"
              @click.stop
            >
              <ContextMenu
                :iconMenu="IconMore"
                :menuItems="() => tagMenu(tag)"
                :smallIcon="true"
              />
            </div>
          </div>
        </li>
      </ul>
      <div
        v-if="dragged !== null"
        data-tag-drop-root
        class="sticky bottom-0 sidebar-item border border-dashed border-primary/40 bg-base-200 text-primary"
        :class="{ 'ring-1 ring-inset ring-primary': dropTarget === null }"
      >
        {{ $t("tag.root") }}
      </div>
      <div v-if="!loading && !rows.length" class="sidebar-empty text-sm">
        {{ $t(failed ? "tag.load_failed" : "tag.not_found") }}
      </div>
    </div>
  </div>
  <MessageBox
    v-if="deleteTarget"
    :title="$t('msgbox.delete_tag.title', { tag: deleteTarget.name })"
    :message="$t('tag.delete_tree')"
    :warningOk="true"
    :isLoading="saving"
    :OkText="$t('msgbox.delete_tag.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="confirmDelete"
    @cancel="deleteTarget = null"
  />
</template>

<script setup lang="ts">
import {
  ref,
  computed,
  watch,
  nextTick,
  onMounted,
  onBeforeUnmount,
} from "vue";
import { emit as emitEvent, listen } from "@tauri-apps/api/event";
import { useI18n } from "vue-i18n";
import { useToast } from "@/common/toast";
import { useUIStore } from "@/stores/uiStore";
import { config, libConfig } from "@/common/config";
import { SIDEBAR } from "@/common/constants";
import {
  getAllTags,
  getTagCounts,
  createTag,
  renameTag,
  deleteTag,
  moveTag,
  getTagSubtreeInfo,
} from "@/common/api";
import {
  tagRows,
  ancestorIds,
  subtreeIds,
  isValidTagName,
  type LibraryTag,
} from "@/common/tagTree";
import {
  IconAdd,
  IconClose,
  IconMore,
  IconSearch,
  IconTag,
  IconTagAdd,
  IconRight,
  IconRename,
  IconTrash,
} from "@/common/icons";
import ContextMenu from "./ContextMenu.vue";
import TButton from "./TButton.vue";
import MessageBox from "./MessageBox.vue";
import TagNameInput from "./TagNameInput.vue";
import SortMenuButton from "./SortMenuButton.vue";

defineProps<{ titlebar: string }>();
defineEmits(["editDataChanged"]);
const { t } = useI18n();
const toast = useToast();
const uiStore = useUIStore();
const tagsChangeSource = crypto.randomUUID();
const tags = ref<LibraryTag[]>([]);
const search = ref("");
const searchInputRef = ref<HTMLInputElement | null>(null);
const isSearchFocused = ref(false);
const loading = ref(true);
const failed = ref(false);
const saving = ref(false);
const error = ref("");
const editor = ref<{
  id: number | null;
  name: string;
  parent: number | null;
} | null>(null);
const deleteTarget = ref<
  (LibraryTag & { tag_count: number; file_count: number }) | null
>(null);
const dragged = ref<number | null>(null);
const tagList = ref<HTMLElement | null>(null);
const dropTarget = ref<number | null | undefined>(undefined);
const rows = computed(() =>
  tagRows(
    editor.value?.id === null
      ? [
          ...tags.value,
          {
            id: -1,
            name: editor.value.name,
            parent_id: editor.value.parent,
            path: editor.value.name,
          },
        ]
      : tags.value,
    search.value,
    libConfig.tag.collapsedTagIds || [],
    config.settings.tagSort,
    -1,
  ),
);
const expanded = (id: number) =>
  !!search.value || !(libConfig.tag.collapsedTagIds || []).includes(id);
function toggle(id: number) {
  if (search.value) return;
  const closed = libConfig.tag.collapsedTagIds || [];
  libConfig.tag.collapsedTagIds = closed.includes(id)
    ? closed.filter((value: number) => value !== id)
    : [...closed, id];
}
function selectTag(tag: LibraryTag) {
  if (suppressDragClick) return;
  libConfig.tag.id = tag.id;
}
function reveal(id: number) {
  const ancestors = new Set(ancestorIds(tags.value, id));
  libConfig.tag.collapsedTagIds = (libConfig.tag.collapsedTagIds || []).filter(
    (value: number) => !ancestors.has(value),
  );
}
function cancelEditor() {
  if (!saving.value) editor.value = null;
}
function clickAddTag(parent: number | null = null) {
  if (saving.value || editor.value) return;
  error.value = "";
  search.value = "";
  const base = t("msgbox.new_tag.title");
  let name = base;
  let index = 0;
  const names = new Set(
    tags.value
      .filter((tag) => tag.parent_id === parent)
      .map((tag) => tag.name.toLocaleLowerCase()),
  );
  while (names.has(name.toLocaleLowerCase())) name = `${base} ${++index}`;
  editor.value = { id: null, name, parent };
  if (parent !== null) {
    const open = new Set([parent, ...ancestorIds(tags.value, parent)]);
    libConfig.tag.collapsedTagIds = (
      libConfig.tag.collapsedTagIds || []
    ).filter((id: number) => !open.has(id));
  }
}
function edit(tag: LibraryTag) {
  if (saving.value || editor.value) return;
  error.value = "";
  search.value = "";
  editor.value = { id: tag.id, name: tag.name, parent: tag.parent_id };
  reveal(tag.id);
}
async function saveEditor(value: string) {
  const target = editor.value;
  if (!target || saving.value) return;
  const name = value.trim();
  if (!isValidTagName(name)) {
    error.value = t("tag.invalid_name");
    return;
  }
  if (
    target.id !== null &&
    tags.value.find((tag) => tag.id === target.id)?.name === name
  ) {
    editor.value = null;
    return;
  }
  if (
    tags.value.some(
      (tag) =>
        tag.id !== target.id &&
        tag.parent_id === target.parent &&
        tag.name.toLocaleLowerCase() === name.toLocaleLowerCase(),
    )
  ) {
    error.value = t("tag.name_exists");
    return;
  }
  saving.value = true;
  try {
    const result =
      target.id === null
        ? await createTag(name, target.parent)
        : await renameTag(target.id, name);
    if (!result) throw new Error("save failed");
    editor.value = null;
    search.value = "";
    await load();
    const id = target.id ?? result.id;
    reveal(id);
    libConfig.tag.id = id;
    await notifyChanged();
    await nextTick();
    document.getElementById(`tag-${id}`)?.scrollIntoView({ block: "nearest" });
  } catch {
    error.value = t("tag.name_save_failed");
  } finally {
    saving.value = false;
  }
}
function tagMenu(tag: LibraryTag): any[] {
  return [
    {
      label: t("tag.add_child"),
      icon: IconTagAdd,
      action: () => clickAddTag(tag.id),
    },
    { label: t("menu.tag.rename"), icon: IconRename, action: () => edit(tag) },
    { label: "-", action: null },
    {
      label: t("tag.delete_tag"),
      icon: IconTrash,
      action: () => prepareDelete(tag),
    },
  ];
}
let dragStart: {
  id: number;
  pointer: number;
  x: number;
  y: number;
  parent: number | null;
} | null = null;
let excludedDropIds = new Set<number>();
let suppressDragClick = false;
function startDrag(event: PointerEvent, tag: LibraryTag) {
  if (
    event.button !== 0 ||
    !event.isPrimary ||
    editor.value ||
    saving.value ||
    (event.target as Element).closest("button, input, [role=button]")
  )
    return;
  stopDrag();
  dragStart = {
    id: tag.id,
    pointer: event.pointerId,
    x: event.clientX,
    y: event.clientY,
    parent: tag.parent_id,
  };
  window.addEventListener("pointermove", dragMove);
  window.addEventListener("pointerup", finishDrag);
  window.addEventListener("pointercancel", stopDrag);
  window.addEventListener("blur", stopDrag);
  window.addEventListener("keydown", cancelDrag);
}
function dragMove(event: PointerEvent) {
  if (!dragStart || event.pointerId !== dragStart.pointer) return;
  if (dragged.value === null) {
    if (
      Math.hypot(event.clientX - dragStart.x, event.clientY - dragStart.y) < 5
    )
      return;
    dragged.value = dragStart.id;
    excludedDropIds = subtreeIds(tags.value, dragStart.id);
    suppressDragClick = true;
    uiStore.pushInputHandler("TagDrag");
  }
  event.preventDefault();
  const element = document.elementFromPoint(event.clientX, event.clientY);
  dropTarget.value = undefined;
  if (!element || !tagList.value?.contains(element)) return;
  if (element.closest("[data-tag-drop-root]")) {
    if (dragStart.parent !== null) dropTarget.value = null;
    return;
  }
  const target = element.closest<HTMLElement>("[data-tag-drop-id]");
  if (!target) return;
  const id = Number(target.dataset.tagDropId);
  if (!excludedDropIds.has(id) && id !== dragStart.parent)
    dropTarget.value = id;
}
function finishDrag(event: PointerEvent) {
  if (!dragStart || event.pointerId !== dragStart.pointer) return;
  if (dragged.value !== null) dragMove(event);
  const id = dragged.value;
  const parent = dropTarget.value;
  stopDrag();
  if (id !== null && parent !== undefined) void move(id, parent);
}
function cancelDrag(event: KeyboardEvent) {
  if (event.key === "Escape") stopDrag();
}
function stopDrag() {
  dragStart = null;
  dragged.value = null;
  dropTarget.value = undefined;
  uiStore.removeInputHandler("TagDrag");
  window.removeEventListener("pointermove", dragMove);
  window.removeEventListener("pointerup", finishDrag);
  window.removeEventListener("pointercancel", stopDrag);
  window.removeEventListener("blur", stopDrag);
  window.removeEventListener("keydown", cancelDrag);
  if (suppressDragClick)
    setTimeout(() => {
      suppressDragClick = false;
    }, 0);
}
async function move(id: number, parent: number | null) {
  if (saving.value) return;
  if (parent !== null && subtreeIds(tags.value, id).has(parent)) {
    toast.error(t("tag.invalid_move"));
    return;
  }
  saving.value = true;
  try {
    await moveTag(id, parent);
    await load();
    reveal(id);
    await notifyChanged();
  } catch {
    toast.error(t("tag.move_failed"));
  } finally {
    saving.value = false;
  }
}
async function prepareDelete(tag: LibraryTag) {
  if (saving.value) return;
  const library = libConfig._libraryId;
  try {
    const info = await getTagSubtreeInfo(tag.id);
    if (!disposed && library === libConfig._libraryId && info.tag_count > 0)
      deleteTarget.value = { ...tag, ...info };
  } catch {
    toast.error(t("tag.load_failed"));
  }
}
async function confirmDelete() {
  if (!deleteTarget.value || saving.value) return;
  saving.value = true;
  try {
    if (!(await deleteTag(deleteTarget.value.id)))
      throw new Error("delete failed");
    deleteTarget.value = null;
    await load();
    await notifyChanged();
  } catch {
    toast.error(t("tag.delete_failed"));
  } finally {
    saving.value = false;
  }
}
async function notifyChanged() {
  await emitEvent("tags-changed", { source: tagsChangeSource });
  if (
    config.main.sidebarIndex === SIDEBAR.TAG &&
    libConfig.activePane === "main"
  )
    libConfig.tag.activateTick++;
}
let request = 0;
let disposed = false;
let unlisten: (() => void) | undefined;
async function load() {
  const current = ++request;
  const library = libConfig._libraryId;
  try {
    const [all, counts] = await Promise.all([getAllTags(0), getTagCounts()]);
    if (disposed || current !== request || library !== libConfig._libraryId)
      return;
    if (!all || !counts) throw new Error("load failed");
    tags.value = all.map((tag: LibraryTag) => ({
      ...tag,
      count: Number(counts[String(tag.id)] || 0),
    }));
    libConfig.tag.counts = counts;
    if (!tags.value.some((tag) => tag.id === libConfig.tag.id))
      libConfig.tag.id = tags.value[0]?.id ?? null;
    failed.value = false;
  } catch {
    if (current === request) failed.value = true;
  } finally {
    if (current === request) loading.value = false;
  }
}
onMounted(async () => {
  const stop = await listen<{ source?: string }>(
    "tags-changed",
    async (event) => {
      if (event.payload?.source === tagsChangeSource) return;
      await load();
      if (!disposed && config.main.sidebarIndex === SIDEBAR.TAG)
        libConfig.tag.activateTick++;
    },
  );
  if (disposed) stop();
  else unlisten = stop;
  await load();
  if (libConfig.tag.id != null) reveal(libConfig.tag.id);
});
onBeforeUnmount(() => {
  stopDrag();
  disposed = true;
  request++;
  unlisten?.();
});
watch(
  () => libConfig._libraryId,
  () => {
    editor.value = null;
    deleteTarget.value = null;
    stopDrag();
    void load();
  },
);
watch(
  () => [config.main.sidebarIndex, libConfig.activePane],
  () => {
    if (config.main.sidebarIndex === SIDEBAR.TAG) void load();
  },
);
defineExpose({ clickAddTag });
</script>
