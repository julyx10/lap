<template>

<div class="p-1 flex items-center justify-between">
  <div class="text-xl">
    {{ titlebar }}
  </div>
  <div class="flex justify-end size-10">
    <img src="/img64/favorites.png" alt="Add Tags" class="p-1 m-1 hover:bg-gray-700 hover:rounded" @click="" />
    <img src="/img64/tags.png" alt="Add Tags" class="p-1 m-1 hover:bg-gray-700 hover:rounded" @click="" />
  </div>
  
</div>


</template>
  
<script setup>
const props = defineProps({
  titlebar: String
});
</script>
  