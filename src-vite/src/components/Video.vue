<template>
  <div ref="playerContainer" class="w-full h-full video-container"></div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue';
import Player from 'xgplayer';

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

// This function initializes or updates the player
const setupPlayer = () => {
  if (!playerContainer.value) return;

  // If player exists, destroy it before creating a new one
  if (player.value) {
    player.value.destroy();
  }

  // Do not create player if src is empty
  if (!props.src) return;

  player.value = new Player({
    el: playerContainer.value,
    url: props.src,
    height: '100%',
    width: '100%',
    fitVideoSize: props.isZoomFit ? 'fixWidth' : 'fixed',
    autoplay: false,
    playsinline: true,
    // 隐藏不需要的控件
    controls: {
      mode: 'normal',
      // 只显示必要的控件
      controls: [
        'play',
        'time',
        'progress',
        'volume',
        'fullscreen'
      ]
    },
    // 隐藏默认的播放按钮
    ignores: ['replay', 'error', 'loading', 'poster'],
    // 确保视频填满容器
    videoInit: true,
    // 禁用一些不需要的功能
    disableVideoTag: false,
    // 自定义样式
    cssFullscreen: true,
    // 确保视频适应屏幕
    videoFillMode: 'cover'
  });
};

// Initialize player on mount
onMounted(setupPlayer);

// Clean up player instance on unmount
onBeforeUnmount(() => {
  if (player.value) {
    player.value.destroy();
  }
});

// Watch for source changes to update the video
// Re-create the player instance when the src changes to ensure a clean state
watch(() => props.src, setupPlayer);

// Watch for rotation changes
watch(() => props.rotate, (newRotate) => {
  if (player.value) {
    // xgplayer 的旋转方法
    player.value.rotate && player.value.rotate(newRotate, true);
  }
});

// Watch for zoom fit changes
watch(() => props.isZoomFit, (newFit) => {
  if (player.value) {
    player.value.setConfig({
      fitVideoSize: newFit ? 'fixWidth' : 'fixed',
    });
  }
});

// Exposed methods for parent component to call
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
.video-container {
  position: relative;
  overflow: hidden;
}

/* 确保 xgplayer 视频元素填满容器 */
.video-container :deep(.xgplayer) {
  width: 100% !important;
  height: 100% !important;
}

.video-container :deep(.xgplayer video) {
  width: 100% !important;
  height: 100% !important;
  object-fit: cover;
}

/* 隐藏不需要的控件 */
.video-container :deep(.xgplayer .xgplayer-controls) {
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.3));
}

/* 确保播放按钮居中 */
.video-container :deep(.xgplayer .xgplayer-play) {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}
</style>