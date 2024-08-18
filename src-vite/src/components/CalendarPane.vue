<template>

  <div>
    <h2 class="text-xl font-bold">{{ titlebar }}</h2>
  </div>

</template>

<script setup>
import { defineProps } from 'vue';

const props = defineProps({
  titlebar: String
});
</script>
