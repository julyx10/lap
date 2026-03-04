<template>
  <div class="flex flex-col items-start justify-start gap-4 h-full text-base-content/70 cursor-default">
    <div class="px-4 flex w-full flex-row items-center justify-start gap-4">
      <div class="shrink-0">
        <img src="@/assets/images/logo.png" class="w-24 h-24 mx-auto rounded-box drop-shadow-lg" />
      </div>
      <div class="flex flex-col text-left">
        <h3 class="text-xl">{{ packageInfo.name }}</h3>
        <p class="mt-2">{{ packageInfo.description }}</p>
      </div>
    </div>

    <div class="w-full max-w-lg rounded-box border border-base-content/5 bg-base-300/30 p-4 shadow-sm">
      <div class="space-y-3 text-left text-base-content/30">
        <div class="grid grid-cols-[84px_1fr] items-start gap-3 text-sm">
          {{ $t('settings.about.package.version') }}
          <div>{{ packageInfo.version }}</div>
        </div>

        <div class="grid grid-cols-[84px_1fr] items-start gap-3 text-sm">
          {{ $t('settings.about.package.build_time') }}
          <div>{{ buildTime }}</div>
        </div>

        <div class="grid grid-cols-[84px_1fr] items-start gap-3 text-sm">
          {{ $t('settings.about.package.license') }}
          <div>{{ packageInfo.license }}</div>
        </div>

        <div class="grid grid-cols-[84px_1fr] items-center gap-1 text-sm">
          {{ $t('settings.about.package.link') }}
          <div class="flex items-center justify-start gap-2">
            <a
              :href="packageInfo.homepage"
              target="_blank"
              class="inline-flex items-center gap-1.5 rounded-box px-2 py-1 text-xs transition-colors hover:bg-base-100/50 hover:text-primary"
            >
              <IconLink class="t-icon-size-sm" />
              <span>{{ $t('settings.about.package.website') }}</span>
            </a>
            <a
              :href="packageInfo.repository"
              target="_blank"
              class="inline-flex items-center gap-1.5 rounded-box px-2 py-1 text-xs transition-colors hover:bg-base-100/50 hover:text-primary"
            >
              <IconGithub class="t-icon-size-sm" />
              <span>{{ $t('settings.about.package.github') }}</span>
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
import { getPackageInfo, getBuildTime } from '@/common/api';
import { IconGithub, IconLink, IconLock } from '@/common/icons';

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
