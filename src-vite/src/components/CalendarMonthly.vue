<template>

  <div class="flex flex-col items-center select-none">

    <!-- title -->
    <div 
      :class="[
        'mt-2 px-2 border-2 rounded-full hover:bg-base-content/10 text-nowrap cursor-pointer',
        isSelected(year, -1) ? 'border-primary' : 'border-transparent'
      ]"
      @click="clickDate(year, -1)"
    >
      {{ yearTitle }}
    </div>

    <!-- month list -->
    <div class="py-2 px-1 gap-4 grid grid-cols-4">
      <div v-for="m in 12" 
        :key="m" 
        class="px-2 text-sm flex items-center justify-center border-2 rounded-full text-nowrap"
        :class="[
          isSelected(year, m) ? 'border-primary' : 'border-transparent',
          sumMonthCount(m) === 0 ? 'text-base-content/30 cursor-default' : 'hover:bg-base-content/10 cursor-pointer',
          isThisMonth(year, m) ? 'bg-base-100' : '',
        ]"
        @click="sumMonthCount(m) > 0 ? clickDate(year, m) : null" 
      >
        {{ localeMsg.calendar.months[m - 1] }}
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

