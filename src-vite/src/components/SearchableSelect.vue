<template>
  <div ref="rootRef" class="relative min-w-0">
    <button
      ref="triggerRef"
      type="button"
      role="combobox"
      aria-haspopup="listbox"
      :aria-expanded="open"
      :disabled="disabled"
      class="select select-sm flex w-full items-center text-left text-xs"
      :class="disabled ? 'cursor-default opacity-50' : 'cursor-pointer'"
      :title="displayLabel"
      @click="toggle"
      @keydown="onTriggerKeydown"
    >
      <span class="min-w-0 flex-1 truncate" :class="{ 'text-base-content/40': !hasSelection }">{{ displayLabel }}</span>
    </button>

    <teleport to="body">
      <div
        v-if="open"
        ref="menuRef"
        class="fixed z-[700] flex flex-col overflow-hidden rounded-box border border-base-content/30 bg-base-200/95 shadow-lg backdrop-blur-md"
        :style="menuStyle"
      >
        <label class="flex items-center gap-1.5 border-b border-base-content/10 px-2 py-1.5">
          <IconSearch class="h-4 w-4 shrink-0 text-base-content/40" />
          <input
            ref="searchRef"
            v-model="query"
            type="text"
            class="min-w-0 w-full bg-transparent px-1 text-sm outline-none"
            :placeholder="searchPlaceholder || placeholder"
            autocomplete="off"
            spellcheck="false"
            @keydown="onSearchKeydown"
            @compositionstart="composing = true"
            @compositionend="composing = false"
          />
        </label>
        <div ref="listRef" class="max-h-56 overflow-y-auto p-1" role="listbox">
          <div
            v-for="(option, index) in filteredOptions"
            :key="String(option.value)"
            role="option"
            :aria-selected="isSelected(option)"
            :data-index="index"
            class="flex cursor-pointer items-center gap-2 rounded-box px-2 py-1 text-xs"
            :class="index === activeIndex ? 'bg-base-100 text-base-content' : 'text-base-content/70'"
            @mouseenter="onOptionHover($event, index, option.label)"
            @mousedown.prevent
            @click="select(option)"
          >
            <span data-option-label class="min-w-0 flex-1 truncate">
              <span
                v-for="(part, partIndex) in highlight(option.label)"
                :key="partIndex"
                :class="part.match ? 'font-semibold text-primary' : ''"
              >{{ part.text }}</span>
            </span>
            <IconOk v-if="isSelected(option)" class="h-3.5 w-3.5 shrink-0 text-primary" />
          </div>
          <div v-if="filteredOptions.length === 0" class="px-2 py-2 text-xs text-base-content/40">{{ emptyText }}</div>
        </div>
      </div>
    </teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { IconOk, IconSearch } from '@/common/icons';
import { filterOptions, highlightSegments } from '@/common/searchableOptions.js';
import {
  initialActiveIndex,
  isOptionSelected,
  moveActiveIndex,
  resolveActiveOption,
  resolveDisplayLabel,
  resolveSelectedOption,
} from '@/common/searchableSelectState.js';

type SelectValue = string | number;

function onOptionHover(event: MouseEvent, index: number, label: string) {
  activeIndex.value = index;
  const row = event.currentTarget as HTMLElement;
  const text = row.querySelector<HTMLElement>('[data-option-label]');
  row.title = text && text.scrollWidth > text.clientWidth ? label : '';
}

interface SelectOption {
  value: SelectValue;
  label: string;
}

const props = withDefaults(defineProps<{
  modelValue?: SelectValue | null;
  options?: SelectOption[];
  placeholder?: string;
  searchPlaceholder?: string;
  emptyText?: string;
  disabled?: boolean;
  maxVisible?: number;
}>(), {
  modelValue: null,
  options: () => [],
  placeholder: '',
  searchPlaceholder: '',
  emptyText: '',
  disabled: false,
  maxVisible: 100,
});

const emit = defineEmits<{ (event: 'update:modelValue', value: SelectValue | null): void }>();

const rootRef = ref<HTMLElement | null>(null);
const triggerRef = ref<HTMLButtonElement | null>(null);
const menuRef = ref<HTMLElement | null>(null);
const searchRef = ref<HTMLInputElement | null>(null);
const listRef = ref<HTMLElement | null>(null);
const open = ref(false);
const query = ref('');
const activeIndex = ref(0);
const composing = ref(false);
const menuStyle = ref<{ top: string; left: string; width: string }>({ top: '0px', left: '0px', width: '0px' });

const selectedOption = computed<SelectOption | null>(() => resolveSelectedOption(props.options, props.modelValue));

const hasSelection = computed(() => selectedOption.value !== null);

const displayLabel = computed(() => resolveDisplayLabel(selectedOption.value, props.modelValue, props.placeholder));

const filteredOptions = computed(() => filterOptions(props.options, query.value, props.maxVisible));

function isSelected(option: SelectOption) {
  return isOptionSelected(option.value, props.modelValue);
}

function highlight(label: string) {
  return highlightSegments(label, query.value);
}

function positionMenu() {
  const trigger = triggerRef.value;
  const menu = menuRef.value;
  if (!trigger || !menu) return;

  const rect = trigger.getBoundingClientRect();
  const padding = 8;
  const width = Math.round(Math.min(360, Math.max(240, rect.width)));
  const height = menu.offsetHeight || 240;

  let top = rect.bottom + 4;
  if (top + height > window.innerHeight - padding) {
    top = Math.max(padding, rect.top - height - 4);
  }

  let left = Math.min(rect.left, window.innerWidth - width - padding);
  left = Math.max(padding, left);

  menuStyle.value = {
    top: `${Math.round(top)}px`,
    left: `${Math.round(left)}px`,
    width: `${width}px`,
  };
}

let scrollFrame = 0;

function onWindowChange() {
  if (!open.value || scrollFrame) return;
  scrollFrame = requestAnimationFrame(() => {
    scrollFrame = 0;
    positionMenu();
  });
}

function addWindowListeners() {
  window.addEventListener('scroll', onWindowChange, true);
  window.addEventListener('resize', onWindowChange);
}

function removeWindowListeners() {
  window.removeEventListener('scroll', onWindowChange, true);
  window.removeEventListener('resize', onWindowChange);
  if (scrollFrame) {
    cancelAnimationFrame(scrollFrame);
    scrollFrame = 0;
  }
}

function onDocumentMousedown(event: MouseEvent) {
  if (!open.value) return;
  const target = event.target as Node | null;
  if (!target) return;
  if (rootRef.value?.contains(target) || menuRef.value?.contains(target)) return;
  closeMenu(false);
}

onMounted(() => {
  document.addEventListener('mousedown', onDocumentMousedown, true);
});

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', onDocumentMousedown, true);
  removeWindowListeners();
});

function scrollActiveIntoView() {
  const list = listRef.value;
  if (!list) return;
  const item = list.querySelector<HTMLElement>(`[data-index="${activeIndex.value}"]`);
  item?.scrollIntoView({ block: 'nearest' });
}

function openMenu() {
  if (props.disabled) return;
  open.value = true;
}

function closeMenu(restoreFocus = false) {
  if (!open.value) return;
  open.value = false;
  if (restoreFocus) {
    void nextTick(() => triggerRef.value?.focus());
  }
}

function toggle() {
  if (props.disabled) return;
  if (open.value) closeMenu(false);
  else openMenu();
}

function select(option: SelectOption) {
  emit('update:modelValue', option.value);
  closeMenu(true);
}

function onTriggerKeydown(event: KeyboardEvent) {
  if (props.disabled) return;
  if (['ArrowDown', 'ArrowUp', 'Enter', ' '].includes(event.key)) {
    event.preventDefault();
    openMenu();
  } else if (event.key === 'Escape' && open.value) {
    event.stopPropagation();
    closeMenu(false);
  }
}

function onSearchKeydown(event: KeyboardEvent) {
  const count = filteredOptions.value.length;

  if (event.key === 'ArrowDown') {
    event.preventDefault();
    if (count > 0) {
      activeIndex.value = moveActiveIndex(activeIndex.value, 1, count);
      scrollActiveIntoView();
    }
    return;
  }

  if (event.key === 'ArrowUp') {
    event.preventDefault();
    if (count > 0) {
      activeIndex.value = moveActiveIndex(activeIndex.value, -1, count);
      scrollActiveIntoView();
    }
    return;
  }

  if (event.key === 'Enter') {
    if (event.isComposing || composing.value) return;
    event.preventDefault();
    const option = resolveActiveOption(filteredOptions.value, activeIndex.value);
    if (option) select(option);
    return;
  }

  if (event.key === 'Escape') {
    event.preventDefault();
    event.stopPropagation();
    closeMenu(true);
    return;
  }

  if (event.key === 'Tab') {
    event.preventDefault();
    closeMenu(true);
  }
}

watch(open, async isOpen => {
  if (!isOpen) {
    removeWindowListeners();
    return;
  }

  query.value = '';
  activeIndex.value = initialActiveIndex(filteredOptions.value, props.modelValue);
  addWindowListeners();

  await nextTick();
  positionMenu();
  scrollActiveIntoView();
  searchRef.value?.focus();
  searchRef.value?.select();
});

watch(query, () => {
  activeIndex.value = 0;
  void nextTick(() => {
    if (open.value) {
      positionMenu();
      if (listRef.value) listRef.value.scrollTop = 0;
    }
  });
});

watch(() => props.options, () => {
  if (!open.value) return;
  const count = filteredOptions.value.length;
  activeIndex.value = count > 0 ? Math.min(activeIndex.value, count - 1) : 0;
  void nextTick(positionMenu);
});

watch(() => props.disabled, disabled => {
  if (disabled) closeMenu(false);
});
</script>
