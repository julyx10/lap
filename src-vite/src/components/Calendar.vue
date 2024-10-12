<template>

  <div class="flex-1 flex flex-col overflow-hidden" style="user-select: none;">

    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between" >
      <span>{{ titlebar }}</span>
      <div class="flex flex-row space-x-2 p-2">
        <IconLeft class="t-icon-hover" @click="clickPrevYear"/>
        <span>{{ selectedYear }}</span>
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
        :year="selectedYear" 
        :month="month"
        :monthName="localeMsg.calendar_months[month - 1]"
      />
    </div>
  
  </div>
  
</template>

<script setup>

import { ref, computed } from 'vue';
import CalendarMonth from './CalendarMonth.vue';

import IconLeft from '@/assets/arrow-left.svg';
import IconRight from '@/assets/arrow-right.svg';
// import IconUp from '@/assets/arrow-up.svg';
// import IconDown from '@/assets/arrow-down.svg';

const props = defineProps({
  titlebar: String
});

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

const selectedYear = ref(2024);

function clickPrevYear() {
  selectedYear.value -= 1;
}

function clickNextYear() {
  selectedYear.value += 1;
}
</script>
