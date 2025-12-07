<template>

  <div class="flex flex-col items-center mx-auto my-1 min-w-64 rounded-box border border-base-content/5">

    <!-- title -->
    <div 
      :class="[
        'mt-2 px-2 rounded-box hover:bg-base-100 text-nowrap cursor-pointer',
        isSelected(year, month, -1) ? 'text-primary bg-base-100' : 'hover:text-base-content'
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
        class="size-8 text-sm flex items-center justify-center rounded-box"
        :class="[
          isSelected(year, month, d.date) ? 'text-primary bg-base-100' : (d.count === 0 ? '' : 'hover:text-base-content'),
          d.count === 0 ? 'text-base-content/30 cursor-default' : 'hover:bg-base-100 cursor-pointer',
          isTodayFn(d.date) ? 'bg-base-100' : '',
        ]"
        @click="d.count > 0 ? clickDate(year, month, d.date): null"
      >
        {{ Number(d.date) }}
      </div>
    </div>

  </div>

</template>


<script setup>

import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { getDaysInMonth, startOfMonth, getDay, isToday } from 'date-fns';
import { config } from '@/common/config';
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
const localeMsg = computed(() => messages.value[locale.value]);

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
const isSelected = (year, month, date) => config.calendar.year === year &&
                                          config.calendar.month === month && 
                                          config.calendar.date === date;

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
      count: dateMap.get(date) || 0 // Use the count from the input if available, else default to 0
    };
  });

  // console.log('getMonthDates:', year, month, dateCountArray);
  return dateCountArray;
}

// click a date to select it
const clickDate = (year, month, date) => {
  config.calendar.year = year;
  config.calendar.month = month; // -1 means selecting a year
  config.calendar.date = date;   // -1 means selecting a month

  console.log('clickDate:', config.calendar.year, config.calendar.month, config.calendar.date);
};

</script>

