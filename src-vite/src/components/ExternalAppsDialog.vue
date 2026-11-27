<template>
  <ModalDialog class="w-120" :title="$t('settings.advanced.external_app.manage_external_apps')" @cancel="close">
    <div class="space-y-4">
      <section v-for="group in externalAppGroups" :key="group.kind" class="space-y-2">
        <div class="flex items-center justify-between gap-3">
          <span class="text-xs uppercase tracking-widest font-bold text-base-content/30">{{ group.label }}</span>
          <TButton
            :icon="IconAdd"
            :buttonSize="'small'"
            :tooltip="$t('settings.advanced.external_app.choose_app')"
            :disabled="group.apps.length >= MAX_EXTERNAL_APPS"
            @click="selectExternalApp(group.kind)"
          />
        </div>

        <div v-if="group.apps.length" class="space-y-1 rounded-box p-1 bg-base-100/30 border border-base-content/5">
          <div v-for="app in group.apps" :key="app.id" :title="app.path" class="flex items-center gap-2 px-2 py-1.5 rounded-box hover:bg-base-100/10">
            <div class="min-w-0 flex-1 truncate text-sm">{{ app.name || app.path }}</div>
            <label class="flex items-center gap-1.5 shrink-0 cursor-pointer text-xs text-base-content/50">
              <input
                type="radio"
                class="radio radio-primary radio-xs"
                :name="`external-app-${group.kind}`"
                :checked="group.defaultId === app.id"
                @change="setDefaultExternalApp(group.kind, app.id)"
              />
              {{ $t('settings.advanced.external_app.default_app') }}
            </label>
            <TButton
              :icon="IconTrash"
              :buttonSize="'small'"
              :tooltip="$t('settings.advanced.external_app.clear_app')"
              @click="removeExternalApp(group.kind, app.id)"
            />
          </div>
        </div>
        <div v-else class="rounded-box px-2 py-3 text-xs text-base-content/30 bg-base-100/20 border border-base-content/5">
          {{ $t('settings.advanced.external_app.external_app_not_selected') }}
        </div>
      </section>
    </div>

    <div class="mt-4 flex justify-end">
      <button class="t-button-primary" @click="close">{{ $t('msgbox.close') }}</button>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { MAX_EXTERNAL_APPS } from '@/common/constants';
import { computed, onBeforeUnmount, onMounted } from 'vue';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { useI18n } from 'vue-i18n';
import { getExternalAppDisplayName } from '@/common/api';
import { config } from '@/common/config';
import { isMac } from '@/common/utils';
import { IconAdd, IconTrash } from '@/common/icons';
import ModalDialog from '@/components/ModalDialog.vue';
import TButton from '@/components/TButton.vue';
import { useUIStore } from '@/stores/uiStore';

type ExternalAppKind = 'image' | 'video';

const emit = defineEmits(['cancel']);
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();
const externalAppGroups = computed(() => ['image', 'video'].map((kind) => ({
  kind: kind as ExternalAppKind,
  label: kind === 'image'
    ? localeMsg.value.settings.advanced.external_app.external_image_editor
    : localeMsg.value.settings.advanced.external_app.external_video_app,
  apps: config.externalAppsFor(kind),
  defaultId: config.settings.externalApps?.[kind]?.defaultId || null,
})));

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('ExternalAppsDialog');
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('ExternalAppsDialog');
});

function close() {
  emit('cancel');
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === 'Escape' && uiStore.isInputActive('ExternalAppsDialog')) {
    event.preventDefault();
    close();
  }
}

async function selectExternalApp(kind: ExternalAppKind) {
  const result = await openDialog({
    title: kind === 'image'
      ? localeMsg.value.settings.advanced.external_app.external_image_editor
      : localeMsg.value.settings.advanced.external_app.external_video_app,
    multiple: false,
    directory: false,
    ...(isMac
      ? {
          defaultPath: '/Applications',
          filters: [{ name: 'Applications', extensions: ['app'] }],
        }
      : {}),
  });

  if (!result || Array.isArray(result)) return;
  let displayName = '';
  try {
    displayName = await getExternalAppDisplayName(result);
  } catch {}

  const externalApps = config.settings.externalApps ??= {
    image: { defaultId: null, apps: [] },
    video: { defaultId: null, apps: [] },
  };
  const group = externalApps[kind];
  if (group.apps.length >= MAX_EXTERNAL_APPS || group.apps.some((app: any) => app.path === result)) return;

  const app = { id: `${kind}:${result}`, name: displayName, path: result };
  group.apps.push(app);
  if (!group.defaultId) group.defaultId = app.id;
}

function setDefaultExternalApp(kind: ExternalAppKind, id: string) {
  config.settings.externalApps[kind].defaultId = id;
}

function removeExternalApp(kind: ExternalAppKind, id: string) {
  const group = config.settings.externalApps[kind];
  group.apps = group.apps.filter((app: any) => app.id !== id);
  if (group.defaultId === id) group.defaultId = group.apps[0]?.id || null;
}
</script>
