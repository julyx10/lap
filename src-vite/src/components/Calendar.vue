<template>

  <div class="sidebar-panel overflow-hidden">
    <div class="sidebar-panel-header">
      <span class="sidebar-panel-header-title flex-1">{{ calendarTitle }}</span>
      <label
        class="swap swap-flip inline-grid w-6 h-6 place-items-center text-base-content/70 hover:text-base-content"
        :title="config.settings.showToolTip ? calendarToggleTooltip : undefined"
        :aria-label="calendarToggleTooltip"
      >
        <input
          type="checkbox"
          :checked="!config.calendar.isMonthly"
          @change="toggleCalendarView"
        />
        <IconCalendarMonth class="swap-off col-start-1 row-start-1 self-center justify-self-center w-4 h-4" />
        <IconCalendarDay class="swap-on col-start-1 row-start-1 self-center justify-self-center w-4 h-4" />
      </label>
    </div>

    <!-- calendar -->
    <div ref="scrollable" v-if="Object.keys(calendar_dates).length > 0"
      class="flex-1 flex flex-col overflow-x-hidden overflow-y-auto"
    >
      <div
        v-if="config.calendar.isMonthly"
        class="grid grid-cols-[repeat(auto-fill,minmax(15rem,1fr))] items-start gap-2 px-1"
      >
        <CalendarMonthly
          v-for="item in sorted_calendar_items"
          :key="item.year"
          :year="Number(item.year)" 
          :months="item.months"
        />
      </div>
      <div
        v-else
        class="grid grid-cols-[repeat(auto-fill,minmax(15rem,1fr))] items-start gap-2 px-1"
      >
        <CalendarDaily
          v-for="item in sorted_daily_items"
          :key="`${item.year}-${item.month}`"
          :year="item.year"
          :month="item.month"
          :dates="item.dates"
        />
      </div>
    </div>

    <div v-else class="mt-2 px-2 flex flex-col items-center justify-center text-base-content/30">
      <!-- <IconCalendar class="w-8 h-8 mb-2" /> -->
      <!-- <span class="text-sm text-center">{{ $t('tooltip.not_found.calendar') }}</span> -->
      <span class="text-sm text-center">{{ $t('tooltip.not_found.calendar_hint') }}</span>
    </div>
  </div>
  
</template>

<script setup lang="ts">

import { ref, computed, onMounted, watch, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { config, libConfig } from '@/common/config';
import { getTakenDates } from '@/common/api';
import { IconCalendarDay, IconCalendarMonth } from '@/common/icons';

import CalendarMonthly from '@/components/CalendarMonthly.vue';
import CalendarDaily from '@/components/CalendarDaily.vue';

// props
const props = defineProps({
  titlebar: String
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const calendarTitle = computed(() =>
  config.calendar.isMonthly
    ? (localeMsg.value.calendar.month_title || localeMsg.value.calendar.month || 'Month')
    : (localeMsg.value.calendar.day_title || localeMsg.value.calendar.day || 'Day')
);
const calendarToggleTooltip = computed(() =>
  config.calendar.isMonthly
    ? (localeMsg.value.calendar.switch_to_day || 'Switch to Day')
    : (localeMsg.value.calendar.switch_to_month || 'Switch to Month')
);

const scrollable = ref<HTMLDivElement | null>(null); // Ref for the scrollable element
type CalendarDates = Record<number, Record<number, { date: number; count: number }[]>>;
const calendar_dates = ref<CalendarDates>({});

const sorted_calendar_items = computed(() => {
  const dates = calendar_dates.value;
  // If array (initial state) or no keys, return empty
  if (!dates || (Array.isArray(dates) && dates.length === 0)) return [];
  
  // keys are years
  const years = Object.keys(dates).map(Number);
  
  // Sort years based on config
  if (config.settings.calendarSort % 2 === 0) {
    years.sort((a, b) => a - b);
  } else {
    years.sort((a, b) => b - a);
  }
  
  return years.map(year => ({
    year: year,
    months: dates[year]
  }));
});

const sorted_daily_items = computed(() => {
  const ascending = config.settings.calendarSort % 2 === 0;
  return sorted_calendar_items.value.flatMap(item => {
    const months = Object.keys(item.months).map(Number);
    months.sort((a, b) => ascending ? a - b : b - a);
    return months.map(month => ({
      year: Number(item.year),
      month,
      dates: item.months[month],
    }));
  });
});

onMounted(async () => {
  console.log('Calendar.vue mounted');
  await getCalendarDates();
  
  // Scroll to selected date after data is loaded and DOM is updated
  scrollToSelected();

  // if (calendar_dates.value.length === 0) {
  //   libConfig.calendar.date = null;
  //   libConfig.calendar.month = null;
  //   libConfig.calendar.year = null;
  // }
});

watch(() => [config.calendar.isMonthly, config.settings.calendarSort], () => {
  scrollToSelected();
});

watch(() => config.settings.calendarSort, async () => {
  await getCalendarDates();
});

function scrollToSelected() {
  nextTick(() => {
    if (scrollable.value) {
      const selectedElement = scrollable.value.querySelector('.selected-item') || scrollable.value.querySelector('.text-primary');
      if (selectedElement) {
        selectedElement.scrollIntoView({
          behavior: 'auto', // 'smooth' is not good when switching view
          block: 'center'
        });
      }
    }
  });
}

function switchToMonthlyView() {
  libConfig.calendar.date = -1;  // -1 means selecting a month
  config.calendar.isMonthly = true;
}

function switchToDailyView() {
  // if a year is selected in month view
  if (config.calendar.isMonthly && libConfig.calendar.month === -1) {
    const year = libConfig.calendar.year;
    if (year !== null && year !== undefined && calendar_dates.value[year]) {
      const months = Object.keys(calendar_dates.value[year]).map(Number);
      if (months.length > 0) {
        if (config.settings.calendarSort % 2 === 0) {
          libConfig.calendar.month = Math.min(...months);
        } else {
          libConfig.calendar.month = Math.max(...months);
        }
      }
    }
  }
  config.calendar.isMonthly = false;
}

function toggleCalendarView() {
  if (config.calendar.isMonthly) {
    switchToDailyView();
  } else {
    switchToMonthlyView();
  }
}

/// fetch calendar dates
async function getCalendarDates() {
  const taken_dates = await getTakenDates(config.settings.calendarSort);
  if(taken_dates) {
    calendar_dates.value = transformArray(taken_dates);
  }
}

/// input: [['2024-10-15', 5], ['2023-01-01', 10]];
/// output: [{2024: {10: {15: 5}}, {2023: {01: {01: 10}}}]
function transformArray(dates: [string, number][]): CalendarDates {
  const result: CalendarDates = {};

  dates.forEach(item => {
    const [dateFormat, count] = item;  // dateForamat: 'yyyy-mm-dd'
    const [yearStr, monthStr, dateStr] = dateFormat.split('-');
    const year  = Number(yearStr);
    const month = Number(monthStr);
    const date  = Number(dateStr);

    if(year > 0 && month > 0 && date > 0) {
      // Initialize the year object if it doesn't exist
      if (!result[year]) {
        result[year] = {};
      }
      // Initialize the month object if it doesn't exist
      if (!result[year][month]) {
        result[year][month] = [];
      }
      // Push the date and count as an object into the month array
      result[year][month].push({ date, count });
    }
  });

  return result;
}

defineExpose({
  switchToMonthlyView,
  switchToDailyView,
});

</script>
