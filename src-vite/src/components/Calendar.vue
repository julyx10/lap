<template>

  <div class="flex px-1 py-3 items-center justify-between" style="user-select: none;">
    {{ titlebar }}
  </div>
  
</template>

<script setup>

const props = defineProps({
  titlebar: String
});

</script>
