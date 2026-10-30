<template>
  <ModalDialog
    :title="
      $t('tag.edit_tag') +
      (allTags.length ? ` (${allTags.length.toLocaleString()})` : '')
    "
    :width="500"
    :position-key="positionKey"
    @cancel="clickCancel"
  >
    <template #title-actions
      ><button
        class="t-button-default btn-outline gap-1.5"
        :disabled="!canEditTags"
        @click="edit()"
      >
        <IconAdd class="size-4" />{{ $t("msgbox.new_tag.title") }}
      </button></template
    >
    <section class="space-y-3">
      <div
        class="flex items-center rounded-box border border-base-content/20 bg-base-100"
      >
        <IconSearch class="ml-2 size-4 text-base-content/40" />
        <input
          ref="tagSearchInputRef"
          v-model="tagSearch"
          class="w-full px-2 py-1 bg-transparent text-sm focus:outline-none"
          :placeholder="$t('menu.tag.search')"
        />
        <button
          v-if="tagSearch"
          class="p-1 text-base-content/40"
          @click="tagSearch = ''"
        >
          <IconClose class="size-4" />
        </button>
      </div>
      <div
        class="min-h-48 max-h-[50vh] overflow-y-auto rounded-box p-2 bg-base-100/30 border border-base-content/10"
        role="tree"
      >
        <div
          v-for="(tag, index) in filteredTags"
          :key="tag.id"
          :id="`dialog-tag-${tag.id}`"
          role="treeitem"
          :aria-level="tag.depth + 1"
          :aria-expanded="
            tag.hasChildren ? !collapsed.includes(tag.id) : undefined
          "
          class="group flex items-center py-2 pr-1 rounded-box hover:bg-base-content/5 cursor-pointer"
          :class="{
            'text-primary': selectedTags.has(tag.id),
            'ring-2 ring-primary': focusedTagIndex === index,
          }"
          :style="{ paddingLeft: `${4 + tag.depth * 16}px` }"
          @click="toggleTag(tag.id)"
        >
          <button
            v-if="tag.hasChildren"
            class="size-5 shrink-0 flex items-center justify-center"
            @click.stop="toggleBranch(tag.id)"
          >
            <IconRight
              class="size-5 p-0.5"
              :class="{ 'rotate-90': !collapsed.includes(tag.id) || tagSearch }"
            />
          </button>
          <span v-else class="size-5 shrink-0" aria-hidden="true" />
          <input
            type="checkbox"
            class="checkbox checkbox-xs mx-1"
            :class="{ 'checkbox-primary': selectedTags.has(tag.id) || intermediateTags.has(tag.id) }"
            :disabled="!canEditTags || !!editor"
            :checked="selectedTags.has(tag.id)"
            :indeterminate="intermediateTags.has(tag.id)"
            @click.stop
            @change="toggleTag(tag.id)"
          />
          <IconTag class="size-5 p-0.5 shrink-0" />
          <TagNameInput
            v-if="editor && tag.id === (editor.id ?? -1)"
            :key="tag.id"
            :initial-name="editor.name"
            :saving="isCreatingTag"
            :error="error"
            @save="saveEditor"
            @cancel="cancelEditor"
            @clear-error="error = ''"
          />
          <span v-else class="min-w-0 flex-1 truncate">{{
            tagSearch ? tag.path : tag.name
          }}</span>
          <div
            v-if="!editor"
            class="hidden group-hover:flex shrink-0"
            @click.stop
          >
            <button
              class="p-1 text-base-content/40 hover:text-primary"
              :disabled="!canEditTags"
              :title="$t('tag.add_child')"
              @click="edit(undefined, tag.id)"
            >
              <IconTagAdd class="size-4" />
            </button>
            <button
              class="p-1 text-base-content/40 hover:text-base-content"
              :disabled="!canEditTags"
              :title="$t('menu.tag.rename')"
              @click="edit(tag)"
            >
              <IconRename class="size-4" />
            </button>
            <button
              class="p-1 text-base-content/40 hover:text-error"
              :disabled="!canEditTags"
              :title="$t('tag.delete_tag')"
              @click="prepareDelete(tag)"
            >
              <IconTrash class="size-4" />
            </button>
          </div>
        </div>
        <div
          v-if="!filteredTags.length && !isLoadingCatalog"
          class="text-center py-10 text-base-content/40"
        >
          {{ $t("tag.not_found") }}
        </div>
      </div>
      <p v-if="tagLoadFailed" class="text-sm text-error">
        {{ $t("tag.load_failed") }}
      </p>
    </section>
    <div class="mt-4 flex justify-end gap-4">
      <button
        class="t-button-default"
        :disabled="isApplyingTags"
        @click="clickCancel"
      >
        {{ $t("msgbox.cancel") }}
      </button>
      <button
        class="t-button-primary"
        :disabled="!canEditTags || editor !== null"
        @click="clickOk"
      >
        {{ $t("msgbox.ok") }}
      </button>
    </div>
  </ModalDialog>
  <MessageBox
    v-if="deleteTarget"
    :title="$t('msgbox.delete_tag.title', { tag: deleteTarget.name })"
    :message="$t('tag.delete_tree')"
    :warningOk="true"
    :isLoading="isCreatingTag"
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
import { emit as tauriEmit, listen } from "@tauri-apps/api/event";
import { useI18n } from "vue-i18n";
import { useToast } from "@/common/toast";
import {
  getAllTags,
  createTag,
  getTagSelectionCounts,
  applyTagsToFiles,
  deleteTag,
  renameTag,
  getTagSubtreeInfo,
} from "@/common/api";
import {
  IconAdd,
  IconTagAdd,
  IconClose,
  IconRename,
  IconSearch,
  IconTag,
  IconTrash,
  IconRight,
} from "@/common/icons";
import {
  tagRows,
  ancestorIds,
  isValidTagName,
  type LibraryTag,
} from "@/common/tagTree";
import { libConfig } from "@/common/config";
import { useUIStore } from "@/stores/uiStore";
import MessageBox from "./MessageBox.vue";
import TagNameInput from "./TagNameInput.vue";
import ModalDialog from "./ModalDialog.vue";

const props = defineProps({
  fileIds: { type: Array as () => number[], default: () => [] },
  positionKey: { type: String, default: "" },
});
const emit = defineEmits(["ok", "cancel", "states-changed"]);
const uiStore = useUIStore();
const toast = useToast();
const { t } = useI18n();
const allTags = ref<LibraryTag[]>([]);
const tagSearch = ref("");
const tagSearchInputRef = ref<HTMLInputElement | null>(null);
const collapsed = ref<number[]>([]);
const focusedTagIndex = ref(-1);
const isLoadingTags = ref(true);
const isLoadingCatalog = ref(true);
const isApplyingTags = ref(false);
const isCreatingTag = ref(false);
const catalogLoadFailed = ref(false);
const selectionLoadFailed = ref(false);
const tagLoadFailed = computed(
  () => catalogLoadFailed.value || selectionLoadFailed.value,
);
const canEditTags = computed(
  () =>
    !isLoadingTags.value &&
    !isLoadingCatalog.value &&
    !isApplyingTags.value &&
    !isCreatingTag.value &&
    !tagLoadFailed.value,
);
const selectedTags = ref<Set<number>>(new Set());
const intermediateTags = ref<Set<number>>(new Set());
const initialSelectedTags = ref<Set<number>>(new Set());
const initialIntermediateTags = ref<Set<number>>(new Set());
const tagChanges = ref<Map<number, "add" | "remove">>(new Map());
const filteredTags = computed(() =>
  tagRows(
    editor.value?.id === null
      ? [
          ...allTags.value,
          {
            id: -1,
            name: editor.value.name,
            parent_id: editor.value.parent,
            path: editor.value.name,
          },
        ]
      : allTags.value,
    tagSearch.value,
    collapsed.value,
    0,
    -1,
  ),
);
const editor = ref<{
  id: number | null;
  name: string;
  parent: number | null;
} | null>(null);
const error = ref("");
const deleteTarget = ref<
  (LibraryTag & { tag_count: number; file_count: number }) | null
>(null);
const tagsChangeSource = crypto.randomUUID();
let disposed = false;
let loadRequest = 0;
let unlisten: (() => void) | undefined;
function toggleBranch(id: number) {
  if (tagSearch.value) return;
  collapsed.value = collapsed.value.includes(id)
    ? collapsed.value.filter((value) => value !== id)
    : [...collapsed.value, id];
}
function cancelEditor() {
  if (!isCreatingTag.value) editor.value = null;
}
function edit(tag?: LibraryTag, parent: number | null = null) {
  if (!canEditTags.value || editor.value) return;
  error.value = "";
  tagSearch.value = "";
  const parentId = tag ? tag.parent_id : parent;
  const base = t("msgbox.new_tag.title");
  let name = tag?.name ?? base;
  let index = 0;
  const names = new Set(
    allTags.value
      .filter((item) => item.parent_id === parentId)
      .map((item) => item.name.toLocaleLowerCase()),
  );
  if (!tag)
    while (names.has(name.toLocaleLowerCase())) name = `${base} ${++index}`;
  editor.value = { id: tag?.id ?? null, name, parent: parentId };
  const open = new Set(
    parentId === null
      ? []
      : [parentId, ...ancestorIds(allTags.value, parentId)],
  );
  collapsed.value = collapsed.value.filter((id) => !open.has(id));
}
async function saveEditor(value: string) {
  const target = editor.value;
  if (!target || !canEditTags.value) return;
  const name = value.trim();
  if (!isValidTagName(name)) {
    error.value = t("tag.invalid_name");
    return;
  }
  if (
    target.id !== null &&
    allTags.value.find((tag) => tag.id === target.id)?.name === name
  ) {
    editor.value = null;
    return;
  }
  if (
    allTags.value.some(
      (tag) =>
        tag.id !== target.id &&
        tag.parent_id === target.parent &&
        tag.name.toLocaleLowerCase() === name.toLocaleLowerCase(),
    )
  ) {
    error.value = t("tag.name_exists");
    return;
  }
  isCreatingTag.value = true;
  try {
    const result =
      target.id === null
        ? await createTag(name, target.parent)
        : await renameTag(target.id, name);
    if (!result) throw new Error("save failed");
    editor.value = null;
    tagSearch.value = "";
    await loadAllTags();
    const id = target.id ?? result.id;
    const ancestors = new Set(ancestorIds(allTags.value, id));
    collapsed.value = collapsed.value.filter((value) => !ancestors.has(value));
    if (target.id === null) {
      const next = new Map(tagChanges.value);
      next.set(id, "add");
      tagChanges.value = next;
      setTagVisualState(id, "selected");
    }
    await tauriEmit("tags-changed", { source: tagsChangeSource });
    await nextTick();
    document
      .getElementById(`dialog-tag-${id}`)
      ?.scrollIntoView({ block: "nearest" });
  } catch {
    error.value = t("tag.name_save_failed");
  } finally {
    isCreatingTag.value = false;
  }
}
async function loadAllTags() {
  const request = ++loadRequest;
  const library = libConfig._libraryId;
  isLoadingCatalog.value = true;
  try {
    const tags = await getAllTags(0);
    if (disposed || request !== loadRequest || library !== libConfig._libraryId)
      return;
    if (!tags) throw new Error("load failed");
    allTags.value = tags;
    const ids = new Set(tags.map((tag: LibraryTag) => tag.id));
    for (const id of tagChanges.value.keys())
      if (!ids.has(id)) tagChanges.value.delete(id);
    catalogLoadFailed.value = false;
  } catch {
    if (request === loadRequest) catalogLoadFailed.value = true;
  } finally {
    if (request === loadRequest) isLoadingCatalog.value = false;
  }
}

let selectionRequest = 0;
async function loadExistingTagsForFiles(preserveChanges = false) {
  const request = ++selectionRequest;
  const library = libConfig._libraryId;
  isLoadingTags.value = true;
  try {
    const counts = props.fileIds.length
      ? await getTagSelectionCounts(props.fileIds)
      : [];
    if (
      disposed ||
      request !== selectionRequest ||
      library !== libConfig._libraryId
    )
      return;
    if (counts === null) {
      selectionLoadFailed.value = true;
      return;
    }
    selectionLoadFailed.value = false;
    const selected = new Set<number>();
    const intermediate = new Set<number>();
    for (const entry of counts) {
      const tagId = Number(entry.tag_id);
      if (Number(entry.count) === props.fileIds.length) selected.add(tagId);
      else if (Number(entry.count) > 0) intermediate.add(tagId);
    }
    initialSelectedTags.value = new Set(selected);
    initialIntermediateTags.value = new Set(intermediate);
    selectedTags.value = selected;
    intermediateTags.value = intermediate;
    if (!preserveChanges) tagChanges.value.clear();
    for (const [id, change] of tagChanges.value)
      setTagVisualState(id, change === "add" ? "selected" : "none");
  } finally {
    if (!disposed && request === selectionRequest) isLoadingTags.value = false;
  }
}

// TAG_SELECTION_TEST_BEGIN: keep these markers for the isolated selection tests.
function toggleTag(tagId: number) {
  if (!canEditTags.value || editor.value) return;
  const normalized = Number(tagId);
  const next = new Map(tagChanges.value);
  const current = next.get(normalized);
  const initialState = initialSelectedTags.value.has(normalized)
    ? "selected"
    : initialIntermediateTags.value.has(normalized)
      ? "intermediate"
      : "none";

  if (current === "add") {
    if (initialState === "none") {
      next.delete(normalized);
    } else {
      next.set(normalized, "remove");
    }
    setTagVisualState(normalized, "none");
  } else if (current === "remove") {
    next.delete(normalized);
    setTagVisualState(normalized, initialState);
  } else if (selectedTags.value.has(normalized)) {
    next.set(normalized, "remove");
    setTagVisualState(normalized, "none");
  } else {
    next.set(normalized, "add");
    setTagVisualState(normalized, "selected");
  }
  tagChanges.value = next;
}

function setTagVisualState(
  tagId: number,
  state: "selected" | "intermediate" | "none",
) {
  const selected = new Set(selectedTags.value);
  const intermediate = new Set(intermediateTags.value);
  selected.delete(tagId);
  intermediate.delete(tagId);
  if (state === "selected") selected.add(tagId);
  if (state === "intermediate") intermediate.add(tagId);
  selectedTags.value = selected;
  intermediateTags.value = intermediate;
}

// TAG_SELECTION_TEST_END
async function prepareDelete(tag: LibraryTag) {
  if (!canEditTags.value) return;
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
  if (!deleteTarget.value || !canEditTags.value) return;
  isCreatingTag.value = true;
  try {
    if (!(await deleteTag(deleteTarget.value.id)))
      throw new Error("delete failed");
    deleteTarget.value = null;
    await loadAllTags();
    await loadExistingTagsForFiles(true);
    const states = await applyTagsToFiles(props.fileIds, [], []);
    if (states !== null) emit("states-changed", states);
    await tauriEmit("tags-changed", { source: tagsChangeSource });
  } catch {
    toast.error(t("tag.delete_failed"));
  } finally {
    isCreatingTag.value = false;
  }
}
async function clickOk() {
  if (!canEditTags.value || editor.value) return;
  isApplyingTags.value = true;
  const add = Array.from(tagChanges.value)
    .filter(([, action]) => action === "add")
    .map(([id]) => id);
  const remove = Array.from(tagChanges.value)
    .filter(([, action]) => action === "remove")
    .map(([id]) => id);
  const library = libConfig._libraryId;
  try {
    const result = await applyTagsToFiles(props.fileIds, add, remove);
    if (library !== libConfig._libraryId || disposed) return;
    if (result !== null) {
      await tauriEmit("tags-changed");
      emit("ok", result);
    }
  } finally {
    isApplyingTags.value = false;
  }
}
function clickCancel() {
  if (!isApplyingTags.value) emit("cancel");
}
function handleKeyDown(event: KeyboardEvent) {
  if (
    !uiStore.isInputActive("TaggingDialog") ||
    editor.value ||
    deleteTarget.value ||
    event.defaultPrevented
  )
    return;
  if (event.key === "Escape") {
    if (focusedTagIndex.value >= 0) {
      focusedTagIndex.value = -1;
      tagSearchInputRef.value?.focus();
    } else clickCancel();
    return;
  }
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    const last = filteredTags.value.length - 1;
    focusedTagIndex.value = Math.max(
      0,
      Math.min(
        last,
        focusedTagIndex.value + (event.key === "ArrowDown" ? 1 : -1),
      ),
    );
    tagSearchInputRef.value?.blur();
    document
      .getElementById(
        `dialog-tag-${filteredTags.value[focusedTagIndex.value]?.id}`,
      )
      ?.scrollIntoView({ block: "nearest" });
  } else if (event.key === " " && focusedTagIndex.value >= 0) {
    event.preventDefault();
    const tag = filteredTags.value[focusedTagIndex.value];
    if (tag) toggleTag(tag.id);
  } else if (
    (event.key === "ArrowRight" || event.key === "ArrowLeft") &&
    focusedTagIndex.value >= 0
  ) {
    event.preventDefault();
    const tag = filteredTags.value[focusedTagIndex.value];
    if (
      tag?.hasChildren &&
      collapsed.value.includes(tag.id) === (event.key === "ArrowRight")
    )
      toggleBranch(tag.id);
  } else if (event.key === "Enter") {
    event.preventDefault();
    void clickOk();
  }
}
onMounted(async () => {
  uiStore.pushInputHandler("TaggingDialog");
  window.addEventListener("keydown", handleKeyDown);
  const stop = await listen<{ source?: string }>(
    "tags-changed",
    async (event) => {
      if (event.payload?.source === tagsChangeSource) return;
      await loadAllTags();
      if (!disposed) await loadExistingTagsForFiles(true);
    },
  );
  if (disposed) {
    stop();
    return;
  }
  unlisten = stop;
  await Promise.all([loadAllTags(), loadExistingTagsForFiles()]);
  await nextTick();
  tagSearchInputRef.value?.focus();
});
onBeforeUnmount(() => {
  disposed = true;
  loadRequest++;
  selectionRequest++;
  unlisten?.();
  window.removeEventListener("keydown", handleKeyDown);
  uiStore.removeInputHandler("TaggingDialog");
});
watch(() => libConfig._libraryId, clickCancel);
watch(tagSearch, () => {
  focusedTagIndex.value = -1;
});
</script>
