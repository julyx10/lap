<template>
  <div ref="videoContainer" class="relative w-full h-full overflow-hidden cursor-pointer" @wheel.prevent="handleWheel">
    <TransitionGroup :name="transitionName" @after-leave="handleTransitionEnd">
      <div
        v-for="index in [0, 1]"
        v-show="activeVideo === index"
        :key="`vid-${index}`"
        class="slide-wrapper absolute inset-0 w-full h-full pointer-events-none overflow-hidden"
      >
        <div class="w-full h-full pointer-events-auto overflow-hidden">
          <video :ref="(el) => { if (el) videoElements[index] = el as HTMLVideoElement }" class="video-js"></video>
        </div>
      </div>
    </TransitionGroup>

    <div v-if="!hasError && !isPlaying && !isLoading" class="absolute inset-0 flex items-center justify-center pointer-events-none z-10">
      <div
        class="w-16 h-16 rounded-full bg-base-100/50 flex items-center justify-center hover:bg-base-100 hover:scale-110 transition-all duration-300 ease-out group pointer-events-auto cursor-pointer"
        @click.stop="clickPlayVideo"
      >
        <component :is="isReplaying ? IconVideoReplay : IconVideoPlay" class="w-8 h-8 text-base-content/50 transition-colors duration-300 group-hover:text-base-content/70" />
      </div>
    </div>

    <div v-if="showSpinner" class="absolute inset-0 flex flex-col items-center justify-center pointer-events-none z-20 bg-base-200/20">
      <span class="loading loading-spinner loading-lg text-primary opacity-80"></span>
      <div class="mt-4 text-sm font-medium text-base-content/80 drop-shadow-md">{{ $t('video.loading') }}</div>
    </div>

    <div v-if="hasError && !isLoading" class="absolute inset-0 flex flex-col items-center justify-center z-10 px-6 text-center overflow-hidden bg-black/50">
      
      <div class="relative z-20 flex flex-col items-center justify-center">
        <IconVideoSlash class="w-10 h-10 mb-3 text-base-content/30" />
        <div class="max-w-md text-sm whitespace-pre-line text-base-content/30 font-medium">{{ errorMessage }}</div>
        <div v-if="canOpenExternalApp" class="mt-4 pointer-events-auto">
          <button class="btn btn-primary btn-sm" @click.stop="openInExternalApp">{{ externalOpenLabel }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { IconVideoSlash, IconVideoPlay, IconVideoReplay } from '@/common/icons';
import videojs from 'video.js/core';
import 'video.js/dist/video-js.min.css';
import { getAssetSrc } from '@/common/utils';
import { openFileWithApp } from '@/common/api';
import zhCN from 'video.js/dist/lang/zh-CN.json';
import { prepareVideo, cancelVideoPrepare } from '@/common/video';

videojs.addLanguage('zh-CN', zhCN);

const props = defineProps({
  filePath: { type: String, required: false },
  rotate: { type: Number, default: 0 },
  isZoomFit: { type: Boolean, default: false },
  isSlideShow: { type: Boolean, default: false },
});

const emit = defineEmits(['message-from-video-viewer', 'slideshow-next', 'scale', 'viewport-change']);
const { t: $t } = useI18n();

const videoContainer = ref<HTMLDivElement | null>(null);
const videoElements = ref<HTMLVideoElement[]>([]);
const players = ref<(ReturnType<typeof videojs> | null)[]>([null, null]);
const videoJsLang = computed(() => (config.settings.language === 'zh' ? 'zh-CN' : config.settings.language));

const hasError = ref(false);
const errorMessage = ref('');
const isLoading = ref(false);
const showSpinner = ref(false);
const isPlaying = ref(false);
const isReplaying = ref(false);
const isFit = ref(false);
const scale = ref(1);
const rotate = ref(0);
const noTransition = ref(false);
const activeVideo = ref(0);
let currentLoadingId = 0;

const externalVideoAppPath = computed(() => String(config.settings?.externalVideoAppPath || '').trim());
const externalVideoAppName = computed(() => String(config.settings?.externalVideoAppName || '').trim());
const canOpenExternalApp = computed(() => !!(props.filePath && externalVideoAppPath.value));
const externalOpenLabel = computed(() => {
  if (externalVideoAppName.value) {
    return $t('video.errors.open_in_external_app_named', { app: externalVideoAppName.value }) || `Open in ${externalVideoAppName.value}`;
  }
  return $t('video.errors.open_in_external_app') || 'Open in external player';
});

async function openInExternalApp() {
  if (!props.filePath || !externalVideoAppPath.value) return;
  await openFileWithApp(props.filePath, externalVideoAppPath.value);
}

let isTouchpadDevice = false;
let horizontalDeltaAccumulator = 0;
let verticalDeltaAccumulator = 0;
let gestureResetTimeout: ReturnType<typeof setTimeout> | null = null;
let hasNavigatedThisGesture = false;
const gestureType = ref<'none' | 'zoom' | 'nav'>('none');
const navDirection = ref<'next' | 'prev' | ''>('');
let lastDeltaX = 0;
const GESTURE_LOCK_THRESHOLD = 10;
const HORIZONTAL_NAV_THRESHOLD = 100;

const transitionName = computed(() => {
  if (props.isSlideShow) return 'slide-next';
  if (navDirection.value) return navDirection.value === 'next' ? 'slide-next' : 'slide-prev';
  return '';
});

function handleTransitionEnd() {
  navDirection.value = '';
}

function resetSwipeState() {
  gestureType.value = 'none';
  horizontalDeltaAccumulator = 0;
  verticalDeltaAccumulator = 0;
  hasNavigatedThisGesture = false;
  lastDeltaX = 0;
  navDirection.value = '';
  if (gestureResetTimeout) {
    clearTimeout(gestureResetTimeout);
    gestureResetTimeout = null;
  }
}

const playerOptions = computed(() => ({
  responsive: false,
  fluid: false,
  width: '100%',
  height: '100%',
  autoplay: false,
  muted: config.video.muted,
  controls: true,
  preload: 'auto',
  language: videoJsLang.value,
  playbackRates: [0.5, 1, 1.25, 1.5, 2],
  disablePictureInPicture: true,
  errorDisplay: false,
  controlBar: {
    pictureInPictureToggle: false,
    playbackRateMenuButton: false,
    fullscreenToggle: true,
    audioTrackButton: false,
    volumePanel: { inline: true },
  },
}));

const getActivePlayer = () => players.value[activeVideo.value];

const updateTransform = (options: boolean | { resetRotation?: boolean, recalcScale?: boolean } = false) => {
  const resetRotation = typeof options === 'boolean' ? options : (options.resetRotation ?? false);
  const recalcScale = typeof options === 'boolean' ? options : (options.recalcScale ?? false);
  const player = getActivePlayer();
  const video = player?.el().querySelector('video') as HTMLVideoElement | null;
  if (!video) return;

  if (noTransition.value) video.classList.add('no-transition');
  else video.classList.remove('no-transition');

  if (resetRotation) rotate.value = props.rotate;

  const videoWidth = player?.videoWidth();
  const videoHeight = player?.videoHeight();
  const containerWidth = videoContainer.value?.clientWidth;
  const containerHeight = videoContainer.value?.clientHeight;
  const isRotated = rotate.value % 180 !== 0;

  // IMPORTANT: Set dimensions to natural size to prevent clipping inside the tag
  if (videoWidth && videoHeight) {
    video.style.width = `${videoWidth}px`;
    video.style.height = `${videoHeight}px`;
  } else {
    video.style.width = 'auto';
    video.style.height = 'auto';
  }

  video.style.position = 'absolute';
  video.style.top = '50%';
  video.style.left = '50%';
  video.style.objectFit = 'fill'; // Use fill since we handle the element size
  video.style.transformOrigin = 'center center';
  video.style.maxWidth = 'none';
  video.style.maxHeight = 'none';

  if (recalcScale) {
    scale.value = 1;
    if (isFit.value && videoWidth && videoHeight && containerWidth && containerHeight) {
      const w = isRotated ? videoHeight : videoWidth;
      const h = isRotated ? videoWidth : videoHeight;
      scale.value = Math.min(containerWidth / w, containerHeight / h);
    }
  }

  video.style.transform = `translate(-50%, -50%) rotate(${rotate.value}deg) scale(${scale.value})`;

  emit('scale', { scale: scale.value, minScale: 0.1, maxScale: 10 });
  emit('viewport-change', { scale: scale.value, isZoomFit: isFit.value, fileType: 2 });
};

const setupPlayer = (index: number) => {
  const el = videoElements.value[index];
  if (!el) return;
  if (!players.value[index]) {
    players.value[index] = videojs(el, playerOptions.value);
    const player = players.value[index]!;
    player.volume(config.video.volume);
    player.muted(config.video.muted);

    player.on('error', () => {
      if (activeVideo.value === index) {
        handlePlayerError(player);
      }
    });

    player.on('play', () => {
      if (activeVideo.value === index) {
        isPlaying.value = true;
        isReplaying.value = false;
      }
    });

    player.on('pause', () => {
      if (activeVideo.value === index) {
        isPlaying.value = false;
        isReplaying.value = false;
      }
    });

    player.on('ended', () => {
      if (activeVideo.value === index) {
        isPlaying.value = false;
        isReplaying.value = true;
        if (props.isSlideShow) {
          emit('slideshow-next');
        }
      }
    });

    player.on('volumechange', () => {
      if (activeVideo.value === index && !isLoading.value) {
        config.setVideoVolume(player.volume());
        config.setVideoMuted(player.muted());
      }
    });
  }
};

const clickPlayVideo = () => getActivePlayer()?.play();

const loadVideo = async (filePath: string) => {
  if (!filePath) return;
  const currentLoadId = ++currentLoadingId;
  
  // IMMEDIATELY set loading state to block volumechange feedbacks from old players
  hasError.value = false;
  isPlaying.value = false;
  isReplaying.value = false;
  isLoading.value = true;
  showSpinner.value = false;

  const currentPlayer = getActivePlayer();
  if (currentPlayer) {
    currentPlayer.pause();
    currentPlayer.reset();
  }

  const nextUpIndex = activeVideo.value ^ 1;
  const player = players.value[nextUpIndex];
  if (!player) return;

  // Sync audio state IMMEDIATELY so the UI reflects the user settings during loading
  player.muted(config.video.muted);
  player.volume(config.video.volume);
  
  setTimeout(() => {
    if (currentLoadId === currentLoadingId && !hasError.value && activeVideo.value !== nextUpIndex) {
      showSpinner.value = true;
    }
  }, 1000);

  const handleSuccessfulLoad = () => {
    if (currentLoadId !== currentLoadingId) return;
    activeVideo.value = nextUpIndex;
    hasError.value = false;
    isLoading.value = false;
    showSpinner.value = false;

    // Pause the other player
    const prevPlayer = players.value[nextUpIndex ^ 1];
    if (prevPlayer) {
      prevPlayer.pause();
      prevPlayer.reset();
    }

    noTransition.value = true;
    isFit.value = props.isZoomFit;
    rotate.value = props.rotate;
    updateTransform({ resetRotation: true, recalcScale: true });

    setTimeout(() => {
      noTransition.value = false;
    }, 100);

    if (config.settings.autoPlayVideo || props.isSlideShow) {
      // Restore user audio settings right before playback starts
      player.volume(config.video.volume);
      player.muted(config.video.muted);
      player.play().catch(() => {});
    } else {
      player.volume(config.video.volume);
      player.muted(config.video.muted);
    }
  };

  const tryPlayProcessed = async () => {
    try {
      // Let the backend choose remux vs transcode for the current platform.
      const result = await prepareVideo(filePath, String(nextUpIndex));
      if (currentLoadId !== currentLoadingId) return;

      player.reset();
      player.src({
        src: getAssetSrc(result.url),
        type: result.action === 'remux' ? 'video/mp4' : (result.url.endsWith('.webm') ? 'video/webm' : 'video/mp4'),
      });

      const onLoadedFallback = () => {
        player.off('error', onErrorFallback);
        handleSuccessfulLoad();
      };

      const onErrorFallback = () => {
        if (currentLoadId !== currentLoadingId) return;
        isLoading.value = false;
        showSpinner.value = false;
        handlePlayerError(player);
        player.off('loadeddata', onLoadedFallback);
        player.off('error', onErrorFallback);
      };

      player.one('loadeddata', onLoadedFallback);
      player.one('error', onErrorFallback);
      player.load();
    } catch (e) {
      if (currentLoadId !== currentLoadingId) return;
      isLoading.value = false;
      showSpinner.value = false;
      console.error('[Video] Prepare failed:', e);
      hasError.value = true;
      errorMessage.value = getFallbackErrorMessage();
    }
  };

  const tryPlayDirect = () => {
    player.src({ src: getAssetSrc(filePath) });

    const onLoadedDirect = () => {
      player.off('error', onErrorDirect);
      handleSuccessfulLoad();
    };

    const onErrorDirect = () => {
      player.off('loadeddata', onLoadedDirect);
      const err = player.error();
      if (err && err.code === 1) {
        return; // Aborted
      }
      if (currentLoadId !== currentLoadingId) return;
      
      console.warn('[Video] Direct play failed (code ' + err?.code + '), falling back to FFmpeg processing...');
      tryPlayProcessed();
    };

    player.one('loadeddata', onLoadedDirect);
    player.one('error', onErrorDirect);
    player.load();
  };

  // Start by checking if we should skip direct play for problematic formats
  const ext = filePath.split('.').pop()?.toLowerCase();
  // TODO: Remove this when we support MPG/MPEG
  if (ext === 'mpg' || ext === 'mpeg') {
    console.warn('[Video] Skipping direct play for legacy format:', ext);
    tryPlayProcessed();
  } else {
    tryPlayDirect();
  }
};

function getFallbackErrorMessage() {
  const formatMsg = $t('video.errors.format');
  if (canOpenExternalApp.value && externalVideoAppName.value) {
    return `${formatMsg}\n${$t('video.errors.use_external_with_app', { app: externalVideoAppName.value })}`;
  }
  if (canOpenExternalApp.value) {
    return `${formatMsg}\n${$t('video.errors.use_external_generic')}`;
  }
  return `${formatMsg}\n${$t('video.errors.use_external')}`;
}

function handlePlayerError(playerInstance: ReturnType<typeof videojs>) {
  const err = playerInstance.error();
  
  // AbortError is frequently thrown locally when the stream is reset/cleared during transitions. 
  // Ignoring it prevents spurious error overlays.
  if (err && err.code === 1) {
    return;
  }

  let msg = $t('video.errors.unknown');
  
  if (err) {
    switch (err.code) {
      case 1: msg = $t('video.errors.aborted'); break;
      case 2: msg = $t('video.errors.network'); break;
      case 3: msg = $t('video.errors.decode'); break;
      case 4: msg = getFallbackErrorMessage(); break;
    }
  } else {
    msg = $t('video.errors.playback_failed') || 'Playback failed';
  }

  hasError.value = true;
  errorMessage.value = msg;
  isPlaying.value = false;
  isReplaying.value = false;
}

let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  nextTick(() => {
    setupPlayer(0);
    setupPlayer(1);
    if (props.filePath) {
      activeVideo.value = 0;
      loadVideo(props.filePath);
    }
  });

  if (videoContainer.value) {
    resizeObserver = new ResizeObserver(() => {
      updateTransform({ recalcScale: true });
    });
    resizeObserver.observe(videoContainer.value);
  }
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  players.value.forEach((p) => {
    if (p) {
      p.off();
      setTimeout(() => {
        try { p.dispose(); } catch (e) {}
      }, 0);
    }
  });
  players.value = [null, null];
  cancelVideoPrepare('0');
  cancelVideoPrepare('1');
});

watch(() => props.filePath, (newPath) => {
  if (newPath) loadVideo(newPath);
});

watch(() => props.rotate, (val) => {
  rotate.value = val;
  updateTransform();
});

watch(() => props.isZoomFit, (val) => {
  isFit.value = val;
  updateTransform({ recalcScale: true });
});

watch(() => props.isSlideShow, (newVal) => {
  if (newVal) {
    const player = getActivePlayer();
    if (player && !isPlaying.value) {
      player.play();
    }
  }
});

const zoomIn = () => {
  scale.value = Math.min(scale.value * 2, 10);
  updateTransform();
};
const zoomOut = () => {
  scale.value = Math.max(scale.value / 2, 0.1);
  updateTransform();
};
const zoomActual = () => {
  scale.value = 1;
  updateTransform();
};
const rotateRight = () => {
  rotate.value = (rotate.value + 90) % 360;
  updateTransform();
};
const togglePlay = () => {
  const player = getActivePlayer();
  if (!player) return;
  if (isPlaying.value) player.pause();
  else player.play();
};

function getViewportState() {
  return { scale: scale.value, isZoomFit: isFit.value, fileType: 2 };
}

function applyViewportState(viewport: { scale?: number; isZoomFit?: boolean }, silent = false) {
  if (!viewport) return;
  if (typeof viewport.isZoomFit === 'boolean') {
    isFit.value = viewport.isZoomFit;
    if (viewport.isZoomFit) {
      updateTransform({ recalcScale: true });
      return;
    }
  }

  if (typeof viewport.scale === 'number') {
    scale.value = Math.max(0.1, Math.min(10, viewport.scale));
    isFit.value = false;
    if (silent) {
      noTransition.value = true;
      updateTransform();
      requestAnimationFrame(() => {
        noTransition.value = false;
      });
      return;
    }
    updateTransform();
  }
}

defineExpose({
  zoomIn,
  zoomOut,
  zoomActual,
  rotateRight,
  togglePlay,
  getViewportState,
  applyViewportState,
  pause: () => {
    players.value.forEach((p) => p?.pause());
  },
});

function handleWheel(event: WheelEvent) {
  event.preventDefault();
  if (event.deltaX !== 0) {
    isTouchpadDevice = true;
  }

  if (gestureResetTimeout) clearTimeout(gestureResetTimeout);
  gestureResetTimeout = setTimeout(() => {
    resetSwipeState();
  }, 150);

  const isTouchPad = isTouchpadDevice;

  if (isTouchPad) {
    if (hasNavigatedThisGesture) {
      const speedIncreased = Math.abs(event.deltaX) > Math.abs(lastDeltaX) + 5;
      if (!speedIncreased) {
        lastDeltaX = event.deltaX;
        return;
      }
      hasNavigatedThisGesture = false;
      horizontalDeltaAccumulator = 0;
    }
    lastDeltaX = event.deltaX;

    if (gestureType.value === 'none') {
      horizontalDeltaAccumulator += event.deltaX;
      verticalDeltaAccumulator += event.deltaY;
      const absX = Math.abs(horizontalDeltaAccumulator);
      const absY = Math.abs(verticalDeltaAccumulator);
      if (absX > GESTURE_LOCK_THRESHOLD || absY > GESTURE_LOCK_THRESHOLD) {
        gestureType.value = absX > absY ? 'nav' : 'zoom';
      }
      return;
    }

    if (gestureType.value === 'nav') {
      horizontalDeltaAccumulator += event.deltaX;
      if (!hasNavigatedThisGesture && Math.abs(horizontalDeltaAccumulator) >= HORIZONTAL_NAV_THRESHOLD) {
        const direction = horizontalDeltaAccumulator > 0 ? 'next' : 'prev';
        navDirection.value = direction;
        emit('message-from-video-viewer', { message: direction });
        hasNavigatedThisGesture = true;
        horizontalDeltaAccumulator = 0;
        gestureType.value = 'none';
      }
      return;
    }

    if (gestureType.value === 'zoom' || Math.abs(event.deltaY) > Math.abs(event.deltaX)) {
      const zoomFactor = 0.01;
      const delta = -event.deltaY * zoomFactor;
      scale.value = Math.max(0.1, Math.min(10, scale.value + delta));
      updateTransform();
    }
  } else {
    if (config.settings.mouseWheelMode === 0) {
      if (event.ctrlKey) {
        const zoomFactor = 0.1;
        scale.value = event.deltaY < 0
          ? Math.min(scale.value * (1 + zoomFactor), 10)
          : Math.max(scale.value * (1 - zoomFactor), 0.1);
        updateTransform();
      } else {
        const direction = event.deltaY < 0 ? 'prev' : 'next';
        emit('message-from-video-viewer', { message: direction });
      }
    } else {
      const zoomFactor = 0.1;
      scale.value = event.deltaY < 0
        ? Math.min(scale.value * (1 + zoomFactor), 10)
        : Math.max(scale.value * (1 - zoomFactor), 0.1);
      updateTransform();
    }
  }
}
</script>

<style>
.video-js {
  width: 100% !important;
  height: 100% !important;
  background-color: transparent !important;
  color: hsl(var(--bc)) !important;
}
.video-js video {
  width: auto !important;
  height: auto !important;
  max-width: none !important;
  max-height: none !important;
  transition: transform 0.3s ease-out !important;
}
.video-js video.no-transition {
  transition: none !important;
}
.video-js .vjs-control-bar {
  background-color: hsl(var(--b2)) !important;
}
.video-js .vjs-big-play-button {
  display: none !important;
}
.vjs-volume-panel {
  position: relative !important;
}
.slide-next-enter-active,
.slide-next-leave-active,
.slide-prev-enter-active,
.slide-prev-leave-active {
  transition: transform 0.6s cubic-bezier(0.4, 0, 0.2, 1);
}
.slide-next-enter-from { transform: translateX(100%); }
.slide-next-leave-to { transform: translateX(-100%); }
.slide-prev-enter-from { transform: translateX(-100%); }
.slide-prev-leave-to { transform: translateX(100%); }
.slide-in-enter-active,
.slide-in-leave-active {
  transition: transform 0.6s cubic-bezier(0.4, 0, 0.2, 1);
}
.slide-in-enter-from { transform: translateX(100%); }
.slide-in-leave-to { transform: translateX(-100%); }
</style>
