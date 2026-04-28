<template>
  <div class="flex flex-col items-start justify-start gap-4 h-full text-base-content/70 cursor-default">

    <!-- logo -->
    <div class="px-4 flex w-full flex-row items-center justify-start gap-4">
      <div class="shrink-0">
        <img src="@/assets/images/logo.png" class="w-24 h-24 mx-auto rounded-box drop-shadow-lg" />
      </div>
      <div class="flex flex-col text-left">
        <h3 class="text-xl">{{ packageInfo.name }}</h3>
        <p class="mt-2">{{ $t('settings.about.package.app_description') }}</p>
      </div>
    </div>

    <!-- package info -->
    <div class="w-full max-w-lg rounded-box border border-base-content/5 bg-base-300/30 p-4 shadow-sm">
      <div class="space-y-3 text-left">
        <div class="grid grid-cols-[84px_1fr] items-start gap-3 text-sm">
          <div class="text-base-content/30">
            {{ $t('settings.about.package.version') }}
          </div>
          <div class="flex items-center gap-2">
            <span>{{ packageInfo.version }}</span>
            <button
              class="badge badge-sm border-0 px-2 py-2 font-medium transition-colors hover:text-primary"
              :class="isUpdateActionEnabled ? 'badge-primary cursor-pointer' : 'badge-neutral/60 cursor-pointer'"
              :disabled="isInstallingUpdate || isCheckingUpdate"
              :title="updateButtonTooltip"
              @click="handleUpdateAction"
            >
              <span v-if="isInstallingUpdate || isCheckingUpdate" class="loading loading-spinner loading-xs"></span>
              <span>{{ updateButtonText }}</span>
            </button>
          </div>
        </div>

        <div class="grid grid-cols-[84px_1fr] items-start gap-3 text-sm">
          <div class="text-base-content/30">
            {{ $t('settings.about.package.build_time') }}
          </div>
          <div>{{ buildTime }}</div>
        </div>

        <div class="grid grid-cols-[84px_1fr] items-start gap-3 text-sm">
          <div class="text-base-content/30">
            {{ $t('settings.about.package.license') }}
          </div>
          <div>{{ packageInfo.license }}</div>
        </div>

        <div class="grid grid-cols-[84px_1fr] items-center gap-1 text-sm">
          <div class="text-base-content/30">
            {{ $t('settings.about.package.link') }}
          </div>
          <div class="flex flex-wrap items-center justify-start">
            <!-- <a
              :href="packageInfo.homepage"
              target="_blank"
              class="inline-flex items-center gap-1.5 rounded-box px-2 py-1 text-xs transition-colors hover:bg-base-100/50 hover:text-primary"
            >
              <IconLink class="t-icon-size-sm" />
              <span>{{ $t('settings.about.package.website') }}</span>
            </a> -->
            <a
              :href="packageInfo.repository"
              target="_blank"
              class="inline-flex items-center gap-1.5 rounded-box px-2 py-1 text-xs transition-colors hover:bg-base-100/50 hover:text-primary"
            >
              <IconGithub class="t-icon-size-sm" />
              <span>{{ $t('settings.about.package.github') }}</span>
            </a>
            <a
              :href="issuesUrl"
              target="_blank"
              class="inline-flex items-center gap-1.5 rounded-box px-2 py-1 text-xs transition-colors hover:bg-base-100/50 hover:text-primary"
            >
              <IconFocus class="t-icon-size-sm" />
              <span>{{ $t('settings.about.package.feedback') }}</span>
            </a>
            <a
              :href="privacyUrl"
              target="_blank"
              class="inline-flex items-center gap-1.5 rounded-box px-2 py-1 text-xs transition-colors hover:bg-base-100/50 hover:text-primary"
            >
              <IconLock class="t-icon-size-sm" />
              <span>{{ $t('settings.about.package.privacy') }}</span>
            </a>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { getPackageInfo, getBuildTime } from '@/common/api';
import { useAppUpdater } from '@/common/updater';
import { IconGithub, IconLink, IconLock, IconFocus } from '@/common/icons';

const packageInfo = ref<any>({
  name: '',
  description: '',
  version: '',
  license: '',
  authors: [],
  homepage: '',
  repository: ''
});
const buildTime = ref('');
const privacyUrl = computed(() => {
  const repo = packageInfo.value.repository || '';
  if (!repo) return 'https://github.com/julyx10/lap/blob/main/PRIVACY.md';
  return repo.endsWith('/') ? `${repo}blob/main/PRIVACY.md` : `${repo}/blob/main/PRIVACY.md`;
});
const issuesUrl = computed(() => {
  const repo = packageInfo.value.repository || '';
  if (!repo) return 'https://github.com/julyx10/lap/issues';
  return repo.endsWith('/') ? `${repo}issues` : `${repo}/issues`;
});
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const {
  isCheckingUpdate,
  isInstallingUpdate,
  updateButtonTooltip,
  updateButtonText,
  isUpdateActionEnabled,
  handleUpdateAction,
} = useAppUpdater(localeMsg, { toastPlacement: 'center' });

onMounted(async () => {
  try {
    packageInfo.value = await getPackageInfo();
    const time = await getBuildTime();
    buildTime.value = time || '';
  } catch (error) {
    console.error('Failed to load about info:', error);
  }
});
</script>
