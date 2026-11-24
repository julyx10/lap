<template>
  <MessageBox
    position-key="rename-file"
    :title="$t('msgbox.rename_file.title')"
    :showInput="true"
    :inputText="name"
    :inputPlaceholder="$t('msgbox.rename_file.placeholder')"
    :inputExtension="extension"
    :needValidateInput="true"
    :OkText="$t('msgbox.rename_file.ok')"
    :cancelText="$t('msgbox.cancel')"
    :errorMessage="errorMessage"
    @ok="$emit('ok', $event)"
    @cancel="$emit('cancel')"
    @reset="$emit('reset')"
  />
</template>

<script setup lang="ts">
import MessageBox from '@/components/MessageBox.vue';

defineProps<{
  name?: string;
  extension?: string;
  errorMessage?: string;
}>();

defineEmits<{
  ok: [value: string];
  cancel: [];
  reset: [];
}>();
</script>
