<template>
  <ModalDialog :title="title" :width="560" :height="560" @cancel="emit('cancel')">
    <div class="flex h-full min-h-0 flex-col gap-3">
      <input
        ref="searchInputRef"
        v-model.trim="searchQuery"
        type="text"
        class="input input-bordered input-sm w-full"
        :placeholder="searchPlaceholder"
      />

      <div class="flex-1 min-h-0 overflow-hidden rounded-box border border-base-content/10 bg-base-100/20">
        <div v-if="loading" class="flex h-full items-center justify-center">
          <span class="loading loading-spinner loading-md text-primary"></span>
        </div>
        <div v-else-if="filteredApps.length === 0" class="flex h-full items-center justify-center px-6 text-center text-sm text-base-content/50">
          {{ emptyLabel }}
        </div>
        <div v-else class="h-full overflow-y-auto p-2">
          <button
            v-for="app in filteredApps"
            :key="app.appId"
            class="w-full rounded-box border px-3 py-2 text-left transition-colors cursor-pointer"
            :class="selectedAppId === app.appId
              ? 'border-primary bg-primary/10 text-base-content'
              : 'border-transparent hover:border-base-content/10 hover:bg-base-100/40'"
            @click="selectedAppId = app.appId"
            @dblclick="selectApp(app)"
          >
            <div class="truncate text-sm font-medium">{{ app.name }}</div>
            <div class="truncate text-xs text-base-content/45">{{ app.appId }}</div>
          </button>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2">
        <button class="btn btn-ghost btn-sm" @click="emit('cancel')">
          {{ cancelLabel }}
        </button>
        <button class="btn btn-primary btn-sm" :disabled="!selectedApp" @click="selectedApp && selectApp(selectedApp)">
          {{ selectLabel }}
        </button>
      </div>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import ModalDialog from '@/components/ModalDialog.vue';

interface WindowsInstalledApp {
  appId: string;
  name: string;
}

const props = defineProps<{
  title: string;
  apps: WindowsInstalledApp[];
  loading?: boolean;
}>();

const emit = defineEmits<{
  cancel: [];
  select: [app: WindowsInstalledApp];
}>();

const { t: $t } = useI18n();
const searchInputRef = ref<HTMLInputElement | null>(null);
const searchQuery = ref('');
const selectedAppId = ref('');

const normalizedApps = computed(() =>
  Array.isArray(props.apps)
    ? props.apps.filter(app => String(app?.appId || '').trim() && String(app?.name || '').trim())
    : []
);

const filteredApps = computed(() => {
  const keyword = searchQuery.value.trim().toLowerCase();
  if (!keyword) return normalizedApps.value;
  return normalizedApps.value.filter((app) => {
    const name = String(app.name || '').toLowerCase();
    const appId = String(app.appId || '').toLowerCase();
    return name.includes(keyword) || appId.includes(keyword);
  });
});

const selectedApp = computed(() =>
  filteredApps.value.find(app => app.appId === selectedAppId.value)
    || normalizedApps.value.find(app => app.appId === selectedAppId.value)
    || null
);

const searchPlaceholder = computed(() =>
  $t('settings.image_view.search_installed_apps') || 'Search installed apps'
);
const emptyLabel = computed(() =>
  props.loading ? '' : ($t('settings.image_view.no_matching_apps') || 'No matching apps found')
);
const cancelLabel = computed(() => $t('msgbox.cancel') || 'Cancel');
const selectLabel = computed(() => $t('settings.image_view.choose_app') || 'Choose');

function selectApp(app: WindowsInstalledApp) {
  emit('select', app);
}

onMounted(async () => {
  await nextTick();
  searchInputRef.value?.focus();
});

watch(normalizedApps, (apps) => {
  if (!apps.length) {
    selectedAppId.value = '';
    return;
  }
  if (!apps.some(app => app.appId === selectedAppId.value)) {
    selectedAppId.value = apps[0].appId;
  }
}, { immediate: true });
</script>
