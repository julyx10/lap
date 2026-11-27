<template>
  <ModalDialog :title="$t('settings.general.theme')" :width="560" @cancel="$emit('close')">
    <div class="max-h-[70vh] overflow-y-auto pr-1 space-y-3 select-none">
      <div class="grid grid-cols-[90px_minmax(0,1fr)] items-center gap-2 text-sm">
        <span class="text-base-content/50">{{ $t('msgbox.batch_rename.name') }}</span>
        <input v-model="name" class="input input-sm w-full" maxlength="80" />
      </div>
      <div class="border-t border-base-content/10 pt-3">
        <div class="mb-2 text-xs font-bold uppercase tracking-widest text-base-content/30">Colors</div>
        <div class="grid grid-cols-1 gap-2 sm:grid-cols-2">
          <div v-for="[token, label] in THEME_COLOR_TOKENS" :key="token" class="flex items-center gap-2 text-xs">
            <input
              type="color"
              :aria-label="label"
              class="h-7 w-8 shrink-0 cursor-pointer rounded border border-base-content/10 bg-transparent p-0"
              :value="colorPickerValue(token)"
              @input="setColorFromPicker(token, $event)"
            />
            <span class="w-24 shrink-0 truncate text-base-content/50">{{ label }}</span>
            <input v-model="colors[token]" :aria-label="label" class="input input-xs min-w-0 flex-1 font-mono" />
          </div>
        </div>
      </div>

      <p v-if="error" class="text-xs text-error">{{ error }}</p>
      <p v-if="exportMessage" class="text-xs text-success" role="status">{{ exportMessage }}</p>

      <div class="flex flex-wrap items-center gap-2 border-t border-base-content/10 pt-3">
        <button type="button" class="btn btn-ghost btn-xs" @click="exportTheme">Export JSON</button>
        <label class="btn btn-ghost btn-xs cursor-pointer">
          Import JSON
          <input type="file" accept=".json,application/json" class="hidden" @change="importTheme" />
        </label>
        <button v-if="theme" type="button" class="btn btn-ghost btn-xs text-error" @click="$emit('delete', theme.id)">
          {{ $t('menu.tag.delete') }}
        </button>
        <span class="flex-1" />
        <button type="button" class="btn btn-ghost btn-sm" @click="$emit('close')">{{ $t('msgbox.cancel') }}</button>
        <button type="button" class="btn btn-primary btn-sm" @click="save">{{ $t('msgbox.image_editor.save') }}</button>
      </div>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import ModalDialog from '@/components/ModalDialog.vue';
import {
  THEME_COLOR_TOKENS,
  DEFAULT_CUSTOM_THEME_COLORS,
  createCustomThemeId,
  importCustomTheme,
  sanitizeCustomTheme,
  type CustomTheme,
  type ThemeAppearance,
} from '@/common/themes';

const props = defineProps<{
  theme?: CustomTheme | null;
  appearance: ThemeAppearance;
}>();
const emit = defineEmits<{
  save: [theme: CustomTheme];
  delete: [id: string];
  close: [];
}>();

const themeId = ref(props.theme?.id || createCustomThemeId());
const name = ref(props.theme?.name || '');
const appearance = ref<ThemeAppearance>(props.theme?.appearance || props.appearance);
const colors = ref<Record<string, string>>({
  ...DEFAULT_CUSTOM_THEME_COLORS[appearance.value],
  ...(props.theme?.colors || {}),
});
const shape = ref<Record<string, string>>({ ...(props.theme?.shape || {}) });
const error = ref('');
const exportMessage = ref('');

function colorPickerValue(token: string) {
  const value = colors.value[token];
  return /^#[0-9a-f]{6}$/i.test(value || '') ? value : '#000000';
}
function setColorFromPicker(token: string, event: Event) {
  colors.value[token] = (event.target as HTMLInputElement).value;
}
function buildTheme(): CustomTheme | null {
  return sanitizeCustomTheme({
    schema: 1,
    id: themeId.value,
    name: name.value,
    appearance: appearance.value,
    colors: colors.value,
    shape: shape.value,
  });
}
function save() {
  const theme = buildTheme();
  if (!theme) {
    error.value = 'Enter a name and valid CSS color values.';
    return;
  }
  emit('save', theme);
}
async function exportTheme() {
  error.value = '';
  exportMessage.value = '';
  const theme = buildTheme();
  if (!theme) {
    error.value = 'Enter a name and valid CSS color values.';
    return;
  }
  try {
    await navigator.clipboard.writeText(JSON.stringify(theme, null, 2));
    exportMessage.value = 'JSON copied to clipboard.';
  } catch {
    error.value = 'Could not copy JSON.';
  }
}
async function importTheme(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;
  try {
    const parsed = JSON.parse(await file.text());
    const theme = importCustomTheme(parsed, appearance.value);
    if (!theme) throw new Error('Invalid theme');
    themeId.value = theme.id;
    name.value = theme.name;
    appearance.value = theme.appearance;
    colors.value = { ...colors.value, ...theme.colors };
    shape.value = { ...(theme.shape || {}) };
    error.value = '';
  } catch {
    error.value = 'Invalid theme JSON.';
  } finally {
    (event.target as HTMLInputElement).value = '';
  }
}
</script>
