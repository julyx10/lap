<template>
  <ModalDialog :title="$t('info_panel.dedup.keep_rules.add_folder')" :width="500" position-key="preferred-folder" @cancel="emit('cancel')">
    <div class="h-[min(400px,55vh)] overflow-auto">
      <AlbumList ref="albumList" selection-source="local" :show-total-count="false" />
    </div>
    <div class="mt-4 flex items-center gap-2 select-none">
      <p role="status" class="mr-auto text-xs text-base-content/60">
        {{ alreadyAdded ? $t('info_panel.dedup.keep_rules.folder_already_added') : (selectionFailed ? $t('msgbox.select_folder.error') : '') }}
      </p>
      <button type="button" class="t-button-default" @click="emit('cancel')">{{ $t('msgbox.cancel') }}</button>
      <button type="button" class="t-button-primary" :disabled="!canAdd" @click="addFolder">{{ $t('info_panel.dedup.keep_rules.add_folder') }}</button>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { selectFolder } from '@/common/api';
import AlbumList from './AlbumList.vue';
import ModalDialog from './ModalDialog.vue';

const props = defineProps<{ folders: (string | undefined)[] }>();
const emit = defineEmits<{ add: [folder: { id: number; path: string }]; cancel: [] }>();
const albumList = ref<InstanceType<typeof AlbumList> | null>(null);
const folder = computed(() => albumList.value?.selectedFolder ?? null);
const alreadyAdded = computed(() => !!folder.value && props.folders.includes(folder.value.path));
const adding = ref(false);
const selectionFailed = ref(false);
const canAdd = computed(() => !!folder.value && !alreadyAdded.value && !adding.value);
let disposed = false;

watch(folder, () => { selectionFailed.value = false; });
onBeforeUnmount(() => { disposed = true; });

async function addFolder() {
  const selected = folder.value;
  if (!selected || !canAdd.value) return;

  adding.value = true;
  selectionFailed.value = false;
  const record = await selectFolder(selected.albumId, selected.path);
  if (disposed) return;
  adding.value = false;
  if (record?.id > 0) emit('add', { id: record.id, path: record.path });
  else if (folder.value === selected) selectionFailed.value = true;
}
</script>
