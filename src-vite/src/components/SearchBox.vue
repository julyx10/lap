<template>

  <div >
    <!-- Search Box -->
    <input
      type="text"
      :placeholder="$t('search_placeholder')"
      class="px-2 py-1 w-full text-sm border rounded-md t-input-color-bg t-color-border t-input-focus"
      @mousedown.stop
    />
    <!-- Search Icon -->
    <IconSearch class="absolute right-2 top-1/2 transform -translate-y-1/2 h-5 w-5" @click="clickSearch" />
  </div>

</template>


<script setup lang="ts">

import IconSearch from '@/assets/search.svg';

const clickSearch = () => {
  console.log('clickSearch');
};

</script>