<template>
  <div 
    class="w-full h-full relative flex items-center justify-center"
    @mousemove="handleMouseMove"
    @mouseleave="handleMouseLeave"
    ref="containerRef"
  >
    <!-- Previous Button -->
    <button 
      v-if="showNavButton && hasPrevious"
      class="absolute left-2 top-1/2 -translate-y-1/2 z-[70] p-2 rounded-full bg-base-100/30 hover:bg-base-100/70 text-base-content/70 transition-opacity duration-300"
      :class="[ isHoverLeft ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none' ]"
      @click.stop="triggerPrev"
      @dblclick.stop
    >
      <IconLeft class="w-8 h-8" />
    </button>

    <!-- Next Button -->
    <button 
      v-if="showNavButton && hasNext"
      class="absolute right-2 top-1/2 -translate-y-1/2 z-[70] p-2 rounded-full bg-base-100/30 hover:bg-base-100/70 text-base-content/70 transition-opacity duration-300"
      :class="[ isHoverRight ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none' ]"
      @click.stop="triggerNext"
      @dblclick.stop
    >
      <IconRight class="w-8 h-8" />
    </button>

    <Image v-if="file?.file_type === 1"
      ref="mediaRef"
      :filePath="file?.file_path" 
      :rotate="file?.rotate ?? 0" 
      :isZoomFit="isZoomFit"
    ></Image>
    
    <Video v-if="file?.file_type === 2"
      ref="mediaRef"
      :filePath="file?.file_path"
      :rotate="file?.rotate ?? 0"
      :isZoomFit="isZoomFit"
    ></Video>

    <ToolTip ref="toolTipRef" />
  </div>
</template>

<script setup lang="ts">
import { defineAsyncComponent, ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import Image from '@/components/Image.vue';
import ToolTip from '@/components/ToolTip.vue';
import { IconLeft, IconRight } from '@/common/icons';

const Video = defineAsyncComponent(() => import('@/components/Video.vue'));

const props = defineProps({
  file: {
    type: Object,
    default: null
  },
  isZoomFit: {
    type: Boolean,
    default: true
  },
  showNavButton: {
    type: Boolean,
    default: false
  },
  hasPrevious: {
    type: Boolean,
    default: false
  },
  hasNext: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits(['prev', 'next']);

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

const containerRef = ref<HTMLElement | null>(null);
const mediaRef = ref<any>(null);
const toolTipRef = ref<any>(null);
const isHoverLeft = ref(false);
const isHoverRight = ref(false);

function handleMouseMove(e: MouseEvent) {
  if (!containerRef.value) return;
  
  const rect = containerRef.value.getBoundingClientRect();
  const x = e.clientX - rect.left;
  const width = rect.width;
  
  if (width > 0) {
    isHoverLeft.value = x < width * 0.2;
    isHoverRight.value = x > width * 0.8;
  }
}

function handleMouseLeave() {
  isHoverLeft.value = false;
  isHoverRight.value = false;
}

// Expose methods for parent component (ImageViewer)
const zoomIn = () => mediaRef.value?.zoomIn();
const zoomOut = () => mediaRef.value?.zoomOut();
const zoomActual = () => mediaRef.value?.zoomActual();
const rotateRight = () => mediaRef.value?.rotateRight();
const showMessage = (message: string, isWarning: boolean = false) => toolTipRef.value?.showTip(message, isWarning);
const showTip = (message: string, isWarning: boolean = false) => toolTipRef.value?.showTip(message, isWarning);

const triggerPrev = () => {
  if (props.hasPrevious) {
    emit('prev');
  } else {
    showTip((localeMsg.value as any).tooltip.image_viewer.first_image);
  }
}

const triggerNext = () => {
  if (props.hasNext) {
    emit('next');
  } else {
    showTip((localeMsg.value as any).tooltip.image_viewer.last_image);
  }
}

defineExpose({
  zoomIn,
  zoomOut,
  zoomActual,
  rotateRight,
  showMessage,
  triggerPrev,
  triggerNext
});
</script>
