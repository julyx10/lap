<template>

  <div class="flex flex-col items-center select-none mx-auto my-1 min-w-48 rounded-box border border-base-content/5">

    <!-- title -->
    <div 
      :class="[
        'mt-2 p-1 rounded-box text-nowrap cursor-pointer',
        isSelected(year, -1) ? 'text-primary bg-base-100 hover:bg-base-100 selected-item' : 'hover:text-base-content hover:bg-base-100/30'
      ]"
      @click="clickDate(year, -1)"
    >
      {{ yearTitle }}
    </div>

    <!-- month list -->
    <div class="p-2 grid grid-cols-6 gap-x-2 gap-y-2 text-center">
      <div v-for="m in 12" 
        :key="m" 
        class="size-8 p-1 text-xs flex items-center justify-center rounded-box"
        :class="{
          'bg-base-100 cursor-default': sumMonthCount(m) === 0,
          'text-base-content/70 hover:text-base-content cursor-pointer': sumMonthCount(m) > 0,
          'bg-base-content/20': sumMonthCount(m) > 0 && sumMonthCount(m) < 100,
          'bg-base-content/50': sumMonthCount(m) >= 100 && sumMonthCount(m) < 1000,
          'bg-base-content/80': sumMonthCount(m) >= 1000,
          'bg-primary/70 text-primary-content/70 hover:text-primary-content/70 selected-item': isSelected(year, m),
          'border border-base-content/20': isThisMonth(year, m),
        }"
        @click="sumMonthCount(m) > 0 ? clickDate(year, m) : null" 
      >
        {{ sumMonthCount(m) > 0 ? (sumMonthCount(m) < 10000 ? sumMonthCount(m) : '9k+') : '' }}
      </div>
    </div>

  </div>

</template>

<script setup>

import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
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

// Title for the year
const yearTitle = computed(() => formatDate(props.year, 1, 1, localeMsg.value.format.year));

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

// Check if the year or month is selected
const isSelected = (year, month) => config.calendar.year === year && config.calendar.month === month;

// click a year or a month to select it
const clickDate = (year, month) => {
  config.calendar.year = year;
  config.calendar.month = month; // -1 means selecting a year
  config.calendar.date = -1;   // -1 means selecting a month

  console.log('clickDate:', config.calendar.year, config.calendar.month, config.calendar.date);
};

</script>

