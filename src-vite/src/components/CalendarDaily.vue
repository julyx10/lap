<template>

  <div class="flex flex-col items-center mx-auto my-1 min-w-48 rounded-box border border-base-content/5">

    <!-- title -->
    <div 
      :class="[
        'mt-2 px-2 rounded-box text-nowrap cursor-pointer',
        isSelected(year, month, -1) ? 'text-primary bg-base-100 hover:bg-base-100 selected-item' : 'hover:text-base-content hover:bg-base-100/30'
      ]"
      @click="clickDate(year, month, -1)"
    >
      {{ monthTitle }}
    </div>

    <!-- date list -->
    <div class="p-2 grid grid-cols-7 gap-2 text-center">
      <div v-for="n in blankDates" :key="'blank' + n"></div>
      <div
        v-for="d in monthDates"
        :key="d.date"
        class="size-6 p-1 text-xs flex items-center justify-center rounded-box"
        :class="{
          'bg-base-content/5 cursor-default scale-80': d.count === 0,
          'text-base-content/70 hover:text-base-content cursor-pointer': d.count > 0,
          'bg-base-content/20': d.count > 0 && d.count < 10,
          'bg-base-content/50': d.count >= 10 && d.count < 100,
          'bg-base-content/80 text-[10px]': d.count >= 100,
          'bg-primary/70 text-primary-content/70 hover:text-primary-content/70 selected-item': isSelected(year, month, d.date),
          'border border-base-content/20': isTodayFn(d.date),
        }"
        @click="d.count > 0 ? clickDate(year, month, d.date): null"
      >
        {{ d.count > 0 ? (d.count < 1000 ? d.count : '999+') : '' }}
        <!-- {{ Number(d.date) }} -->
      </div>
    </div>

  </div>

</template>


<script setup lang="ts">

import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { getDaysInMonth, startOfMonth, getDay, isToday } from 'date-fns';
import { libConfig } from '@/common/config';
import { formatDate } from '@/common/utils';

const props = defineProps({
  year: {
    type: Number,
    required: true,
  },
  month: {
    type: Number,
    required: true,
  },
  dates: {
    type: Array,
    required: true,
  },
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

// Title of the month
const monthTitle = computed(() => formatDate(props.year, props.month, 1, localeMsg.value.format.month));

// Blank days at the start of the calendar (for proper alignment)
const blankDates = computed(() => {
  const firstDayOfMonth = getDay(startOfMonth(new Date(props.year, props.month - 1)));
  return [...Array(firstDayOfMonth).keys()];
});

// Array of { date, count } objects for the month
const monthDates = getMonthDates(props.year, props.month, props.dates);

// Check if the given date is today
const isTodayFn = (date) => isToday(new Date(props.year, props.month - 1, date));

// Check if the date is selected
const isSelected = (year, month, date) => libConfig.calendar.year === year &&
                                          libConfig.calendar.month === month && 
                                          libConfig.calendar.date === date;

// Generate an array of { date, count } objects for the month
function getMonthDates(year, month, dates = []) {
  // Get the number of days in the month
  const daysInMonth = getDaysInMonth(new Date(year, month - 1));

  // Create a map from the input dates for quick lookup
  const dateMap = new Map(dates.map(item => [Number(item.date), item.count]));

  // Create an array with { date, count } objects
  const dateCountArray = Array.from({ length: daysInMonth }, (v, i) => {
    const date = i + 1;
    return {
      date: date,
      count: Number(dateMap.get(date) || 0), // Use the count from the input if available, else default to 0
    };
  });

  // console.log('getMonthDates:', year, month, dateCountArray);
  return dateCountArray;
}

// click a date to select it
const clickDate = (year, month, date) => {
  libConfig.calendar.year = year;
  libConfig.calendar.month = month; // -1 means selecting a year
  libConfig.calendar.date = date;   // -1 means selecting a month

  console.log('clickDate:', libConfig.calendar.year, libConfig.calendar.month, libConfig.calendar.date);
};

</script>

