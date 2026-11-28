<template>
  <div class="flex items-center gap-2 shrink-0">
    <TButton :tooltip="disabled ? tr('disabled') : filterTooltip">
      <button
        ref="trigger"
        class="inline-flex h-8 items-center justify-center rounded-box border px-2 text-sm outline-none"
        :class="[
          disabled
            ? 'border-0! border-transparent text-base-content/30 cursor-default'
            : count > 0
              ? 'border-transparent bg-base-100 hover:bg-base-100 text-primary! cursor-pointer'
              : 'border-transparent text-base-content/70 hover:text-base-content hover:bg-base-100/30 cursor-pointer',
        ]"
        :disabled="disabled"
        :aria-expanded="open"
        :aria-label="disabled ? tr('disabled') : undefined"
        @click="togglePanel"
      >
        <IconFilter class="w-5 h-5 shrink-0" />
        <span v-if="!disabled && count > 0" class="badge badge-primary badge-xs ml-1">{{
          count
        }}</span>
      </button>
    </TButton>
  </div>
  <Teleport to="body">
    <Transition name="fade">
    <div
      v-if="open"
      ref="panel"
      role="dialog"
      :aria-label="tr('title')"
      class="fixed z-700 w-90 max-w-[calc(100vw-16px)] rounded-box border border-base-content/30 bg-base-200/80 backdrop-blur-md shadow-lg text-base-content overflow-y-auto px-3 py-2 select-none"
      :style="position"
      @keydown.esc.stop.prevent="close"
    >
      <div class="flex items-center gap-2 mb-2">
        <span class="flex-1 text-xs text-base-content/70">{{ tr("title") }}</span>
        <span
          v-if="resultPending"
          class="loading loading-spinner loading-xs text-base-content/40"
          :aria-label="tr('counting')"
        ></span>
        <span v-else class="whitespace-nowrap text-xs text-base-content/60">{{ resultSummary }}</span>
        <button
          class="btn btn-ghost btn-xs text-xs btn-square shrink-0"
          :disabled="!count"
          :title="tr('reset')"
          :aria-label="tr('reset')"
          @click="reset"
        >
          <IconRestore class="size-3.5" aria-hidden="true" />
        </button>
      </div>
      <div v-if="count > 0" class="mb-2 flex flex-wrap gap-1.5">
        <button
          v-for="chip in chips"
          :key="chip.key"
          class="badge badge-outline h-6 max-w-full min-w-0 px-2 text-xs border-primary/30 bg-primary/15 font-medium text-primary hover:bg-primary/25 cursor-pointer gap-1"
          @click="chip.clear()"
        >
          <span class="min-w-0 truncate">{{ chip.label }}</span
          ><span class="shrink-0" aria-hidden="true">×</span
          ><span class="sr-only">{{ tr("remove") }}</span>
        </button>
      </div>
      <div class="border-t border-base-content/10 py-2 space-y-2">
      <section>
        <div class="text-xs text-base-content/60 mb-2">
          {{ tr("file_type") }}
        </div>
        <div
          class="join flex border border-base-content/15 rounded-box overflow-hidden"
        >
          <button
            v-for="option in types"
            :key="option.value"
            class="join-item btn btn-ghost btn-xs text-xs flex-1 border-transparent text-base-content/70 hover:text-base-content hover:bg-base-100/30"
            :class="{ 'bg-base-100 hover:bg-base-100 text-primary!': type === option.value }"
            @click="$emit('type', option.value)"
          >
            {{ option.label }}
          </button>
        </div>
      </section>
      <section>
        <div class="text-xs text-base-content/60 mb-2">{{ tr("rating") }}</div>
        <div
          class="join flex border border-base-content/15 rounded-box overflow-hidden"
        >
          <button
            v-for="mode in [
              { value: -1, label: 'all' },
              { value: -2, label: 'rated' },
              { value: 0, label: 'unrated' },
            ]"
            :key="mode.value"
            class="join-item btn btn-ghost btn-xs text-xs flex-1 border-transparent text-base-content/70 hover:text-base-content hover:bg-base-100/30"
            :class="{
              'bg-base-100 hover:bg-base-100 text-primary!': filters.rating === mode.value,
            }"
            @click="filters.rating = mode.value"
          >
            <component
              v-if="mode.value !== -1"
              :is="mode.value === -2 ? IconStarFilled : IconStar"
              class="size-3.5 shrink-0"
              aria-hidden="true"
            />{{ tr(mode.label) }}
          </button>
        </div>
        <div class="flex items-center justify-between gap-2 mt-2">
          <div class="flex items-center">
            <button
              v-for="star in 5"
              :key="star"
              class="btn btn-ghost btn-xs text-xs btn-square"
              :aria-label="String(star)"
              :class="
                star <= filters.rating ? 'text-warning' : 'text-base-content/30'
              "
              @click="filters.rating = filters.rating === star ? -1 : star"
            >
              <IconStarFilled class="size-4" />
            </button>
          </div>
          <div class="join shrink-0">
            <button
              v-for="op in operators"
              :key="op.value"
              :disabled="filters.rating < 1 || filters.rating > 5"
              class="join-item btn btn-ghost btn-xs text-xs"
              :class="{
                'bg-primary/15 text-primary':
                  filters.rating > 0 && filters.ratingOp === op.value,
              }"
              :title="op.label"
              :aria-label="op.label"
              @click="filters.ratingOp = op.value"
            >
              <component :is="op.icon" class="size-3" aria-hidden="true" />
            </button>
          </div>
        </div>
      </section>
      <section
        v-for="field in segments"
        :key="field.key"
      >
        <div class="text-xs text-base-content/60 mb-2">{{ tr(field.key) }}</div>
        <div
          class="join flex border border-base-content/15 rounded-box overflow-hidden"
        >
          <button
            v-for="option in field.options"
            :key="option.value"
            class="join-item btn btn-ghost btn-xs text-xs flex-1 border-transparent px-2 text-base-content/70 hover:text-base-content hover:bg-base-100/30"
            :class="{
              'bg-base-100 hover:bg-base-100 text-primary!': filters[field.key] === option.value,
            }"
            @click="filters[field.key] = option.value"
          >
            <component
              v-if="option.value !== -1 && !(field.key === 'tags' && option.value === 0)"
              :is="
                field.key === 'tags'
                  ? IconTag
                  : field.key === 'favorite'
                    ? option.value === 1
                      ? IconHeartFilled
                      : IconHeart
                    : option.value === 1
                      ? IconFlagFilled
                      : option.value === 2
                        ? IconFlagOff
                        : IconFlag
              "
              class="size-3.5 shrink-0"
              aria-hidden="true"
            />{{ tr(option.label) }}
          </button>
        </div>
      </section>
      </div>
    </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import TButton from "@/components/TButton.vue";
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import {
  browserFilters as filters,
  resetBrowserFilterState,
} from "@/common/browserFilters";
import { useUIStore } from "@/stores/uiStore";
import {
  IconEqualGreater,
  IconEqual,
  IconEqualLower,
  IconRestore,
  IconFilter,
  IconStar,
  IconStarFilled,
  IconTag,
  IconHeart,
  IconHeartFilled,
  IconFlag,
  IconFlagFilled,
  IconFlagOff,
} from "@/common/icons";
const props = defineProps<{
  type: number;
  types: Array<{ value: number; label: string }>;
  disabled: boolean;
  resultCount: number | null;
  baseline: number | null;
}>();
const emit = defineEmits<{ type: [value: number]; open: [value: boolean] }>();
const { t } = useI18n();
const uiStore = useUIStore();
const tr = (key: string, values: Record<string, string> = {}) =>
  t("toolbar.browser_filters." + key, values);
const open = ref(false),
  trigger = ref<HTMLElement>(),
  panel = ref<HTMLElement>();
const position = ref<Record<string, string>>({});
const operators = [
  { value: "gte", label: "≥", icon: IconEqualGreater },
  { value: "eq", label: "=", icon: IconEqual },
  { value: "lte", label: "≤", icon: IconEqualLower },
];
const ratingLabel = computed(() =>
  filters.rating === -1
    ? tr("all")
    : filters.rating === 0
      ? tr("unrated")
      : filters.rating === -2
        ? tr("rated")
        : `${operators.find((op) => op.value === filters.ratingOp)?.label} ${filters.rating}`,
);
const segments = [
  {
    key: "favorite" as const,
    options: [
      { value: -1, label: "all" },
      { value: 1, label: "favorited" },
      { value: 0, label: "not_favorited" },
    ],
  },
  {
    key: "culling" as const,
    options: [
      { value: -1, label: "all" },
      { value: 1, label: "pick" },
      { value: 2, label: "reject" },
      { value: 0, label: "unreviewed" },
    ],
  },
  {
    key: "tags" as const,
    options: [
      { value: -1, label: "all" },
      { value: 1, label: "tagged" },
      { value: 0, label: "untagged" },
    ],
  },
];
const chips = computed(() => {
  const items: Array<{ key: string; label: string; clear: () => void }> = [];
  if (props.type)
    items.push({
      key: "type",
      label: props.types.find((type) => type.value === props.type)?.label || "",
      clear: () => emit("type", 0),
    });
  if (filters.rating !== -1)
    items.push({
      key: "rating",
      // Rated/unrated are plain state badges; only a numeric star threshold
      // needs the extra star glyph.
      label: filters.rating === -2 || filters.rating === 0 ? ratingLabel.value : `★ ${ratingLabel.value}`,
      clear: () => {
        filters.rating = -1;
      },
    });
  for (const field of segments)
    if (filters[field.key] !== -1)
      items.push({
        key: field.key,
        label: tr(
          field.options.find((option) => option.value === filters[field.key])!
            .label,
        ),
        clear: () => {
          filters[field.key] = -1;
        },
      });
  return items;
});
const count = computed(() => chips.value.length);
const resultPending = computed(() =>
  props.baseline == null || props.resultCount == null
);
const resultSummary = computed(() => {
  const baseline = props.baseline;
  const resultCount = props.resultCount;
  if (resultPending.value) return "";
  // Keep the denominator visible when a filter excludes every result,
  // e.g. "0 of 100 items"; only an actually empty result set is "No items found".
  if (baseline === 0 && resultCount === 0) return t("tooltip.not_found.files");
  if (resultCount === baseline) {
    return tr("total_items", { total: baseline.toLocaleString() });
  }
  return tr("results", {
    count: resultCount.toLocaleString(),
    total: baseline.toLocaleString(),
  });
});
const filterTooltip = computed(() =>
  chips.value.length
    ? chips.value.map((chip) => chip.label).join(" · ")
    : tr("tooltip"),
);
function reset() {
  resetBrowserFilterState();
  emit("type", 0);
}
function locate() {
  const rect = trigger.value?.getBoundingClientRect();
  if (!rect) return;
  const width = Math.min(440, window.innerWidth - 16);
  position.value = {
    left: `${Math.max(8, Math.min(rect.left, window.innerWidth - width - 8))}px`,
    top: `${Math.min(rect.bottom + 6, window.innerHeight - 100)}px`,
    maxHeight: `${Math.max(80, window.innerHeight - rect.bottom - 14)}px`,
  };
}
watch(open, (value) => {
  emit("open", value);
  if (value) uiStore.pushInputHandler("FilterPanel");
  else uiStore.removeInputHandler("FilterPanel");
});
function close() {
  open.value = false;
}
watch(() => props.disabled, disabled => {
  if (disabled) close();
});
function togglePanel() {
  if (open.value) return close();
  open.value = true;
  locate();
}
function outside(event: PointerEvent) {
  if (
    open.value &&
    !panel.value?.contains(event.target as Node) &&
    !trigger.value?.parentElement?.contains(event.target as Node)
  )
    close();
}
function escape(event: KeyboardEvent) {
  if (event.key === "Escape" && open.value) {
    event.stopPropagation();
    close();
  }
}
onMounted(() => {
  document.addEventListener("pointerdown", outside);
  document.addEventListener("keydown", escape, true);
  window.addEventListener("resize", locate);
});
onBeforeUnmount(() => {
  uiStore.removeInputHandler("FilterPanel");
  document.removeEventListener("pointerdown", outside);
  document.removeEventListener("keydown", escape, true);
  window.removeEventListener("resize", locate);
});
</script>
