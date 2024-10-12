<template>

  <span class="p-4 font-lg">{{ monthName }}</span>

  <div class="grid grid-cols-7 gap-2 text-center">
    <div v-for="n in blankDays" :key="'blank' + n"></div>
    <div
      v-for="date in daysInMonth"
      :key="date"
      class="size-8 flex items-center justify-center border rounded-full t-color-bg-hover cursor-pointer"
      :class="{
        'bg-sky-900': isTodayFn(date),
        'border-sky-500': isSelectedDate(date),
        'border-transparent': !isSelectedDate(date),
      }"
      @click="selectDate(date)"
    >
      {{ date }}
    </div>
  </div>

</template>

<script setup>
import { ref, inject, computed, watch } from 'vue';
import { format, getDaysInMonth, startOfMonth, getDay, isToday } from 'date-fns';

const props = defineProps({
  year: {
    type: Number,
    required: true,
  },
  month: {
    type: Number,
    required: true,
  },
  monthName: {
    type: String,
    required: true,
  },
});

const gCalendarMonth = inject('gCalendarMonth');
const gCalendarDate = inject('gCalendarDate');

// Computed property to get the number of days in the month
const daysInMonth = computed(() => getDaysInMonth(new Date(props.year, props.month - 1)));

// Blank days at the start of the calendar (for proper alignment)
const blankDays = computed(() => {
  const firstDayOfMonth = getDay(startOfMonth(new Date(props.year, props.month - 1)));
  return [...Array(firstDayOfMonth).keys()];
});

// Check if the given date is today
const isTodayFn = (date) => isToday(new Date(props.year, props.month - 1, date));

// Check if the date is selected
const isSelectedDate = (date) => gCalendarMonth.value === props.month && gCalendarDate.value === date;


// Method to select a date
const selectDate = (date) => {
  gCalendarMonth.value = props.month;
  gCalendarDate.value = date;

  console.log('Selected date:', format(new Date(props.year, props.month - 1, date), 'yyyy-MM-dd'));
};
</script>

