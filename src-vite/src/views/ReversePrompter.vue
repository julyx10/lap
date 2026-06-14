<template>
  <div
    class="w-screen h-screen flex flex-col overflow-hidden select-none bg-base-300 text-base-content/70"
    @dragenter.prevent="isDragging = true"
    @dragover.prevent="isDragging = true"
    @dragleave.prevent="isDragging = false"
    @drop.prevent="handleDrop"
  >
    <TitleBar titlebar="Reverse Prompter" viewName="ReversePrompter" class="shrink-0 z-50" />

    <main class="reverse-prompter-layout flex-1 min-h-0 p-2 gap-2">
      <section class="min-h-0 flex flex-col overflow-hidden rounded-box bg-base-200">
        <div class="h-11 px-3 shrink-0 flex items-center gap-2 border-b border-base-content/10" data-tauri-drag-region>
          <IconSmartTag class="w-5 h-5 text-primary shrink-0" />
          <span class="font-medium truncate">Image</span>
          <div class="flex-1" data-tauri-drag-region></div>
          <button class="btn btn-sm btn-ghost rounded-box gap-2" @click="selectFile">
            <IconUpload class="w-4 h-4" />
            <span>Select</span>
          </button>
          <button class="btn btn-sm btn-ghost rounded-box gap-2" :disabled="!previewUrl" @click="clearImage">
            <IconClose class="w-4 h-4" />
            <span>Clear</span>
          </button>
        </div>

        <button
          class="relative flex-1 min-h-0 m-2 rounded-box overflow-hidden border border-dashed transition-colors bg-base-300/50"
          :class="isDragging ? 'border-primary text-primary' : 'border-base-content/15 hover:border-base-content/30'"
          @click="selectFile"
        >
          <img
            v-if="previewUrl"
            :src="previewUrl"
            class="absolute inset-0 w-full h-full object-contain"
            draggable="false"
            alt=""
          />
          <div v-else class="absolute inset-0 flex flex-col items-center justify-center gap-3 text-base-content/45">
            <IconDragDrop class="w-12 h-12" />
            <div class="text-sm font-medium">Paste / drop / select</div>
          </div>
        </button>

        <div class="shrink-0 p-3 grid grid-cols-2 gap-2 border-t border-base-content/10">
          <div v-for="item in metadataItems" :key="item.label" class="min-w-0 rounded-box bg-base-300/50 px-3 py-2">
            <div class="text-[10px] uppercase font-bold text-base-content/35">{{ item.label }}</div>
            <div class="truncate text-sm text-base-content/75" :title="item.value">{{ item.value }}</div>
          </div>
        </div>
      </section>

      <section class="min-h-0 flex flex-col overflow-hidden rounded-box bg-base-200">
        <div class="h-11 px-3 shrink-0 flex items-center gap-2 border-b border-base-content/10" data-tauri-drag-region>
          <IconComment class="w-5 h-5 text-primary shrink-0" />
          <span class="font-medium truncate">Prompt</span>
          <div class="flex-1" data-tauri-drag-region></div>
          <button class="btn btn-sm btn-primary rounded-box gap-2" :disabled="!canGenerate" @click="generatePrompt">
            <span v-if="isGenerating" class="loading loading-spinner loading-xs"></span>
            <IconRefresh v-else class="w-4 h-4" />
            <span>Generate</span>
          </button>
          <button class="btn btn-sm btn-ghost rounded-box gap-2" :disabled="!promptText" @click="copyPrompt">
            <IconCopy class="w-4 h-4" />
            <span>Copy</span>
          </button>
        </div>

        <div class="shrink-0 p-3 grid gap-2 border-b border-base-content/10 reverse-prompter-controls">
          <label class="form-control min-w-0">
            <span class="label-text text-xs text-base-content/50">API key</span>
            <input
              v-model="apiKey"
              type="password"
              class="input input-sm input-bordered rounded-box w-full"
              autocomplete="off"
              placeholder="sk-..."
            />
          </label>
          <label class="form-control min-w-0">
            <span class="label-text text-xs text-base-content/50">Model</span>
            <input v-model="model" class="input input-sm input-bordered rounded-box w-full" />
          </label>
          <label class="form-control min-w-0">
            <span class="label-text text-xs text-base-content/50">Detail</span>
            <select v-model="detail" class="select select-sm select-bordered rounded-box w-full">
              <option value="high">High</option>
              <option value="low">Low</option>
              <option value="auto">Auto</option>
              <option value="original">Original</option>
            </select>
          </label>
          <label class="form-control min-w-0">
            <span class="label-text text-xs text-base-content/50">Mode</span>
            <select v-model="tone" class="select select-sm select-bordered rounded-box w-full">
              <option value="detailed">Detailed</option>
              <option value="concise">Concise</option>
              <option value="tags">Tags</option>
            </select>
          </label>
          <label class="form-control min-w-0 reverse-prompter-endpoint">
            <span class="label-text text-xs text-base-content/50">Endpoint</span>
            <input v-model="endpoint" class="input input-sm input-bordered rounded-box w-full" />
          </label>
        </div>

        <div class="flex-1 min-h-0 p-3 grid gap-3 reverse-prompter-output">
          <label class="form-control min-h-0">
            <span class="label-text text-xs text-base-content/50">Intent</span>
            <textarea
              v-model="extraContext"
              class="textarea textarea-bordered rounded-box min-h-18 resize-none"
              placeholder="Optional"
            ></textarea>
          </label>

          <label class="form-control min-h-0 flex-1">
            <span class="label-text text-xs text-base-content/50">Output</span>
            <textarea
              v-model="promptText"
              class="textarea textarea-bordered rounded-box h-full min-h-48 resize-none font-mono text-sm leading-6"
              spellcheck="false"
            ></textarea>
          </label>
        </div>

        <div class="h-9 shrink-0 px-3 flex items-center gap-2 border-t border-base-content/10 text-xs">
          <span class="w-2 h-2 rounded-full" :class="statusDotClass"></span>
          <span class="truncate" :title="statusText">{{ statusText }}</span>
        </div>
      </section>
    </main>

    <input
      ref="fileInputRef"
      class="hidden"
      type="file"
      accept="image/*"
      @change="handleFileSelection"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import TitleBar from '@/components/TitleBar.vue';
import { reversePromptImage } from '@/common/api';
import {
  DEFAULT_REVERSE_PROMPT_ENDPOINT,
  DEFAULT_REVERSE_PROMPT_MODEL,
  buildFallbackPrompt,
  buildReversePromptInstruction,
} from '@/common/reversePrompt';
import {
  IconClose,
  IconComment,
  IconCopy,
  IconDragDrop,
  IconRefresh,
  IconSmartTag,
  IconUpload,
} from '@/common/icons';

type ImageAnalysis = {
  fileName: string;
  width: number;
  height: number;
  mimeType: string;
  sizeBytes: number;
  hasAlpha: boolean;
  averageBrightness: number;
  averageSaturation: number;
  contrast: number;
  palette: string[];
};

const fileInputRef = ref<HTMLInputElement | null>(null);
const previewUrl = ref('');
const imageDataUrl = ref('');
const imageInfo = ref<ImageAnalysis | null>(null);
const promptText = ref('');
const localPrompt = ref('');
const apiKey = ref('');
const model = ref(DEFAULT_REVERSE_PROMPT_MODEL);
const endpoint = ref(DEFAULT_REVERSE_PROMPT_ENDPOINT);
const detail = ref('high');
const tone = ref('detailed');
const extraContext = ref('');
const statusText = ref('Ready');
const statusKind = ref<'idle' | 'ok' | 'error' | 'loading'>('idle');
const isGenerating = ref(false);
const isDragging = ref(false);

const canGenerate = computed(() => Boolean(imageDataUrl.value) && !isGenerating.value);
const statusDotClass = computed(() => ({
  'bg-base-content/30': statusKind.value === 'idle',
  'bg-success': statusKind.value === 'ok',
  'bg-error': statusKind.value === 'error',
  'bg-primary animate-pulse': statusKind.value === 'loading',
}));

const metadataItems = computed(() => {
  const info = imageInfo.value;
  if (!info) {
    return [
      { label: 'Dimensions', value: '-' },
      { label: 'Type', value: '-' },
      { label: 'Size', value: '-' },
      { label: 'Palette', value: '-' },
    ];
  }

  return [
    { label: 'Dimensions', value: `${info.width}x${info.height}` },
    { label: 'Type', value: info.mimeType || 'image' },
    { label: 'Size', value: formatBytes(info.sizeBytes) },
    { label: 'Palette', value: info.palette.join(', ') || '-' },
  ];
});

onMounted(() => {
  window.addEventListener('paste', handlePaste);
});

onBeforeUnmount(() => {
  window.removeEventListener('paste', handlePaste);
  revokePreviewUrl();
});

function selectFile() {
  fileInputRef.value?.click();
}

function handleFileSelection(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (file) {
    void loadImageFile(file);
  }
  input.value = '';
}

function handlePaste(event: ClipboardEvent) {
  const file = findImageFile(event.clipboardData?.items);
  if (!file) return;
  event.preventDefault();
  void loadImageFile(file);
}

function handleDrop(event: DragEvent) {
  isDragging.value = false;
  const file = Array.from(event.dataTransfer?.files || []).find((item) => item.type.startsWith('image/'));
  if (!file) {
    setStatus('No image found in drop.', 'error');
    return;
  }
  void loadImageFile(file);
}

async function loadImageFile(file: File) {
  if (!file.type.startsWith('image/')) {
    setStatus('Unsupported file type.', 'error');
    return;
  }

  setStatus('Loading image...', 'loading');
  const objectUrl = URL.createObjectURL(file);

  try {
    const image = await loadHtmlImage(objectUrl);
    const analysis = analyzeImage(image, file);
    const preparedDataUrl = prepareImageDataUrl(image, analysis.hasAlpha);

    revokePreviewUrl();
    previewUrl.value = objectUrl;
    imageDataUrl.value = preparedDataUrl;
    imageInfo.value = analysis;
    localPrompt.value = buildFallbackPrompt(analysis);
    promptText.value = localPrompt.value;
    setStatus('Local prompt ready.', 'ok');
  } catch (error) {
    URL.revokeObjectURL(objectUrl);
    setStatus(normalizeError(error), 'error');
  }
}

async function generatePrompt() {
  if (!imageDataUrl.value) {
    setStatus('No image selected.', 'error');
    return;
  }

  if (!apiKey.value.trim()) {
    promptText.value = localPrompt.value;
    setStatus('Local prompt ready.', 'ok');
    return;
  }

  isGenerating.value = true;
  setStatus('Generating prompt...', 'loading');

  try {
    const result = await reversePromptImage({
      apiKey: apiKey.value.trim(),
      imageDataUrl: imageDataUrl.value,
      model: model.value.trim() || DEFAULT_REVERSE_PROMPT_MODEL,
      endpoint: endpoint.value.trim() || DEFAULT_REVERSE_PROMPT_ENDPOINT,
      detail: detail.value,
      instruction: buildReversePromptInstruction({
        tone: tone.value,
        extraContext: extraContext.value,
      }),
    });

    promptText.value = String(result?.prompt || '').trim();
    setStatus('AI prompt ready.', 'ok');
  } catch (error) {
    if (localPrompt.value) {
      promptText.value = localPrompt.value;
    }
    setStatus(normalizeError(error), 'error');
  } finally {
    isGenerating.value = false;
  }
}

async function copyPrompt() {
  if (!promptText.value) return;

  try {
    await navigator.clipboard.writeText(promptText.value);
    setStatus('Prompt copied.', 'ok');
  } catch (error) {
    setStatus(normalizeError(error), 'error');
  }
}

function clearImage() {
  revokePreviewUrl();
  imageDataUrl.value = '';
  imageInfo.value = null;
  promptText.value = '';
  localPrompt.value = '';
  setStatus('Ready', 'idle');
}

function findImageFile(items?: DataTransferItemList | null) {
  if (!items) return null;

  for (const item of Array.from(items)) {
    if (item.kind === 'file' && item.type.startsWith('image/')) {
      return item.getAsFile();
    }
  }

  return null;
}

function loadHtmlImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error('Image could not be decoded.'));
    image.src = src;
  });
}

function analyzeImage(image: HTMLImageElement, file: File): ImageAnalysis {
  const canvas = document.createElement('canvas');
  const sampleSize = 72;
  canvas.width = sampleSize;
  canvas.height = sampleSize;
  const context = canvas.getContext('2d', { willReadFrequently: true });
  if (!context) {
    throw new Error('Canvas is not available.');
  }

  context.drawImage(image, 0, 0, sampleSize, sampleSize);
  const pixels = context.getImageData(0, 0, sampleSize, sampleSize).data;
  const paletteCounts = new Map<string, number>();
  const luminanceValues: number[] = [];
  let brightnessTotal = 0;
  let saturationTotal = 0;
  let opaqueCount = 0;
  let transparentCount = 0;

  for (let index = 0; index < pixels.length; index += 4) {
    const alpha = pixels[index + 3];
    if (alpha < 250) {
      transparentCount++;
    }
    if (alpha < 32) {
      continue;
    }

    const red = pixels[index];
    const green = pixels[index + 1];
    const blue = pixels[index + 2];
    const luminance = (0.2126 * red + 0.7152 * green + 0.0722 * blue) / 255;
    const saturation = getRgbSaturation(red, green, blue);
    const key = quantizeHex(red, green, blue);

    brightnessTotal += luminance;
    saturationTotal += saturation;
    luminanceValues.push(luminance);
    paletteCounts.set(key, (paletteCounts.get(key) || 0) + 1);
    opaqueCount++;
  }

  const count = Math.max(opaqueCount, 1);
  const averageBrightness = brightnessTotal / count;
  const contrast = Math.sqrt(
    luminanceValues.reduce((total, value) => total + (value - averageBrightness) ** 2, 0) / count
  );

  return {
    fileName: file.name,
    width: image.naturalWidth || image.width,
    height: image.naturalHeight || image.height,
    mimeType: file.type,
    sizeBytes: file.size,
    hasAlpha: transparentCount > sampleSize * sampleSize * 0.01,
    averageBrightness,
    averageSaturation: saturationTotal / count,
    contrast,
    palette: Array.from(paletteCounts.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, 5)
      .map(([color]) => color),
  };
}

function prepareImageDataUrl(image: HTMLImageElement, hasAlpha: boolean) {
  const sourceWidth = image.naturalWidth || image.width;
  const sourceHeight = image.naturalHeight || image.height;
  let maxDimension = 2048;
  let quality = 0.92;
  let latestDataUrl = '';

  for (let attempt = 0; attempt < 4; attempt++) {
    const scale = Math.min(1, maxDimension / Math.max(sourceWidth, sourceHeight));
    const width = Math.max(1, Math.round(sourceWidth * scale));
    const height = Math.max(1, Math.round(sourceHeight * scale));
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    const context = canvas.getContext('2d');
    if (!context) {
      throw new Error('Canvas is not available.');
    }

    if (!hasAlpha) {
      context.fillStyle = '#ffffff';
      context.fillRect(0, 0, width, height);
    }
    context.drawImage(image, 0, 0, width, height);

    latestDataUrl = hasAlpha
      ? canvas.toDataURL('image/png')
      : canvas.toDataURL('image/jpeg', quality);
    if (latestDataUrl.length < 19_500_000) {
      return latestDataUrl;
    }

    maxDimension = Math.round(maxDimension * 0.75);
    quality = Math.max(0.78, quality - 0.06);
  }

  return latestDataUrl;
}

function getRgbSaturation(red: number, green: number, blue: number) {
  const max = Math.max(red, green, blue) / 255;
  const min = Math.min(red, green, blue) / 255;
  if (max <= 0) return 0;
  return (max - min) / max;
}

function quantizeHex(red: number, green: number, blue: number) {
  const quantize = (value: number) => Math.max(0, Math.min(255, Math.round(value / 32) * 32));
  return `#${toHex(quantize(red))}${toHex(quantize(green))}${toHex(quantize(blue))}`;
}

function toHex(value: number) {
  return value.toString(16).padStart(2, '0');
}

function revokePreviewUrl() {
  if (previewUrl.value) {
    URL.revokeObjectURL(previewUrl.value);
    previewUrl.value = '';
  }
}

function setStatus(text: string, kind: 'idle' | 'ok' | 'error' | 'loading') {
  statusText.value = text;
  statusKind.value = kind;
}

function normalizeError(error: unknown) {
  if (typeof error === 'string') return error;
  if (error instanceof Error) return error.message;
  return 'Request failed.';
}

function formatBytes(bytes: number) {
  if (!Number.isFinite(bytes) || bytes <= 0) return '-';
  const units = ['B', 'KB', 'MB', 'GB'];
  let value = bytes;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex++;
  }
  return `${value.toFixed(unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
}
</script>

<style scoped>
.reverse-prompter-layout {
  display: grid;
  grid-template-columns: minmax(320px, 0.85fr) minmax(440px, 1.15fr);
}

.reverse-prompter-controls {
  grid-template-columns: minmax(180px, 1.2fr) minmax(120px, 0.8fr) minmax(96px, 0.45fr) minmax(104px, 0.45fr);
}

.reverse-prompter-endpoint {
  grid-column: 1 / -1;
}

.reverse-prompter-output {
  grid-template-rows: auto minmax(0, 1fr);
}

@media (max-width: 900px) {
  .reverse-prompter-layout {
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: minmax(240px, 42%) minmax(360px, 1fr);
  }

  .reverse-prompter-controls {
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  }
}
</style>
