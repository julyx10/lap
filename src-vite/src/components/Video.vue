<template>
  <div ref="playerContainer" class="relative w-full h-full overflow-hidden cursor-pointer bg-base-200">

    <!-- Video.js element -->
    <video v-show="!hasError && !showLoading" ref="videoElement" class="video-js"></video>
    
    <!-- Loading indicator -->
    <div v-if="showLoading" class="absolute inset-0 flex items-center justify-center text-base-content/30">
      <div class="text-center">
        <div class="loading loading-spinner loading-lg mb-4"></div>
        <div>{{ $t('video.loading') }}</div>
      </div>
    </div>
    
    <!-- Error display -->
    <div v-if="hasError" class="absolute inset-0 flex flex-col items-center justify-center text-base-content/30">
      <IconVideoSlash class="w-8 h-8 mb-2" />
      <div class="text-center">
        <!-- <div class="mb-2">{{ $t('video.failed') }}</div> -->
        <div>{{ errorMessage }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/utils';

import { IconVideoSlash } from '@/common/icons';

// Import video.js
import videojs from 'video.js';
import 'video.js/dist/video-js.css';

// Import language JSON data
import ja from 'video.js/dist/lang/ja.json';
import zhCN from 'video.js/dist/lang/zh-CN.json';

// Register the languages with video.js
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

const playerContainer = ref<HTMLElement | null>(null);
const videoElement = ref<HTMLVideoElement | null>(null);
const player = ref<any>(null); // video.js player

// Error state management
const hasError = ref(false); // error state
const errorMessage = ref(''); // error message

// Loading state management
const isLoading = ref(false); // loading state
const showLoading = ref(false); // show loading indicator
const loadTimeout = ref<NodeJS.Timeout | null>(null); // timeout for loading the video
const loadingDelayTimeout = ref<NodeJS.Timeout | null>(null); // delay for showing the loading indicator

// Map app language to video.js language codes
const videoJsLang = computed(() => {
  if (config.language === 'zh') {
    return 'zh-CN';
  }
  return config.language;
});

const playerOptions = computed(() => ({
  responsive: true,
  fluid: true,
  // aspectRatio: '16:9',
  autoplay: true,
  controls: true,
  preload: 'auto',
  language: videoJsLang.value, // Use mapped language
  playbackRates: [0.5, 1, 1.25, 1.5, 2],
  disablePictureInPicture: true,
  errorDisplay: false,
  controlBar: {
    pictureInPictureToggle: false, // hide picture in picture toggle
    playbackRateMenuButton: false, // hide playback rate menu button
    fullscreenToggle: false,
    audioTrackButton: false,
    volumePanel: {
      inline: false
    }
  }
}));

const clearLoadTimeout = () => {
  if (loadTimeout.value) {
    clearTimeout(loadTimeout.value);
    loadTimeout.value = null;
  }
};

const clearLoadingDelayTimeout = () => {
  if (loadingDelayTimeout.value) {
    clearTimeout(loadingDelayTimeout.value);
    loadingDelayTimeout.value = null;
  }
};

const setupLoadingDelay = () => {
  clearLoadingDelayTimeout();
  loadingDelayTimeout.value = setTimeout(() => {
    if (isLoading.value) {
      showLoading.value = true;
    }
  }, 500); // 0.5 second delay
};

const setupLoadTimeout = () => {
  clearLoadTimeout();
  loadTimeout.value = setTimeout(() => {
    if (isLoading.value) {
      hasError.value = true;
      isLoading.value = false;
      showLoading.value = false;
      errorMessage.value = $t('video.errors.timeout');
    }
  }, 5000); // 5 seconds timeout
};

const setupPlayer = () => {
  if (!videoElement.value) return;

  if (!props.src) {
    if (player.value) {
      player.value.dispose();
      player.value = null;
    }
    clearLoadTimeout();
    isLoading.value = false;
    hasError.value = false;
    return;
  }

  // Reset states
  hasError.value = false;
  errorMessage.value = '';
  isLoading.value = true;
  showLoading.value = false;
  setupLoadTimeout();
  setupLoadingDelay();

  if (player.value) {
    // record the previous playing state
    const wasPlaying = !player.value.paused();
    
    // update the video source
    player.value.src(props.src);

    // if the previous state was playing, then play the new video after loading
    if (wasPlaying) {
      player.value.one('loadedmetadata', () => {
        clearLoadTimeout();
        clearLoadingDelayTimeout();
        isLoading.value = false;
        showLoading.value = false;
        player.value.play();
      });
    } else {
      player.value.one('loadedmetadata', () => {
        clearLoadTimeout();
        clearLoadingDelayTimeout();
        isLoading.value = false;
        showLoading.value = false;
      });
    }
  } else {
    // create new player
    player.value = videojs(videoElement.value, playerOptions.value);
    player.value.src(props.src);

    player.value.on('error', (error) => {
      clearLoadTimeout();
      clearLoadingDelayTimeout();
      isLoading.value = false;
      showLoading.value = false;
      hasError.value = true;
      const errorObj = player.value.error();
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

    player.value.on('loadedmetadata', () => {
      clearLoadTimeout();
      clearLoadingDelayTimeout();
      isLoading.value = false;
      showLoading.value = false;
    });

    player.value.ready(() => {
      const video = player.value.el().querySelector('video');
      if (video) {
        video.style.objectFit = props.isZoomFit ? 'contain' : 'cover';
      }
    });
  }
};

// Setup the player
onMounted(setupPlayer);

onBeforeUnmount(() => {
  clearLoadTimeout();
  clearLoadingDelayTimeout();
  if (player.value) {
    player.value.dispose();
  }
});

watch(() => props.src, (newSrc, oldSrc) => {
  if (newSrc !== oldSrc) {
    clearLoadTimeout();
    clearLoadingDelayTimeout();
    hasError.value = false;
    errorMessage.value = '';
    isLoading.value = false;
    showLoading.value = false;
    setupPlayer();
  }
});

watch(videoJsLang, (newLang) => {
  if (player.value) {
    player.value.language(newLang);
  }
});

watch(() => props.rotate, (newRotate) => {
  if (player.value) {
    const video = player.value.el().querySelector('video');
    if (video) {
      video.style.transform = `rotate(${newRotate}deg)`;
    }
  }
});

watch(() => props.isZoomFit, (newFit) => {
  if (player.value) {
    const video = player.value.el().querySelector('video');
    if (video) {
      video.style.objectFit = newFit ? 'contain' : 'cover';
    }
  }
});

const zoomIn = () => {
  if (player.value) {
    const video = player.value.el().querySelector('video');
    if (video) {
      const currentScale = parseFloat(video.style.transform?.match(/scale\(([^)]+)\)/)?.[1] || '1');
      video.style.transform = `scale(${currentScale * 1.5})`;
    }
  }
}

const zoomOut = () => {
  if (player.value) {
    const video = player.value.el().querySelector('video');
    if (video) {
      const currentScale = parseFloat(video.style.transform?.match(/scale\(([^)]+)\)/)?.[1] || '1');
      video.style.transform = `scale(${currentScale / 1.5})`;
    }
  }
}

const zoomActual = () => {
  if (player.value) {
    const video = player.value.el().querySelector('video');
    if (video) {
      video.style.transform = 'scale(1)';
    }
  }
}

const rotateRight = () => {
  if (player.value) {
    const video = player.value.el().querySelector('video');
    if (video) {
      const currentRotate = parseInt(video.style.transform?.match(/rotate\(([^)]+)deg\)/)?.[1] || '0');
      video.style.transform = `rotate(${currentRotate + 90}deg)`;
    }
  }
}

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
  background-color: hsl(var(--b1)) !important;
  color: hsl(var(--bc)) !important;
}
.video-js .vjs-control-bar {
  background-color: hsl(var(--b2)) !important;
}
.video-js .vjs-big-play-button {
  display: none !important;
}
</style>
