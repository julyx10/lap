<template>
  <div class="sidebar-panel">
    <div class="px-2 pb-3 shrink-0 space-y-2">
      <div class="space-y-1">
        <label class="text-[10px] uppercase tracking-widest font-bold text-base-content/40">Civitai URL</label>
        <div class="relative">
          <IconLink class="absolute left-2 top-1/2 -translate-y-1/2 w-4 h-4 text-base-content/30" />
          <input
            v-model="imageUrl"
            class="input input-bordered input-sm w-full pl-7 pr-7 bg-transparent text-xs"
            placeholder="https://civitai.red/images/12097475"
            spellcheck="false"
            @keydown.enter="analyze"
          />
          <button
            v-if="imageUrl"
            type="button"
            class="absolute right-2 top-1/2 -translate-y-1/2 text-base-content/30 hover:text-base-content"
            @click="imageUrl = ''"
          >
            <IconClose class="w-4 h-4" />
          </button>
        </div>
      </div>

      <div class="space-y-1">
        <label class="text-[10px] uppercase tracking-widest font-bold text-base-content/40">Model folders</label>
        <textarea
          v-model="modelRootsText"
          class="textarea textarea-bordered textarea-sm w-full min-h-16 bg-transparent text-xs resize-none"
          placeholder="C:\stable-diffusion-webui\models"
          spellcheck="false"
        ></textarea>
      </div>

      <button
        type="button"
        class="t-button-primary w-full"
        :disabled="isLoading || !imageUrl.trim()"
        @click="analyze"
      >
        <span v-if="isLoading" class="loading loading-spinner loading-xs"></span>
        <IconSearch v-else class="w-4 h-4" />
        <span>Analyze</span>
      </button>
    </div>

    <div v-if="errorMessage" class="mx-2 mb-3 rounded-box border border-error/20 bg-error/10 p-3 text-xs text-error">
      {{ errorMessage }}
    </div>

    <div v-if="!report && !errorMessage && !isLoading" class="sidebar-empty text-sm">
      <IconLink class="w-8 h-8 mb-2 text-base-content/20" />
      <span>No image analyzed</span>
    </div>

    <div v-if="report" class="flex-1 overflow-y-auto px-2 pb-4 space-y-4">
      <section class="border-t border-base-content/5 pt-3 space-y-2">
        <div class="flex items-center gap-2 text-base-content/70">
          <span class="font-bold mr-auto uppercase text-xs tracking-wide">Prompt</span>
          <button
            type="button"
            class="btn btn-ghost btn-xs btn-square rounded-box"
            title="Copy prompt"
            :disabled="!report.prompt"
            @click="copyText(report.prompt || '')"
          >
            <IconCopy class="w-4 h-4" />
          </button>
        </div>
        <textarea
          class="textarea textarea-bordered w-full min-h-32 bg-base-300/30 text-xs resize-none"
          :value="report.prompt || ''"
          readonly
        ></textarea>
      </section>

      <section v-if="report.negativePrompt" class="border-t border-base-content/5 pt-3 space-y-2">
        <div class="flex items-center gap-2 text-base-content/70">
          <span class="font-bold mr-auto uppercase text-xs tracking-wide">Negative prompt</span>
          <button
            type="button"
            class="btn btn-ghost btn-xs btn-square rounded-box"
            title="Copy negative prompt"
            @click="copyText(report.negativePrompt || '')"
          >
            <IconCopy class="w-4 h-4" />
          </button>
        </div>
        <textarea
          class="textarea textarea-bordered w-full min-h-20 bg-base-300/30 text-xs resize-none"
          :value="report.negativePrompt"
          readonly
        ></textarea>
      </section>

      <section class="border-t border-base-content/5 pt-3 space-y-2">
        <div class="flex items-center gap-2 text-base-content/70">
          <span class="font-bold mr-auto uppercase text-xs tracking-wide">Settings</span>
          <button
            type="button"
            class="btn btn-ghost btn-xs btn-square rounded-box"
            title="Copy generation data"
            @click="copyText(generationText)"
          >
            <IconCopy class="w-4 h-4" />
          </button>
        </div>
        <div v-if="report.settings.length" class="grid grid-cols-[96px_1fr] gap-x-3 gap-y-1 text-xs">
          <template v-for="setting in report.settings" :key="`${setting.key}-${setting.value}`">
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/30 py-1">
              {{ setting.key }}
            </div>
            <div class="font-semibold text-base-content/70 break-all py-1">
              {{ setting.value }}
            </div>
          </template>
        </div>
        <div v-else class="text-xs text-base-content/30">No settings found</div>
      </section>

      <section class="border-t border-base-content/5 pt-3 space-y-2">
        <div class="flex items-center gap-2 text-base-content/70">
          <span class="font-bold mr-auto uppercase text-xs tracking-wide">Models</span>
        </div>

        <div v-if="report.resources.length" class="space-y-2">
          <div
            v-for="resource in report.resources"
            :key="resourceKey(resource)"
            class="rounded-box border border-base-content/5 bg-base-300/30 p-2 space-y-1"
          >
            <div class="flex items-start gap-2">
              <div class="min-w-0 flex-1">
                <div class="text-xs font-semibold text-base-content/80 break-words">{{ resource.modelName }}</div>
                <div class="text-[11px] text-base-content/40 break-words">
                  {{ resource.modelType }}<template v-if="resource.versionName"> · {{ resource.versionName }}</template>
                </div>
              </div>
              <span
                class="badge badge-xs shrink-0"
                :class="availabilityClass(resource.availability)"
              >
                {{ availabilityLabel(resource.availability) }}
              </span>
            </div>
            <div v-if="resource.baseModel" class="text-[11px] text-base-content/40">
              Base: {{ resource.baseModel }}
            </div>
            <div v-if="resource.strength !== null && resource.strength !== undefined" class="text-[11px] text-base-content/40">
              Strength: {{ resource.strength }}
            </div>
            <div v-if="resource.matchedPath" class="text-[11px] text-success break-all">
              {{ resource.matchedPath }}
            </div>
          </div>
        </div>

        <div v-else class="text-xs text-base-content/30">No model records found</div>
      </section>

      <section v-if="report.warnings.length" class="border-t border-base-content/5 pt-3 space-y-2">
        <div class="flex items-center gap-2 text-warning">
          <IconError class="w-4 h-4" />
          <span class="font-bold mr-auto uppercase text-xs tracking-wide">Notices</span>
        </div>
        <ul class="space-y-1 text-xs text-base-content/50">
          <li v-for="warning in report.warnings" :key="warning" class="break-words">{{ warning }}</li>
        </ul>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { analyzeCivitaiImageUrl } from '@/common/api';
import { IconClose, IconCopy, IconError, IconLink, IconSearch } from '@/common/icons';

interface CivitaiSetting {
  key: string;
  value: string;
}

interface CivitaiResource {
  modelName: string;
  versionName: string;
  modelType: string;
  baseModel?: string | null;
  modelId?: number | null;
  modelVersionId?: number | null;
  strength?: number | null;
  availability: string;
  matchedPath?: string | null;
}

interface CivitaiReport {
  sourceUrl: string;
  imageId?: number | null;
  imageUrl?: string | null;
  prompt?: string | null;
  negativePrompt?: string | null;
  settings: CivitaiSetting[];
  resources: CivitaiResource[];
  warnings: string[];
}

const imageUrl = ref('');
const modelRootsText = ref('');
const report = ref<CivitaiReport | null>(null);
const isLoading = ref(false);
const errorMessage = ref('');

const modelRoots = computed(() =>
  modelRootsText.value
    .split(/\r?\n/)
    .map((item) => item.trim())
    .filter(Boolean)
);

const generationText = computed(() => {
  if (!report.value) return '';
  const lines: string[] = [];
  if (report.value.prompt) lines.push(report.value.prompt);
  if (report.value.negativePrompt) lines.push(`Negative prompt: ${report.value.negativePrompt}`);
  if (report.value.settings.length) {
    lines.push(report.value.settings.map((item) => `${item.key}: ${item.value}`).join(', '));
  }
  return lines.join('\n');
});

async function analyze() {
  const url = imageUrl.value.trim();
  if (!url || isLoading.value) return;

  isLoading.value = true;
  errorMessage.value = '';

  try {
    report.value = await analyzeCivitaiImageUrl(url, modelRoots.value);
  } catch (error: any) {
    report.value = null;
    errorMessage.value = String(error?.message || error || 'Failed to analyze Civitai image');
  } finally {
    isLoading.value = false;
  }
}

async function copyText(text: string) {
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
  } catch (error) {
    console.error('Failed to copy text:', error);
  }
}

function availabilityClass(status: string) {
  if (status === 'found') return 'badge-success';
  if (status === 'missing') return 'badge-error';
  return 'badge-neutral';
}

function availabilityLabel(status: string) {
  if (status === 'found') return 'Found';
  if (status === 'missing') return 'Missing';
  return 'Unchecked';
}

function resourceKey(resource: CivitaiResource) {
  return [
    resource.modelVersionId || '',
    resource.modelName,
    resource.versionName,
    resource.modelType,
  ].join(':');
}
</script>
