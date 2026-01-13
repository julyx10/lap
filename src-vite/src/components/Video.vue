<template>
  <div ref="videoContainer" class="relative w-full h-full cursor-pointer">
    
    <TransitionGroup :name="isSlideShow ? 'slide-in' : ''">
      <div 
        v-for="index in [0, 1]" 
        v-show="activeVideo === index"
        :key="`vid-${index}`"
        class="slide-wrapper absolute inset-0 w-full h-full pointer-events-none"
      >
        <div class="w-full h-full pointer-events-auto">
          <video :ref="(el) => { if(el) videoElements[index] = el as HTMLVideoElement }" class="video-js"></video>
        </div>
      </div>
    </TransitionGroup>

    <!-- Play button overlay when video is paused -->
    <div v-if="!hasError && !isPlaying"
      class="absolute inset-0 flex items-center justify-center pointer-events-none z-10"
    >
      <div class="w-16 h-16 rounded-full bg-base-100/50 flex items-center justify-center 
                  hover:bg-base-100 hover:scale-110 transition-all duration-300 ease-out group pointer-events-auto cursor-pointer"
           @click.stop="clickPlayVideo"
      >
        <component :is="isReplaying ? IconVideoReplay : IconVideoPlay"
          class="w-8 h-8 text-base-content/50 transition-colors duration-300 group-hover:text-base-content/70"
        />
      </div>
    </div>

    <div v-if="hasError" class="absolute inset-0 flex flex-col items-center justify-center text-base-content/30 z-10">
      <IconVideoSlash class="w-8 h-8 mb-2" />
      <div class="text-center">{{ errorMessage }}</div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { IconVideoSlash } from '@/common/icons';
import videojs from 'video.js/core';
import 'video.js/dist/video-js.min.css';
import { getAssetSrc } from '@/common/utils';
import { IconVideoPlay, IconVideoReplay } from '@/common/icons';

import zhCN from 'video.js/dist/lang/zh-CN.json';
videojs.addLanguage('zh-CN', zhCN);

const props = defineProps({
  filePath: {
    type: String,
    required: false,
  },
  rotate: {
    type: Number,
    default: 0,
  },
  isZoomFit: {
    type: Boolean,
    default: false,
  },
  isSlideShow: {
    type: Boolean,
    default: false,
  },
});

const { t: $t } = useI18n(); // i18n
 
const videoContainer = ref<HTMLDivElement | null>(null);
const videoElements = ref<HTMLVideoElement[]>([]);
const players = ref<(ReturnType<typeof videojs> | null)[]>([null, null]);

const videoJsLang = computed(() => (config.settings.language === 'zh' ? 'zh-CN' : config.settings.language));

const hasError = ref(false);
const errorMessage = ref('');
const isPlaying = ref(false);
const isReplaying = ref(false);

const isFit = ref(false);
const scale = ref(1);
const rotate = ref(0);
const noTransition = ref(false);

// Double buffering state
const activeVideo = ref(0);
let currentLoadingId = 0;

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

const getActivePlayer = () => players.value[activeVideo.value];

const updateTransform = (resetZoom = false) => {
  const player = getActivePlayer();
  const video = player?.el().querySelector('video');
  if (!video) return;

  // Toggle no-transition class
  if (noTransition.value) {
    video.classList.add('no-transition');
  } else {
    video.classList.remove('no-transition');
  }

  const videoWidth = player?.videoWidth();
  const videoHeight = player?.videoHeight();
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

    rotate.value = props.rotate; // use prop rotate for reset
  }
  video.style.transform = `translate(-50%, -50%) rotate(${rotate.value}deg) scale(${scale.value})`;
};

const setupPlayer = (index: number) => {
  const el = videoElements.value[index];
  if (!el) return;

  if (!players.value[index]) {
    players.value[index] = videojs(el, playerOptions.value);
    
    const player = players.value[index]!;
    // Set volume/mute from config immediately
    player.volume(config.video.volume);
    player.muted(config.video.muted);

    player.on('error', () => {
      // Only handle error if this player is the active one or the one being loaded
      const isActive = activeVideo.value === index;
      
      const errorObj = player.error();
      if (!errorObj) return;
      
      let msg = $t('video.errors.unknown');
      switch (errorObj.code) {
        case 1: msg = $t('video.errors.aborted'); break;
        case 2: msg = $t('video.errors.network'); break;
        case 3: msg = $t('video.errors.decode'); break;
        case 4: msg = $t('video.errors.format'); break;
        default: msg = $t('video.errors.unknown');
      }

      if (isActive) {
        hasError.value = true;
        errorMessage.value = msg;
        isPlaying.value = false;
        isReplaying.value = false;
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
      }
    });

    player.on('volumechange', () => {
      if (activeVideo.value === index) {
        config.setVideoVolume(player.volume());
        config.setVideoMuted(player.muted());
      }
    });
  }
};

const clickPlayVideo = () => getActivePlayer()?.play();

const loadVideo = (filePath: string) => {
  currentLoadingId++;
  const loadingId = currentLoadingId;
  const nextUpIndex = activeVideo.value ^ 1;

  // Ensure player exists
  if (!players.value[nextUpIndex]) {
    setupPlayer(nextUpIndex);
  }
  
  const player = players.value[nextUpIndex];
  if (!player) return;

  // Stop previous playback
  player.pause();
  player.currentTime(0);

  // Check file format
  const assetSrc = getAssetSrc(filePath);
  if(!canPlay(assetSrc)) {
    // If format invalid, switch immediately to show error
    activeVideo.value = nextUpIndex;
    hasError.value = true;
    errorMessage.value = $t('video.errors.format');
    player.src('');
    return;
  }

  // Preload
  hasError.value = false;
  player.src(assetSrc);

  // One-time event listeners
  const onLoaded = () => {
    if (loadingId !== currentLoadingId) return; // Cancelled
    
    // Swap
    activeVideo.value = nextUpIndex;
    
    // Reset state for new video
    isPlaying.value = config.settings.autoPlayVideo;
    isReplaying.value = false;
    hasError.value = false;
    
    // Pause previous
    const prevPlayer = players.value[nextUpIndex ^ 1];
    if (prevPlayer) {
      prevPlayer.pause();
      prevPlayer.currentTime(0);
    }
    
    // Setup for display
    noTransition.value = true;
    isFit.value = props.isZoomFit;
    
    // Apply transform immediately (before transition removal)
    updateTransform(true);
    
    setTimeout(() => {
      noTransition.value = false;
      // Re-evaluate transform just in case
      // updateTransform(false);
    }, 100);

    if (config.settings.autoPlayVideo) {
      player.play();
    }
    
    // Clean up
    player.off('loadeddata', onLoaded);
    player.off('error', onError);
  };

  const onError = () => {
    if (loadingId !== currentLoadingId) return;
    
    activeVideo.value = nextUpIndex;
    hasError.value = true;
    // Error message set by player error handler
    
    player.off('loadeddata', onLoaded);
    player.off('error', onError);
  };

  player.one('loadeddata', onLoaded);
  player.one('error', onError);
};


// check if the file can be played
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
  // Initialize both players
  nextTick(() => {
    setupPlayer(0);
    setupPlayer(1);
    
    // Start loading first video
    if (props.filePath) {
      // Force load into index 0 directly for first run
      const index = 0;
      activeVideo.value = index;
      const player = players.value[index];
      if (player) {
         loadVideo(props.filePath);
      }
    }
  });

  if (videoContainer.value) {
    resizeObserver = new ResizeObserver(() => {
      updateTransform(props.isZoomFit)
    });
    resizeObserver.observe(videoContainer.value);
  }
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  players.value.forEach(p => {
    if (p) {
      p.off('play');
      p.off('pause');
      p.off('ended');
      p.off('loadeddata');
      p.off('volumechange');
      p.off('error');
      p.dispose();
    }
  });
  players.value = [null, null];
});

watch(videoJsLang, (newLang) => {
  players.value.forEach(p => p?.language(newLang));
});

watch(() => props.filePath, (newPath) => { 
  if (newPath) {
    loadVideo(newPath); 
  }
});

watch(() => props.rotate, (val) => { 
  rotate.value = val; 
  updateTransform(); 
});

watch(() => props.isZoomFit, (val) => { 
  isFit.value = val; 
  updateTransform(true); 
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
  pause: () => {
    players.value.forEach(p => p?.pause());
  },
});
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

/* Slideshow transition */
.slide-in-enter-active,
.slide-in-leave-active {
  transition: transform 0.8s ease;
}

.slide-in-enter-from {
  transform: translateX(calc(100% + 10px));
}

.slide-in-leave-to {
  transform: translateX(calc(-100% - 10px));
}
</style>
