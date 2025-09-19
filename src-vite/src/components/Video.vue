<template>
  <div class="relative w-full h-full overflow-hidden cursor-pointer bg-base-200">
    <video v-show="!hasError" ref="videoElement" class="video-js"></video>

    <div v-if="hasError" class="absolute inset-0 flex flex-col items-center justify-center text-base-content/30">
      <IconVideoSlash class="w-8 h-8 mb-2" />
      <div class="text-center">{{ errorMessage }}</div>
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

// transform state
const transformState = reactive({ rotate: 0, scale: 1 });

const videoJsLang = computed(() => (config.language === 'zh' ? 'zh-CN' : config.language));

const playerOptions = computed(() => ({
  responsive: false,
  fluid: false,
  width: '100%',
  height: '100%',
  autoplay: true,
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
    volumePanel: { inline: false },
  },
}));

const updateTransform = () => {
  const video = player.value?.el().querySelector('video');
  if (video) {
    video.style.transform = `rotate(${transformState.rotate}deg) scale(${transformState.scale})`;
    video.style.objectFit = props.isZoomFit ? 'contain' : 'cover';
  }
};

const setupPlayer = () => {
  if (!videoElement.value) return;

  if (!player.value) {
    player.value = videojs(videoElement.value, playerOptions.value);

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
  }

  if (props.src) {
    if(!canPlay(props.src)) {
      hasError.value = true;
      errorMessage.value = $t('video.errors.format');
      return;
    }
    hasError.value = false;
    errorMessage.value = '';
    player.value.src(props.src);
    nextTick(() => player.value?.play());
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

onMounted(setupPlayer);

onBeforeUnmount(() => {
  if (player.value) {
    player.value.off();
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

watch(() => props.isZoomFit, () => updateTransform());

const zoomIn = () => {
  transformState.scale *= 1.5;
  updateTransform();
};
const zoomOut = () => {
  transformState.scale /= 1.5;
  updateTransform();
};
const zoomActual = () => {
  transformState.scale = 1;
  updateTransform();
};
const rotateRight = () => {
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
  background-color: hsl(var(--b2)) !important;
}
.video-js .vjs-big-play-button {
  display: none !important;
}
</style>
