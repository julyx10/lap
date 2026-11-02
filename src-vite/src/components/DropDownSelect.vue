<template>
  <div ref="dropdown" class="relative inline-block text-left">

    <!-- Dropdown Trigger -->
    <button tabindex="-1"
      :title="title"
      class="flex items-center outline-none rounded-box transition-colors duration-300 whitespace-nowrap"
      :class="[
        size === 'small' ? 'h-6 text-xs' : 'h-8 py-1 text-sm',
        iconOnly
          ? (size === 'small' ? 'px-0 w-6 justify-center' : 'px-0 w-8 justify-center')
          : (size === 'small' ? 'px-1.5 w-full gap-1 border' : 'px-2 w-full gap-1 border'),
        iconOnly ? '' : (selected ? 'border-primary' : 'border-base-content/15'),
        disabled
          ? 'text-base-content/30 cursor-default'
          : (selected ? 'text-primary' : 'text-base-content/70'),
        disabled ? '' : 'hover:bg-base-100/30 hover:text-base-content cursor-pointer'
      ]"
      :disabled="disabled"
      @click="toggleDropdown"
    >
      <component v-if="icon" :is="icon" class="w-4 h-4 shrink-0" />
      <span v-if="!iconOnly">{{ triggerLabel }}</span>
      <slot name="trigger-suffix" />
      <IconArrowDown v-if="!iconOnly" class="w-3 h-3 shrink-0 opacity-50" />
    </button>

    <!-- Dropdown Menu -->
    <teleport to="body">
      <transition name="fade">
        <div v-if="isDropDown"
          ref="menu"
          class="menu mt-1 text-base-content/70 bg-base-200/80 backdrop-blur-md border border-base-content/30 absolute rounded-box shadow-lg z-500"
          :style="menuStyle"
        >
          <!-- menu group 1 -->
          <template v-for="(option, index) in options" :key="index">
            <button
              class="p-1 flex flex-row text-sm whitespace-nowrap"
              :disabled="optionsDisabled"
              :class="optionsDisabled ? 'opacity-40 cursor-default' : 'hover:bg-base-100/30 hover:text-base-content hover:rounded-box cursor-pointer'"
              @click="selectOption(index)"
            >
              <div class="w-5">
                <IconOk v-if="multiSelect ? normalizedSelectedValues.includes(Number(option.value)) : false" class="w-4 h-4" />
                <IconDot v-else-if="!multiSelect && optionIndex === index" class="w-5" />
              </div>
              <span class="mr-4">{{ option.label }}</span>
            </button>
            <div v-if="separatorsAfter.includes(index)" class="mx-2 my-1 border-t border-base-content/10"></div>
          </template>

          <slot name="menu-after" />

          <!-- seperator -->
          <div v-if="extendOptions.length > 0 " class="mx-2 my-1 border-t border-base-content/10"></div>

          <!-- menu group 2 -->
          <button v-for="(option, index) in extendOptions"
            :class="[
              'p-1 flex flex-row text-sm whitespace-nowrap',
              extendDisabled
                ? 'text-base-content/30 cursor-default'
                : 'hover:bg-base-100/30 hover:text-base-content hover:rounded-box cursor-pointer',
            ]"
            :key="index"
            :disabled="extendDisabled"
            @click="selectExtendOption(index)"
          >
            <IconDot v-if="extendIndex === index" class="w-5" /> 
            <span v-else class="w-5"></span>
            <span>{{ option.label }}</span>
          </button>

        </div>
      </transition>
    </teleport>

  </div>
  
</template>
  
<script setup>
import { computed, ref, onMounted, onBeforeUnmount, nextTick, watch } from 'vue';
import { IconArrowDown, IconDot, IconOk } from '@/common/icons';

  // Props
const props = defineProps({
  options: {
    type: Array,
    required: true,
  },
  defaultIndex: {
    type: Number,
    default: 0,
  },
  extendOptions: {
    type: Array,
    default: () => [],
  },
  defaultExtendIndex: {
    type: Number,
    default: 0,
  },
  disabled: {
    type: Boolean,
    default: false,
  },
  optionsDisabled: {
    type: Boolean,
    default: false,
  },
  extendDisabled: {
    type: Boolean,
    default: false,
  },
  selected: {
    type: Boolean,
    default: false,
  },
  multiSelect: {
    type: Boolean,
    default: false,
  },
  selectedValues: {
    type: Array,
    default: () => [],
  },
  summaryLabel: {
    type: String,
    default: '',
  },
  separatorsAfter: {
    type: Array,
    default: () => [],
  },
  icon: {
    type: Object,
    default: null,
  },
  iconOnly: {
    type: Boolean,
    default: false,
  },
  size: {
    type: String,
    default: 'medium', // 'small' | 'medium'
  },
  title: {
    type: String,
    default: '',
  },
});

// Emits
const emit = defineEmits(['select', 'multi-select']);

// State
const dropdown = ref(null);
const menu = ref(null);
const isDropDown = ref(false);
const menuStyle = ref({});
const optionIndex = ref(props.defaultIndex);
const extendIndex = ref(props.defaultExtendIndex);
const normalizedSelectedValues = computed(() =>
  Array.isArray(props.selectedValues) ? props.selectedValues.map(value => Number(value)) : []
);
const triggerLabel = computed(() => {
  if (props.multiSelect) {
    return props.summaryLabel || props.options[0]?.label || '';
  }
  return props.options[optionIndex.value]?.label || '';
});

watch(() => props.defaultIndex, (value) => {
  optionIndex.value = value;
});

watch(() => props.defaultExtendIndex, (value) => {
  extendIndex.value = value;
});

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside, { capture: true });
  document.addEventListener('keydown', handleKeyDown);
});

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleClickOutside, { capture: true });
  document.removeEventListener('keydown', handleKeyDown);
});

// Handle Escape key press
const handleKeyDown = (event) => {
  if (isDropDown.value && event.key === 'Escape') {
    isDropDown.value = false;
  }
};

const toggleDropdown = async () => {
  if (props.disabled) return;
  isDropDown.value = !isDropDown.value;

  if (isDropDown.value) {
    await nextTick(); // Ensure menu is rendered before measuring

    const rect = dropdown.value.getBoundingClientRect();
    const menuRect = menu.value.getBoundingClientRect();
    
    const padding = 8; 
    const menuWidth = menuRect.width;
    const menuHeight = menuRect.height;
    const winWidth = window.innerWidth;
    const winHeight = window.innerHeight;
    const scrollY = window.scrollY;

    let top = rect.bottom + scrollY;
    let left = rect.left + window.scrollX;

    // Check bottom boundary
    if (top + menuHeight > winHeight + scrollY - padding) {
      // Not enough space below, try to place above
      top = rect.top - menuHeight + scrollY;
    }

    // Check top boundary (after potentially flipping)
    if (top < scrollY + padding) {
      // Still not enough space (menu is too tall), align to top
      top = scrollY + padding;
    }

    // Check right boundary
    if (left + menuWidth > winWidth - padding) {
      // Align to the right edge
      left = winWidth - menuWidth - padding;
    }

    // Check left boundary
    if (left < padding) {
      // Align to the left edge
      left = padding;
    }

    menuStyle.value = { top: `${top}px`, left: `${left}px` };
  }
};

// Close dropdown when clicking outside
const handleClickOutside = (event) => {
  if (dropdown.value && !dropdown.value.contains(event.target) && menu.value && !menu.value.contains(event.target)) {
    isDropDown.value = false;
  }
};

const selectOption = (index) => {
  if (props.disabled || props.optionsDisabled) return;
  if (props.multiSelect) {
    const optionValue = Number(props.options[index]?.value);
    if (Number.isNaN(optionValue)) return;
    let nextValues = normalizedSelectedValues.value;
    if (optionValue === 0) {
      nextValues = [0];
    } else if (normalizedSelectedValues.value.includes(optionValue)) {
      nextValues = normalizedSelectedValues.value.filter(value => value !== optionValue && value !== 0);
    } else {
      nextValues = [...normalizedSelectedValues.value.filter(value => value !== 0), optionValue];
    }
    emit('multi-select', nextValues);
    return;
  }
  optionIndex.value = index;
  emit('select', optionIndex.value, extendIndex.value);
  isDropDown.value = false;
};

const selectExtendOption = (index) => { 
  if (props.extendDisabled) return;
  extendIndex.value = index;
  emit('select', optionIndex.value, extendIndex.value);
  isDropDown.value = false;
};

</script>
