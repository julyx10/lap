<template>
  <MessageBox
    position-key="file-delete"
    :title="permanently ? $t('msgbox.file_delete.title', confirmationCount) : $t('msgbox.move_to_trash.title')"
    :message="plan ? undefined : trashMessage"
    :OkText="permanently ? $t('msgbox.file_delete.confirm', confirmationCount) : $t('msgbox.move_to_trash.ok')"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    :focusCancel="true"
    :isLoading="isDeleting"
    @ok="$emit('ok')"
    @cancel="$emit('cancel')"
  >
    <template v-if="plan">
      <p class="text-sm">
        {{ subtitle }}
        {{ permanently ? $t('msgbox.file_delete.irreversible') : '' }}
      </p>
      <p v-if="plan.trashUnavailable" class="mt-2 text-sm text-warning">
        {{ $t('msgbox.file_delete.trash_unavailable') }}
      </p>
    </template>
    <div v-if="previewFiles.length" class="my-4 flex flex-wrap gap-3 rounded-box border border-base-content/10 bg-base-300/30 p-3">
      <div v-for="file in previewFiles" :key="file.id"
        class="relative h-12 w-12 shrink-0" :title="file.name">
        <div v-if="file.rawPair" class="absolute inset-0 translate-x-1 translate-y-1 rounded-box border border-base-content/20 bg-base-300"></div>
        <img :src="thumbnails[file.id] || file.thumbnail || thumbnailPlaceholder" :alt="file.name"
          class="relative h-full w-full rounded-box border border-base-content/20 bg-base-300 object-cover" />
      </div>
      <span v-if="previewItemCount > previewLimit"
        class="flex h-12 w-12 shrink-0 items-center justify-center rounded-box bg-base-300 text-sm">
        +{{ previewItemCount - previewLimit }}
      </span>
    </div>
    <template v-if="plan">
      <div v-if="rawPairDeleteCount" class="rounded-box border border-base-content/5 bg-base-300/30 p-3">
        <label class="flex items-center justify-between gap-4 text-sm">
          <span>{{ $t('msgbox.raw_pair_delete.scope') }}</span>
          <select :value="rawPairScope" class="select select-bordered select-sm w-40 shrink-0" :disabled="isDeleting"
            @change="$emit('update:rawPairScope', ($event.target as HTMLSelectElement).value)">
            <option v-for="choice in rawPairDeleteChoices" :key="choice.value" :value="choice.value">
              {{ choice.label }}
            </option>
          </select>
        </label>
        <p v-if="singleDeleteCount" class="mt-2 text-xs text-base-content/70">{{ $t('msgbox.file_delete.mixed_hint') }}</p>
      </div>
    </template>
  </MessageBox>
</template>

<script setup lang="ts">
import MessageBox from '@/components/MessageBox.vue';
import type { FileDeletePlan, FileDeleteTarget } from '@/common/fileActions';

defineProps<{
  plan: FileDeletePlan | null;
  permanently: boolean;
  confirmationCount: number;
  subtitle: string;
  trashMessage: string;
  previewFiles: FileDeleteTarget[];
  previewItemCount: number;
  previewLimit: number;
  thumbnails: Record<number, string>;
  rawPairDeleteCount: number;
  singleDeleteCount: number;
  rawPairDeleteChoices: Array<{ value: string; label: string }>;
  rawPairScope: 'both' | 'raw' | 'companion';
  isDeleting: boolean;
}>();

defineEmits<{
  ok: [];
  cancel: [];
  'update:rawPairScope': ['both' | 'raw' | 'companion'];
}>();

const thumbnailPlaceholder = new URL('@/assets/images/image-file.png', import.meta.url).href;
</script>
