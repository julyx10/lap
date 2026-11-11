<template>
  <div class="flex flex-col items-start justify-start gap-4 h-full text-base-content/70 cursor-default">

    <!-- logo -->
    <div class="px-2 flex w-full flex-row items-center justify-start gap-4">
      <div class="shrink-0">
        <img :src="iconLogo" class="w-20 h-20 select-none [-webkit-app-region:no-drag]" draggable="false" />
      </div>
      <div class="flex flex-col text-left">
        <h3 class="text-xl">{{ packageInfo.name }}</h3>
        <p class="mt-2 text-sm">{{ $t('settings.about.package.app_description') }}</p>
      </div>
    </div>

    <!-- package info -->
    <div class="w-full rounded-box border border-base-content/5 bg-base-300/30 p-4 shadow-sm">
      <div class="space-y-3 text-left">
        <div class="grid grid-cols-[84px_1fr] items-start gap-3 text-sm">
          <div class="text-base-content/30">
            {{ $t('settings.about.package.version') }}
          </div>
          <div>{{ displayVersion }}</div>
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

    <!-- updates -->
    <div class="w-full rounded-box p-2 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
      <div class="flex items-center gap-2 text-base-content/30">
        <span class="font-bold uppercase text-[10px] tracking-widest">{{ $t('settings.about.section_updates') }}</span>
      </div>
      <div class="flex items-center justify-between gap-4 px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
        <div class="min-w-0 flex-1 flex flex-col gap-0.5 text-sm leading-5">
          <div>{{ updateStatusText }}</div>
          <progress
            v-if="isDownloadingUpdate"
            class="progress progress-primary mt-1 h-1.5 w-full"
            :value="downloadPercent === null ? undefined : downloadPercent"
            max="100"
          ></progress>
        </div>
        <button
          class="btn btn-sm btn-ghost rounded-box bg-base-100 border border-base-content/30 text-base-content/70 hover:text-base-content shrink-0"
          :disabled="isCheckingUpdate || isInstallingUpdate || isDownloadingUpdate"
          @click="handleUpdateAction"
        >
          <span v-if="isCheckingUpdate || isInstallingUpdate || isDownloadingUpdate" class="loading loading-spinner loading-xs"></span>
          <span v-else>{{ updateButtonText }}</span>
        </button>
      </div>
      <div class="flex items-center justify-between gap-4 px-1 rounded-box hover:bg-base-100/10 transition-colors duration-200">
        <div class="flex flex-col gap-0.5 text-sm leading-5">
          <div>{{ $t('settings.general.auto_check_updates') }}</div>
        </div>
        <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="config.settings.autoCheckUpdates" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { getPackageInfo, getBuildTime } from '@/common/api';
import { useAppUpdater } from '@/common/updater';
import { config } from '@/common/config';
import { IconGithub, IconLink, IconLock, IconFocus } from '@/common/icons';
import iconLogo from '@/assets/images/icon.png';

const packageInfo = ref<any>({
  name: '',
  description: '',
  version: '',
  commit_hash: '',
  license: '',
  authors: [],
  homepage: '',
  repository: ''
});
const buildTime = ref('');
const displayVersion = computed(() => {
  const version = packageInfo.value.version || '';
  const commitHash = packageInfo.value.commit_hash || packageInfo.value.commitHash || '';
  return commitHash ? `${version} (${commitHash})` : version;
});
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
  isDownloadingUpdate,
  isUpdateReadyToRestart,
  updateAvailable,
  updateVersion,
  downloadProgressLabel,
  downloadPercent,
  checkError,
  updateButtonText,
  handleUpdateAction,
} = useAppUpdater(localeMsg, { toastPlacement: 'center', silent: true });

// Status line for the Updates section: reflects the current update state.
const updateStatusText = computed(() => {
  const au = localeMsg.value.settings.about.auto_update;
  if (isDownloadingUpdate.value) return downloadProgressLabel.value;
  if (isInstallingUpdate.value) return au.installing;
  if (isCheckingUpdate.value) return au.checking;
  if (checkError.value) return checkError.value;
  if (isUpdateReadyToRestart.value) return au.update_installed_waiting_restart;
  if (updateAvailable.value && updateVersion.value) {
    return au.new_version_available.replace('{version}', updateVersion.value);
  }
  return au.latest_version;
});

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
