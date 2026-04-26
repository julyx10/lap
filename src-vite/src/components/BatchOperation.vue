<template>
  <ModalDialog :title="title" :width="700" @cancel="clickCancel">
    <div class="space-y-4">
      <!-- Operation Type Selection -->
      <div class="flex gap-4">
        <div
          v-for="op in operations"
          :key="op.type"
          :class="[
            'flex-1 p-4 rounded-lg border-2 cursor-pointer transition-all',
            selectedOperation === op.type
              ? 'border-primary bg-primary/10'
              : 'border-base-content/10 hover:border-primary/50'
          ]"
          @click="selectedOperation = op.type"
        >
          <div class="flex items-center gap-3">
            <component :is="op.icon" class="w-6 h-6 text-primary" />
            <div>
              <div class="font-semibold">{{ op.label }}</div>
              <div class="text-xs text-base-content/50">{{ op.description }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- Batch Rename Section -->
      <template v-if="selectedOperation === 'rename'">
        <div class="space-y-3">
          <div class="form-control">
            <label class="label">
              <span class="label-text font-medium">{{ $t('batch_rename.pattern') }}</span>
            </label>
            <input
              v-model="renamePattern"
              type="text"
              class="input input-bordered w-full"
              :placeholder="$t('batch_rename.pattern_placeholder')"
            />
          </div>

          <div class="text-xs text-base-content/50 space-y-1">
            <div>{{ $t('batch_rename.variables') }}:</div>
            <div class="flex flex-wrap gap-2">
              <span
                v-for="variable in renameVariables"
                :key="variable"
                class="px-2 py-1 bg-base-300 rounded text-xs cursor-pointer hover:bg-primary/20"
                @click="insertVariable(variable)"
              >
                {{ variable }}
              </span>
            </div>
          </div>

          <!-- Preview -->
          <div v-if="files.length > 0" class="border rounded-lg p-3 bg-base-300/30 max-h-48 overflow-y-auto">
            <div class="text-xs font-medium text-base-content/50 mb-2">{{ $t('batch_rename.preview') }}</div>
            <div v-for="(file, index) in previewRename" :key="index" class="text-sm py-1">
              <span class="text-base-content/50">{{ file.original }}</span>
              <span class="mx-2">→</span>
              <span class="text-primary font-medium">{{ file.new }}</span>
            </div>
          </div>
        </div>
      </template>

      <!-- Batch Resize Section -->
      <template v-if="selectedOperation === 'resize'">
        <div class="space-y-3">
          <div class="grid grid-cols-2 gap-4">
            <div class="form-control">
              <label class="label">
                <span class="label-text font-medium">{{ $t('batch_resize.width') }}</span>
              </label>
              <input
                v-model="resizeWidth"
                type="number"
                min="1"
                class="input input-bordered w-full"
                :disabled="resizeMode === 'percentage'"
              />
            </div>
            <div class="form-control">
              <label class="label">
                <span class="label-text font-medium">{{ $t('batch_resize.height') }}</span>
              </label>
              <input
                v-model="resizeHeight"
                type="number"
                min="1"
                class="input input-bordered w-full"
                :disabled="resizeMode === 'percentage'"
              />
            </div>
          </div>

          <div class="flex items-center gap-2">
            <input type="checkbox" v-model="keepAspectRatio" class="checkbox checkbox-sm" />
            <span class="text-sm">{{ $t('batch_resize.keep_aspect_ratio') }}</span>
          </div>

          <div class="form-control">
            <label class="label">
              <span class="label-text font-medium">{{ $t('batch_resize.mode') }}</span>
            </label>
            <select v-model="resizeMode" class="select select-bordered w-full">
              <option value="pixels">{{ $t('batch_resize.mode_pixels') }}</option>
              <option value="percentage">{{ $t('batch_resize.mode_percentage') }}</option>
            </select>
          </div>

          <div v-if="resizeMode === 'percentage'" class="form-control">
            <label class="label">
              <span class="label-text font-medium">{{ $t('batch_resize.percentage') }}</span>
            </label>
            <input
              v-model="resizePercentage"
              type="number"
              min="1"
              max="500"
              class="input input-bordered w-full"
            />
          </div>

          <div class="form-control">
            <label class="label">
              <span class="label-text font-medium">{{ $t('batch_resize.quality') }}</span>
            </label>
            <select v-model="resizeQuality" class="select select-bordered w-full">
              <option value="90">High (90%)</option>
              <option value="80">Medium (80%)</option>
              <option value="60">Low (60%)</option>
            </select>
          </div>

          <div class="form-control">
            <label class="label">
              <span class="label-text font-medium">{{ $t('batch_resize.output_format') }}</span>
            </label>
            <select v-model="outputFormat" class="select select-bordered w-full">
              <option value="original">{{ $t('batch_resize.format_original') }}</option>
              <option value="jpg">JPEG</option>
              <option value="png">PNG</option>
              <option value="webp">WebP</option>
            </select>
          </div>
        </div>
      </template>

      <!-- Output Folder Selection -->
      <div class="form-control">
        <label class="label">
          <span class="label-text font-medium">{{ $t('batch_operation.output_folder') }}</span>
        </label>
        <div class="flex gap-2">
          <input
            v-model="outputFolder"
            type="text"
            class="input input-bordered flex-1"
            readonly
          />
          <button class="btn btn-outline" @click="selectOutputFolder">
            {{ $t('batch_operation.browse') }}
          </button>
        </div>
      </div>

      <!-- Selected Files Count -->
      <div class="text-sm text-base-content/50">
        {{ $t('batch_operation.selected_files', { count: files.length }) }}
      </div>
    </div>

    <!-- Action Buttons -->
    <div class="mt-6 flex justify-end space-x-4">
      <button class="btn btn-ghost" @click="clickCancel">
        {{ $t('batch_operation.cancel') }}
      </button>
      <button
        class="btn btn-primary"
        :disabled="files.length === 0 || isProcessing"
        @click="executeOperation"
      >
        <span v-if="isProcessing" class="loading loading-spinner loading-sm"></span>
        {{ selectedOperation === 'rename' ? $t('batch_operation.rename') : $t('batch_operation.resize') }}
      </button>
    </div>

    <!-- Progress Modal -->
    <div v-if="showProgress" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-base-100 rounded-lg p-6 w-96 space-y-4">
        <div class="text-lg font-semibold">{{ $t('batch_operation.processing') }}</div>
        <div class="space-y-2">
          <div class="text-sm">{{ currentFile }}</div>
          <div class="progress-bar h-2 bg-base-300 rounded-full overflow-hidden">
            <div
              class="h-full bg-primary transition-all"
              :style="{ width: progressPercent + '%' }"
            ></div>
          </div>
          <div class="text-xs text-base-content/50 text-right">
            {{ completed }}/{{ total }}
          </div>
        </div>
        <button class="btn btn-outline btn-sm w-full" @click="cancelOperation">
          {{ $t('batch_operation.cancel_operation') }}
        </button>
      </div>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { listen, type Event } from '@tauri-apps/api/event';
import { open as openDialog } from '@tauri-apps/api/dialog';
import { batchRename, batchResize } from '@/common/api';
import { getFolderPath, extractFileName, combineFileName } from '@/common/utils';
import ModalDialog from '@/components/ModalDialog.vue';
import { IconRename, IconResize } from '@/common/icons';

interface FileInfo {
  id: number;
  name: string;
  file_path: string;
  file_type: number;
}

interface RenamePreview {
  original: string;
  new: string;
}

const props = defineProps<{
  files: FileInfo[];
  operation: 'rename' | 'resize';
}>();

const emit = defineEmits(['success', 'cancel']);

const selectedOperation = ref<'rename' | 'resize'>(props.operation || 'rename');

// Rename state
const renamePattern = ref('{name}_{index}');
const renameVariables = ['{name}', '{index}', '{date}', '{time}'];

// Resize state
const resizeWidth = ref(1920);
const resizeHeight = ref(1080);
const keepAspectRatio = ref(true);
const resizeMode = ref<'pixels' | 'percentage'>('pixels');
const resizePercentage = ref(50);
const resizeQuality = ref(80);
const outputFormat = ref('original');

// Output folder
const outputFolder = ref(getFolderPath(props.files[0]?.file_path || ''));

// Processing state
const isProcessing = ref(false);
const showProgress = ref(false);
const progressPercent = ref(0);
const completed = ref(0);
const total = ref(0);
const currentFile = ref('');

let unlistenKeydown: () => void;
let cancelled = false;

const operations = computed(() => [
  {
    type: 'rename',
    label: 'Batch Rename',
    description: 'Rename multiple files with pattern',
    icon: IconRename,
  },
  {
    type: 'resize',
    label: 'Batch Resize',
    description: 'Resize multiple images',
    icon: IconResize,
  },
]);

const previewRename = computed<RenamePreview[]>(() => {
  if (selectedOperation.value !== 'rename' || props.files.length === 0) return [];

  const previews: RenamePreview[] = [];
  const pattern = renamePattern.value;

  props.files.forEach((file, index) => {
    const { name, ext } = extractFileName(file.name);
    const now = new Date();
    const date = now.toISOString().split('T')[0];
    const time = now.toTimeString().split(' ')[0].replace(/:/g, '-');

    let newName = pattern
      .replace('{name}', name)
      .replace('{index}', String(index + 1).padStart(3, '0'))
      .replace('{date}', date)
      .replace('{time}', time);

    if (!newName.includes('.') && ext) {
      newName = combineFileName(newName, ext);
    }

    previews.push({
      original: file.name,
      new: newName,
    });
  });

  return previews;
});

const title = computed(() =>
  selectedOperation.value === 'rename'
    ? 'Batch Rename Files'
    : 'Batch Resize Images'
);

onMounted(async () => {
  unlistenKeydown = await listen<KeyPayload>('global-keydown', handleKeyDown);
});

onUnmounted(() => {
  if (unlistenKeydown) {
    unlistenKeydown();
  }
});

interface KeyPayload {
  key: string;
}

function handleKeyDown(event: Event<KeyPayload>) {
  const { key } = event.payload;
  switch (key) {
    case 'Escape':
      clickCancel();
      break;
  }
}

function insertVariable(variable: string) {
  renamePattern.value += variable;
}

async function selectOutputFolder() {
  const selected = await openDialog({
    directory: true,
    multiple: false,
  });
  if (selected) {
    outputFolder.value = selected as string;
  }
}

async function executeOperation() {
  cancelled = false;
  isProcessing.value = true;
  total.value = props.files.length;
  completed.value = 0;
  progressPercent.value = 0;
  showProgress.value = true;

  try {
    if (selectedOperation.value === 'rename') {
      await executeRename();
    } else {
      await executeResize();
    }
    emit('success', { count: completed.value });
  } catch (error) {
    console.error('Batch operation failed:', error);
  } finally {
    isProcessing.value = false;
    showProgress.value = false;
  }
}

async function executeRename() {
  for (let i = 0; i < props.files.length; i++) {
    if (cancelled) break;

    const file = props.files[i];
    const preview = previewRename.value[i];

    currentFile.value = file.name;
    completed.value = i + 1;
    progressPercent.value = Math.round((completed.value / total.value) * 100);

    try {
      await batchRename({
        file_id: file.id,
        source_path: file.file_path,
        new_name: preview.new,
      });
    } catch (error) {
      console.error(`Failed to rename ${file.name}:`, error);
    }
  }
}

async function executeResize() {
  const params = {
    file_ids: props.files.map(f => f.id),
    width: resizeMode.value === 'pixels' ? parseInt(resizeWidth.value as unknown as string) : null,
    height: resizeMode.value === 'pixels' ? parseInt(resizeHeight.value as unknown as string) : null,
    percentage: resizeMode.value === 'percentage' ? parseInt(resizePercentage.value as unknown as string) : null,
    keep_aspect_ratio: keepAspectRatio.value,
    quality: parseInt(resizeQuality.value as unknown as string),
    output_format: outputFormat.value === 'original' ? null : outputFormat.value,
    output_folder: outputFolder.value || null,
  };

  for (let i = 0; i < props.files.length; i++) {
    if (cancelled) break;

    const file = props.files[i];
    currentFile.value = file.name;
    completed.value = i + 1;
    progressPercent.value = Math.round((completed.value / total.value) * 100);

    try {
      await batchResize({
        file_id: file.id,
        file_path: file.file_path,
        width: params.width,
        height: params.height,
        percentage: params.percentage,
        keep_aspect_ratio: params.keep_aspect_ratio,
        quality: params.quality,
        output_format: params.output_format,
        output_folder: params.output_folder,
      });
    } catch (error) {
      console.error(`Failed to resize ${file.name}:`, error);
    }
  }
}

function cancelOperation() {
  cancelled = true;
  showProgress.value = false;
}

function clickCancel() {
  emit('cancel');
}
</script>