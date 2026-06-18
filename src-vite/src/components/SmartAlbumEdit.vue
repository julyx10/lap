<template>
  <ModalDialog :title="isNew ? $t('album.smart_edit.title_add') : $t('album.smart_edit.title_edit')" :width="620" @cancel="clickCancel">
    <section
      class="min-h-0 overflow-y-auto pr-1"
      @dragenter.stop
      @dragover.stop
      @dragleave.stop
      @drop.stop
    >
      <div class="w-full grid grid-cols-[92px_1fr] gap-x-4 gap-y-3 items-start text-xs select-none">
        <div class="h-8 flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/70">{{ $t('album.smart_edit.name') }}</div>
        <input ref="inputNameRef" v-model="name" type="text" maxlength="255" class="w-full input input-sm text-xs font-semibold" :placeholder="$t('album.smart_edit.validation_name')" />

        <div class="h-8 flex items-start pt-2 text-[10px] uppercase tracking-widest font-bold text-base-content/70">{{ $t('album.smart_edit.description') }}</div>
        <textarea
          v-if="showDescription"
          ref="descriptionRef"
          v-model="description"
          rows="2"
          maxlength="1024"
          class="w-full textarea textarea-sm min-h-[56px] max-h-[120px] text-xs font-semibold"
        ></textarea>
        <TButton
          v-else
          :icon="IconEdit"
          :buttonSize="'small'"
          :tooltip="$t('album.smart_edit.description')"
          @click="showDescriptionInput"
        />

        <div class="h-8 flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/70">{{ $t('album.smart_edit.match') }}</div>
        <div class="flex h-8 items-center gap-4">
          <label class="flex items-center gap-1.5 text-xs cursor-pointer">
            <input v-model="matchMode" type="radio" value="all" class="radio radio-xs radio-primary" />
            {{ $t('album.smart_edit.match_all') }}
          </label>
          <label class="flex items-center gap-1.5 text-xs cursor-pointer">
            <input v-model="matchMode" type="radio" value="any" class="radio radio-xs radio-primary" />
            {{ $t('album.smart_edit.match_any') }}
          </label>
        </div>

        <div class="pt-2 text-[10px] uppercase tracking-widest font-bold text-base-content/70">{{ $t('album.smart_edit.rules') }}</div>
        <div class="space-y-2">
          <VueDraggable
            v-model="rules"
            handle=".smart-rule-drag-handle"
            draggable=".smart-rule-row"
            :animation="150"
            :forceFallback="true"
            :fallbackOnBody="true"
            :fallbackTolerance="3"
            ghostClass="smart-rule-ghost"
            chosenClass="smart-rule-chosen"
            fallbackClass="smart-rule-fallback"
            :class="[
              'space-y-1.5 pr-1',
              rules.length > 6 ? 'max-h-[252px] overflow-y-auto' : ''
            ]"
            @dragenter.stop
            @dragover.stop
            @dragleave.stop
            @drop.stop
          >
            <div
              v-for="(rule, index) in rules"
              :key="rule.id"
              class="smart-rule-row grid grid-cols-[28px_minmax(0,1fr)_88px_minmax(0,1.45fr)_28px] gap-2 items-center rounded-box px-1 py-1 transition-colors hover:bg-base-content/5"
            >
              <div class="smart-rule-drag-handle cursor-move text-base-content/40 hover:text-base-content/70">
                <IconDragHandle class="w-4 h-4 mx-auto" />
              </div>
              <select v-model="rule.field" class="select select-sm text-xs w-full" @change="resetRule(rule)">
                <option v-for="field in fieldOptions" :key="field.value" :value="field.value">{{ field.label }}</option>
              </select>
              <select v-model="rule.operator" class="select select-sm text-xs w-[88px]" @change="resetRuleValue(rule)">
                <option v-for="operator in getOperatorOptions(rule.field)" :key="operator.value" :value="operator.value">{{ operator.label }}</option>
              </select>
              <RuleValueControl
                :rule="rule"
                :tag-options="tagOptions"
                :person-options="personOptions"
                :camera-options="cameraOptions"
                :lens-options="lensOptions"
                :location-options="locationOptions"
                :file-type-options="fileTypeOptions"
                :extension-options="extensionOptions"
                :date-field-options="dateFieldOptions"
              />
              <TButton
                :icon="IconRemove"
                :buttonSize="'small'"
                :tooltip="$t('album.smart_edit.remove_rule')"
                :disabled="rules.length <= 1"
                @click="removeRule(index)"
              />
            </div>
          </VueDraggable>
          <button class="btn btn-sm rounded-box" :disabled="rules.length >= maxRules" @click="addRule">
            <IconAdd class="w-4 h-4" />
            {{ $t('album.smart_edit.add_rule') }}
          </button>
        </div>
      </div>
    </section>

    <div v-if="validationMessage" class="mt-3 text-xs text-warning">{{ validationMessage }}</div>

    <div class="mt-4 flex justify-end space-x-4">
      <button class="t-button-default" @click="clickCancel">{{ $t('msgbox.cancel') }}</button>
      <button class="t-button-primary" :disabled="!canSave" @click="clickOk">{{ $t('msgbox.ok') }}</button>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { computed, defineComponent, h, nextTick, onBeforeUnmount, onMounted, ref, Teleport, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { VueDraggable } from 'vue-draggable-plus';
import { getAllTags, getCameraInfo, getLensInfo, getLocationInfo, getPersons, getSupportedFormatExtensions } from '@/common/api';
import { IconAdd, IconDragHandle, IconEdit, IconRemove } from '@/common/icons';
import ModalDialog from '@/components/ModalDialog.vue';
import TButton from '@/components/TButton.vue';

const props = defineProps({
  smartAlbum: {
    type: Object,
    default: null,
  },
});

const emit = defineEmits(['ok', 'cancel']);
const { t, locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const inputNameRef = ref<HTMLInputElement | null>(null);
const descriptionRef = ref<HTMLTextAreaElement | null>(null);
const maxRules = 20;

const isNew = computed(() => !props.smartAlbum);
const name = ref(props.smartAlbum?.name || '');
const description = ref(props.smartAlbum?.description || '');
const showDescription = ref(description.value.trim().length > 0);
const matchMode = ref(props.smartAlbum?.query?.match === 'any' ? 'any' : 'all');

const tagOptions = ref<any[]>([]);
const personOptions = ref<any[]>([]);
const cameraOptions = ref<any[]>([]);
const lensOptions = ref<any[]>([]);
const locationOptions = ref<any[]>([]);
const extensionOptions = ref<any[]>([]);
const imageExtensionValues = ref<Set<string>>(new Set());
const rawExtensionValues = ref<Set<string>>(new Set());
const videoExtensionValues = ref<Set<string>>(new Set());
const rules = ref<any[]>([]);

const fileTypeOptions = computed(() => [
  { value: 1, label: localeMsg.value.toolbar.filter?.file_type_options?.[1] || 'Image' },
  { value: 4, label: localeMsg.value.toolbar.filter?.file_type_options?.[2] || 'RAW' },
  { value: 2, label: localeMsg.value.toolbar.filter?.file_type_options?.[3] || 'Video' },
]);

const fieldOptions = computed(() => [
  { value: 'file_type', label: t('album.smart_edit.fields.file_type') },
  { value: 'favorite', label: t('album.smart_edit.fields.favorite') },
  { value: 'rating', label: t('album.smart_edit.fields.rating') },
  { value: 'date', label: t('album.smart_edit.fields.date') },
  { value: 'size', label: t('album.smart_edit.fields.size') },
  { value: 'width', label: t('album.smart_edit.fields.width') },
  { value: 'height', label: t('album.smart_edit.fields.height') },
  { value: 'duration', label: t('album.smart_edit.fields.duration') },
  { value: 'has_gps', label: t('album.smart_edit.fields.has_gps') },
  { value: 'tag', label: t('album.smart_edit.fields.tag') },
  { value: 'person', label: t('album.smart_edit.fields.person') },
  { value: 'camera', label: t('album.smart_edit.fields.camera') },
  { value: 'lens', label: t('album.smart_edit.fields.lens') },
  { value: 'location', label: t('album.smart_edit.fields.location') },
]);

const dateFieldOptions = computed(() => [
  { value: 'taken', label: t('album.smart_edit.date_fields.taken') },
  { value: 'added', label: t('album.smart_edit.date_fields.added') },
  { value: 'modified', label: t('album.smart_edit.date_fields.modified') },
]);

const validationMessage = computed(() => {
  if (rules.value.length === 0) return t('album.smart_edit.validation_rule');
  if (!rules.value.every(isRuleValid)) return t('album.smart_edit.validation_rule');
  return '';
});
const canSave = computed(() => name.value.trim().length > 0 && validationMessage.value.length === 0);

function makeRule(field = 'file_type') {
  const rule = {
    id: crypto.randomUUID?.() || `${Date.now()}-${Math.random()}`,
    field,
    operator: getOperatorOptions(field)[0]?.value || 'is',
    value: null,
  };
  setDefaultRuleValue(rule);
  return rule;
}

function cloneRules(source: any[] | undefined) {
  const cloned = Array.isArray(source)
    ? source.map(rule => normalizeRuleForEdit(rule))
    : [];
  return cloned.length > 0 ? cloned.slice(0, maxRules) : [makeRule()];
}

function normalizeRuleForEdit(rule: any) {
  const sourceField = rule.field || 'file_type';
  const field = sourceField === 'extension' ? 'file_type' : sourceField;
  return {
    id: rule.id || crypto.randomUUID?.() || `${Date.now()}-${Math.random()}`,
    field,
    operator: rule.operator || getOperatorOptions(field)[0]?.value || 'is',
    value: normalizeRuleValueForEdit(sourceField, cloneValue(rule.value)),
  };
}

function getOperatorOptions(field: string) {
  const op = (key: string) => ({ value: key, label: t(`album.smart_edit.operators.${key}`) });
  if (['file_type', 'extension', 'camera', 'lens', 'location'].includes(field)) return [op('is'), op('is_not')];
  if (field === 'favorite' || field === 'has_gps') return [op('is')];
  if (field === 'rating') return [op('is'), op('is_not'), op('gt'), op('gte'), op('lt'), op('lte'), op('between'), op('empty'), op('not_empty')];
  if (field === 'date') return [op('before'), op('after'), op('between'), op('empty'), op('not_empty')];
  if (['size', 'width', 'height', 'duration'].includes(field)) return [op('gt'), op('gte'), op('lt'), op('lte'), op('between'), op('empty'), op('not_empty')];
  if (field === 'tag' || field === 'person') return [op('has'), op('not_has')];
  return [op('is')];
}

function resetRule(rule: any) {
  rule.operator = getOperatorOptions(rule.field)[0]?.value || 'is';
  setDefaultRuleValue(rule);
}

function resetRuleValue(rule: any) {
  setDefaultRuleValue(rule);
}

function setDefaultRuleValue(rule: any) {
  if (rule.field === 'file_type') rule.value = { fileType: 1, extension: '' };
  else if (rule.field === 'extension') rule.value = { fileType: 1, extension: 'jpg' };
  else if (rule.field === 'favorite' || rule.field === 'has_gps') rule.value = true;
  else if (rule.field === 'rating') rule.value = rule.operator === 'between' ? { min: 1, max: 5 } : 1;
  else if (rule.field === 'date') rule.value = rule.operator === 'between'
    ? { field: 'taken', start: dateToUnix(new Date()), end: dateToUnix(addDays(new Date(), 1)) }
    : { field: 'taken', value: dateToUnix(new Date()) };
  else if (rule.field === 'size') rule.value = rule.operator === 'between' ? { min: 1, max: 10 } : 1;
  else if (rule.field === 'duration') rule.value = rule.operator === 'between' ? { min: 60, max: 600 } : 60;
  else if (rule.field === 'width' || rule.field === 'height') rule.value = rule.operator === 'between' ? { min: 1000, max: 4000 } : 1000;
  else if (rule.field === 'tag') rule.value = tagOptions.value[0]?.value || null;
  else if (rule.field === 'person') rule.value = personOptions.value[0]?.value || null;
  else if (rule.field === 'camera') rule.value = cameraOptions.value[0]?.value || null;
  else if (rule.field === 'lens') rule.value = lensOptions.value[0]?.value || null;
  else if (rule.field === 'location') rule.value = locationOptions.value[0]?.value || null;
}

function isRuleValid(rule: any) {
  if (!rule.field || !rule.operator) return false;
  if (['empty', 'not_empty'].includes(rule.operator)) return true;
  if (rule.operator === 'between') {
    return Boolean(
      (rule.value && rule.value.min !== '' && rule.value.max !== '')
      || (rule.value && rule.value.start && rule.value.end)
    );
  }
  if (rule.field === 'file_type') return Boolean(getFileTypeValue(rule.value));
  if (rule.field === 'date') return Boolean(rule.value?.field && rule.value?.value);
  return rule.value !== null && rule.value !== undefined && rule.value !== '';
}

function addRule() {
  if (rules.value.length >= maxRules) return;
  rules.value.push(makeRule());
}

function removeRule(index: number) {
  if (rules.value.length <= 1) return;
  rules.value.splice(index, 1);
}

async function showDescriptionInput() {
  showDescription.value = true;
  await nextTick();
  descriptionRef.value?.focus();
}

function clickCancel() {
  emit('cancel');
}

function clickOk() {
  if (!canSave.value) return;
  const now = Math.floor(Date.now() / 1000);
  emit('ok', {
    id: props.smartAlbum?.id || crypto.randomUUID?.() || `${Date.now()}`,
    name: name.value.trim(),
    description: description.value.trim(),
    source: 'rules',
    query: {
      version: 1,
      match: matchMode.value,
      rules: rules.value.slice(0, maxRules).map(rule => ({
        id: rule.id,
        field: rule.field,
        operator: rule.operator,
        value: normalizeRuleValue(rule),
      })),
    },
    sort: { type: Number(props.smartAlbum?.sort?.type ?? 0), order: Number(props.smartAlbum?.sort?.order ?? 1) },
    createdAt: props.smartAlbum?.createdAt || now,
    updatedAt: now,
  });
}

function normalizeRuleValue(rule: any) {
  if (rule.field === 'file_type') {
    const fileType = getFileTypeValue(rule.value);
    const extension = getExtensionValue(rule.value);
    return extension ? { fileType, extension } : { fileType };
  }
  if (rule.field === 'size') return normalizeNumericValue(rule.value, 1_000_000);
  if (rule.field === 'date') return rule.value;
  return normalizeNumericValue(rule.value, 1);
}

function cloneValue(value: any) {
  if (value === undefined) return value;
  return JSON.parse(JSON.stringify(value));
}

function normalizeRuleValueForEdit(field: string, value: any) {
  if (field === 'file_type') {
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      return {
        fileType: getFileTypeValue(value),
        extension: getExtensionValue(value),
      };
    }
    return { fileType: getFileTypeValue(value), extension: '' };
  }
  if (field === 'extension') {
    const extension = Array.isArray(value) ? String(value[0] || '') : String(value || '');
    return { fileType: inferFileTypeFromExtension(extension), extension };
  }
  if (field !== 'size') return value;
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    const next: any = { ...value };
    if (typeof next.min === 'number') next.min = next.min / 1_000_000;
    if (typeof next.max === 'number') next.max = next.max / 1_000_000;
    return next;
  }
  return typeof value === 'number' ? value / 1_000_000 : value;
}

function getFileTypeValue(value: any) {
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    return Number(value.fileType ?? value.file_type ?? 1);
  }
  return Number(value || 1);
}

function getExtensionValue(value: any) {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return '';
  return String(value.extension || '').trim().replace(/^\./, '').toLowerCase();
}

function inferFileTypeFromExtension(extension: string) {
  const value = String(extension || '').trim().replace(/^\./, '').toLowerCase();
  if (rawExtensionValues.value.has(value)) return 4;
  if (videoExtensionValues.value.has(value)) return 2;
  return 1;
}

function getExtensionOptionsForFileType(fileType: number) {
  const valueSet = fileType === 4
    ? rawExtensionValues.value
    : fileType === 2
      ? videoExtensionValues.value
      : imageExtensionValues.value;
  return [
    { value: '', label: 'All' },
    ...extensionOptions.value.filter(option => valueSet.has(option.value)),
  ];
}

function normalizeNumericValue(value: any, multiplier: number) {
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    const next: any = {};
    for (const key of Object.keys(value)) {
      next[key] = typeof value[key] === 'number' ? Math.round(value[key] * multiplier) : value[key];
    }
    return next;
  }
  return typeof value === 'number' ? Math.round(value * multiplier) : value;
}

function dateToUnix(date: Date) {
  return Math.floor(new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime() / 1000);
}

function addDays(date: Date, days: number) {
  const next = new Date(date);
  next.setDate(next.getDate() + days);
  return next;
}

function unixToDateInput(value: number) {
  if (!value) return '';
  const date = new Date(value * 1000);
  const month = `${date.getMonth() + 1}`.padStart(2, '0');
  const day = `${date.getDate()}`.padStart(2, '0');
  return `${date.getFullYear()}-${month}-${day}`;
}

function dateInputToUnix(value: string) {
  if (!value) return 0;
  const [year, month, day] = value.split('-').map(Number);
  return Math.floor(new Date(year, month - 1, day).getTime() / 1000);
}

function isValidDateInput(value: string) {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) return false;
  const [year, month, day] = value.split('-').map(Number);
  const date = new Date(year, month - 1, day);
  return date.getFullYear() === year && date.getMonth() === month - 1 && date.getDate() === day;
}

function unixToDateInputInclusiveEnd(value: number) {
  if (!value) return '';
  return unixToDateInput(value - 1);
}

function dateInputToUnixExclusiveEnd(value: string) {
  if (!value) return 0;
  return dateInputToUnix(value) + 86400;
}

async function loadOptions() {
  const [tags, persons, cameras, lenses, locations] = await Promise.all([
    getAllTags(),
    getPersons(),
    getCameraInfo(),
    getLensInfo(),
    getLocationInfo(),
  ]);
  tagOptions.value = (tags || []).map((tag: any) => ({ value: tag.id, label: tag.name }));
  personOptions.value = (persons || []).map((person: any) => ({ value: person.id, label: person.name || `Person ${person.id}` }));
  cameraOptions.value = flattenMakeModelOptions(cameras || []);
  lensOptions.value = flattenMakeModelOptions(lenses || []);
  locationOptions.value = flattenLocationOptions(locations || []);
  for (const rule of rules.value) {
    if (rule.value === null || rule.value === undefined || rule.value === '') {
      setDefaultRuleValue(rule);
    }
  }
}

async function loadSupportedFormatOptions() {
  const formats = await getSupportedFormatExtensions();
  extensionOptions.value = (formats.options || [])
    .map((option: any) => ({
      value: String(option.value || '').toLowerCase(),
      label: option.label || String(option.value || '').toUpperCase(),
    }))
    .filter((option: any) => option.value)
    .sort((a: any, b: any) => a.value.localeCompare(b.value));
  imageExtensionValues.value = new Set((formats.image || []).map((value: string) => String(value).toLowerCase()));
  rawExtensionValues.value = new Set((formats.raw || []).map((value: string) => String(value).toLowerCase()));
  videoExtensionValues.value = new Set((formats.video || []).map((value: string) => String(value).toLowerCase()));
  rules.value = cloneRules(props.smartAlbum?.query?.rules);
}

function flattenMakeModelOptions(items: any[]) {
  const options: any[] = [];
  for (const item of items) {
    if (!item.make) continue;
    options.push({ value: `${item.make}||`, label: item.make });
    for (const model of item.models || []) {
      options.push({ value: `${item.make}||${model}`, label: `${item.make} › ${model}` });
    }
  }
  return options;
}

function flattenLocationOptions(items: any[]) {
  const options: any[] = [];
  for (const item of items) {
    if (!item.admin1) continue;
    options.push({ value: `${item.cc || ''}||${item.admin1}||`, label: item.admin1 });
    for (const name of item.names || []) {
      options.push({ value: `${item.cc || ''}||${item.admin1}||${name}`, label: `${item.admin1} › ${name}` });
    }
  }
  return options;
}

onMounted(async () => {
  await loadSupportedFormatOptions();
  await loadOptions();
  await nextTick();
  inputNameRef.value?.focus();
});

watch([tagOptions, personOptions, cameraOptions, lensOptions, locationOptions], () => {
  for (const rule of rules.value) {
    if (rule.value === null || rule.value === undefined || rule.value === '') setDefaultRuleValue(rule);
  }
});

const RuleValueControl = defineComponent({
  props: {
    rule: { type: Object, required: true },
    tagOptions: { type: Array, default: () => [] },
    personOptions: { type: Array, default: () => [] },
    cameraOptions: { type: Array, default: () => [] },
    lensOptions: { type: Array, default: () => [] },
    locationOptions: { type: Array, default: () => [] },
    fileTypeOptions: { type: Array, default: () => [] },
    extensionOptions: { type: Array, default: () => [] },
    dateFieldOptions: { type: Array, default: () => [] },
  },
  setup(props: any) {
    const suppressDatePopoverUntil = ref(0);
    const extensionMenuOpen = ref(false);
    const extensionTriggerRef = ref<HTMLElement | null>(null);
    const extensionMenuRef = ref<HTMLElement | null>(null);
    const extensionMenuStyle = ref<Record<string, string>>({
      left: '0px',
      top: '0px',
      minWidth: '0px',
    });
    const extensionMenuPlacement = ref<'top' | 'bottom'>('bottom');
    let removeExtensionMenuListeners: (() => void) | null = null;
    const inputClass = 'input input-sm text-xs w-full';
    const selectClass = 'select select-sm text-xs w-full';
    const numberInput = (value: any, onInput: any) => h('input', { class: inputClass, type: 'number', value, onInput });
    const inputWithUnit = (inputNode: any, unit: string) => {
      if (!unit) return inputNode;
      return h('label', { class: 'input input-sm w-full flex items-center gap-1 px-0 overflow-hidden' }, [
        h('div', { class: 'min-w-0 flex-1' }, [
          inputNode,
        ]),
        h('span', { class: 'pr-2 text-[10px] uppercase text-base-content/40 shrink-0' }, unit),
      ]);
    };
    const unitNumberInput = (value: any, unit: string, onInput: any) => inputWithUnit(
      h('input', {
        class: 'w-full min-w-0 bg-transparent px-2 text-xs outline-none',
        type: 'number',
        value,
        onInput,
      }),
      unit,
    );
    const calendarIcon = (label: string, path: string, slot: string) => h('svg', {
      'aria-label': label,
      class: 'fill-current size-4',
      slot,
      xmlns: 'http://www.w3.org/2000/svg',
      viewBox: '0 0 24 24',
    }, [
      h('path', { fill: 'currentColor', d: path }),
    ]);
    const closeDateDropdown = (popoverId: string) => {
      const popover = document.getElementById(popoverId) as HTMLElement & { hidePopover?: () => void } | null;
      popover?.hidePopover?.();
    };
    const dateInput = (value: string, onSelect: any, key: string) => {
      const popoverId = `smart-date-${props.rule.id}-${key}`;
      const calendarId = `${popoverId}-calendar`;
      const anchorName = `--${popoverId}`;
      const showDateDropdown = () => {
        if (Date.now() < suppressDatePopoverUntil.value) return;
        const popover = document.getElementById(popoverId) as HTMLElement & { showPopover?: () => void } | null;
        popover?.showPopover?.();
      };
      const closeDateDropdownAfterSelect = () => {
        suppressDatePopoverUntil.value = Date.now() + 250;
        setTimeout(() => {
          closeDateDropdown(popoverId);
          (document.activeElement as HTMLElement | null)?.blur?.();
        });
      };
      const syncDateValue = (nextValue: string) => {
        onSelect(nextValue);
        const calendar = document.getElementById(calendarId) as HTMLElement & { value?: string; focusedDate?: string } | null;
        if (calendar) {
          calendar.value = nextValue;
          calendar.focusedDate = nextValue;
        }
      };
      const commitDateInput = (event: Event) => {
        const target = event.currentTarget as HTMLInputElement;
        if (target.value === '' || isValidDateInput(target.value)) syncDateValue(target.value);
      };
      return h('div', { class: 'w-full' }, [
        h('input', {
          type: 'text',
          class: 'input input-sm w-full text-xs',
          value,
          placeholder: 'YYYY-MM-DD',
          style: `anchor-name:${anchorName}`,
          onFocus: showDateDropdown,
          onClick: showDateDropdown,
          onInput: commitDateInput,
          onChange: commitDateInput,
          onKeydown: (event: KeyboardEvent) => {
            if (event.key === 'Enter') {
              commitDateInput(event);
              (event.currentTarget as HTMLInputElement).blur();
            }
          },
        }),
        h('div', {
          id: popoverId,
          popover: 'auto',
          class: 'dropdown bg-base-100 rounded-box shadow-lg',
          style: `position-anchor:${anchorName}`,
          onClick: (event: Event) => event.stopPropagation(),
        }, [
          h('calendar-date', {
            id: calendarId,
            class: 'cally',
            value,
            focusedDate: value || undefined,
            onSelectday: closeDateDropdownAfterSelect,
            onChange: (event: Event) => {
              const target = event.currentTarget as HTMLElement & { value?: string };
              syncDateValue(target.value || '');
              closeDateDropdownAfterSelect();
            },
            onKeydown: (event: KeyboardEvent) => {
              if (event.key === 'Escape') closeDateDropdown(popoverId);
            },
          }, [
            calendarIcon('Previous', 'M15.75 19.5 8.25 12l7.5-7.5', 'previous'),
            calendarIcon('Next', 'm8.25 4.5 7.5 7.5-7.5 7.5', 'next'),
            h('calendar-month'),
          ]),
        ]),
      ]);
    };
    const textInput = () => h('input', {
      class: inputClass,
      type: 'text',
      value: props.rule.value,
      onInput: (event: any) => { props.rule.value = event.target.value; },
    });
    const selectInput = (options: any[]) => h('select', {
      class: selectClass,
      value: props.rule.value,
      onChange: (event: any) => { props.rule.value = coerceOptionValue(event.target.value, options); },
    }, options.map(option => h('option', { value: option.value }, option.label)));

    const removeExtensionPositionListeners = () => {
      if (removeExtensionMenuListeners) {
        removeExtensionMenuListeners();
        removeExtensionMenuListeners = null;
      }
    };
    const closeExtensionMenu = () => {
      extensionMenuOpen.value = false;
      removeExtensionPositionListeners();
    };
    const updateExtensionMenuPosition = async () => {
      await nextTick();
      if (!extensionTriggerRef.value || !extensionMenuRef.value) return;

      const padding = 8;
      const gap = 4;
      const triggerRect = extensionTriggerRef.value.getBoundingClientRect();
      const menuRect = extensionMenuRef.value.getBoundingClientRect();
      const viewportWidth = window.innerWidth;
      const viewportHeight = window.innerHeight;
      const maxWidth = Math.min(420, viewportWidth - padding * 2);
      const menuWidth = Math.min(maxWidth, Math.max(triggerRect.width, menuRect.width));
      const spaceBelow = viewportHeight - triggerRect.bottom - gap - padding;
      const spaceAbove = triggerRect.top - gap - padding;
      const showAbove = spaceBelow < Math.min(menuRect.height, 256) && spaceAbove > spaceBelow;
      const maxHeight = Math.max(96, Math.min(256, showAbove ? spaceAbove : spaceBelow));
      let left = triggerRect.left;
      let top = showAbove ? triggerRect.top - Math.min(menuRect.height, maxHeight) - gap : triggerRect.bottom + gap;

      left = Math.max(padding, Math.min(left, viewportWidth - menuWidth - padding));
      top = Math.max(padding, Math.min(top, viewportHeight - maxHeight - padding));

      extensionMenuPlacement.value = showAbove ? 'top' : 'bottom';
      extensionMenuStyle.value = {
        position: 'fixed',
        left: `${left}px`,
        top: `${top}px`,
        minWidth: `${triggerRect.width}px`,
        width: `${menuWidth}px`,
        maxWidth: `${maxWidth}px`,
        maxHeight: `${maxHeight}px`,
      };
    };
    const addExtensionPositionListeners = () => {
      removeExtensionPositionListeners();
      const onPointerDown = (event: PointerEvent) => {
        const target = event.target as Node | null;
        if (
          target
          && (extensionTriggerRef.value?.contains(target) || extensionMenuRef.value?.contains(target))
        ) {
          return;
        }
        closeExtensionMenu();
      };
      const onPositionChange = () => {
        if (extensionMenuOpen.value) void updateExtensionMenuPosition();
      };
      document.addEventListener('pointerdown', onPointerDown, true);
      window.addEventListener('resize', onPositionChange);
      window.addEventListener('scroll', onPositionChange, true);
      removeExtensionMenuListeners = () => {
        document.removeEventListener('pointerdown', onPointerDown, true);
        window.removeEventListener('resize', onPositionChange);
        window.removeEventListener('scroll', onPositionChange, true);
      };
    };
    const toggleExtensionMenu = async () => {
      if (extensionMenuOpen.value) {
        closeExtensionMenu();
        return;
      }
      extensionMenuOpen.value = true;
      await updateExtensionMenuPosition();
      addExtensionPositionListeners();
    };
    onBeforeUnmount(() => {
      removeExtensionPositionListeners();
    });
    const extensionSelect = (options: any[], extension: string, onSelect: (value: string) => void) => {
      const selectedText = extension ? extension.toUpperCase() : 'All';
      return h('div', { class: 'w-full' }, [
        h('button', {
          ref: extensionTriggerRef,
          type: 'button',
          class: 'select select-sm text-xs w-full cursor-pointer flex items-center text-left',
          onClick: toggleExtensionMenu,
        }, selectedText),
        h(Teleport, { to: 'body' }, [
          extensionMenuOpen.value ? h('ul', {
            ref: extensionMenuRef,
            class: [
              'smart-extension-menu menu bg-base-100 rounded-box shadow-lg z-[10000] p-1 overflow-y-auto',
              extensionMenuPlacement.value === 'top' ? 'origin-bottom' : 'origin-top',
            ],
            style: extensionMenuStyle.value,
          }, options.map((option: any) => h('li', { key: option.value }, [
            h('button', {
              type: 'button',
              class: 'smart-extension-option text-xs text-left leading-4',
              onClick: () => {
                onSelect(option.value);
                closeExtensionMenu();
              },
            }, option.label),
          ]))) : null,
        ]),
      ]);
    };
    const fileTypeInput = () => {
      const value = props.rule.value && typeof props.rule.value === 'object'
        ? props.rule.value
        : { fileType: getFileTypeValue(props.rule.value), extension: '' };
      const fileType = getFileTypeValue(value);
      const extension = getExtensionValue(value);
      const extensionOptions = getExtensionOptionsForFileType(fileType);
      const options = !extension || extensionOptions.some((option: any) => option.value === extension)
        ? extensionOptions
        : [
          ...extensionOptions,
          { value: extension, label: `${extension.toUpperCase()} (Custom Extension)` },
        ];
      return h('div', { class: 'grid grid-cols-[0.75fr_1.25fr] gap-1' }, [
        h('select', {
          class: selectClass,
          value: fileType,
          onChange: (event: any) => {
            props.rule.value = { fileType: Number(event.target.value), extension: '' };
          },
        }, props.fileTypeOptions.map((option: any) => h('option', { value: option.value }, option.label))),
        extensionSelect(options, extension, (nextExtension: string) => {
          props.rule.value = { fileType, extension: nextExtension };
        }),
      ]);
    };
    const boolSelect = () => selectInput([
      { value: 'true', label: t('album.smart_edit.boolean.yes') },
      { value: 'false', label: t('album.smart_edit.boolean.no') },
    ]);
    const rangeInput = (unit = '', scale = 1) => h('div', { class: 'grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-1' }, [
      unitNumberInput(scaleValue(props.rule.value?.min, scale), unit, (event: any) => {
        props.rule.value = { ...(props.rule.value || {}), min: Number(event.target.value) };
      }),
      h('span', { class: 'text-base-content/40 select-none' }, '-'),
      unitNumberInput(scaleValue(props.rule.value?.max, scale), unit, (event: any) => {
        props.rule.value = { ...(props.rule.value || {}), max: Number(event.target.value) };
      }),
    ]);
    const dateControl = () => {
      const value = props.rule.value || { field: 'taken' };
      if (['empty', 'not_empty'].includes(props.rule.operator)) {
        return selectDateField(value);
      }
      if (props.rule.operator === 'between') {
        return h('div', { class: 'grid grid-cols-[0.8fr_1fr_1fr] gap-1' }, [
          selectDateField(value),
          dateInput(unixToDateInput(value.start), (nextValue: string) => { props.rule.value = { ...value, start: dateInputToUnix(nextValue) }; }, 'start'),
          dateInput(unixToDateInputInclusiveEnd(value.end), (nextValue: string) => { props.rule.value = { ...value, end: dateInputToUnixExclusiveEnd(nextValue) }; }, 'end'),
        ]);
      }
      return h('div', { class: 'grid grid-cols-[0.8fr_1fr] gap-1' }, [
        selectDateField(value),
        dateInput(unixToDateInput(value.value), (nextValue: string) => { props.rule.value = { ...value, value: dateInputToUnix(nextValue) }; }, 'value'),
      ]);
    };
    const selectDateField = (value: any) => h('select', {
      class: selectClass,
      value: value.field || 'taken',
      onChange: (event: any) => { props.rule.value = { ...value, field: event.target.value }; },
    }, props.dateFieldOptions.map((option: any) => h('option', { value: option.value }, option.label)));
    const valueUnit = () => {
      if (props.rule.field === 'size') return 'MB';
      if (props.rule.field === 'width' || props.rule.field === 'height') return 'px';
      if (props.rule.field === 'duration') return 's';
      return '';
    };
    return () => {
      if (['empty', 'not_empty'].includes(props.rule.operator) && props.rule.field !== 'date') return h('span', { class: 'text-xs text-base-content/30' }, '-');
      if (props.rule.field === 'file_type') return fileTypeInput();
      if (props.rule.field === 'extension') return fileTypeInput();
      if (props.rule.field === 'favorite' || props.rule.field === 'has_gps') return boolSelect();
      if (props.rule.field === 'rating') return props.rule.operator === 'between' ? rangeInput() : numberInput(props.rule.value, (event: any) => { props.rule.value = Number(event.target.value); });
      if (props.rule.field === 'date') return dateControl();
      if (props.rule.field === 'size') return props.rule.operator === 'between' ? rangeInput(valueUnit()) : unitNumberInput(props.rule.value, valueUnit(), (event: any) => { props.rule.value = Number(event.target.value); });
      if (['width', 'height', 'duration'].includes(props.rule.field)) return props.rule.operator === 'between' ? rangeInput(valueUnit()) : unitNumberInput(props.rule.value, valueUnit(), (event: any) => { props.rule.value = Number(event.target.value); });
      if (props.rule.field === 'tag') return selectInput(props.tagOptions);
      if (props.rule.field === 'person') return selectInput(props.personOptions);
      if (props.rule.field === 'camera') return selectInput(props.cameraOptions);
      if (props.rule.field === 'lens') return selectInput(props.lensOptions);
      if (props.rule.field === 'location') return selectInput(props.locationOptions);
      return textInput();
    };
  },
});

function scaleValue(value: any, scale: number) {
  return typeof value === 'number' && scale !== 1 ? value / scale : value;
}

function coerceOptionValue(value: string, options: any[]) {
  const option = options.find(item => String(item.value) === String(value));
  return option ? option.value : value;
}
</script>

<style>
.smart-rule-ghost {
  opacity: 0.35;
  background: color-mix(in oklab, var(--color-base-content) 8%, transparent);
}

.smart-rule-chosen {
  cursor: grabbing;
}

.smart-rule-fallback {
  z-index: 10000;
  pointer-events: none;
  opacity: 0.92;
  background: var(--color-base-100);
  box-shadow: 0 8px 24px color-mix(in oklab, var(--color-base-content) 18%, transparent);
}

.smart-extension-menu {
  position: fixed;
}

.smart-extension-option {
  white-space: normal;
  overflow-wrap: anywhere;
}
</style>
