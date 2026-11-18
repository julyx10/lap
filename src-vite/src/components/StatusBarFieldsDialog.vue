<template>
  <ModalDialog :title="$t('statusbar.customize_title')" :width="400" @cancel="emit('cancel')">
    <VueDraggable
      v-model="fields"
      handle=".status-bar-drag-handle"
      direction="vertical"
      :group="{ name: 'status-bar-fields', pull: false, put: false }"
      :force-fallback="true"
      :revert-on-spill="true"
      :bubble-scroll="false"
      ghost-class="sidebar-reorder-placeholder"
      :animation="150"
      class="relative transform-gpu space-y-1 overflow-x-hidden overflow-y-auto max-h-[60vh]"
    >
      <div v-for="field in fields" :key="field.key" class="flex items-center gap-2 rounded-box bg-base-300/30 px-2 py-1 text-sm">
        <IconDragHandle class="status-bar-drag-handle p-1 w-6 h-6 shrink-0 cursor-grab text-base-content/30 touch-none" aria-hidden="true" />
        <label class="flex min-w-0 flex-1 items-center gap-2">
          <input
            type="checkbox"
            class="checkbox checkbox-primary checkbox-sm"
            :checked="field.visible"
            @change="setFieldVisibility(field.key, ($event.target as HTMLInputElement).checked)"
          />
          <span>{{ $t(statusBarFields[field.key]) }}</span>
        </label>
      </div>
    </VueDraggable>
    <div class="mt-4 flex justify-between gap-2">
      <button class="t-button-default" @click="fields = normalizeStatusBarFields(null)">{{ $t('statusbar.restore_defaults') }}</button>
      <button class="t-button-primary" @click="emit('cancel')">{{ $t('msgbox.close') }}</button>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from 'vue';
import { emit as emitWindowEvent } from '@tauri-apps/api/event';
import { VueDraggable } from 'vue-draggable-plus';
import ModalDialog from '@/components/ModalDialog.vue';
import { config } from '@/common/config';
import { IconDragHandle } from '@/common/icons';
import { normalizeStatusBarFields, statusBarFields } from '@/common/statusBarFields';
import { useUIStore } from '@/stores/uiStore';

const emit = defineEmits(['cancel']);
const ui = useUIStore();

const fields = computed({
  get: () => normalizeStatusBarFields(config.settings.statusBarFields),
  set: value => {
    const updated = normalizeStatusBarFields(value);
    config.settings.statusBarFields = updated;
    void emitWindowEvent('status-bar-fields-changed', updated);
  },
});

function setFieldVisibility(key: string, visible: boolean) {
  fields.value = fields.value.map(field => field.key === key ? { ...field, visible } : field);
}

function onKeyDown(event: KeyboardEvent) {
  if (event.key !== 'Escape') return;
  event.preventDefault();
  event.stopImmediatePropagation();
  emit('cancel');
}

onMounted(() => {
  ui.pushInputHandler('StatusBarFieldsDialog');
  window.addEventListener('keydown', onKeyDown, true);
});
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeyDown, true);
  ui.removeInputHandler('StatusBarFieldsDialog');
});
</script>
