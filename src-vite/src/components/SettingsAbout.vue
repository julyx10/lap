<template>
  <div class="flex flex-col items-center justify-start p-2 gap-4 text-center h-full text-base-content/70 cursor-default">
    
    <!-- App Header -->
    <div class="ml-4 flex flex-row items-center justify-center gap-2">
      <!-- App Logo -->
      <div class="">
        <img :src="LogoFull" class="w-24 h-24 mx-auto rounded-box drop-shadow-lg" />
      </div>
      <div class="flex flex-col">
        <h3 class="text-xl font-bold">{{ packageInfo.name }}</h3>
        <p class="mt-2">{{ packageInfo.description }}</p>
      </div>
    </div>

    <!-- Info Grid -->
    <div class="w-full max-w-md bg-base-200/30 rounded-lg p-4">
      <div class="grid grid-cols-[100px_1fr] gap-4 text-left text-sm">
        
        <div class="font-semibold">{{ $t('settings.about.package.version') }}</div>
        <div class="flex flex-col gap-1">
          <div class="flex items-center gap-2">
            <div class="font-mono mr-2">{{ packageInfo.version }}</div>
            
            <button 
              v-if="!updateAvailable"
              class="btn btn-primary btn-xs" 
              @click="checkForUpdates" 
              :disabled="checkingUpdate"
            >
              <span v-if="checkingUpdate" class="loading loading-spinner loading-xs"></span>
              {{ checkingUpdate ? $t('settings.about.auto_update.checking') : $t('settings.about.auto_update.check') }}
            </button>

            <button 
              v-if="updateAvailable"
              class="btn btn-primary btn-xs" 
              @click="installUpdate" 
              :disabled="checkingUpdate"
            >
              <span v-if="checkingUpdate" class="loading loading-spinner loading-xs"></span>
              {{ checkingUpdate ? $t('settings.about.auto_update.installing') : $t('settings.about.auto_update.install') }}
            </button>
          </div>
        </div>

        <div class="font-semibold">{{ $t('settings.about.package.build_time') }}</div>
        <div>{{ buildTime }}</div>

        <div class="font-semibold">{{ $t('settings.about.package.license') }}</div>
        <div>{{ packageInfo.license }}</div>

        <!-- <div class="font-semibold">{{ $t('settings.about.package.author') }}</div>
        <div>{{ packageInfo.authors && packageInfo.authors[0] }}</div> -->

        <div class="font-semibold">{{ $t('settings.about.package.website') }}</div>
        <div>
          <a :href="packageInfo.homepage" target="_blank" class="link hover:text-primary-focus">
            {{ packageInfo.homepage }}
          </a>
        </div>

        <div class="font-semibold">GitHub</div>
        <div>
          <a :href="packageInfo.repository" target="_blank" class="link hover:text-primary-focus">
            {{ packageInfo.repository }}
          </a>
        </div>

      </div>
    </div>
    
    <ToolTip ref="toolTip" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { getPackageInfo, getBuildTime } from '@/common/api';
import LogoFull from '@/assets/images/logo.png';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import ToolTip from '@/components/ToolTip.vue';

const { t } = useI18n();

const packageInfo = ref<any>({
  name: '',
  description: '',
  version: '',
  license: '',
  authors: [],
  homepage: ''
});
const buildTime = ref('');

// Auto Updater State
const checkingUpdate = ref(false);
const updateAvailable = ref(false);
const updateVersion = ref('');
const updateBody = ref('');
const downloaded = ref(0);
const contentLength = ref(0);

const toolTip = ref<InstanceType<typeof ToolTip> | null>(null);

let currentUpdate: any = null;

async function checkForUpdates() {
  checkingUpdate.value = true;
  updateAvailable.value = false;
  
  try {
    const update = await check();
    if (update) {
      updateAvailable.value = true;
      updateVersion.value = update.version;
      updateBody.value = update.body || '';
      currentUpdate = update;
      toolTip.value?.showTip(t('settings.about.auto_update.new_version_available', { version: update.version }));
    } else {
      toolTip.value?.showTip(t('settings.about.auto_update.latest_version'));
    }
  } catch (error: any) {
    console.error('Failed to check for updates:', error);
    toolTip.value?.showTip(error.message || t('settings.about.auto_update.failed_check'), true);
  } finally {
    checkingUpdate.value = false;
  }
}

async function installUpdate() {
  if (!currentUpdate) return;
  
  try {
    checkingUpdate.value = true;
    toolTip.value?.showTip(t('settings.about.auto_update.downloading_update'));
    
    await currentUpdate.downloadAndInstall((event: any) => {
      switch (event.event) {
        case 'Started':
          contentLength.value = event.data.contentLength || 0;
          break;
        case 'Progress':
          downloaded.value += event.data.chunkLength;
          break;
        case 'Finished':
          toolTip.value?.showTip(t('settings.about.auto_update.download_finished'));
          break;
      }
    });

    toolTip.value?.showTip(t('settings.about.auto_update.update_installed'));
    await relaunch();
  } catch (error: any) {
    console.error('Failed to install update:', error);
    toolTip.value?.showTip(error.message || t('settings.about.auto_update.failed_install'), true);
    checkingUpdate.value = false;
  }
}

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
