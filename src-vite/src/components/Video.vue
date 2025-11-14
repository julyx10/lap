<template>
  <div ref="videoContainer" class="relative w-full h-full cursor-pointer bg-base-200">
    <video v-show="!hasError" ref="videoElement" class="video-js"></video>

    <!-- Play button overlay when video is paused -->
    <div v-if="!hasError && !isPlaying"
      class="absolute inset-0 flex items-center justify-center cursor-pointer"
      @click.stop="clickPlayVideo"
    >
      <div class="w-16 h-16 rounded-full bg-base-100/50 flex items-center justify-center 
                  hover:bg-base-100/80 hover:scale-110 transition-all duration-300 ease-out group">
        <component :is="isReplaying ? IconVideoReplay : IconVideoPlay"
          class="w-8 h-8 text-base-content/50 transition-colors duration-300 group-hover:text-base-content/80"
        />
      </div>
    </div>

    <div v-if="hasError" class="absolute inset-0 flex flex-col items-center justify-center text-base-content/30">
      <IconVideoSlash class="w-8 h-8 mb-2" />
      <div class="text-center">{{ errorMessage }}</div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed, nextTick } from 'vue';
import { emit } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { IconVideoSlash } from '@/common/icons';
import videojs from 'video.js/core';
import 'video.js/dist/video-js.min.css';
import { IconVideoPlay, IconVideoReplay } from '@/common/icons';

import ja from 'video.js/dist/lang/ja.json';
import zhCN from 'video.js/dist/lang/zh-CN.json';
videojs.addLanguage('ja', ja);
videojs.addLanguage('zh-CN', zhCN);

const props = defineProps({
  src: {
    type: String,
    required: true,
  },
  rotate: {
    type: Number,
    default: 0,
  },
  isZoomFit: {
    type: Boolean,
    default: false,
  },
});

const { t: $t } = useI18n(); // i18n

const videoContainer = ref<HTMLDivElement | null>(null);
const videoElement = ref<HTMLVideoElement | null>(null);
const player = ref<ReturnType<typeof videojs> | null>(null);

const videoJsLang = computed(() => (config.settings.language === 'zh' ? 'zh-CN' : config.settings.language));

const hasError = ref(false);
const errorMessage = ref('');
const isPlaying = ref(false);
const isReplaying = ref(false);

const isFit = ref(false);
const scale = ref(1);
const rotate = ref(0);

let isSrcChanging = false;  // disable transform when src changing

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
    fullscreenToggle: false,
    audioTrackButton: false,
    volumePanel: { inline: true },
  },
}));

const updateTransform = (resetZoom = false) => {
  if (isSrcChanging) return;
  const video = player.value?.el().querySelector('video');
  if (!video) return;

  const videoWidth = player.value?.videoWidth();
  const videoHeight = player.value?.videoHeight();
  const containerWidth = videoContainer.value?.clientWidth;
  const containerHeight = videoContainer.value?.clientHeight;
  const isRotated = rotate.value % 180 !== 0;

  // center the video
  video.style.position = 'absolute';
  video.style.top = '50%';
  video.style.left = '50%';
  video.style.objectFit = 'none';
  video.style.transformOrigin = 'center center';
  video.style.width = 'auto';
  video.style.height = 'auto';

  // reset zoom and rotate when loading new video or when zoom fit is changed
  if(resetZoom) {
    scale.value = 1;
    if (isFit.value && videoWidth && videoHeight && containerWidth && containerHeight) {
      const w = isRotated ? videoHeight : videoWidth;
      const h = isRotated ? videoWidth : videoHeight;
      scale.value = Math.min(containerWidth / w, containerHeight / h);
    }

    rotate.value = props.rotate;
  }
  video.style.transform = `translate(-50%, -50%) rotate(${rotate.value}deg) scale(${scale.value})`;
};

const setupPlayer = () => {
  if (!videoElement.value) return;

  if (!player.value) {
    player.value = videojs(videoElement.value, playerOptions.value);
    player.value.volume(config.video.volume);
    player.value.muted(config.video.muted);

    player.value.on('error', () => {
      hasError.value = true;
      const errorObj = player.value?.error();
      if (errorObj) {
        switch (errorObj.code) {
          case 1:
            errorMessage.value = $t('video.errors.aborted');
            break;
          case 2:
            errorMessage.value = $t('video.errors.network');
            break;
          case 3:
            errorMessage.value = $t('video.errors.decode');
            break;
          case 4:
            errorMessage.value = $t('video.errors.format');
            break;
          default:
            errorMessage.value = $t('video.errors.unknown');
        }
      } else {
        errorMessage.value = $t('video.errors.unknown');
      }
    });

    // Add event listeners for play/pause state
    player.value.on('play', () => {
      isPlaying.value = true;
      isReplaying.value = false;
    });

    player.value.on('pause', () => {
      isPlaying.value = false;
      isReplaying.value = false;
    });

    player.value.on('ended', () => {
      isPlaying.value = false;
      isReplaying.value = true;
    });

    player.value.on('loadeddata', () => {
      isSrcChanging = false;
      isPlaying.value = config.settings.autoPlayVideo;
      isReplaying.value = false;
      updateTransform(true);
    });
    player.value.on('volumechange', () => {
      config.setVideoVolume(player.value?.volume());
      config.setVideoMuted(player.value?.muted());
    });
  }

  if (props.src) {
    if(!canPlay(props.src)) {
      hasError.value = true;
      errorMessage.value = $t('video.errors.format');

      // set player src to empty
      player.value?.src('');
      player.value?.pause();
      return;
    }
    hasError.value = false;
    errorMessage.value = '';
    isPlaying.value = true;
    player.value.src(props.src);
    nextTick(() => {
      if(config.settings.autoPlayVideo) {
        player.value?.play();
      }
    });
  } else {
    player.value.pause();
    player.value.currentTime(0);
  }
};

// check if the file can be played
// only support mp4, webm, ogg, mov format
function canPlay(file: string): boolean {
  const ext = file.split('.').pop()?.toLowerCase();
  const video = document.createElement('video');

  switch (ext) {
    case 'mp4':
    case 'm4v':
      return !!video.canPlayType('video/mp4');
    case 'webm':
      return !!video.canPlayType('video/webm');
    case 'ogg':
    case 'ogv':
      return !!video.canPlayType('video/ogg');
    case 'mov':
      return !!video.canPlayType('video/quicktime');
    default:
      return false; // unsupported format
  }
}

const clickPlayVideo = () => player.value?.play();

let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  setupPlayer();
  if (videoElement.value?.parentElement) {
    resizeObserver = new ResizeObserver(() => {
      updateTransform(props.isZoomFit)
    });
    resizeObserver.observe(videoElement.value.parentElement);
  }
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  if (player.value) {
    player.value.off('play');
    player.value.off('pause');
    player.value.off('ended');
    player.value.off('loadeddata');
    player.value.off('volumechange');
    player.value.dispose();
    player.value = null;
  }
});

watch(videoJsLang, (newLang) => player.value?.language(newLang), { immediate: true });

watch(() => props.src, () => { 
  isSrcChanging = true;
  if (!props.isZoomFit) scale.value = 1; 
  setupPlayer(); 
});

watch(() => props.rotate, (val) => { 
  rotate.value = val; 
  updateTransform(); 
}, { immediate: true });

watch(() => props.isZoomFit, (val) => { 
  isFit.value = val; 
  updateTransform(true); 
}, { immediate: true });

watch(() => scale.value, (val) => {
  emit('message-from-image', { message: 'scale', scale: val, minScale: 0.1, maxScale: 10 });
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

defineExpose({ 
  zoomIn, 
  zoomOut, 
  zoomActual, 
  rotateRight,
  pause: () => player.value?.pause(),
});
</script>

<style>
.video-js {
  width: 100% !important;
  height: 100% !important;
  background-color: hsl(var(--b1)) !important;
  color: hsl(var(--bc)) !important;
}
.video-js video {
  width: auto !important;
  height: auto !important;
  max-width: none !important;
  max-height: none !important;
  transition: transform 0.3s ease-out !important;
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
</style>
