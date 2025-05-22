<template>
    <div
      ref="container"
      class="relative w-full h-full overflow-hidden cursor-pointer"
    >
      <video
        :src="videoSrc"
        controls
        class="w-full rounded-lg shadow"
        :style="videoStyle"
      >
        Your browser does not support the video tag.
      </video>
    
    </div> 

</template>

<script setup lang="ts">
import { computed } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'

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

// Convert file path to a valid src for <video> using Tauri's convertFileSrc
const videoSrc = computed(() => convertFileSrc(props.src))
const videoStyle = computed(() => {
  const styles = {
    transform: `rotate(${props.rotate}deg)`,
  }
  if (props.isZoomFit) {
    styles.width = '100%'
    styles.height = '100%'
  }
  return styles
})
</script>

<style scoped>
video {
  background-color: black;
}
</style>
