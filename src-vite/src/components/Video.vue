<template>
  <div ref="playerContainer" class="relative w-full h-full overflow-hidden cursor-pointer bg-base-200">
    <video v-show="!hasError" ref="videoElement" class="video-js"></video>
    <div v-if="hasError" class="absolute inset-0 flex items-center justify-center text-base-content">
      <div class="text-center">
        <div class="text-lg font-medium mb-2">{{ $t('video.failed') }}</div>
        <div class="text-sm">{{ $t('video.error') }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed } from 'vue';
import { config } from '@/common/utils';

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
  autoplay: {
    type: Boolean,
    default: false,
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

const emit = defineEmits(['play', 'pause', 'error']);

const playerContainer = ref<HTMLElement | null>(null);
const videoElement = ref<HTMLVideoElement | null>(null);
const player = ref<any>(null);

// Error state management
const hasError = ref(false);
const errorMessage = ref('');

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
  aspectRatio: '16:9',
  autoplay: props.autoplay,
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

const setupPlayer = () => {
  if (!videoElement.value) return;

  if (!props.src) {
    if (player.value) {
      player.value.dispose();
      player.value = null;
    }
    return;
  }

  if (player.value) {
    // record the previous playing state
    const wasPlaying = !player.value.paused();
    
    // update the video source
    player.value.src(props.src);

    // if the previous state was playing, then play the new video after loading
    if (wasPlaying) {
      player.value.one('loadedmetadata', () => {
        player.value.play();
      });
    }
  } else {
    // create new player
    player.value = videojs(videoElement.value, playerOptions.value);
    player.value.src(props.src);

    player.value.on('play', () => emit('play'));
    player.value.on('pause', () => emit('pause'));

    player.value.on('error', () => {
      hasError.value = true;
      errorMessage.value = player.value.error().message;
      emit('error', errorMessage.value);
    });

    player.value.ready(() => {
      const video = player.value.el().querySelector('video');
      if (video) {
        video.style.objectFit = props.isZoomFit ? 'contain' : 'cover';
      }
    });
  }
};

onMounted(setupPlayer);

onBeforeUnmount(() => {
  if (player.value) {
    player.value.dispose();
  }
});

watch(() => props.src, (newSrc, oldSrc) => {
  if (newSrc !== oldSrc) {
    hasError.value = false;
    errorMessage.value = '';
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
</style>
