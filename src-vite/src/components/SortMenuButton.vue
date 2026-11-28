<template>
  <DropDownSelect
    :disabled="disabled"
    :options="options"
    :extendOptions="extendOptions"
    :defaultIndex="dimOf(modelValue)"
    :defaultExtendIndex="dirOf(modelValue)"
    @select="(dim, dir) => emit('update:modelValue', combine(dim, dir))"
  >
    <template #trigger="{ toggle }">
      <TButton
        :icon="iconFor(modelValue)"
        buttonSize="small"
        :tooltip="resolvedTitle"
        :disabled="disabled"
        @click="toggle"
      />
    </template>
  </DropDownSelect>
</template>

<script setup lang="ts">
// Compact in-view sort control for the sidebar panels and the calendar.
// Renders a two-tier menu like the main toolbar's sort: a dimension group
// (Name / Count, or Taken / Created / Modified), a separator, then the
// Ascending / Descending group (reusing the toolbar's sort_order_options).
// The stored value encodes both: value = dimension * 2 + direction.
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import DropDownSelect from './DropDownSelect.vue';
import TButton from './TButton.vue';
import { getSelectOptions } from '@/common/utils';
import { IconSortingAsc, IconSortingDesc } from '@/common/icons';
import { SORT_TYPE } from '@/common/constants';

const props = withDefaults(defineProps<{
  modelValue: number;
  kind?: 'category' | 'calendar' | 'folder';
  title?: string;
  disabled?: boolean;
}>(), { kind: 'category', title: '', disabled: false });

const emit = defineEmits<{ (e: 'update:modelValue', value: number): void }>();

const { t, locale, messages } = useI18n();
const localeMsg = computed(() => (messages.value as any)[locale.value]);
const resolvedTitle = computed(() => props.title || t('toolbar.tooltip.sort'));

const options = computed(() => {
  if (props.kind === 'folder') {
    const labels = localeMsg.value.toolbar.filter.sort_type_options;
    return getSelectOptions([labels[SORT_TYPE.NAME], labels[SORT_TYPE.MODIFIED_DATE]]);
  }
  const key = props.kind === 'calendar' ? 'calendar_sort_options' : 'category_sort_options';
  return getSelectOptions(localeMsg.value?.settings?.browse?.[key] || []);
});
// Direction labels reuse the toolbar's sort order options: [Ascending, Descending].
const extendOptions = computed(() =>
  getSelectOptions(localeMsg.value?.toolbar?.filter?.sort_order_options || []),
);

const dimOf = (v: number) => Math.floor(Number(v) / 2);
const dirOf = (v: number) => Number(v) % 2;
const combine = (dim: number, dir: number) => Number(dim) * 2 + Number(dir);
// Icon reflects the direction (ascending / descending), matching the main
// toolbar's sort control.
const iconFor = (v: number) => (Number(v) % 2 === 0 ? IconSortingAsc : IconSortingDesc);
</script>
