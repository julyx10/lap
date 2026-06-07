<template>
  <ModalDialog :title="$t('msgbox.file_conflict.title')" :width="440" @cancel="resolve('skip')">
    <p class="text-sm whitespace-pre-line wrap-break-word">
      {{ $t('msgbox.file_conflict.message', { name, destination }) }}
    </p>

    <label v-if="showApplyAll" class="mt-4 flex cursor-pointer items-center gap-2 text-sm">
      <input v-model="applyAll" type="checkbox" class="checkbox checkbox-xs" />
      <span>{{ $t('msgbox.file_conflict.apply_all') }}</span>
    </label>

    <div class="mt-5 flex justify-end gap-2">
      <button class="px-3 py-1 rounded-box hover:bg-base-100 cursor-pointer" @click="resolve('skip')">
        {{ $t('msgbox.file_conflict.skip') }}
      </button>
      <button class="px-3 py-1 rounded-box hover:bg-base-100 cursor-pointer" @click="resolve('keep_both')">
        {{ $t('msgbox.file_conflict.keep_both') }}
      </button>
      <button
        v-if="allowReplace"
        class="px-3 py-1 rounded-box bg-error text-error-content hover:bg-error/90 cursor-pointer"
        @click="resolve('replace')"
      >
        {{ $t('msgbox.file_conflict.replace') }}
      </button>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { useUIStore } from '@/stores/uiStore';
import ModalDialog from '@/components/ModalDialog.vue';

type FileConflictPolicy = 'skip' | 'keep_both' | 'replace';

defineProps({
  name: {
    type: String,
    required: true,
  },
  destination: {
    type: String,
    required: true,
  },
  showApplyAll: {
    type: Boolean,
    default: false,
  },
  allowReplace: {
    type: Boolean,
    default: true,
  },
});

const emit = defineEmits<{
  resolve: [result: { policy: FileConflictPolicy; applyAll: boolean }];
}>();

const uiStore = useUIStore();
const applyAll = ref(false);

onMounted(() => uiStore.pushInputHandler('FileConflictDialog'));
onUnmounted(() => uiStore.removeInputHandler('FileConflictDialog'));

function resolve(policy: FileConflictPolicy) {
  emit('resolve', { policy, applyAll: applyAll.value });
}
</script>
