<template>

  <div class="flex flex-col items-center">

    <!-- title -->
    <span class="pt-2 pb-4 t-color-text-dark">
      {{ monthTitle }}
    </span>

    <!-- date list -->
    <div class="pb-2 px-1 grid grid-cols-7 gap-2 text-center">
      <div v-for="n in blankDates" :key="'blank' + n"></div>
      <div
        v-for="d in monthDates"
        :key="d.date"
        class="size-8 flex items-center justify-center border rounded-full t-color-bg-hover cursor-pointer"
        :class="[
          {
            'bg-sky-900': isTodayFn(d.date),
            'border-sky-500': isSelectedDate(d.date),
            'border-transparent': !isSelectedDate(d.date),
          },
          d.count > 0 ? 't-color-text' : 't-color-text-dark'
        ]"
        @click="clickDate(d.date)"
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
import { useConfigStore } from '@/stores/configStore';
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

// config store
const config = useConfigStore();

// Title of the month
const monthTitle = computed(() => formatDate(props.year, props.month, 1, localeMsg.value.month_format));

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
const isSelectedDate = (date) => config.calendarYear === props.year &&
                                 config.calendarMonth === props.month && 
                                 config.calendarDate === date;

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
const clickDate = (date) => {
  config.calendarYear = props.year;
  config.calendarMonth = props.month;
  config.calendarDate = date;

  console.log('clickDate:', config.calendarYear, config.calendarMonth, config.calendarDate);
};

</script>

