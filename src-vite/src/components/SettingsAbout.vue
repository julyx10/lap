<template>
  <div class="flex flex-col items-center justify-start p-2 gap-4 text-center h-full text-base-content/70 cursor-default">
    
    <!-- App Header -->
    <div class="px-4 flex flex-row items-center justify-start gap-4">
      <!-- App Logo -->
      <div class="shrink-0">
        <img src="@/assets/images/logo.png" class="w-24 h-24 mx-auto rounded-box drop-shadow-lg" />
      </div>
      <div class="flex flex-col">
        <h3 class="text-xl font-bold">{{ packageInfo.name }}</h3>
        <p class="mt-2">{{ packageInfo.description }}</p>
      </div>
    </div>

    <!-- Info Grid -->
    <div class="w-full max-w-md text-base-content/30 bg-base-200/30 rounded-lg p-4">
      <div class="grid grid-cols-[100px_1fr] gap-4 text-left text-sm">
        
        <div class="font-semibold">{{ $t('settings.about.package.version') }}</div>
        <div class="font-mono">{{ packageInfo.version }}</div>

        <div class="font-semibold">{{ $t('settings.about.package.build_time') }}</div>
        <div>{{ buildTime }}</div>

        <div class="font-semibold">{{ $t('settings.about.package.license') }}</div>
        <div>{{ packageInfo.license }}</div>

        <div class="font-semibold">{{ $t('settings.about.package.author') }}</div>
        <div>{{ packageInfo.authors && packageInfo.authors[0] }}</div>

        <div class="font-semibold">{{ $t('settings.about.package.website') }}</div>
        <div>
          <a :href="packageInfo.homepage" target="_blank" class="link hover:text-primary-focus">
            {{ packageInfo.homepage }}
          </a>
        </div>

        <!-- <div class="font-semibold">GitHub</div>
        <div>
          <a :href="packageInfo.repository" target="_blank" class="link hover:text-primary-focus">
            {{ packageInfo.repository }}
          </a>
        </div> -->

      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { getPackageInfo, getBuildTime } from '@/common/api';

const packageInfo = ref<any>({
  name: '',
  description: '',
  version: '',
  license: '',
  authors: [],
  homepage: ''
});
const buildTime = ref('');

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
