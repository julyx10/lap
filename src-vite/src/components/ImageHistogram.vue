<template>
  <div class="space-y-1.5">
    <div
      class="relative w-full aspect-4/1 px-0.5"
      @mousemove="onHistogramMove"
      @mouseleave="onHistogramLeave"
    >
      <div
        v-if="hoverBin !== null"
        class="absolute right-1 top-1 z-10 rounded-md border border-base-content/5 bg-base-100/95 px-2 py-1 text-[9px] font-semibold leading-tight shadow-sm backdrop-blur-sm"
      >
        <button
          type="button"
          class="mb-1 cursor-pointer text-[8px] uppercase tracking-[0.18em] transition-colors hover:text-base-content/80"
          :class="allChannelsVisible ? 'text-base-content/80' : 'text-base-content/30'"
          :aria-pressed="allChannelsVisible"
          @click.stop="showAllChannels"
        >
          {{ $t('msgbox.image_editor.tone') }} {{ hoverBin }}
        </button>
        <button
          v-for="item in legendItems"
          :key="item.key"
          type="button"
          class="flex w-full cursor-pointer items-center gap-1.5 rounded-sm transition-colors hover:bg-base-content/5"
          :aria-pressed="visibleChannels[item.key]"
          @click.stop="toggleChannel(item.key)"
        >
          <span :class="visibleChannels[item.key] ? item.activeClass : item.inactiveClass">{{ item.label }}</span>
          <span
            class="w-10 text-right tabular-nums transition-colors"
            :class="visibleChannels[item.key] ? 'text-base-content/70' : 'text-base-content/30'"
          >{{ formatLegendValue(item.value) }}</span>
        </button>
      </div>

      <svg
        viewBox="0 0 256 64"
        class="h-full w-full"
        preserveAspectRatio="none"
      >
        <defs>
          <linearGradient :id="gradientId" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="rgba(148,163,184,0.62)" />
            <stop offset="100%" stop-color="rgba(148,163,184,0.12)" />
          </linearGradient>
        </defs>
        <g class="text-base-content/30">
          <line x1="64" y1="0" x2="64" y2="64" stroke="currentColor" stroke-width="0.5" />
          <line x1="128" y1="0" x2="128" y2="64" stroke="currentColor" stroke-width="0.5" />
          <line x1="192" y1="0" x2="192" y2="64" stroke="currentColor" stroke-width="0.5" />
        </g>
        <path v-if="visibleChannels.luma" :d="histogramPath" :fill="`url(#${gradientId})`" />
        <path v-if="visibleChannels.red" :d="histogramPathR" fill="rgba(239,68,68,0.35)" style="mix-blend-mode: screen" />
        <path v-if="visibleChannels.green" :d="histogramPathG" fill="rgba(34,197,94,0.35)" style="mix-blend-mode: screen" />
        <path v-if="visibleChannels.blue" :d="histogramPathB" fill="rgba(59,130,246,0.35)" style="mix-blend-mode: screen" />
        <line
          v-if="hoverBin !== null"
          :x1="hoverX"
          y1="0"
          :x2="hoverX"
          y2="64"
          class="text-primary"
          stroke="currentColor"
          stroke-width="0.5"
        />
      </svg>
    </div>

    <div class="flex justify-between px-0.5 text-[8px] uppercase tracking-tighter font-black text-base-content/30">
      <span>{{ $t('msgbox.image_editor.shadows') }}</span>
      <span>{{ $t('msgbox.image_editor.midtones') }}</span>
      <span>{{ $t('msgbox.image_editor.highlights') }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { useConfigStore } from '@/stores/configStore';

const config = useConfigStore();
const props = defineProps({
  source: {
    type: String,
    default: '',
  },
  adjustments: {
    type: Object as () => Partial<AdjustmentValues> | null,
    default: null,
  },
  applyAdjustments: {
    type: Boolean,
    default: false,
  },
  crop: {
    type: Object as () => { x: number; y: number; width: number; height: number } | null,
    default: null,
  },
  rotate: {
    type: Number,
    default: 0,
  },
  flipHorizontal: {
    type: Boolean,
    default: false,
  },
  flipVertical: {
    type: Boolean,
    default: false,
  },
});

type AdjustmentValues = {
  brightness: number;
  contrast: number;
  saturation: number;
  hue: number;
  blur: number;
  filter: string;
};

const HISTOGRAM_BIN_COUNT = 256;
const HISTOGRAM_HEIGHT = 58;
const HISTOGRAM_TONE_RANGE_EDGE_BINS = 8;
const histogramData = new Float32Array(HISTOGRAM_BIN_COUNT);
const histogramDataR = new Float32Array(HISTOGRAM_BIN_COUNT);
const histogramDataG = new Float32Array(HISTOGRAM_BIN_COUNT);
const histogramDataB = new Float32Array(HISTOGRAM_BIN_COUNT);
const displayedHistData = new Float32Array(HISTOGRAM_BIN_COUNT);
const displayedHistDataR = new Float32Array(HISTOGRAM_BIN_COUNT);
const displayedHistDataG = new Float32Array(HISTOGRAM_BIN_COUNT);
const displayedHistDataB = new Float32Array(HISTOGRAM_BIN_COUNT);
const smoothedHistData = new Float32Array(HISTOGRAM_BIN_COUNT);
const smoothedHistDataR = new Float32Array(HISTOGRAM_BIN_COUNT);
const smoothedHistDataG = new Float32Array(HISTOGRAM_BIN_COUNT);
const smoothedHistDataB = new Float32Array(HISTOGRAM_BIN_COUNT);
const histogramVersion = ref(0);
const gradientId = `histGradient-${Math.random().toString(36).slice(2)}`;
const hoverBin = ref<number | null>(null);
const hoverX = ref(0);
type HistogramChannel = 'luma' | 'red' | 'green' | 'blue';
const HISTOGRAM_ALL_CHANNELS = 15;
const HISTOGRAM_CHANNEL_BITS: Record<HistogramChannel, number> = {
  luma: 1,
  red: 2,
  green: 4,
  blue: 8,
};
const histogramChannelMask = computed(() => {
  const storedMask = Number(config.infoPanel.histogramChannels);
  if (storedMask === 16) return HISTOGRAM_ALL_CHANNELS;
  if (!Number.isInteger(storedMask) || storedMask < 0 || storedMask > HISTOGRAM_ALL_CHANNELS) {
    return HISTOGRAM_ALL_CHANNELS;
  }
  return storedMask;
});
const visibleChannels = computed<Record<HistogramChannel, boolean>>(() => {
  const mask = histogramChannelMask.value;
  return {
    luma: Boolean(mask & HISTOGRAM_CHANNEL_BITS.luma),
    red: Boolean(mask & HISTOGRAM_CHANNEL_BITS.red),
    green: Boolean(mask & HISTOGRAM_CHANNEL_BITS.green),
    blue: Boolean(mask & HISTOGRAM_CHANNEL_BITS.blue),
  };
});
const allChannelsVisible = computed(() => histogramChannelMask.value === HISTOGRAM_ALL_CHANNELS);
let histogramAnimRaf: number | null = null;
let histogramLoadId = 0;
let histogramSourceImage: HTMLImageElement | null = null;
let histogramSourceObjectUrl = '';
let recomputeHistogramTimer: ReturnType<typeof setTimeout> | null = null;
let autoPresetValues: AdjustmentValues | null = null;

const naturalAdjustments: AdjustmentValues = {
  brightness: 0,
  contrast: 0,
  saturation: 100,
  hue: 0,
  blur: 0,
  filter: '',
};

function resolvedAdjustments(): AdjustmentValues {
  return {
    ...naturalAdjustments,
    ...(props.adjustments || {}),
  };
}

function clearHistogram() {
  releaseHistogramSourceObjectUrl();
  histogramSourceImage = null;
  autoPresetValues = null;
  if (histogramAnimRaf !== null) {
    cancelAnimationFrame(histogramAnimRaf);
    histogramAnimRaf = null;
  }
  if (recomputeHistogramTimer !== null) {
    clearTimeout(recomputeHistogramTimer);
    recomputeHistogramTimer = null;
  }
  histogramData.fill(0);
  histogramDataR.fill(0);
  histogramDataG.fill(0);
  histogramDataB.fill(0);
  displayedHistData.fill(0);
  displayedHistDataR.fill(0);
  displayedHistDataG.fill(0);
  displayedHistDataB.fill(0);
  smoothedHistData.fill(0);
  smoothedHistDataR.fill(0);
  smoothedHistDataG.fill(0);
  smoothedHistDataB.fill(0);
  hoverBin.value = null;
  hoverX.value = 0;
  histogramVersion.value++;
}

function releaseHistogramSourceObjectUrl() {
  if (histogramSourceObjectUrl) {
    URL.revokeObjectURL(histogramSourceObjectUrl);
    histogramSourceObjectUrl = '';
  }
}

function writeNormalizedHistogram(counts: Float32Array, output: Float32Array) {
  let maxVal = 0;
  for (let i = 0; i < HISTOGRAM_BIN_COUNT; i++) {
    if (counts[i] > maxVal) maxVal = counts[i];
  }

  if (maxVal <= 0) {
    output.fill(0);
    return;
  }

  for (let i = 0; i < HISTOGRAM_BIN_COUNT; i++) {
    output[i] = Math.min((counts[i] / maxVal) * HISTOGRAM_HEIGHT, HISTOGRAM_HEIGHT);
  }
}

function updateHistogramTargets(data: Uint8ClampedArray) {
  const hist = new Float32Array(HISTOGRAM_BIN_COUNT);
  const histR = new Float32Array(HISTOGRAM_BIN_COUNT);
  const histG = new Float32Array(HISTOGRAM_BIN_COUNT);
  const histB = new Float32Array(HISTOGRAM_BIN_COUNT);

  for (let i = 0; i < data.length; i += 4) {
    const alpha = data[i + 3] / 255;
    if (alpha <= 0) continue;

    const r = data[i];
    const g = data[i + 1];
    const b = data[i + 2];
    const gray = Math.round(0.2126 * r + 0.7152 * g + 0.0722 * b);
    hist[gray] += alpha;
    histR[r] += alpha;
    histG[g] += alpha;
    histB[b] += alpha;
  }

  writeNormalizedHistogram(hist, histogramData);
  writeNormalizedHistogram(histR, histogramDataR);
  writeNormalizedHistogram(histG, histogramDataG);
  writeNormalizedHistogram(histB, histogramDataB);
}

function clampColor(value: number) {
  return Math.max(0, Math.min(255, value));
}

function clampNumber(value: number, min: number, max: number) {
  return Math.max(min, Math.min(max, value));
}

function applyPerPixel(data: Uint8ClampedArray, transform: (r: number, g: number, b: number) => [number, number, number]) {
  for (let i = 0; i < data.length; i += 4) {
    const [r, g, b] = transform(data[i], data[i + 1], data[i + 2]);
    data[i] = clampColor(r);
    data[i + 1] = clampColor(g);
    data[i + 2] = clampColor(b);
  }
}

function applyBoxBlur(data: Uint8ClampedArray, width: number, height: number, radius: number) {
  if (radius <= 0) return;

  const temp = new Uint8ClampedArray(data.length);
  const output = new Uint8ClampedArray(data.length);
  const windowSize = radius * 2 + 1;

  for (let y = 0; y < height; y++) {
    let r = 0, g = 0, b = 0, a = 0;
    for (let x = -radius; x <= radius; x++) {
      const clampedX = Math.max(0, Math.min(width - 1, x));
      const idx = (y * width + clampedX) * 4;
      r += data[idx];
      g += data[idx + 1];
      b += data[idx + 2];
      a += data[idx + 3];
    }

    for (let x = 0; x < width; x++) {
      const idx = (y * width + x) * 4;
      temp[idx] = r / windowSize;
      temp[idx + 1] = g / windowSize;
      temp[idx + 2] = b / windowSize;
      temp[idx + 3] = a / windowSize;

      const removeX = Math.max(0, Math.min(width - 1, x - radius));
      const addX = Math.max(0, Math.min(width - 1, x + radius + 1));
      const removeIdx = (y * width + removeX) * 4;
      const addIdx = (y * width + addX) * 4;
      r += data[addIdx] - data[removeIdx];
      g += data[addIdx + 1] - data[removeIdx + 1];
      b += data[addIdx + 2] - data[removeIdx + 2];
      a += data[addIdx + 3] - data[removeIdx + 3];
    }
  }

  for (let x = 0; x < width; x++) {
    let r = 0, g = 0, b = 0, a = 0;
    for (let y = -radius; y <= radius; y++) {
      const clampedY = Math.max(0, Math.min(height - 1, y));
      const idx = (clampedY * width + x) * 4;
      r += temp[idx];
      g += temp[idx + 1];
      b += temp[idx + 2];
      a += temp[idx + 3];
    }

    for (let y = 0; y < height; y++) {
      const idx = (y * width + x) * 4;
      output[idx] = r / windowSize;
      output[idx + 1] = g / windowSize;
      output[idx + 2] = b / windowSize;
      output[idx + 3] = a / windowSize;

      const removeY = Math.max(0, Math.min(height - 1, y - radius));
      const addY = Math.max(0, Math.min(height - 1, y + radius + 1));
      const removeIdx = (removeY * width + x) * 4;
      const addIdx = (addY * width + x) * 4;
      r += temp[addIdx] - temp[removeIdx];
      g += temp[addIdx + 1] - temp[removeIdx + 1];
      b += temp[addIdx + 2] - temp[removeIdx + 2];
      a += temp[addIdx + 3] - temp[removeIdx + 3];
    }
  }

  data.set(output);
}

function applyHistogramAdjustments(data: Uint8ClampedArray, width: number, height: number) {
  const adj = resolvedAdjustments();
  const br = (100 + adj.brightness) / 100;
  const ct = (100 + adj.contrast) / 100;

  applyPerPixel(data, (r, g, b) => [r * br, g * br, b * br]);
  applyPerPixel(data, (r, g, b) => [
    (r - 128) * ct + 128,
    (g - 128) * ct + 128,
    (b - 128) * ct + 128,
  ]);

  applyBoxBlur(data, width, height, Math.round(adj.blur));

  const hueRad = adj.hue * Math.PI / 180;
  const cos = Math.cos(hueRad);
  const sin = Math.sin(hueRad);
  const sat = adj.saturation / 100;

  applyPerPixel(data, (r, g, b) => [
    (0.213 + 0.787 * cos - 0.213 * sin) * r + (0.715 - 0.715 * cos - 0.715 * sin) * g + (0.072 - 0.072 * cos + 0.928 * sin) * b,
    (0.213 - 0.213 * cos + 0.143 * sin) * r + (0.715 + 0.285 * cos + 0.140 * sin) * g + (0.072 - 0.072 * cos - 0.283 * sin) * b,
    (0.213 - 0.213 * cos - 0.787 * sin) * r + (0.715 - 0.715 * cos + 0.715 * sin) * g + (0.072 + 0.928 * cos + 0.072 * sin) * b,
  ]);

  applyPerPixel(data, (r, g, b) => [
    (0.213 + 0.787 * sat) * r + (0.715 - 0.715 * sat) * g + (0.072 - 0.072 * sat) * b,
    (0.213 - 0.213 * sat) * r + (0.715 + 0.285 * sat) * g + (0.072 - 0.072 * sat) * b,
    (0.213 - 0.213 * sat) * r + (0.715 - 0.715 * sat) * g + (0.072 + 0.928 * sat) * b,
  ]);

  if (adj.filter === 'grayscale') {
    applyPerPixel(data, (r, g, b) => {
      const gray = 0.2126 * r + 0.7152 * g + 0.0722 * b;
      return [gray, gray, gray];
    });
  } else if (adj.filter === 'sepia') {
    applyPerPixel(data, (r, g, b) => [
      0.393 * r + 0.769 * g + 0.189 * b,
      0.349 * r + 0.686 * g + 0.168 * b,
      0.272 * r + 0.534 * g + 0.131 * b,
    ]);
  } else if (adj.filter === 'invert') {
    applyPerPixel(data, (r, g, b) => [255 - r, 255 - g, 255 - b]);
  }
}

function normalizeRotate(degrees: number) {
  return ((Math.round(degrees / 90) * 90) % 360 + 360) % 360;
}

function getHistogramSampleRect(img: HTMLImageElement) {
  const sourceWidth = img.naturalWidth || img.width;
  const sourceHeight = img.naturalHeight || img.height;
  const rotate = normalizeRotate(props.rotate);
  const processedWidth = rotate === 90 || rotate === 270 ? sourceHeight : sourceWidth;
  const processedHeight = rotate === 90 || rotate === 270 ? sourceWidth : sourceHeight;
  const crop = props.crop;

  if (!crop || crop.width <= 0 || crop.height <= 0) {
    return { x: 0, y: 0, width: processedWidth, height: processedHeight, sourceWidth, sourceHeight, rotate };
  }

  const x = Math.max(0, Math.min(Math.round(crop.x), processedWidth - 1));
  const y = Math.max(0, Math.min(Math.round(crop.y), processedHeight - 1));
  const width = Math.max(1, Math.min(Math.round(crop.width), processedWidth - x));
  const height = Math.max(1, Math.min(Math.round(crop.height), processedHeight - y));
  return { x, y, width, height, sourceWidth, sourceHeight, rotate };
}

function applyHistogramSourceTransform(
  ctx: CanvasRenderingContext2D,
  sourceWidth: number,
  sourceHeight: number,
  rotate: number,
) {
  if (rotate === 90) {
    ctx.translate(sourceHeight, 0);
    ctx.rotate(Math.PI / 2);
  } else if (rotate === 180) {
    ctx.translate(sourceWidth, sourceHeight);
    ctx.rotate(Math.PI);
  } else if (rotate === 270) {
    ctx.translate(0, sourceWidth);
    ctx.rotate((Math.PI * 3) / 2);
  }

  if (props.flipHorizontal) {
    ctx.translate(sourceWidth, 0);
    ctx.scale(-1, 1);
  }
  if (props.flipVertical) {
    ctx.translate(0, sourceHeight);
    ctx.scale(1, -1);
  }
}

function drawHistogramSample(
  ctx: CanvasRenderingContext2D,
  img: HTMLImageElement,
  size: number,
) {
  const sample = getHistogramSampleRect(img);
  ctx.save();
  ctx.scale(size / sample.width, size / sample.height);
  ctx.translate(-sample.x, -sample.y);
  applyHistogramSourceTransform(ctx, sample.sourceWidth, sample.sourceHeight, sample.rotate);
  ctx.drawImage(img, 0, 0);
  ctx.restore();
}

function buildHistogramData(img: HTMLImageElement) {
  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d', { willReadFrequently: true });
  if (!ctx) {
    clearHistogram();
    return;
  }

  const size = 512;
  canvas.width = size;
  canvas.height = size;
  drawHistogramSample(ctx, img, size);

  try {
    const imageData = ctx.getImageData(0, 0, size, size);
    if (props.applyAdjustments) {
      applyHistogramAdjustments(imageData.data, size, size);
    }
    updateHistogramTargets(imageData.data);
  } catch {
    clearHistogram();
    return;
  }
  startHistogramAnimation();
}

async function loadHistogramImage(source: string) {
  const response = await fetch(source);
  if (!response.ok) throw new Error(`Failed to fetch histogram source: ${response.status}`);
  const blob = await response.blob();

  return await new Promise<{ img: HTMLImageElement; objectUrl: string }>((resolve, reject) => {
    const objectUrl = URL.createObjectURL(blob);
    const img = new Image();

    img.onload = () => resolve({ img, objectUrl });
    img.onerror = () => {
      URL.revokeObjectURL(objectUrl);
      reject(new Error('Failed to decode histogram source'));
    };
    img.src = objectUrl;
  });
}

async function updateHistogram(source: string) {
  if (!source) {
    clearHistogram();
    return;
  }

  const loadId = ++histogramLoadId;

  try {
    const loaded = await loadHistogramImage(source);
    if (loadId !== histogramLoadId) {
      URL.revokeObjectURL(loaded.objectUrl);
      return;
    }

    releaseHistogramSourceObjectUrl();
    histogramSourceObjectUrl = loaded.objectUrl;
    histogramSourceImage = loaded.img;
    autoPresetValues = computeAutoPresetValues(loaded.img);
    buildHistogramData(loaded.img);
  } catch {
    if (loadId !== histogramLoadId) return;
    clearHistogram();
  }
}

function recomputeHistogramWithFilter() {
  if (!histogramSourceImage) return;
  buildHistogramData(histogramSourceImage);
}

function scheduleHistogramRecompute() {
  if (recomputeHistogramTimer !== null) return;
  recomputeHistogramTimer = setTimeout(() => {
    recomputeHistogramTimer = null;
    recomputeHistogramWithFilter();
  }, 32);
}

function percentileFromCounts(counts: Float32Array, total: number, percentile: number) {
  if (total <= 0) return 0;
  const target = total * percentile;
  let cumulative = 0;
  for (let i = 0; i < counts.length; i++) {
    cumulative += counts[i];
    if (cumulative >= target) return i;
  }
  return counts.length - 1;
}

function computeAutoPresetValues(img: HTMLImageElement): AdjustmentValues {
  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d', { willReadFrequently: true });
  if (!ctx) return naturalAdjustments;

  const size = 256;
  canvas.width = size;
  canvas.height = size;
  drawHistogramSample(ctx, img, size);

  try {
    const data = ctx.getImageData(0, 0, size, size).data;
    const lumaCounts = new Float32Array(HISTOGRAM_BIN_COUNT);
    let total = 0;
    let lumaSum = 0;
    let saturationSum = 0;

    for (let i = 0; i < data.length; i += 4) {
      const alpha = data[i + 3] / 255;
      if (alpha <= 0) continue;

      const r = data[i];
      const g = data[i + 1];
      const b = data[i + 2];
      const luma = Math.round(0.2126 * r + 0.7152 * g + 0.0722 * b);
      const maxChannel = Math.max(r, g, b);
      const minChannel = Math.min(r, g, b);
      const chroma = maxChannel > 0 ? (maxChannel - minChannel) / maxChannel : 0;

      lumaCounts[luma] += alpha;
      total += alpha;
      lumaSum += luma * alpha;
      saturationSum += chroma * alpha;
    }

    if (total <= 0) return naturalAdjustments;

    const low = percentileFromCounts(lumaCounts, total, 0.01);
    const high = percentileFromCounts(lumaCounts, total, 0.99);
    const range = Math.max(1, high - low);
    const mean = Math.max(1, lumaSum / total);
    const avgSaturation = saturationSum / total;

    const isUnderexposed = mean < 92 && high < 210;
    const isOverexposed = mean > 168 && low > 35;
    const isFlat = range < 150;
    const isHealthyExposure = !isUnderexposed && !isOverexposed && mean >= 108 && mean <= 148;

    let contrastFactor = 1;
    if (isFlat) {
      contrastFactor = clampNumber(170 / range, 1, 1.28);
    }

    let targetMean = mean;
    if (isUnderexposed) {
      targetMean = 126;
    } else if (isOverexposed) {
      targetMean = 136;
    } else if (isFlat && !isHealthyExposure) {
      targetMean = 132;
    }

    const brightnessFactor = targetMean === mean
      ? 1
      : clampNumber((((targetMean - 128) / contrastFactor) + 128) / mean, 0.82, 1.22);

    let saturationBoost = 0;
    if (avgSaturation >= 0.04 && avgSaturation < 0.16) {
      saturationBoost = 10;
    } else if (avgSaturation >= 0.16 && avgSaturation < 0.24) {
      saturationBoost = 5;
    } else if (avgSaturation > 0.62) {
      saturationBoost = -4;
    }

    return {
      brightness: Math.round((brightnessFactor - 1) * 100),
      contrast: Math.round((contrastFactor - 1) * 100),
      saturation: clampNumber(Math.round(100 + saturationBoost), 85, 120),
      hue: 0,
      blur: 0,
      filter: '',
    };
  } catch {
    return naturalAdjustments;
  }
}

async function getAutoPresetValues() {
  if (autoPresetValues) return autoPresetValues;
  if (histogramSourceImage) {
    autoPresetValues = computeAutoPresetValues(histogramSourceImage);
    return autoPresetValues;
  }

  if (!props.source) return naturalAdjustments;

  try {
    const loaded = await loadHistogramImage(props.source);
    try {
      autoPresetValues = computeAutoPresetValues(loaded.img);
      return autoPresetValues;
    } finally {
      URL.revokeObjectURL(loaded.objectUrl);
    }
  } catch {
    autoPresetValues = naturalAdjustments;
    return autoPresetValues;
  }
}

function gaussianSmooth(source: Float32Array, target: Float32Array, radius = 7, sigma = 3.2) {
  for (let i = 0; i < HISTOGRAM_BIN_COUNT; i++) {
    let sum = 0;
    let weight = 0;
    for (let j = -radius; j <= radius; j++) {
      const idx = i + j;
      if (idx < 0 || idx >= HISTOGRAM_BIN_COUNT) continue;
      const w = Math.exp(-(j * j) / (2 * sigma * sigma));
      sum += source[idx] * w;
      weight += w;
    }
    target[i] = sum / weight;
  }
}

function sampleSmoothedHistogram(smoothed: Float32Array, center: number, radius = 3) {
  let sum = 0;
  let count = 0;
  for (let i = Math.max(0, center - radius); i <= Math.min(HISTOGRAM_BIN_COUNT - 1, center + radius); i++) {
    sum += smoothed[i];
    count++;
  }
  return count > 0 ? sum / count : 0;
}

function getToneRangeDisplayMax(smoothed: Float32Array) {
  let maxVal = 0;
  for (
    let i = HISTOGRAM_TONE_RANGE_EDGE_BINS;
    i < HISTOGRAM_BIN_COUNT - HISTOGRAM_TONE_RANGE_EDGE_BINS;
    i++
  ) {
    const val = Math.max(0, sampleSmoothedHistogram(smoothed, i));
    if (val > maxVal) maxVal = val;
  }
  return maxVal;
}

function buildPathFromSmoothed(smoothed: Float32Array) {
  const width = 256;
  const height = 64;
  const sampledPoints: { x: number; y: number }[] = [];
  const toneRangeMax = getToneRangeDisplayMax(smoothed);
  const scale = toneRangeMax > 0 ? HISTOGRAM_HEIGHT / toneRangeMax : 1;

  for (let i = 0; i < HISTOGRAM_BIN_COUNT; i += 2) {
    const val = Math.min(Math.max(0, sampleSmoothedHistogram(smoothed, i)) * scale, HISTOGRAM_HEIGHT);
    sampledPoints.push({ x: i, y: height - val });
  }
  const lastVal = Math.min(
    Math.max(0, sampleSmoothedHistogram(smoothed, HISTOGRAM_BIN_COUNT - 1)) * scale,
    HISTOGRAM_HEIGHT,
  );
  sampledPoints.push({ x: width, y: height - lastVal });

  if (sampledPoints.length < 2) return '';

  let path = `M 0,${height}`;
  for (let i = 0; i < sampledPoints.length; i++) {
    const p = sampledPoints[i];
    if (i === 0) {
      path += ` L ${p.x.toFixed(1)},${p.y.toFixed(1)}`;
    } else {
      const prev = sampledPoints[i - 1];
      const cp1x = prev.x + (p.x - prev.x) / 2;
      const cp2x = cp1x;
      path += ` C ${cp1x.toFixed(1)},${prev.y.toFixed(1)} ${cp2x.toFixed(1)},${p.y.toFixed(1)} ${p.x.toFixed(1)},${p.y.toFixed(1)}`;
    }
  }
  path += ` L ${width},${height} Z`;
  return path;
}

function getHistogramValueAt(display: Float32Array, bin: number) {
  const index = Math.max(0, Math.min(HISTOGRAM_BIN_COUNT - 1, bin));
  return display[index] || 0;
}

const hoveredValues = computed(() => {
  const bin = hoverBin.value ?? 0;
  return {
    luma: getHistogramValueAt(displayedHistData, bin),
    red: getHistogramValueAt(displayedHistDataR, bin),
    green: getHistogramValueAt(displayedHistDataG, bin),
    blue: getHistogramValueAt(displayedHistDataB, bin),
  };
});

const legendItems = computed(() => [
  {
    key: 'luma' as const,
    label: 'L',
    value: hoveredValues.value.luma,
    activeClass: 'text-base-content/70',
    inactiveClass: 'text-base-content/30',
  },
  {
    key: 'red' as const,
    label: 'R',
    value: hoveredValues.value.red,
    activeClass: 'text-red-500',
    inactiveClass: 'text-red-500/30',
  },
  {
    key: 'green' as const,
    label: 'G',
    value: hoveredValues.value.green,
    activeClass: 'text-green-500',
    inactiveClass: 'text-green-500/30',
  },
  {
    key: 'blue' as const,
    label: 'B',
    value: hoveredValues.value.blue,
    activeClass: 'text-blue-500',
    inactiveClass: 'text-blue-500/30',
  },
]);

function toggleChannel(channel: HistogramChannel) {
  const channelBit = HISTOGRAM_CHANNEL_BITS[channel];
  const currentMask = histogramChannelMask.value;
  const isOnlyVisibleChannel = currentMask === channelBit;
  if (isOnlyVisibleChannel) return;
  config.infoPanel.histogramChannels = currentMask ^ channelBit;
}

function showAllChannels() {
  config.infoPanel.histogramChannels = HISTOGRAM_ALL_CHANNELS;
}

const histogramPath = computed(() => {
  histogramVersion.value;
  return buildPathFromSmoothed(smoothedHistData);
});
const histogramPathR = computed(() => {
  histogramVersion.value;
  return buildPathFromSmoothed(smoothedHistDataR);
});
const histogramPathG = computed(() => {
  histogramVersion.value;
  return buildPathFromSmoothed(smoothedHistDataG);
});
const histogramPathB = computed(() => {
  histogramVersion.value;
  return buildPathFromSmoothed(smoothedHistDataB);
});

function lerpHistogram(target: Float32Array, display: Float32Array): boolean {
  let changed = false;
  for (let i = 0; i < HISTOGRAM_BIN_COUNT; i++) {
    const diff = target[i] - display[i];
    if (Math.abs(diff) < 0.5) {
      display[i] = target[i];
    } else {
      display[i] += diff * 0.3;
      changed = true;
    }
  }
  return changed;
}

function startHistogramAnimation() {
  if (histogramAnimRaf !== null) {
    cancelAnimationFrame(histogramAnimRaf);
    histogramAnimRaf = null;
  }
  const step = () => {
    const changedL = lerpHistogram(histogramData, displayedHistData);
    const changedR = lerpHistogram(histogramDataR, displayedHistDataR);
    const changedG = lerpHistogram(histogramDataG, displayedHistDataG);
    const changedB = lerpHistogram(histogramDataB, displayedHistDataB);
    gaussianSmooth(displayedHistData, smoothedHistData);
    gaussianSmooth(displayedHistDataR, smoothedHistDataR);
    gaussianSmooth(displayedHistDataG, smoothedHistDataG);
    gaussianSmooth(displayedHistDataB, smoothedHistDataB);
    histogramVersion.value++;
    if (changedL || changedR || changedG || changedB) {
      histogramAnimRaf = requestAnimationFrame(step);
    } else {
      histogramAnimRaf = null;
    }
  };
  histogramAnimRaf = requestAnimationFrame(step);
}

function onHistogramMove(event: MouseEvent) {
  const target = event.currentTarget as HTMLElement | null;
  if (!target) return;
  const rect = target.getBoundingClientRect();
  if (rect.width <= 0) return;
  const ratio = Math.max(0, Math.min(1, (event.clientX - rect.left) / rect.width));
  hoverX.value = ratio * 256;
  hoverBin.value = Math.max(0, Math.min(HISTOGRAM_BIN_COUNT - 1, Math.round(ratio * (HISTOGRAM_BIN_COUNT - 1))));
}

function onHistogramLeave() {
  hoverBin.value = null;
}

function formatLegendValue(value: number) {
  return `${Math.round((value / HISTOGRAM_HEIGHT) * 100)}%`;
}

watch(() => props.source, updateHistogram, { immediate: true });
watch(
  () => [props.crop, props.rotate, props.flipHorizontal, props.flipVertical],
  () => {
    autoPresetValues = null;
    scheduleHistogramRecompute();
  },
  { deep: true }
);
watch(
  () => props.adjustments,
  () => {
    if (props.applyAdjustments) scheduleHistogramRecompute();
  },
  { deep: true }
);

onBeforeUnmount(() => {
  ++histogramLoadId;
  clearHistogram();
});

defineExpose({
  getAutoPresetValues,
});
</script>
