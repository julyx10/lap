import { computed, ref, type Ref } from 'vue';
import { check, type Update, type DownloadEvent } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { openExternalUrl } from '@/common/api';
import { useToast, type ToastPlacement } from '@/common/toast';

const UPDATE_CHECK_INTERVAL = 24 * 60 * 60 * 1000;
const UPDATE_CHECK_KEY = 'lap_last_update_check';
const UPDATE_RELEASE_NOTE_KEY = 'lap_update_release_note_version';
const UPDATE_CHECK_TIMEOUT_MS = 8_000;

function extractRawErrorMessage(error: unknown) {
  if (typeof error === 'string') return error.trim();
  if (error && typeof error === 'object' && 'message' in error && typeof (error as any).message === 'string') {
    return (error as any).message.trim();
  }
  return '';
}

function isSensitiveNetworkError(message: string) {
  const normalizedMessage = message.toLowerCase();
  return (
    normalizedMessage.includes('error sending request') ||
    normalizedMessage.includes('failed to send request') ||
    normalizedMessage.includes('failed to download') ||
    normalizedMessage.includes('timed out') ||
    normalizedMessage.includes('dns') ||
    normalizedMessage.includes('getaddrinfo') ||
    normalizedMessage.includes('connection refused') ||
    normalizedMessage.includes('connection reset') ||
    normalizedMessage.includes('network') ||
    normalizedMessage.includes('http://') ||
    normalizedMessage.includes('https://') ||
    normalizedMessage.includes('url')
  );
}

function getErrorMessage(error: unknown, fallback: string) {
  const message = extractRawErrorMessage(error);
  if (!message || isSensitiveNetworkError(message)) {
    return fallback;
  }
  return message;
}

interface AppUpdaterOptions {
  toastPlacement?: ToastPlacement;
}

export function useAppUpdater(localeMsg: Ref<any>, options: AppUpdaterOptions = {}) {
  const toast = useToast();
  const toastPlacement = options.toastPlacement ?? 'bottom-right';
  const updateAvailable = ref(false);
  const isCheckingUpdate = ref(false);
  const isInstallingUpdate = ref(false);
  const isDownloadingUpdate = ref(false);
  const isUpdateReadyToRestart = ref(false);
  const isReleaseNoteVisible = ref(false);
  const updateVersion = ref('');
  const downloadPercent = ref<number | null>(null);
  let downloadTotalBytes = 0;
  let downloadedBytes = 0;
  let currentUpdate: Update | null = null;

  const restartLabel = computed(() => localeMsg.value.settings.about.auto_update.restart);
  const releaseNoteLabel = computed(() => localeMsg.value.settings.about.auto_update.release_note);
  const releaseNoteVersion = computed(() => {
    return isReleaseNoteVisible.value ? localStorage.getItem(UPDATE_RELEASE_NOTE_KEY) || '' : '';
  });
  const releaseNoteUrl = computed(() => {
    if (!releaseNoteVersion.value) return '';
    return `https://github.com/julyx10/lap/releases/tag/v${releaseNoteVersion.value}`;
  });
  const downloadProgressLabel = computed(() => {
    if (downloadPercent.value === null) {
      return localeMsg.value.settings.about.auto_update.downloading_update;
    }
    return localeMsg.value.settings.about.auto_update.downloading.replace(
      '{percent}',
      String(downloadPercent.value)
    );
  });

  const updateButtonTooltip = computed(() => {
    if (isDownloadingUpdate.value) {
      return downloadProgressLabel.value;
    }
    if (isInstallingUpdate.value) {
      return localeMsg.value.settings.about.auto_update.installing;
    }
    if (isCheckingUpdate.value) {
      return localeMsg.value.settings.about.auto_update.checking;
    }
    if (isUpdateReadyToRestart.value) {
      return restartLabel.value;
    }
    if (isReleaseNoteVisible.value) {
      return releaseNoteLabel.value;
    }
    if (updateAvailable.value && updateVersion.value) {
      return localeMsg.value.settings.about.auto_update.new_version_available.replace('{version}', updateVersion.value);
    }
    return localeMsg.value.settings.about.auto_update.check;
  });

  const updateButtonText = computed(() => {
    if (isDownloadingUpdate.value) return downloadProgressLabel.value;
    if (isInstallingUpdate.value) return localeMsg.value.settings.about.auto_update.installing;
    if (isCheckingUpdate.value) return localeMsg.value.settings.about.auto_update.checking;
    if (isUpdateReadyToRestart.value) return restartLabel.value;
    if (isReleaseNoteVisible.value) return releaseNoteLabel.value;
    if (updateAvailable.value) return localeMsg.value.settings.about.auto_update.update;
    return localeMsg.value.settings.about.auto_update.check;
  });

  const isUpdateActionEnabled = computed(() =>
    updateAvailable.value || isUpdateReadyToRestart.value || isReleaseNoteVisible.value
  );

  function resetDownloadProgress() {
    isDownloadingUpdate.value = false;
    downloadPercent.value = null;
    downloadTotalBytes = 0;
    downloadedBytes = 0;
  }

  function resetPendingUpdateState() {
    updateAvailable.value = false;
    updateVersion.value = '';
    currentUpdate = null;
  }

  function markUpdateReadyToRestart() {
    localStorage.setItem(UPDATE_RELEASE_NOTE_KEY, updateVersion.value);
    resetPendingUpdateState();
    isUpdateReadyToRestart.value = true;
  }

  function loadReleaseNoteState() {
    isReleaseNoteVisible.value = !!localStorage.getItem(UPDATE_RELEASE_NOTE_KEY);
  }

  async function checkWithTimeout() {
    let timeoutId: number | null = null;
    try {
      return await Promise.race([
        check(),
        new Promise<never>((_, reject) => {
          timeoutId = window.setTimeout(() => {
            reject(new Error('Update check timed out after 8 seconds'));
          }, UPDATE_CHECK_TIMEOUT_MS);
        }),
      ]);
    } finally {
      if (timeoutId !== null) {
        window.clearTimeout(timeoutId);
      }
    }
  }

  function handleDownloadEvent(event: DownloadEvent) {
    if (event.event === 'Started') {
      isDownloadingUpdate.value = true;
      downloadTotalBytes = event.data.contentLength || 0;
      downloadedBytes = 0;
      downloadPercent.value = downloadTotalBytes > 0 ? 0 : null;
      toast.info(localeMsg.value.settings.about.auto_update.downloading_started, { placement: toastPlacement });
      return;
    }

    if (event.event === 'Progress') {
      downloadedBytes += event.data.chunkLength;
      if (downloadTotalBytes > 0) {
        downloadPercent.value = Math.min(100, Math.round((downloadedBytes / downloadTotalBytes) * 100));
      }
      return;
    }

    if (event.event === 'Finished') {
      isDownloadingUpdate.value = false;
      downloadPercent.value = 100;
      toast.success(localeMsg.value.settings.about.auto_update.download_finished, { placement: toastPlacement });
    }
  }

  async function checkForUpdates(manual = false) {
    if (isCheckingUpdate.value || isInstallingUpdate.value) return;

    if (!manual) {
      const lastCheck = localStorage.getItem(UPDATE_CHECK_KEY);
      if (lastCheck && Date.now() - Number(lastCheck) < UPDATE_CHECK_INTERVAL) {
        return;
      }
    }

    isCheckingUpdate.value = true;
    resetPendingUpdateState();
    resetDownloadProgress();

    try {
      const update = await checkWithTimeout();
      localStorage.setItem(UPDATE_CHECK_KEY, String(Date.now()));
      if (!update) {
        if (manual) {
          toast.info(localeMsg.value.settings.about.auto_update.latest_version, { placement: toastPlacement });
        }
        return;
      }

      updateAvailable.value = true;
      updateVersion.value = update.version;
      currentUpdate = update;
      toast.info(
        localeMsg.value.settings.about.auto_update.new_version_available.replace('{version}', update.version),
        { placement: toastPlacement }
      );
    } catch (error: unknown) {
      const message = getErrorMessage(error, localeMsg.value.settings.about.auto_update.failed_check);
      console.error('Failed to check for updates:', error);
      if (manual) {
        toast.error(message, { placement: toastPlacement });
      }
    } finally {
      isCheckingUpdate.value = false;
    }
  }

  async function installAvailableUpdate() {
    if (isInstallingUpdate.value) return;

    if (isUpdateReadyToRestart.value) {
      try {
        await relaunch();
      } catch (error: unknown) {
        const message = getErrorMessage(error, localeMsg.value.settings.about.auto_update.failed_install);
        console.error('Failed to relaunch after update:', error);
        toast.error(message, { placement: toastPlacement });
      }
      return;
    }

    if (!currentUpdate) return;

    try {
      isInstallingUpdate.value = true;
      resetDownloadProgress();
      toast.info(localeMsg.value.settings.about.auto_update.downloading_update, { placement: toastPlacement });
      await currentUpdate.downloadAndInstall(handleDownloadEvent);
      markUpdateReadyToRestart();
      toast.success(localeMsg.value.settings.about.auto_update.update_installed_waiting_restart, { placement: toastPlacement });
    } catch (error: unknown) {
      resetDownloadProgress();
      const message = getErrorMessage(error, localeMsg.value.settings.about.auto_update.failed_install);
      console.error('Failed to install update:', error);
      toast.error(message, { placement: toastPlacement });
    } finally {
      resetDownloadProgress();
      isInstallingUpdate.value = false;
    }
  }

  async function handleUpdateAction() {
    if (isInstallingUpdate.value || isCheckingUpdate.value) return;
    if (updateAvailable.value || isUpdateReadyToRestart.value) {
      await installAvailableUpdate();
      return;
    }
    if (isReleaseNoteVisible.value && releaseNoteUrl.value) {
      await openExternalUrl(releaseNoteUrl.value);
      localStorage.removeItem(UPDATE_RELEASE_NOTE_KEY);
      isReleaseNoteVisible.value = false;
      return;
    }
    await checkForUpdates(true);
  }

  loadReleaseNoteState();

  return {
    updateAvailable,
    isCheckingUpdate,
    isInstallingUpdate,
    isUpdateReadyToRestart,
    isReleaseNoteVisible,
    updateVersion,
    isDownloadingUpdate,
    downloadPercent,
    updateButtonTooltip,
    updateButtonText,
    downloadProgressLabel,
    isUpdateActionEnabled,
    checkForUpdates,
    installAvailableUpdate,
    handleUpdateAction,
  };
}
