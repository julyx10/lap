<template>

  <div class="sidebar-panel overflow-hidden">
    <!-- on this day -->
    <div 
      :class="[ 
        'sidebar-item',
        libConfig.calendar.year === -1 ? 'sidebar-item-selected' : 'sidebar-item-hover',
      ]"
      @click="libConfig.calendar.year = -1"
    >
      <IconHistory class="mx-1 w-5 h-5 shrink-0" />
      <div class="sidebar-item-label">
        {{ $t('calendar.on_this_day') }}
      </div>
    </div>

    <template v-if="Object.keys(calendar_dates).length > 0" >
      <div class="sidebar-panel-header">
        <div role="tablist" class="sidebar-header-tabs">
          <a
            role="tab"
            :class="['sidebar-header-tab', { 'tab-active': config.calendar.isMonthly }]"
            @click="switchToMonthlyView()"
          >
            {{ localeMsg.menu.calendar.monthly }}
          </a>
          <a
            role="tab"
            :class="['sidebar-header-tab', { 'tab-active': !config.calendar.isMonthly }]"
            @click="switchToDailyView()"
          >
            {{ localeMsg.menu.calendar.daily }}
          </a>
        </div>
        <ContextMenu class="sidebar-panel-action" :menuItems="calendarMenuItems" :iconMenu="IconMore" :smallIcon="true" />
      </div>
      <!-- calendar -->
      <div ref="scrollable"
        class="flex-1 flex flex-col overflow-x-hidden overflow-y-auto"
      >
        <div v-for="item in sorted_calendar_items" 
          :key="item.year"
          :class="[
            'flex min-w-48',
            config.calendar.sortingAsc ? 'flex-col' : 'flex-col-reverse'
          ]"
        >
          <CalendarMonthly v-if="config.calendar.isMonthly"
            :year="Number(item.year)" 
            :months="item.months"
          />
          <CalendarDaily v-else v-for="(dates, month) in item.months" 
            :year="Number(item.year)" 
            :month="Number(month)"
            :dates="dates"
          />
        </div>
      </div>
    </template>

    <!-- Display message if no data are found -->
    <div v-else class="mt-8 px-2 flex flex-col items-center justify-center text-base-content/30">
      <IconCalendar class="w-8 h-8 mb-2" />
      <span class="text-sm text-center">{{ $t('tooltip.not_found.calendar') }}</span>
    </div>
  </div>
  
</template>

<script setup lang="ts">

import { ref, computed, onMounted, watch, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { config, libConfig } from '@/common/config';
import { getTakenDates } from '@/common/api';

import { IconCalendar, IconDot, IconHistory, IconMore } from '@/common/icons';
import CalendarMonthly from '@/components/CalendarMonthly.vue';
import CalendarDaily from '@/components/CalendarDaily.vue';
import ContextMenu from '@/components/ContextMenu.vue';

// props
const props = defineProps({
  titlebar: String
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

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
  if (config.calendar.sortingAsc) {
    years.sort((a, b) => a - b);
  } else {
    years.sort((a, b) => b - a);
  }
  
  return years.map(year => ({
    year: year,
    months: dates[year]
  }));
});

const calendarMenuItems = computed(() => [
  {
    label: localeMsg.value.menu.calendar.time_asc,
    icon: config.calendar.sortingAsc ? IconDot : null,
    action: () => { config.calendar.sortingAsc = true; },
  },
  {
    label: localeMsg.value.menu.calendar.time_desc,
    icon: config.calendar.sortingAsc ? null : IconDot,
    action: () => { config.calendar.sortingAsc = false; },
  },
]);

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

watch(() => [config.calendar.isMonthly, config.calendar.sortingAsc], () => {
  scrollToSelected();
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
        if (config.calendar.sortingAsc) {
          libConfig.calendar.month = Math.min(...months);
        } else {
          libConfig.calendar.month = Math.max(...months);
        }
      }
    }
  }
  config.calendar.isMonthly = false;
}

const toggleSortingOrder = () => {
  config.calendar.sortingAsc = !config.calendar.sortingAsc;
}

/// fetch calendar dates
async function getCalendarDates() {
  const taken_dates = await getTakenDates();
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
