<template>

  <div class="flex-1 flex flex-col overflow-hidden" style="user-select: none;">

    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between" >
      <span>{{ titlebar }}</span>
      <div class="flex flex-row space-x-2 p-2">
        <IconLeft class="t-icon-hover" @click="clickPrevYear"/>
        <span>{{ gCalendarYear }}</span>
        <IconRight class="t-icon-hover" @click="clickNextYear" />
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
    <div class="flex flex-col items-center overflow-auto t-scrollbar-dark">
      <CalendarMonth 
        v-for="month in 12" 
        :key="month" 
        :year="gCalendarYear" 
        :month="month"
        :monthName="localeMsg.calendar_months_long[month - 1]"
      />
    </div>
  
  </div>
  
</template>

<script setup>

import { ref, computed, inject, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api';
import CalendarMonth from './CalendarMonth.vue';

import IconLeft from '@/assets/arrow-left.svg';
import IconRight from '@/assets/arrow-right.svg';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

// props
const props = defineProps({
  titlebar: String
});

const taken_dates = ref([]);

// select date
const gCalendarYear = inject('gCalendarYear');

onMounted(() => {
  console.log('Calendar.vue mounted');
  if(taken_dates.value.length === 0) {
    getTakenDates();
  }
});

function clickPrevYear() {
  gCalendarYear.value -= 1;
}

function clickNextYear() {
  gCalendarYear.value += 1;
}

/// refresh taken dates
const clickRefresh = async () => {
  console.log('clickRefresh...');
  await getTakenDates();
};

async function getTakenDates() {
  try {
    taken_dates.value = await invoke('get_taken_dates');
    console.log('getTakenDates...', taken_dates.value);
  } catch (error) {
    console.error('Failed to fetch taken dates:', error);
  }
}

</script>
