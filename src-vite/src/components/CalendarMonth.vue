<template>

  <span class="p-4 font-lg">{{ year }} - {{ month }}</span>

  <div class="grid grid-cols-7 gap-2 text-center">
    <div v-for="n in blankDays" :key="'blank' + n"></div>
    <div
      v-for="date in dates"
      :key="date"
      class="size-8 flex items-center justify-center border rounded-full t-color-bg-hover cursor-pointer"
      :class="{
        'bg-sky-900': isTodayFn(date[0]),
        'border-sky-500': isSelectedDate(date[0]),
        'border-transparent': !isSelectedDate(date[0]),
      }"
      @click="clickDate(date)"
    >
      <div class="flex flex-col">
        {{ Number(date.date) }} 
        <!-- <span v-if="date.count > 0" class="text-xs">{{ date.count }}</span> -->
      </div>
    </div>
  </div>

</template>


<script setup>

import { ref, inject, computed, watch, onMounted } from 'vue';
import { getDaysInMonth, startOfMonth, getDay, isToday } from 'date-fns';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

const props = defineProps({
  year: {
    type: Number,
    required: true,
  },
  month: {
    type: Number,
    required: true,
  },
  dates: {      // [[date, count]], for example [ [1, 0], [2, 12], [3, 13] ];
    type: Array,
    required: true,
  },
});

const gCalendarYear = inject('gCalendarYear');
const gCalendarMonth = inject('gCalendarMonth');
const gCalendarDate = inject('gCalendarDate');

onMounted(() => {
  console.log('CalendarMonth:', props.year, props.month, props.dates);
});

// Computed property to get the number of days in the month
// const daysInMonth = computed(() => getDaysInMonth(new Date(props.year, props.month - 1)));

// Blank days at the start of the calendar (for proper alignment)
const blankDays = computed(() => {
  const firstDayOfMonth = getDay(startOfMonth(new Date(props.year, props.month - 1)));
  return [...Array(firstDayOfMonth).keys()];
});

// Check if the given date is today
const isTodayFn = (date) => isToday(new Date(props.year, props.month - 1, date));

// Check if the date is selected
const isSelectedDate = (date) => gCalendarYear.value === props.year &&
                                 gCalendarMonth.value === props.month && 
                                 gCalendarDate.value === date;

// click a date to select it
const clickDate = (date) => {
  gCalendarYear.value = props.year;
  gCalendarMonth.value = props.month;
  gCalendarDate.value = date;

  console.log('clickDate:', gCalendarYear.value, gCalendarMonth.value, gCalendarDate.value);
};

</script>

