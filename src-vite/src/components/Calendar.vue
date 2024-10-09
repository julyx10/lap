<template>

  <div class="flex-1 flex flex-col overflow-auto">

    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between" style="user-select: none;">
      <div>
        {{ titlebar }}
      </div>
      <div class="flex">
        <div>Year</div>
        <div>Month</div>
        <div>Day</div>
      </div>
    </div>

    <!-- calendar -->
    <div  class="flex-grow overflow-auto t-scrollbar">
    </div>
  
  </div>
  
</template>

<script setup>

import IconUp from '@/assets/arrow-up.svg';
import IconDown from '@/assets/arrow-down.svg';

const props = defineProps({
  titlebar: String
});


import { ref, computed } from 'vue';

const currentView = ref('year');
const currentYear = ref(new Date().getFullYear());
const selectedDate = ref(new Date().toISOString().substr(0, 10));
const months = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'];
const currentMonthIndex = ref(new Date().getMonth());
const currentMonth = computed(() => months[currentMonthIndex.value]);

const toggleView = (view) => {
  currentView.value = view;
};

const getDaysInMonth = (year, month) => {
  return new Array(new Date(year, month, 0).getDate()).fill(0).map((_, i) => i + 1);
};

const updateDate = () => {
  // This will be where you handle date updates, if necessary
};
</script>
