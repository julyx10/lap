<template>
  <div
    ref="root"
    class="flex items-center shrink-0 overflow-hidden transition-[width,padding-left] duration-200 ease-out motion-reduce:transition-none"
    :class="expanded || modelValue ? 'w-57' : 'w-8'"
    @transitionend.self="locateHistory"
  >
    <TButton v-if="disabled || (!expanded && !modelValue)" :tooltip="$t('search.title')">
      <button
        class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-box border border-transparent text-sm outline-none"
        :class="
          disabled
            ? 'text-base-content/30 cursor-default'
            : 'text-base-content/70 hover:text-base-content hover:bg-base-100/30 cursor-pointer'
        "
        :disabled="disabled"
        :aria-label="$t('search.title')"
        @click="focus"
      >
        <IconSearch class="w-5 h-5 shrink-0" />
      </button>
    </TButton>
    <div
      v-else
      ref="inputBox"
      class="flex items-center shrink-0 h-8 w-56 rounded-box border border-base-content/20 bg-base-100/40 px-2 gap-2"
    >
      <IconSearch class="w-5 h-5 shrink-0" />
      <input
        id="toolbar-search-input"
        ref="input"
        v-model="draft"
        role="combobox"
        :aria-label="$t('search.title')"
        :aria-expanded="historyOpen"
        aria-controls="toolbar-search-history"
        class="min-w-0 w-full text-sm bg-transparent outline-none"
        :placeholder="$t('search.image_search_placeholder')"
        maxlength="255"
        @focus="showHistory"
        @keydown.enter.prevent="submit(draft)"
        @keydown.esc.stop.prevent="hide"
      />
      <button
        v-if="draft || modelValue"
        :aria-label="$t('menu.tag.clear_search')"
        @click="clear"
      >
        <IconClose class="w-4 h-4" />
      </button>
    </div>
    <Teleport to="body">
      <Transition name="fade">
        <div
          v-if="historyOpen"
          id="toolbar-search-history"
          ref="popup"
          role="listbox"
          class="fixed z-700 bg-base-200/80 backdrop-blur-md border border-base-content/30 rounded-box shadow-lg p-2 max-h-72 overflow-y-auto text-sm select-none"
          :style="position"
          @keydown.esc.stop.prevent="hide"
        >
          <div
            class="flex items-center gap-2 px-2 py-1 text-xs text-base-content/40"
          >
            <span class="flex-1 min-w-0 truncate">{{
              $t("search.in_scope", { scope })
            }}</span>
            <button
              v-if="history.length"
              type="button"
              class="shrink-0 cursor-pointer hover:text-base-content"
              :title="$t('menu.search.clear_history')"
              :aria-label="$t('menu.search.clear_history')"
              @mousedown.prevent
              @click="clearHistory"
            >
              <IconTrash class="w-4 h-4" aria-hidden="true" />
            </button>
          </div>
          <button
            v-for="text in history"
            :key="text"
            class="w-full text-left px-2 py-1 rounded-box hover:bg-base-content/5 truncate"
            role="option"
            aria-selected="false"
            @mousedown.prevent
            @click="submit(text)"
          >
            <IconSearch class="inline-block w-4 h-4 mr-2" />{{ text }}
          </button>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>
<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import TButton from "@/components/TButton.vue";
import { config, libConfig } from "@/common/config";
import { IconSearch, IconClose, IconTrash } from "@/common/icons";
import { useUIStore } from "@/stores/uiStore";
const props = defineProps<{
  modelValue: string;
  scope: string;
  disabled?: boolean;
}>();
const emit = defineEmits<{ "update:modelValue": [value: string] }>();
const ui = useUIStore();
const expanded = ref(false),
  historyOpen = ref(false),
  draft = ref(props.modelValue);
const inputBox = ref<HTMLElement>();
const root = ref<HTMLElement>(),
  input = ref<HTMLInputElement>(),
  popup = ref<HTMLElement>();
const position = ref({ left: "0px", top: "0px", width: "224px" });
const history = computed(
  () =>
    [
      ...new Set(
        (libConfig.search.searchHistory || [])
          .map((item: any) => (typeof item === "string" ? item : item.text))
          .filter(Boolean),
      ),
    ] as string[],
);
watch(
  () => props.modelValue,
  (value) => {
    draft.value = value;
  },
);
async function focus() {
  if (props.disabled) return;
  expanded.value = true;
  await nextTick();
  if (props.disabled) return;
  input.value?.focus();
  input.value?.select();
}
let historyRequest = 0;
async function showHistory() {
  if (props.disabled) return;
  const request = ++historyRequest;
  historyOpen.value = false;
  if (!ui.isInputActive("ToolbarSearch")) ui.pushInputHandler("ToolbarSearch");
  await nextTick();
  const animations = root.value?.getAnimations() || [];
  await Promise.allSettled(animations.map((animation) => animation.finished));
  if (request !== historyRequest || document.activeElement !== input.value)
    return;
  historyOpen.value = true;
  locateHistory();
}
function locateHistory() {
  if (!historyOpen.value) return;
  const rect = (inputBox.value || root.value)!.getBoundingClientRect();
  position.value = {
    left:
      Math.max(8, Math.min(rect.left, window.innerWidth - rect.width - 8)) +
      "px",
    top: rect.bottom + 6 + "px",
    width: rect.width + "px",
  };
}
function hide() {
  historyRequest++;
  historyOpen.value = false;
  input.value?.blur();
  ui.removeInputHandler("ToolbarSearch");
  draft.value = props.modelValue;
  if (!props.modelValue) expanded.value = false;
}
function submit(value: string) {
  if (props.disabled) return;
  const text = value.trim();
  if (!text) return clear();
  libConfig.search.searchHistory = [
    { text, count: null },
    ...history.value
      .filter((item) => item !== text)
      .map((text) => ({ text, count: null })),
  ].slice(0, config.search.maxSearchHistory || 20);
  hide();
  draft.value = text;
  emit("update:modelValue", text);
}
function clearHistory() {
  libConfig.search.searchHistory = [];
  libConfig.search.searchHistoryIndex = -1;
  historyOpen.value = false;
}
function clear() {
  historyRequest++;
  draft.value = "";
  emit("update:modelValue", "");
  historyOpen.value = false;
  expanded.value = false;
  input.value?.blur();
  ui.removeInputHandler("ToolbarSearch");
}
function outside(event: PointerEvent) {
  if (
    !root.value?.contains(event.target as Node) &&
    !popup.value?.contains(event.target as Node)
  )
    hide();
}
watch(
  () => props.disabled,
  (disabled) => {
    if (disabled) hide();
  },
);
onMounted(() => {
  document.addEventListener("pointerdown", outside);
});
onBeforeUnmount(() => {
  historyRequest++;
  document.removeEventListener("pointerdown", outside);
  ui.removeInputHandler("ToolbarSearch");
});
defineExpose({ focus });
</script>
