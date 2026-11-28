<template>

  <div class="w-full min-w-60 flex flex-col items-center my-1 rounded-box border border-base-content/5">

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

    <div class="px-2 pt-2 grid grid-cols-7 gap-2 text-center text-xs font-semibold text-base-content/30">
      <div
        v-for="(weekday, index) in weekdayLabels"
        :key="index"
        class="size-6 flex items-center justify-center"
      >
        {{ weekday }}
      </div>
    </div>

    <!-- date list -->
    <div class="p-2 grid grid-cols-7 gap-2 text-center">
      <div v-for="n in blankDates" :key="'blank' + n"></div>
      <div
        v-for="d in monthDates"
        :key="d.date"
        class="size-6 p-1 text-xs flex items-center justify-center rounded-box"
        :class="{
          'bg-base-content/5 cursor-default scale-80': d.count === 0 && isWeekend(d.date),
          'bg-base-content/10 cursor-default scale-80': d.count === 0 && !isWeekend(d.date),
          'text-base-100/70 hover:text-base-100 hover:bg-base-content/80 hover:scale-110 transition-all cursor-pointer': d.count > 0,
          'bg-base-content/30': heatLevel(d.count) === 1,
          'bg-base-content/40': heatLevel(d.count) === 2,
          'bg-base-content/50': heatLevel(d.count) === 3,
          'bg-base-content/60': heatLevel(d.count) === 4 && !isSelected(year, month, d.date),
          'text-base-100! bg-primary hover:bg-primary scale-110': isSelected(year, month, d.date),
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

import { computed, PropType } from 'vue';
import { useI18n } from 'vue-i18n';
import { getDaysInMonth, startOfMonth, getDay, isToday } from 'date-fns';
import { libConfig } from '@/common/config';
import { formatDate } from '@/common/utils';

interface DateItem {
  date: number;
  count: number;
}

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
    type: Array as PropType<DateItem[]>,
    required: true,
  },
  heatmapThresholds: {
    type: Array as PropType<number[] | null>,
    default: null,
  },
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const weekdayLabels = computed(() => localeMsg.value.calendar?.weekdays || ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']);

// Title of the month
const monthTitle = computed(() => formatDate(props.year, props.month, 1, 'month'));

// Blank days at the start of the calendar (for proper alignment)
const blankDates = computed(() => {
  const firstDayOfMonth = getDay(startOfMonth(new Date(props.year, props.month - 1)));
  return [...Array(firstDayOfMonth).keys()];
});

// Array of { date, count } objects for the month
const monthDates = getMonthDates(props.year, props.month, props.dates);

// Check if the given date is today
const isTodayFn = (date: number) => isToday(new Date(props.year, props.month - 1, date));
const isWeekend = (date: number) => {
  const weekday = getDay(new Date(props.year, props.month - 1, date));
  return weekday === 0 || weekday === 6;
};

function heatLevel(count: number) {
  if (count === 0) return 0;

  const thresholds = props.heatmapThresholds;
  if (thresholds) {
    if (count < thresholds[0]) return 1;
    if (count < thresholds[1]) return 2;
    if (count < thresholds[2]) return 3;
    return 4;
  }

  if (count < 10) return 1;
  if (count < 100) return 2;
  return 4;
}

// Check if the date is selected
const isSelected = (year: number, month: number, date: number) => libConfig.calendar.year === year &&
                                          libConfig.calendar.month === month && 
                                          libConfig.calendar.date === date;

// Generate an array of { date, count } objects for the month
function getMonthDates(year: number, month: number, dates: DateItem[] = []) {
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
const clickDate = (year: number, month: number, date: number) => {
  libConfig.calendar.year = year;
  libConfig.calendar.month = month; // -1 means selecting a year
  libConfig.calendar.date = date;   // -1 means selecting a month

  console.log('clickDate:', libConfig.calendar.year, libConfig.calendar.month, libConfig.calendar.date);
};

</script>
