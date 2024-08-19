<template>

  <div class="text-xl">
    {{ titlebar }}
  </div>

</template>

<script setup>
const props = defineProps({
  titlebar: String
});
</script>
