<template>
  <div ref="playerContainer" class="relative w-full h-full overflow-hidden cursor-pointer"></div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue';
import Player from 'xgplayer';
import 'xgplayer/dist/index.min.css';
import { config } from '@/common/utils';

// Props
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

const playerContainer = ref<HTMLElement | null>(null);
const player = ref<Player | null>(null);

const setupPlayer = () => {
  if (!playerContainer.value) return;

  if (!props.src) {
    if (player.value) {
      player.value.destroy();
      player.value = null;
    }
    return;
  }

  if (player.value) {
    // If player exists, just update the src
    if (player.value.src !== props.src) {
      player.value.src = props.src;
    }
  } else {
    // If player doesn't exist, create a new one
    player.value = new Player({
      el: playerContainer.value,
      url: props.src,
      height: '100%',
      width: '100%',
      fitVideoSize: props.isZoomFit ? 'fixWidth' : 'fixed',
      autoplay: false,
      playsinline: true,
      // hide default play button
      ignores: ['replay'],
      // ensure video fill container
      videoInit: true,
      // disable some unnecessary features
      disableVideoTag: true,
      // custom style
      fullscreen: true,
      cssFullscreen: true,
      // ensure video fit screen
      videoFillMode: 'cover',
      // set language
      lang: config.language,
    });
  }
};

onMounted(setupPlayer);

onBeforeUnmount(() => {
  if (player.value) {
    player.value.destroy();
  }
});

watch(() => props.src, setupPlayer);

watch(() => props.rotate, (newRotate) => {
  if (player.value) {
    // player.value.rotate && player.value.rotate(newRotate, true);
    player.value.on('rotate', (newRotate) => {
      console.log('rotate:', newRotate);
    });
  }
});

watch(() => props.isZoomFit, (newFit) => {
  if (player.value) {
    player.value.setConfig({
      fitVideoSize: newFit ? 'fixWidth' : 'fixed',
    });
  }
});

const zoomIn = () => {
  if (player.value && player.value.zoom) {
    const currentZoom = player.value.zoom() || 1;
    player.value.zoom(currentZoom * 1.5);
  }
}

const zoomOut = () => {
  if (player.value && player.value.zoom) {
    const currentZoom = player.value.zoom() || 1;
    player.value.zoom(currentZoom / 1.5);
  }
}

const zoomActual = () => {
  if (player.value && player.value.zoom) {
    player.value.zoom(1);
  }
}

const rotateRight = () => {
  if (player.value && player.value.rotate) {
    player.value.rotate(); // Rotates 90 degrees clockwise
  }
}

defineExpose({
  zoomIn,
  zoomOut,
  zoomActual,
  rotateRight
});

</script>

<style scoped>
</style>