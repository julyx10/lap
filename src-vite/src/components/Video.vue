<template>
  <div ref="playerContainer" class="relative w-full h-full overflow-hidden cursor-pointer">
    <video
      ref="videoElement"
      class="video-js vjs-default-skin w-full h-full"
      preload="metadata"
      data-setup="{}"
    ></video>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed } from 'vue';
import videojs from 'video.js';
import 'video.js/dist/video-js.css';
import { config } from '@/common/utils';

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
const videoElement = ref<HTMLVideoElement | null>(null);
const player = ref<any>(null);

// 播放器配置
const playerOptions = computed(() => ({
  responsive: true,
  fluid: true,
  aspectRatio: '16:9',
  autoplay: true,
  controls: true,
  preload: 'metadata',
  language: config.language,
  playbackRates: [0.5, 1, 1.25, 1.5, 2],
  controlBar: {
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
    // 更新视频源
    player.value.src(props.src);
  } else {
    // 创建新播放器
    player.value = videojs(videoElement.value, playerOptions.value);
    
    // 设置视频源
    player.value.src(props.src);
    
    // 添加自定义样式
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

watch(() => props.src, setupPlayer);

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

<style scoped>
.video-js {
  width: 100% !important;
  height: 100% !important;
}

.video-js .vjs-tech {
  object-fit: cover;
  width: 100% !important;
  height: 100% !important;
}

/* 自定义控件样式 */
.video-js .vjs-control-bar {
  background: linear-gradient(transparent, rgba(0, 0, 0, 0.6));
}

/* 播放按钮居中 */
.video-js .vjs-big-play-button {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  border-radius: 50%;
  width: 80px;
  height: 80px;
  line-height: 80px;
  font-size: 30px;
}

/* 确保视频元素居中显示 */
.video-js video {
  object-fit: cover;
  width: 100% !important;
  height: 100% !important;
}
</style>