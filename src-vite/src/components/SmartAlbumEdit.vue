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
        <div class="h-8 flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/30">{{ $t('album.smart_edit.name') }}</div>
        <input ref="inputNameRef" v-model="name" type="text" maxlength="255" class="w-full input input-sm text-xs font-semibold" :placeholder="$t('album.smart_edit.validation_name')" />

        <div class="h-8 flex items-start pt-2 text-[10px] uppercase tracking-widest font-bold text-base-content/30">{{ $t('album.smart_edit.description') }}</div>
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

        <section class="col-span-2 rounded-box border border-base-content/10 bg-base-300/20 p-3 shadow-sm">
          <div class="grid grid-cols-3 gap-2">
            <label class="min-w-0 space-y-1">
              <span class="block text-[10px] uppercase tracking-widest font-bold text-base-content/30">{{ $t('album.smart_edit.group') }}</span>
              <select v-model.number="groupType" class="select select-sm text-xs w-full">
                <option v-for="option in groupOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
            </label>
            <label class="min-w-0 space-y-1">
              <span class="block text-[10px] uppercase tracking-widest font-bold text-base-content/30">{{ $t('album.smart_edit.sort') }}</span>
              <select v-model.number="sortType" class="select select-sm text-xs w-full">
                <option v-for="option in sortOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
            </label>
            <label class="min-w-0 space-y-1">
              <span class="block text-[10px] uppercase tracking-widest font-bold text-base-content/30">{{ $t('album.smart_edit.order') }}</span>
              <select v-model.number="sortOrder" class="select select-sm text-xs w-full">
                <option v-for="option in sortOrderOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
            </label>
          </div>
        </section>

        <section class="col-span-2 rounded-box border border-base-content/10 bg-base-300/20 p-3 shadow-sm">
          <div class="mb-2 flex min-h-8 items-center justify-between gap-3">
            <span class="text-[10px] uppercase tracking-widest font-bold text-base-content/30">{{ $t('album.smart_edit.match') }}</span>
            <div role="tablist" class="tabs tabs-box tabs-xs shrink-0 bg-base-100/40 shadow-inner">
              <button
                role="tab"
                :class="['tab', matchMode === 'all' ? 'tab-active text-primary' : '']"
                @click="matchMode = 'all'"
              >
                {{ $t('album.smart_edit.match_all') }}
              </button>
              <button
                role="tab"
                :class="['tab', matchMode === 'any' ? 'tab-active text-primary' : '']"
                @click="matchMode = 'any'"
              >
                {{ $t('album.smart_edit.match_any') }}
              </button>
            </div>
          </div>

          <div class="space-y-2">
            <div
              ref="rulesListRef"
              :class="[
                'pr-1',
                rules.length > 4 ? 'max-h-[236px] overflow-y-auto' : ''
              ]"
            >
              <template
              v-for="(rule, index) in rules"
              :key="rule.id"
              >
                <div class="grid grid-cols-[minmax(0,1fr)_88px_minmax(0,1.45fr)_28px] gap-2 items-center rounded-box border border-transparent px-1 py-1 transition-colors hover:border-base-content/5 hover:bg-base-100/40">
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
                  />
                  <TButton
                    :icon="IconRemove"
                    :buttonSize="'small'"
                    :tooltip="$t('album.smart_edit.remove_rule')"
                    :disabled="rules.length <= 1"
                    @click="removeRule(index)"
                  />
                </div>
                <div
                  v-if="index < rules.length - 1"
                  class="flex h-5 items-center gap-2 px-2 text-[9px] font-bold uppercase tracking-widest text-base-content/30"
                >
                  <span class="h-px flex-1 bg-base-content/5"></span>
                  <span>{{ matchMode === 'all' ? $t('album.smart_edit.match_and') : $t('album.smart_edit.match_or') }}</span>
                  <span class="h-px flex-1 bg-base-content/5"></span>
                </div>
              </template>
            </div>
            <div class="border-t border-base-content/5 pt-2 flex items-center justify-between">
              <button class="btn btn-sm rounded-box" :disabled="rules.length >= maxRules" @click="addRule">
                <IconAdd class="w-4 h-4" />
                {{ $t('album.smart_edit.add_rule') }}
              </button>
              <span class="text-base-content/30 tabular-nums">{{ $t('album.smart_edit.rules_count', { count: rules.length, max: maxRules }) }}</span>
            </div>
          </div>
        </section>
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
import { computed, defineComponent, h, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { getAllTags, getCameraInfo, getLensInfo, getLocationInfo, getPersons, getSupportedFormatExtensions } from '@/common/api';
import { IconAdd, IconEdit, IconRemove } from '@/common/icons';
import { useUIStore } from '@/stores/uiStore';
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
const uiStore = useUIStore();
const localeMsg = computed(() => messages.value[locale.value] as any);
const inputNameRef = ref<HTMLInputElement | null>(null);
const descriptionRef = ref<HTMLTextAreaElement | null>(null);
const rulesListRef = ref<HTMLElement | null>(null);
const maxRules = 20;

const isNew = computed(() => !props.smartAlbum);
const name = ref(props.smartAlbum?.name || '');
const description = ref(props.smartAlbum?.description || '');
const showDescription = ref(description.value.trim().length > 0);
const matchMode = ref(props.smartAlbum?.query?.match === 'any' ? 'any' : 'all');
const groupType = ref(Number(props.smartAlbum?.group?.type ?? 0));
const sortType = ref(Number(props.smartAlbum?.sort?.type ?? 0));
const sortOrder = ref(Number(props.smartAlbum?.sort?.order ?? 1));

const tagOptions = ref<any[]>([]);
const personOptions = ref<any[]>([]);
const cameraOptions = ref<any[]>([]);
const lensOptions = ref<any[]>([]);
const locationOptions = ref<any[]>([]);
const extensionOptions = ref<any[]>([]);
const rules = ref<any[]>([]);

const fileTypeOptions = computed(() => [
  { value: 1, label: localeMsg.value.toolbar.filter?.file_type_options?.[1] || 'Image' },
  { value: 4, label: localeMsg.value.toolbar.filter?.file_type_options?.[2] || 'RAW' },
  { value: 2, label: localeMsg.value.toolbar.filter?.file_type_options?.[3] || 'Video' },
]);

function indexedOptions(labels: unknown, fallbacks: string[]) {
  const values = Array.isArray(labels) ? labels : [];
  return fallbacks.map((fallback, value) => ({
    value,
    label: String(values[value] || fallback),
  }));
}

const groupOptions = computed(() => {
  const labels = indexedOptions(
    localeMsg.value.toolbar.filter?.group_type_options,
    ['None', 'Folder', 'Day', 'Month', 'Rating', 'Location', 'Camera', 'Lens', 'Year'],
  );
  return [0, 1, 2, 3, 8, 4, 5, 6, 7].map(value => labels[value]);
});

const sortOptions = computed(() => indexedOptions(
  localeMsg.value.toolbar.filter?.sort_type_options,
  ['Taken Date', 'Created Date', 'Modified Date', 'Name', 'Size', 'Dimension', 'Duration', 'Rating', 'Random'],
));

const sortOrderOptions = computed(() => indexedOptions(
  localeMsg.value.toolbar.filter?.sort_order_options,
  ['Ascending', 'Descending'],
));

const fieldOptions = computed(() => [
  { value: 'name', label: t('album.smart_edit.fields.name') },
  { value: 'date_taken', label: t('album.smart_edit.fields.date_taken') },
  { value: 'date_created', label: t('album.smart_edit.fields.date_created') },
  { value: 'date_modified', label: t('album.smart_edit.fields.date_modified') },
  { value: 'favorite', label: t('album.smart_edit.fields.favorite') },
  { value: 'rating', label: t('album.smart_edit.fields.rating') },
  { value: 'tag', label: t('album.smart_edit.fields.tag') },
  { value: 'person', label: t('album.smart_edit.fields.person') },
  { value: 'file_type', label: t('album.smart_edit.fields.file_type') },
  { value: 'extension', label: t('album.smart_edit.fields.extension') },
  { value: 'orientation', label: t('album.smart_edit.fields.orientation') },
  { value: 'width', label: t('album.smart_edit.fields.width') },
  { value: 'height', label: t('album.smart_edit.fields.height') },
  { value: 'duration', label: t('album.smart_edit.fields.duration') },
  { value: 'size', label: t('album.smart_edit.fields.size') },
  { value: 'camera', label: t('album.smart_edit.fields.camera') },
  { value: 'lens', label: t('album.smart_edit.fields.lens') },
  { value: 'location', label: t('album.smart_edit.fields.location') },
  { value: 'has_gps', label: t('album.smart_edit.fields.has_gps') },
]);

const validationMessage = computed(() => {
  if (rules.value.length === 0) return t('album.smart_edit.validation_rule');
  if (!rules.value.every(isRuleValid)) return t('album.smart_edit.validation_rule');
  return '';
});
const canSave = computed(() => name.value.trim().length > 0 && validationMessage.value.length === 0);

function makeRule(field = 'date_taken') {
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
  const field = normalizeRuleFieldForEdit(rule);
  return {
    id: rule.id || crypto.randomUUID?.() || `${Date.now()}-${Math.random()}`,
    field,
    operator: rule.operator || getOperatorOptions(field)[0]?.value || 'is',
    value: normalizeRuleValueForEdit(sourceField, cloneValue(rule.value)),
  };
}

function getOperatorOptions(field: string) {
  const op = (key: string) => ({ value: key, label: t(`album.smart_edit.operators.${key}`) });
  if (field === 'name') return [op('contains'), op('not_contains')];
  if (['file_type', 'extension', 'camera', 'lens', 'location'].includes(field)) return [op('is'), op('is_not')];
  if (field === 'favorite' || field === 'has_gps') return [op('is')];
  if (field === 'orientation') return [op('is')];
  if (field === 'rating') return [op('is'), op('is_not'), op('gt'), op('gte'), op('lt'), op('lte'), op('between'), op('empty'), op('not_empty')];
  if (isDateRuleField(field)) return [op('is'), op('before'), op('after'), op('between'), op('in_last'), op('older_than')];
  if (['size', 'width', 'height', 'duration'].includes(field)) return [op('gt'), op('gte'), op('lt'), op('lte'), op('between'), op('empty'), op('not_empty')];
  if (field === 'tag' || field === 'person') return [op('has'), op('not_has'), op('empty'), op('not_empty')];
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
  if (rule.field === 'name') rule.value = '';
  else if (rule.field === 'file_type') rule.value = 1;
  else if (rule.field === 'extension') rule.value = extensionOptions.value[0]?.value || 'jpg';
  else if (rule.field === 'favorite' || rule.field === 'has_gps') rule.value = true;
  else if (rule.field === 'orientation') rule.value = 'landscape';
  else if (rule.field === 'rating') rule.value = rule.operator === 'between' ? { min: 1, max: 5 } : 1;
  else if (isDateRuleField(rule.field)) {
    if (rule.operator === 'between') rule.value = { start: dateToUnix(new Date()), end: dateToUnix(addDays(new Date(), 1)) };
    else if (isRelativeDateOperator(rule.operator)) rule.value = { amount: 7, unit: 'day' };
    else if (rule.operator === 'is') rule.value = 'this_month';
    else rule.value = { value: dateToUnix(new Date()) };
  }
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
  if (isDateRuleField(rule.field) && isRelativeDateOperator(rule.operator)) return isRelativeDateValueValid(rule.value);
  if (isDateRuleField(rule.field) && rule.operator === 'is') return isDatePeriodValueValid(rule.value);
  if (rule.field === 'orientation') return isOrientationValueValid(rule.value);
  if (rule.field === 'file_type') return Boolean(getFileTypeValue(rule.value));
  if (isDateRuleField(rule.field)) return Boolean(rule.value?.value);
  return rule.value !== null && rule.value !== undefined && rule.value !== '';
}

async function addRule() {
  if (rules.value.length >= maxRules) return;
  rules.value.push(makeRule());
  await nextTick();
  rulesListRef.value?.scrollTo({
    top: rulesListRef.value.scrollHeight,
    behavior: 'smooth',
  });
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

function handleKeyDown(event: KeyboardEvent) {
  if (!uiStore.isInputActive('SmartAlbumEdit')) return;

  if (event.key === 'Escape') {
    event.preventDefault();
    if (closeActiveDateDropdown()) return;
    clickCancel();
  }
}

function closeActiveDateDropdown() {
  for (const popover of document.querySelectorAll<HTMLElement>('[id^="smart-date-"]')) {
    if (popover.matches(':popover-open')) {
      popover.hidePopover?.();
      return true;
    }
  }
  return false;
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
    group: { type: groupType.value },
    sort: { type: sortType.value, order: sortOrder.value },
    createdAt: props.smartAlbum?.createdAt || now,
    updatedAt: now,
  });
}

function normalizeRuleValue(rule: any) {
  if (rule.field === 'file_type') return getFileTypeValue(rule.value);
  if (rule.field === 'extension') return getExtensionValue(rule.value);
  if (rule.field === 'size') return normalizeNumericValue(rule.value, 1_000_000);
  if (isDateRuleField(rule.field)) return normalizeDateValue(rule.value);
  return normalizeNumericValue(rule.value, 1);
}

function cloneValue(value: any) {
  if (value === undefined) return value;
  return JSON.parse(JSON.stringify(value));
}

function normalizeRuleValueForEdit(field: string, value: any) {
  if (field === 'file_type') {
    return getFileTypeValue(value);
  }
  if (field === 'extension') {
    return getExtensionValue(value);
  }
  if (isDateRuleField(field)) return normalizeDateValue(value);
  if (field !== 'size') return value;
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    const next: any = { ...value };
    if (typeof next.min === 'number') next.min = next.min / 1_000_000;
    if (typeof next.max === 'number') next.max = next.max / 1_000_000;
    return next;
  }
  return typeof value === 'number' ? value / 1_000_000 : value;
}

function normalizeRuleFieldForEdit(rule: any) {
  const field = rule.field || 'file_type';
  return field;
}

function isDateRuleField(field: string) {
  return ['date_taken', 'date_created', 'date_modified'].includes(field);
}

function isRelativeDateOperator(operator: string) {
  return ['in_last', 'older_than'].includes(operator);
}

function isRelativeDateValueValid(value: any) {
  const amount = Number(value?.amount);
  return Number.isFinite(amount) && amount > 0 && ['day', 'month', 'year'].includes(value?.unit);
}

function isDatePeriodValueValid(value: any) {
  return ['this_week', 'this_month', 'this_year'].includes(value);
}

function isOrientationValueValid(value: any) {
  return ['landscape', 'portrait', 'square'].includes(value);
}

function normalizeDateValue(value: any) {
  return value;
}

function getFileTypeValue(value: any) {
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    return Number(value.fileType ?? value.file_type ?? 1);
  }
  return Number(value || 1);
}

function getExtensionValue(value: any) {
  if (Array.isArray(value)) return String(value[0] || '').trim().replace(/^\./, '').toLowerCase();
  if (typeof value === 'string') return value.trim().replace(/^\./, '').toLowerCase();
  if (!value || typeof value !== 'object' || Array.isArray(value)) return '';
  return String(value.extension || '').trim().replace(/^\./, '').toLowerCase();
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
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('SmartAlbumEdit');
  await loadSupportedFormatOptions();
  await loadOptions();
  await nextTick();
  inputNameRef.value?.focus();
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('SmartAlbumEdit');
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
  },
  setup(props: any) {
    const suppressDatePopoverUntil = ref(0);
    const inputClass = 'input input-sm text-xs w-full';
    const selectClass = 'select select-sm text-xs w-full';
    const numberInput = (value: any, onInput: any, attrs: Record<string, any> = {}) => h('input', { class: inputClass, type: 'number', value, onInput, ...attrs });
    const inputWithUnit = (inputNode: any, unit: string) => {
      if (!unit) return inputNode;
      return h('label', { class: 'input input-sm w-full flex items-center gap-1 px-0 overflow-hidden' }, [
        h('div', { class: 'min-w-0 flex-1' }, [
          inputNode,
        ]),
        h('span', { class: 'pr-2 text-[10px] uppercase text-base-content/40 shrink-0' }, unit),
      ]);
    };
    const unitNumberInput = (value: any, unit: string, onInput: any, attrs: Record<string, any> = {}) => inputWithUnit(
      h('input', {
        class: 'w-full min-w-0 bg-transparent px-2 text-xs outline-none',
        type: 'number',
        value,
        onInput,
        ...attrs,
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
              if (event.key === 'Escape') {
                event.stopPropagation();
                closeDateDropdown(popoverId);
              }
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

    const extensionSelect = (options: any[], extension: string, onSelect: (value: string) => void) => {
      return h('select', {
        class: selectClass,
        value: extension,
        onChange: (event: any) => { onSelect(event.target.value); },
      }, options.map((option: any) => h('option', { value: option.value }, option.label)));
    };
    const fileTypeInput = () => h('select', {
      class: selectClass,
      value: getFileTypeValue(props.rule.value),
      onChange: (event: any) => { props.rule.value = Number(event.target.value); },
    }, props.fileTypeOptions.map((option: any) => h('option', { value: option.value }, option.label)));
    const extensionInput = () => {
      const extension = getExtensionValue(props.rule.value);
      const options = !extension || props.extensionOptions.some((option: any) => option.value === extension)
        ? props.extensionOptions
        : [
          ...props.extensionOptions,
          { value: extension, label: `${extension.toUpperCase()} (Custom Extension)` },
        ];
      return extensionSelect(options, extension, (nextExtension: string) => {
        props.rule.value = nextExtension;
      });
    };
    const boolSelect = () => selectInput([
      { value: 'true', label: t('album.smart_edit.boolean.yes') },
      { value: 'false', label: t('album.smart_edit.boolean.no') },
    ]);
    const rangeInput = (unit = '', scale = 1, attrs: Record<string, any> = {}) => h('div', { class: 'grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-1' }, [
      unitNumberInput(scaleValue(props.rule.value?.min, scale), unit, (event: any) => {
        props.rule.value = { ...(props.rule.value || {}), min: Number(event.target.value) };
      }, attrs),
      h('span', { class: 'text-base-content/40 select-none' }, '-'),
      unitNumberInput(scaleValue(props.rule.value?.max, scale), unit, (event: any) => {
        props.rule.value = { ...(props.rule.value || {}), max: Number(event.target.value) };
      }, attrs),
    ]);
    const dateControl = () => {
      const value = props.rule.value || {};
      if (props.rule.operator === 'is') {
        return h('select', {
          class: selectClass,
          value: isDatePeriodValueValid(props.rule.value) ? props.rule.value : 'this_week',
          onChange: (event: any) => { props.rule.value = event.target.value; },
        }, ['this_week', 'this_month', 'this_year'].map(period => h('option', { value: period }, t(`album.smart_edit.date_periods.${period}`))));
      }
      if (isRelativeDateOperator(props.rule.operator)) {
        return h('div', { class: 'grid grid-cols-[minmax(0,1fr)_110px] gap-1' }, [
          h('input', {
            class: inputClass,
            type: 'number',
            min: 1,
            step: 1,
            value: value.amount ?? 7,
            onInput: (event: any) => {
              props.rule.value = { amount: Number(event.target.value), unit: value.unit || 'day' };
            },
          }),
          h('select', {
            class: selectClass,
            value: value.unit || 'day',
            onChange: (event: any) => {
              props.rule.value = { amount: Number(value.amount || 7), unit: event.target.value };
            },
          }, ['day', 'month', 'year'].map(unit => h('option', { value: unit }, t(`album.smart_edit.date_units.${unit}`)))),
        ]);
      }
      if (props.rule.operator === 'between') {
        return h('div', { class: 'grid grid-cols-[1fr_1fr] gap-1' }, [
          dateInput(unixToDateInput(value.start), (nextValue: string) => { props.rule.value = { ...value, start: dateInputToUnix(nextValue) }; }, 'start'),
          dateInput(unixToDateInputInclusiveEnd(value.end), (nextValue: string) => { props.rule.value = { ...value, end: dateInputToUnixExclusiveEnd(nextValue) }; }, 'end'),
        ]);
      }
      return dateInput(unixToDateInput(value.value), (nextValue: string) => { props.rule.value = { ...value, value: dateInputToUnix(nextValue) }; }, 'value');
    };
    const orientationSelect = () => h('select', {
      class: selectClass,
      value: isOrientationValueValid(props.rule.value) ? props.rule.value : 'landscape',
      onChange: (event: any) => { props.rule.value = event.target.value; },
    }, ['landscape', 'portrait', 'square'].map(orientation => h('option', { value: orientation }, t(`album.smart_edit.orientation_values.${orientation}`))));
    const valueUnit = () => {
      if (props.rule.field === 'size') return 'MB';
      if (props.rule.field === 'width' || props.rule.field === 'height') return 'px';
      if (props.rule.field === 'duration') return 's';
      return '';
    };
    return () => {
      if (['empty', 'not_empty'].includes(props.rule.operator) && !isDateRuleField(props.rule.field)) return h('span', { class: 'text-xs text-base-content/30' }, '-');
      if (props.rule.field === 'file_type') return fileTypeInput();
      if (props.rule.field === 'extension') return extensionInput();
      if (props.rule.field === 'favorite' || props.rule.field === 'has_gps') return boolSelect();
      if (props.rule.field === 'orientation') return orientationSelect();
      if (props.rule.field === 'rating') return props.rule.operator === 'between' ? rangeInput('', 1, { min: 0, max: 5, step: 1 }) : numberInput(props.rule.value, (event: any) => { props.rule.value = Number(event.target.value); }, { min: 0, max: 5, step: 1 });
      if (isDateRuleField(props.rule.field)) return dateControl();
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
