<template>
  <div ref="dropdown" class="relative inline-block text-left">

    <!-- Dropdown Trigger -->
    <slot name="trigger" :toggle="toggleDropdown">
      <!-- Default trigger button when no slot provided -->
      <TButton
        :icon="rawIconMenu"
        :buttonSize="smallIcon ? 'small' : 'medium'"
        :disabled="disabled"
        @click="toggleDropdown" 
        @dblclick.stop
      />
    </slot>
    <!-- Dropdown Menu -->
    <teleport to="body">
      <transition name="fade">
        <div v-if="isDropDown" 
          ref="menu"
          class="menu text-base-content/70 bg-base-200/80 backdrop-blur-md border border-base-content/30 absolute rounded-box shadow-lg z-500"
          :style="menuStyle"
        >
          <!-- menu items -->
          <template v-for="(item, index) in resolvedMenuItems">
            <div
              v-if="!item.hidden"
              :key="index"
              class="relative"
              @mouseenter="handleItemMouseEnter(index, item, $event)"
              @mouseleave="handleItemMouseLeave(index)"
            >
              <div
                v-if="item.label === '-'"
                class="mx-2 my-1 border-t border-base-content/5"
              ></div>
              <button
                v-else
                :class="[
                  'w-full px-2 py-1 flex justify-between text-sm whitespace-nowrap',
                  item.disabled ? 'text-base-content/30' : 'hover:bg-base-100/30 hover:text-base-content hover:rounded-box cursor-pointer group',
                ]"
                @click="handleMainItemClick(index, item, $event)"
              >
                <div class="w-full flex items-center">
                  <div class="w-5">
                    <component class="t-icon-size-sm" :is="item.icon" ></component>
                  </div>
                  <span class="ml-2 mr-4">{{ item.label }}</span>
                  <span class="ml-auto text-base-content/30">{{ item.shortcut }}</span>
                  <IconRight v-if="item.children?.length" class="ml-2 w-3 h-3 text-base-content/70 group-hover:text-base-content"/>
                </div>
              </button>
            </div>
          </template>
        </div>
      </transition> 
    </teleport>
    <teleport to="body">
      <transition name="fade">
        <div
          v-if="activeSubmenuItem?.children?.length"
          ref="submenu"
          class="menu fixed text-base-content/70 bg-base-200/80 backdrop-blur-md border border-base-content/30 rounded-box shadow-lg min-w-max z-510"
          :style="submenuStyle"
          @mouseenter="cancelSubmenuClose"
          @mouseleave="scheduleSubmenuClose(activeSubmenuIndex)"
        >
          <template v-for="(child, childIndex) in activeSubmenuItem.children">
            <div
              v-if="!child.hidden"
              :key="childIndex"
            >
              <div
                v-if="child.label === '-'"
                class="mx-2 my-1 border-t border-base-content/5"
              />
              <button
                v-else
                :class="[
                  'w-full px-2 py-1 flex justify-between text-sm whitespace-nowrap',
                  child.disabled ? 'text-base-content/30' : 'hover:bg-base-100/30 hover:text-base-content hover:rounded-box cursor-pointer',
                ]"
                @click="handleLeafClick(child)"
              >
                <div class="w-full flex items-center">
                  <div class="w-5">
                    <component class="t-icon-size-sm" :is="child.icon" ></component>
                  </div>
                  <span class="ml-2 mr-4">{{ child.label }}</span>
                  <span class="ml-auto text-base-content/30">{{ child.shortcut }}</span>
                </div>
              </button>
            </div>
          </template>
        </div>
      </transition>
    </teleport>

  </div>
  
</template>
  
<script setup>
import { ref, shallowRef, onBeforeUnmount, nextTick, watch, computed, markRaw } from 'vue';

import TButton from '@/components/TButton.vue';
import { IconRight } from '@/common/icons';

// Props
const props = defineProps({
  iconMenu : {
    type: Object, // SVG is typically imported as an object
    required: false, // Not required when using slot
    default: null,
  },
  menuItems: {
    type: [Array, Function], // Accept Array or Function for lazy generation
    required: true,
  },
  smallIcon: {
    type: Boolean,
    default: false,
  },
  disabled: {
    type: Boolean,
    default: false,    
  }
});

const rawIconMenu = computed(() => (props.iconMenu ? markRaw(props.iconMenu) : null));

// Resolved menu items (computed on demand)
// Use shallowRef to avoid making icon components reactive
const resolvedMenuItems = shallowRef([]);

// Emits
const emit = defineEmits(['open-change']);

// State
const dropdown = ref(null);
const menu = ref(null);
const submenu = ref(null);
const isDropDown = ref(false);
const menuStyle = ref({});
const submenuStyle = ref({});
const activeSubmenuIndex = ref(null);
const activeSubmenuItem = ref(null);
let submenuCloseTimeout = null;
let submenuOpenTimeout = null;

// Remove event listener when the component is destroyed
onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleClickOutside, { capture: true });
  document.removeEventListener('keydown', handleKeyDown);
  cancelSubmenuClose();
  cancelSubmenuOpen();
});

// Watch for dropdown state to attach/detach listeners
watch(isDropDown, (val) => {
  emit('open-change', val);
  if (val) {
    document.addEventListener('mousedown', handleClickOutside, { capture: true });
    document.addEventListener('keydown', handleKeyDown);
  } else {
    document.removeEventListener('mousedown', handleClickOutside, { capture: true });
    document.removeEventListener('keydown', handleKeyDown);
    activeSubmenuIndex.value = null;
    activeSubmenuItem.value = null;
    cancelSubmenuClose();
    cancelSubmenuOpen();
  }
});

// Handle Escape key press
const handleKeyDown = (event) => {
  if (isDropDown.value && event.key === 'Escape') {
    event.stopPropagation();
    isDropDown.value = false;
  }
};

// Open dropdown menu at specific coordinates or relative to the trigger
const open = async (x, y) => {
  if (props.disabled) return;

  isDropDown.value = true;

  // Resolve menu items (call function if provided)
  resolvedMenuItems.value = typeof props.menuItems === 'function' 
    ? props.menuItems() 
    : props.menuItems;

  await nextTick(); // Ensure menu is rendered before measuring

  const menuRect = menu.value.getBoundingClientRect();
  
  const padding = 8; // A smaller padding for a snug fit.
  const menuWidth = menuRect.width;
  const menuHeight = menuRect.height;
  const winWidth = window.innerWidth;
  const winHeight = window.innerHeight;
  const scrollY = window.scrollY;

  let top, left;

  if (x !== undefined && y !== undefined) {
    // Open at specific coordinates (e.g., context menu)
    top = y + scrollY;
    left = x + window.scrollX;
  } else {
    // Open relative to trigger button
    const rect = dropdown.value.getBoundingClientRect();
    top = rect.bottom + scrollY;
    left = rect.left + window.scrollX;

    // Check bottom boundary for trigger-relative positioning
    if (top + menuHeight > winHeight + scrollY - padding) {
      top = rect.top - menuHeight + scrollY;
    }
  }

  // Common boundary checks
  // Check bottom boundary (for context menu or general overflow)
  if (top + menuHeight > winHeight + scrollY - padding) {
     top = winHeight + scrollY - menuHeight - padding;
  }

  // Check top boundary
  if (top < scrollY + padding) {
    top = scrollY + padding;
  }

  // Check right boundary
  if (left + menuWidth > winWidth - padding) {
    left = winWidth - menuWidth - padding;
  }

  // Check left boundary
  if (left < padding) {
    left = padding;
  }

  menuStyle.value = { top: `${top}px`, left: `${left}px` };
};

// Toggle dropdown menu
const toggleDropdown = async () => {
  if (isDropDown.value) {
    isDropDown.value = false;
  } else {
    open();
  }
};

// Close dropdown when clicking outside
const handleClickOutside = (event) => {
  if (
    dropdown.value &&
    !dropdown.value.contains(event.target) &&
    menu.value &&
    !menu.value.contains(event.target) &&
    (!submenu.value || !submenu.value.contains(event.target))
  ) {
    isDropDown.value = false;
  }
};

const positionSubmenu = async (targetEl) => {
  await nextTick();
  if (!submenu.value || !targetEl || !menu.value) return;

  const buttonEl = targetEl.querySelector('button') || targetEl;
  const parentButtonRect = buttonEl.getBoundingClientRect();
  const menuRect = menu.value.getBoundingClientRect();
  const submenuRect = submenu.value.getBoundingClientRect();
  const padding = 8;
  let left = menuRect.right - 4;
  let top = parentButtonRect.top;

  if (left + submenuRect.width > window.innerWidth - padding) {
    left = menuRect.left - submenuRect.width + 4;
  }

  if (left < padding) {
    left = padding;
  }

  if (top + submenuRect.height > window.innerHeight - padding) {
    top = window.innerHeight - submenuRect.height - padding;
  }

  if (top < padding) {
    top = padding;
  }

  submenuStyle.value = {
    left: `${left}px`,
    top: `${top}px`,
  };

  await nextTick();

  const firstSubmenuButton = submenu.value?.querySelector('button');
  if (firstSubmenuButton) {
    const firstSubmenuButtonRect = firstSubmenuButton.getBoundingClientRect();
    const offset = parentButtonRect.top - firstSubmenuButtonRect.top;
    top += offset;

    if (top + submenuRect.height > window.innerHeight - padding) {
      top = window.innerHeight - submenuRect.height - padding;
    }

    if (top < padding) {
      top = padding;
    }
  }

  submenuStyle.value = {
    left: `${left}px`,
    top: `${top}px`,
  };
};

const handleItemMouseEnter = (index, item, event) => {
  cancelSubmenuClose();
  cancelSubmenuOpen();
  activeSubmenuIndex.value = item.children?.length ? index : null;
  if (item.children?.length) {
    const targetEl = event?.currentTarget;
    const openDelay = Number(item.submenuOpenDelay || 0);
    if (openDelay > 0) {
      submenuOpenTimeout = window.setTimeout(() => {
        activeSubmenuItem.value = item;
        void positionSubmenu(targetEl);
        submenuOpenTimeout = null;
      }, openDelay);
    } else {
      activeSubmenuItem.value = item;
      void positionSubmenu(targetEl);
    }
  } else {
    activeSubmenuItem.value = null;
  }
};

const handleItemMouseLeave = (index) => {
  if (activeSubmenuIndex.value === index) {
    scheduleSubmenuClose(index);
  }
};

const scheduleSubmenuClose = (index) => {
  cancelSubmenuClose();
  cancelSubmenuOpen();
  submenuCloseTimeout = window.setTimeout(() => {
    if (activeSubmenuIndex.value === index) {
      activeSubmenuIndex.value = null;
      activeSubmenuItem.value = null;
    }
    submenuCloseTimeout = null;
  }, 180);
};

const cancelSubmenuClose = () => {
  if (submenuCloseTimeout !== null) {
    window.clearTimeout(submenuCloseTimeout);
    submenuCloseTimeout = null;
  }
};

const cancelSubmenuOpen = () => {
  if (submenuOpenTimeout !== null) {
    window.clearTimeout(submenuOpenTimeout);
    submenuOpenTimeout = null;
  }
};

const handleLeafClick = (item) => {
  if (!item.disabled && item.action && typeof item.action === 'function') {
    item.action();
    activeSubmenuItem.value = null;
    isDropDown.value = false;
  }
};

const handleMainItemClick = (index, item, event) => {
  if (item.children?.length) {
    cancelSubmenuClose();
    cancelSubmenuOpen();
    activeSubmenuIndex.value = index;
    activeSubmenuItem.value = item;
    const target = event?.currentTarget?.closest('.relative') || event?.currentTarget;
    void positionSubmenu(target);
    return;
  }
  handleLeafClick(item);
};

defineExpose({
  open
});


</script>
