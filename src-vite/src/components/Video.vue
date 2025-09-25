<template>
  <div class="relative w-full h-full overflow-hidden cursor-pointer bg-base-200">
    <video v-show="!hasError" ref="videoElement" class="video-js"></video>

    <!-- Play button overlay when video is paused -->
    <div v-if="!hasError && !isPlaying"
      class="absolute inset-0 flex items-center justify-center cursor-pointer"
      @click.stop="playVideo"
    >
      <div class="w-16 h-16 rounded-full bg-base-100/50 flex items-center justify-center 
                  hover:bg-base-100/80 hover:scale-110 transition-all duration-300 ease-out group">
        <component :is="isReplaying ? IconVideoReplay : IconVideoPlay"
          class="w-8 h-8 text-base-content/50 transition-colors duration-300 group-hover:text-base-content"
        />
      </div>
    </div>

    <div v-if="hasError" class="absolute inset-0 flex flex-col items-center justify-center text-base-content/30">
      <IconVideoSlash class="w-8 h-8 mb-2" />
      <div class="text-center">{{ errorMessage }}</div>
    </div>

    <!-- show video config info -->
    <div v-if="config.debugMode" class="absolute top-0 left-0 right-0 text-sm bg-base-100 opacity-60 rounded-lg">
      <div>video isZoomFit: {{ props.isZoomFit }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed, reactive, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/utils';
import { IconVideoSlash } from '@/common/icons';
import videojs from 'video.js';
import 'video.js/dist/video-js.css';

import { IconVideoPlay, IconVideoReplay } from '@/common/icons';

// Import language JSON data for video.js
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

const videoElement = ref<HTMLVideoElement | null>(null);
const player = ref<ReturnType<typeof videojs> | null>(null);

// status
const hasError = ref(false);
const errorMessage = ref('');
const isPlaying = ref(false);
const isReplaying = ref(false);

// transform state
const transformState = reactive({ rotate: 0, scale: 1 });

const videoJsLang = computed(() => (config.language === 'zh' ? 'zh-CN' : config.language));

const playerOptions = computed(() => ({
  responsive: false,
  fluid: false,
  width: '100%',
  height: '100%',
  autoplay: false,
  muted: config.videoMuted,
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

const updateTransform = () => {
  const video = player.value?.el().querySelector('video');
  if (!video) return;

  if (props.isZoomFit) {
    const videoWidth = player.value?.videoWidth();
    const videoHeight = player.value?.videoHeight();
    const containerWidth = videoElement.value?.parentElement?.clientWidth || 0;
    const containerHeight = videoElement.value?.parentElement?.clientHeight || 0;
    const isRotated = transformState.rotate % 180 !== 0;

    video.style.objectFit = 'contain';
    video.style.position = '';
    video.style.top = '';
    video.style.left = '';
    video.style.transformOrigin = 'center center';

    if (isRotated && videoWidth && videoHeight && containerWidth && containerHeight) {
      const scaleContain = Math.min(containerWidth / videoWidth, containerHeight / videoHeight);
      const scaleCorrect = (1 / scaleContain) * Math.min(containerWidth / videoHeight, containerHeight / videoWidth);
      video.style.transform = `rotate(${transformState.rotate}deg) scale(${scaleCorrect})`;
    } else {
      video.style.transform = `rotate(${transformState.rotate}deg) scale(1)`;
    }
  } else {
    video.style.objectFit = 'none';
    video.style.position = 'absolute';
    video.style.top = '50%';
    video.style.left = '50%';
    video.style.transform = `translate(-50%, -50%) rotate(${transformState.rotate}deg) scale(${transformState.scale})`;
    video.style.transformOrigin = 'center center';
  }
};

const playVideo = () => {
  if (player.value) {
    player.value.play();
  }
};

const setupPlayer = () => {
  if (!videoElement.value) return;

  if (!player.value) {
    player.value = videojs(videoElement.value, playerOptions.value);
    player.value.volume(config.videoVolume);
    player.value.muted(config.videoMuted);

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

    player.value.ready(() => updateTransform());

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
      isPlaying.value = config.autoPlayVideo;
      isReplaying.value = false;
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
      return;
    }
    hasError.value = false;
    errorMessage.value = '';
    isPlaying.value = true;
    player.value.src(props.src);
    nextTick(() => {
      if(config.autoPlayVideo) {
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

let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  setupPlayer();
  if (videoElement.value?.parentElement) {
    resizeObserver = new ResizeObserver(() => {
      updateTransform();
    });
    resizeObserver.observe(videoElement.value.parentElement);
  }
});

onBeforeUnmount(() => {
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
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

watch(() => props.src, () => setupPlayer());
watch(videoJsLang, (newLang) => player.value?.language(newLang), { immediate: true });

watch(() => props.rotate, (val) => {
  transformState.rotate = val;
  updateTransform();
});

watch(() => props.isZoomFit, () => updateTransform(), { immediate: true });

const zoomIn = () => {
  console.log('zoomIn');
  transformState.scale *= 1.5;
  updateTransform();
};
const zoomOut = () => {
  console.log('zoomOut');
  transformState.scale /= 1.5;
  updateTransform();
};
const zoomActual = () => {
  console.log('zoomActual');
  transformState.scale = 1;
  updateTransform();
};
const rotateRight = () => {
  console.log('rotateRight');
  transformState.rotate += 90;
  updateTransform();
};

defineExpose({ 
  zoomIn, 
  zoomOut, 
  zoomActual, 
  rotateRight 
});
</script>

<style>
.video-js {
  width: 100% !important;
  height: 100% !important;
  max-height: 100% !important;
  background-color: hsl(var(--b1)) !important;
  color: hsl(var(--bc)) !important;
  object-fit: contain !important;
}
.video-js video {
  width: 100% !important;
  height: 100% !important;
  max-height: 100% !important;
  object-fit: contain !important;
}
.video-js .vjs-control-bar {
  z-index: 9999 !important;
  background-color: hsl(var(--b2)) !important;
}
.video-js .vjs-big-play-button {
  z-index: 9999 !important;
  display: none !important;
}
.vjs-volume-panel {
  z-index: 9999 !important;
  position: relative !important;
}
</style>
