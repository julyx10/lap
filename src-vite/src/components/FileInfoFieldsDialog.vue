<template>
  <ModalDialog class="w-115" :title="$t('file_info.customize_fields')" position-key="file-info-fields" @cancel="emit('cancel')">
    <div class="overflow-y-auto max-h-[65vh] space-y-2 p-1 select-none">
      <section aria-labelledby="info-fields-general-heading" class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5">
        <h2 id="info-fields-general-heading" class="flex items-center font-bold uppercase text-xs tracking-widest text-base-content/30">{{ $t('file_info.general') }}</h2>
        <label class="flex items-center justify-between gap-4 text-sm">
          <span>{{ $t('file_info.field_preset') }}</span>
          <select class="select select-bordered select-sm w-40 shrink-0" :value="preset" @change="applyPreset(($event.target as HTMLSelectElement).value)">
            <option value="default">{{ $t('file_info.fields_default') }}</option>
            <option value="all">{{ $t('file_info.fields_all') }}</option>
            <option value="custom" disabled>{{ $t('file_info.fields_custom') }}</option>
          </select>
        </label>
        <label class="flex items-center justify-between gap-4 text-sm">
          <span>{{ $t('file_info.hide_empty_fields') }}</span>
          <input v-model="config.infoPanel.hideEmptyFields" type="checkbox" class="toggle toggle-primary toggle-sm shrink-0" />
        </label>
      </section>
      <section v-for="group in fileInfoFieldGroups" :key="group.key" class="rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5" :aria-labelledby="`info-fields-${group.key}`">
        <div class="flex items-center justify-between gap-2">
          <h2 :id="`info-fields-${group.key}`" class="flex items-center font-bold uppercase text-xs tracking-widest text-base-content/30">{{ $t(group.label) }}</h2>
          <button
            v-if="group.key === 'cameraSettings'"
            type="button"
            class="text-sm text-primary hover:underline cursor-pointer"
            @click="toggleVisibleCameraSettings"
          >
            {{ allVisibleCameraSettingsSelected ? $t('menu.select.none') : $t('menu.select.all') }}
          </button>
        </div>
        <div v-if="group.key === 'cameraSettings'" class="flex flex-wrap gap-1">
          <button
            v-for="brand in cameraBrandFilterOptions"
            :key="brand.key"
            type="button"
            class="px-2 py-0.5 rounded-full text-sm transition-colors cursor-pointer"
            :class="selectedCameraBrand === brand.key
              ? 'bg-primary text-primary-content font-medium'
              : 'bg-base-content/10 text-base-content/70 hover:bg-base-content/15'"
            @click="selectedCameraBrand = brand.key"
          >
            {{ brand.key === 'all' ? $t('file_info.all_brands') : brand.label }}
          </button>
        </div>
        <div class="grid grid-cols-2 gap-x-3 gap-y-2">
          <label v-for="field in visibleFieldsFor(group)" :key="field.key" class="flex items-center gap-2 text-sm">
            <input type="checkbox" class="checkbox checkbox-primary checkbox-sm shrink-0"
              :checked="selected.includes(field.key)"
              @change="toggle(field.key, ($event.target as HTMLInputElement).checked)" />
            <span>{{ field.label === 'GPS' ? 'GPS' : $t(field.label) }}</span>
          </label>
        </div>
      </section>
    </div>
    <div class="flex justify-end gap-2 pt-4 select-none">
      <button class="t-button-primary" @click="emit('cancel')">{{ $t('msgbox.close') }}</button>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { computed, ref, watch, onMounted, onBeforeUnmount } from 'vue';
import ModalDialog from '@/components/ModalDialog.vue';
import { config } from '@/common/config';
import {
  fileInfoFieldGroups,
  allFileInfoFields,
  defaultFileInfoFields,
  normalizeFileInfoFields,
  cameraSettingsBrands,
  filterCameraSettingsFields,
  type CameraSettingsBrand,
} from '@/common/fileInfoFields';
import { useUIStore } from '@/stores/uiStore';

const emit = defineEmits(['cancel']);
const uiStore = useUIStore();
const selected = computed(() => normalizeFileInfoFields(config.infoPanel.fields));
const selectedCameraBrand = ref<'all' | CameraSettingsBrand>('all');
const cameraBrandFilterOptions: ReadonlyArray<{ key: 'all' | CameraSettingsBrand; label: string }> = [
  { key: 'all', label: 'All' },
  ...cameraSettingsBrands,
];
const visibleCameraSettingsFields = computed(() => filterCameraSettingsFields(selectedCameraBrand.value));
const allVisibleCameraSettingsSelected = computed(() =>
  visibleCameraSettingsFields.value.length > 0
  && visibleCameraSettingsFields.value.every(field => selected.value.includes(field.key))
);
function visibleFieldsFor(group: (typeof fileInfoFieldGroups)[number]) {
  return group.key === 'cameraSettings' ? visibleCameraSettingsFields.value : group.fields;
}
function toggleVisibleCameraSettings() {
  const keys = visibleCameraSettingsFields.value.map(field => field.key);
  const next = allVisibleCameraSettingsSelected.value
    ? selected.value.filter(key => !keys.includes(key))
    : [...selected.value, ...keys];
  config.infoPanel.fields = normalizeFileInfoFields(next);
}
const matchesFields = (fields: string[]) => selected.value.length === fields.length && fields.every(key => selected.value.includes(key));
const preset = ref(matchesFields(defaultFileInfoFields) ? 'default' : matchesFields(allFileInfoFields) ? 'all' : 'custom');
watch(selected, () => { preset.value = 'custom'; }, { flush: 'sync' });
function applyPreset(value: string) {
  if (value === 'default') config.infoPanel.fields = [...defaultFileInfoFields];
  else if (value === 'all') config.infoPanel.fields = [...allFileInfoFields];
  preset.value = value;
}
function toggle(key: string, checked: boolean) {
  config.infoPanel.fields = normalizeFileInfoFields(checked ? [...selected.value, key] : selected.value.filter(value => value !== key));
}
function onKeyDown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    event.stopImmediatePropagation();
    emit('cancel');
  }
}
onMounted(() => {
  uiStore.pushInputHandler('FileInfoFieldsDialog');
  window.addEventListener('keydown', onKeyDown);
});
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeyDown);
  uiStore.removeInputHandler('FileInfoFieldsDialog');
});
</script>
