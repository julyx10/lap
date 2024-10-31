<template>

  <div class="flex flex-col items-center">

    <!-- title -->
    <span class="p-4 t-color-text-dark">
      {{ yearTitle }}
    </span>

    <!-- month list -->
    <div class="gap-4 grid grid-cols-4">
      <div v-for="m in 12" 
        :key="m" 
        class="px-2 flex items-center justify-center border rounded-full t-color-bg-hover cursor-pointer"
        :class="{
          'bg-sky-900': isThisMonth(year, m),
          'border-sky-500': isSelectedMonth(year, m),
          'border-transparent': !isSelectedMonth(year, m),
          't-color-text-dark': sumMonthCount(m) === 0
        }"
        @click="clickMonth(year, m)" 
      >
        {{ localeMsg.calendar_months[m - 1] }}
      </div>
    </div>

  </div>

</template>


<script setup>

import { inject, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { formatDate } from '@/common/utils';

const props = defineProps({
  year: {
    type: Number,
    required: true,
  },
  months: {
    type: Object,
    required: true,
  }
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

const gCalendarYear = inject('gCalendarYear');
const gCalendarMonth = inject('gCalendarMonth');
const gCalendarDate = inject('gCalendarDate');

// Title for the year
const yearTitle = computed(() => formatDate(props.year, 1, 1, localeMsg.value.year_format));

// Sum the count values for the given month
function sumMonthCount(month) {
  let sum = 0;
  if (props.months[month]) {
    props.months[month].forEach(entry => {
      sum += Number(entry.count) || 0; // Sum the count values, defaulting to 0 if missing
    });
  }
  return sum;
}

// Check if the given month is this month
function isThisMonth(year, month) {
  const now = new Date();
  // Check if the given year and month match the current year and month
  return year === now.getFullYear() && (month - 1) === now.getMonth();
}

// Check if the month is selected
const isSelectedMonth = (year, month) => gCalendarYear.value === year && gCalendarMonth.value === month;

// click a month to select it
const clickMonth = (year, month) => {
  gCalendarYear.value = year;
  gCalendarMonth.value = month;
  gCalendarDate.value = -1;   // -1 means selecting a month

  console.log('clickDate:', gCalendarYear.value, gCalendarMonth.value, gCalendarDate.value);
};

</script>

