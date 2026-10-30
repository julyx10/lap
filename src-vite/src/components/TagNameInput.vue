<template>
  <div class="flex-1 min-w-0" @click.stop @mousedown.stop @dblclick.stop>
    <input
      ref="input"
      v-model="name"
      type="text"
      maxlength="255"
      class="input px-1 w-full text-base"
      :readonly="saving"
      :aria-invalid="!!error"
      @keydown.stop
      @keydown.enter.prevent="!$event.isComposing && emit('save', name)"
      @keydown.esc.prevent="emit('cancel')"
      @blur="emit('save', name)"
      @input="emit('clear-error')"
    />
    <p v-if="error" class="text-xs text-error">{{ error }}</p>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, onMounted, onBeforeUnmount } from "vue";
import { useUIStore } from "@/stores/uiStore";

const props = defineProps<{
  initialName: string;
  saving: boolean;
  error: string;
}>();
const emit = defineEmits<{
  save: [name: string];
  cancel: [];
  "clear-error": [];
}>();
const name = ref(props.initialName);
const input = ref<HTMLInputElement | null>(null);
const uiStore = useUIStore();
onMounted(() => {
  uiStore.pushInputHandler("TagNameInput");
  input.value?.scrollIntoView({ block: "nearest" });
  input.value?.focus();
  input.value?.select();
});
onBeforeUnmount(() => uiStore.removeInputHandler("TagNameInput"));
watch(
  () => props.error,
  async (error) => {
    if (error) {
      await nextTick();
      input.value?.focus();
    }
  },
);
</script>
