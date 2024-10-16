<template>

  <div class="flex-1 flex flex-col overflow-hidden" style="user-select: none;">

    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between" >
      <span>{{ titlebar }}</span>

      <div class="flex">
        <IconRefresh class="p-0.5 t-icon-hover" @click="clickRefresh"/>
      </div>
    </div>
    
    <!-- days of the week -->
    <div class="flex flex-col items-center mr-4">

      <div class="grid grid-cols-7 gap-2 text-center">
        <div 
          v-for="(day, index) in localeMsg.calendar_weekdays" 
          :key="index" 
          class="p-2 w-8 flex items-center justify-center"
        >
          {{ day }}
        </div>
      </div>
    </div>  
    <!-- calendar -->
    <div class="overflow-auto t-scrollbar-dark">
      <div v-for="(months, year) in calendar_dates" class="flex flex-col items-center">
        <CalendarMonth v-for="(dates, month) in months" 
          :year="Number(year)" 
          :month="Number(month)"
          :dates="dates"
        />
      </div>
    </div>
  
  </div>
  
</template>

<script setup>

import { ref, computed, inject, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api';
import CalendarMonth from './CalendarMonth.vue';

import IconRefresh from '@/assets/refresh.svg';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

// props
const props = defineProps({
  titlebar: String
});

const calendar_dates = ref([]);


onMounted( () => {
  console.log('Calendar.vue mounted');
  // if(gCalendarDays.value.length === 0) {
    getCalendarDates();
  // }
});


/// refresh taken dates
function clickRefresh() {
  console.log('clickRefresh...');
  getCalendarDates();
};


/// fetch calendar dates
async function getCalendarDates() {
  try {
    let taken_dates = await invoke('get_taken_dates');
    calendar_dates.value = transformArray(taken_dates);
    console.log('getCalendarDates...', calendar_dates.value);
  } catch (error) {
    console.error('Failed to fetch calendar dates:', error);
  }
}


/// input: [['2024-10-15', 5], ['2023-01-01', 10]];
/// output: [{2024: {10: {15: 5}}, {2023: {01: {01: 10}}}]
function transformArray(dates) {
  const result = {};

  dates.forEach(item => {
    const [dateStr, count] = item;
    const [year, month, date] = dateStr.split('-');

    // Initialize the year object if it doesn't exist
    if (!result[year]) {
      result[year] = {};
    }

    // Initialize the month object if it doesn't exist
    if (!result[year][month]) {
      result[year][month] = [];
    }

    // Push the date and count as an object into the month array
    result[year][month].push({ date, count });
  });

  return result;
}

</script>
