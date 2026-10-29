<template>

  <div class="sidebar-panel overflow-hidden">
    <div class="sidebar-panel-header">
      <div class="sidebar-header-tabs" role="tablist" :aria-label="props.titlebar || $t('calendar.title')">
        <button
          v-for="tab in calendarTabs"
          :key="tab.value"
          type="button"
          role="tab"
          class="sidebar-header-tab"
          :class="{ 'tab-active': calendarView === tab.value }"
          :aria-selected="calendarView === tab.value"
          @click="setCalendarView(tab.value)"
        >
          {{ tab.label }}
        </button>
      </div>
      <SortMenuButton v-model="config.settings.calendarSort" kind="calendar" />
    </div>

    <!-- calendar -->
    <div ref="scrollable" v-if="Object.keys(calendar_dates).length > 0"
      class="flex-1 flex flex-col overflow-x-hidden overflow-y-auto"
    >
      <template v-if="calendarView === 'years'">
        <ul>
          <li v-for="item in calendarYearsItems" :key="item.year">
            <div
              :class="[
                'sidebar-item',
                isYearSelected(item.year) ? 'sidebar-item-selected' : 'sidebar-item-hover',
              ]"
              @click="selectYear(item.year)"
            >
              <IconRight
                :class="['p-1 w-6 h-6 shrink-0 transition-transform', isYearExpanded(item.year) ? 'rotate-90' : '']"
                @click.stop="toggleYear(item.year)"
              />
              <span class="sidebar-item-label">{{ formatDate(item.year, 1, 1, 'year') }}</span>
              <span class="sidebar-item-count">{{ item.count.toLocaleString() }}</span>
            </div>
            <ul v-if="isYearExpanded(item.year)">
              <li v-for="month in item.months" :key="`${item.year}-${month.month}`" class="pl-4">
                <div
                  :class="[
                    'sidebar-item sidebar-item-compact ml-2',
                    isMonthSelected(item.year, month.month) ? 'sidebar-item-selected' : 'sidebar-item-hover',
                  ]"
                  @click="selectMonth(item.year, month.month)"
                >
                  <IconRight
                    :class="['p-1 w-6 h-6 shrink-0 transition-transform', isMonthExpanded(item.year, month.month) ? 'rotate-90' : '']"
                    @click.stop="toggleMonth(item.year, month.month)"
                  />
                  <span class="sidebar-item-label">{{ formatMonth(item.year, month.month) }}</span>
                  <span class="sidebar-item-count">{{ month.count.toLocaleString() }}</span>
                </div>
                <ul v-if="isMonthExpanded(item.year, month.month)">
                  <li v-for="day in month.days" :key="`${item.year}-${month.month}-${day.date}`" class="pl-8">
                    <div
                      :class="[
                        'sidebar-item sidebar-item-compact ml-2',
                        isDaySelected(item.year, month.month, day.date) ? 'sidebar-item-selected' : 'sidebar-item-hover',
                      ]"
                      @click="selectDay(item.year, month.month, day.date)"
                    >
                      <IconCalendarDay class="mx-1 w-4 h-4 shrink-0" />
                      <span class="sidebar-item-label">{{ formatDay(item.year, month.month, day.date) }}</span>
                      <span class="sidebar-item-count">{{ day.count.toLocaleString() }}</span>
                    </div>
                  </li>
                </ul>
              </li>
            </ul>
          </li>
        </ul>
      </template>
      <div
      v-else-if="calendarView === 'months'"
        class="grid grid-cols-[repeat(auto-fill,minmax(15rem,1fr))] items-start gap-2 px-1"
      >
        <CalendarMonthly
          v-for="item in sorted_calendar_items"
          :key="item.year"
          :year="Number(item.year)" 
          :months="item.months"
          :heatmap-thresholds="monthlyHeatmapThresholds"
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
          :heatmap-thresholds="dailyHeatmapThresholds"
        />
      </div>
    </div>

    <div v-else-if="!isLoading" class="mt-2 px-2 flex flex-col items-center justify-center text-base-content/30">
      <!-- <IconCalendar class="w-8 h-8 mb-2" /> -->
      <!-- <span class="text-sm text-center">{{ $t('tooltip.not_found.calendar') }}</span> -->
      <span class="text-sm text-center">{{ $t('tooltip.not_found.calendar_hint') }}</span>
    </div>
  </div>
  
</template>

<script setup lang="ts">

import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { fileInfoRevision } from '@/common/fileInfoRefresh';
import { config, libConfig } from '@/common/config';
import { getTakenDates } from '@/common/api';
import { formatDate } from '@/common/utils';
import { SIDEBAR } from '@/common/constants';
import { IconCalendarDay, IconRight } from '@/common/icons';

import CalendarMonthly from '@/components/CalendarMonthly.vue';
import CalendarDaily from '@/components/CalendarDaily.vue';
import SortMenuButton from '@/components/SortMenuButton.vue';

const props = defineProps({
  titlebar: String,
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
type CalendarView = 'years' | 'months' | 'days';
const calendarView = computed<CalendarView>(() => {
  const view = config.calendar.view;
  return view === 'years' || view === 'months' || view === 'days' ? view : 'years';
});
const calendarTabs = computed(() => [
  { value: 'years' as const, label: localeMsg.value.calendar.tabs?.years || 'Years' },
  { value: 'months' as const, label: localeMsg.value.calendar.tabs?.months || 'Months' },
  { value: 'days' as const, label: localeMsg.value.calendar.tabs?.days || 'Days' },
]);

const scrollable = ref<HTMLDivElement | null>(null); // Ref for the scrollable element
type CalendarDates = Record<number, Record<number, { date: number; count: number }[]>>;
const calendar_dates = ref<CalendarDates>({});
const isLoading = ref(true);
const expandedYears = ref<number[]>([]);
const expandedMonths = ref<string[]>([]);
let isCalendarMounted = true;
let calendarRequestVersion = 0;

onUnmounted(() => {
  isCalendarMounted = false;
  calendarRequestVersion++;
});

function buildHeatmapThresholds(values: number[]): number[] | null {
  const sorted = values.filter(value => value > 0).sort((a, b) => a - b);
  if (sorted.length < 4) return null;

  const percentile = (ratio: number) => sorted[Math.ceil(sorted.length * ratio) - 1];
  const thresholds = [percentile(0.5), percentile(0.75), percentile(0.95)];

  return thresholds[0] < thresholds[1] && thresholds[1] < thresholds[2]
    ? thresholds
    : null;
}

const dailyHeatmapThresholds = computed(() =>
  buildHeatmapThresholds(
    Object.values(calendar_dates.value).flatMap(months =>
      Object.values(months).flatMap(dates => dates.map(({ count }) => count))
    )
  )
);

const monthlyHeatmapThresholds = computed(() =>
  buildHeatmapThresholds(
    Object.values(calendar_dates.value).flatMap(months =>
      Object.values(months).map(dates =>
        dates.reduce((total, { count }) => total + count, 0)
      )
    )
  )
);

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

const calendarYearsItems = computed(() => {
  const ascending = config.settings.calendarSort % 2 === 0;
  return sorted_calendar_items.value.map(({ year, months }) => {
    const monthItems = Object.keys(months).map(Number)
      .sort((a, b) => ascending ? a - b : b - a)
      .map(month => {
        const days = [...months[month]].sort((a, b) => ascending ? a.date - b.date : b.date - a.date);
        return {
          month,
          days,
          count: days.reduce((total, day) => total + Number(day.count || 0), 0),
        };
      });
    return {
      year,
      months: monthItems,
      count: monthItems.reduce((total, month) => total + month.count, 0),
    };
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

watch(() => [config.calendar.view, config.settings.calendarSort], () => {
  if (calendarView.value === 'years') expandSelectedCalendarPath();
  scrollToSelected();
});

// Keep the current filter even if its last file moves out of this category.
watch(fileInfoRevision, async () => {
  if (libConfig.activePane === 'main' && config.main.sidebarIndex === SIDEBAR.CALENDAR) await getCalendarDates(true);
});

watch(() => config.settings.groupRawJpegPairs, async () => {
  if (libConfig.activePane === 'main' && config.main.sidebarIndex === SIDEBAR.CALENDAR) await getCalendarDates(true);
});

// Only refresh the active view. Inactive panel data is refreshed on re-entry.
watch(() => [config.settings.calendarSort], async () => {
  if (libConfig.activePane === 'main' && config.main.sidebarIndex === SIDEBAR.CALENDAR) await getCalendarDates();
});

watch(() => [config.main.sidebarIndex, libConfig.activePane], async () => {
  if (libConfig.activePane === 'main' && config.main.sidebarIndex === SIDEBAR.CALENDAR) await getCalendarDates();
});

watch(
  () => [libConfig.calendar.year, libConfig.calendar.month, libConfig.calendar.date],
  () => {
    if (calendarView.value === 'years') expandSelectedCalendarPath();
    scrollToSelected();
  },
);

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

function setCalendarView(view: CalendarView) {
  if (view === calendarView.value) return;

  if (view === 'months') {
    libConfig.calendar.date = -1;
  } else if (view === 'days' && libConfig.calendar.month === -1) {
    const year = libConfig.calendar.year;
    if (year !== null && year !== undefined && calendar_dates.value[year]) {
      const months = Object.keys(calendar_dates.value[year]).map(Number);
      if (months.length > 0) {
        libConfig.calendar.month = config.settings.calendarSort % 2 === 0
          ? Math.min(...months)
          : Math.max(...months);
      }
    }
  }

  config.calendar.view = view;
}

const monthKey = (year: number, month: number) => `${year}-${month}`;
const isYearExpanded = (year: number) => expandedYears.value.includes(year);
const isMonthExpanded = (year: number, month: number) => expandedMonths.value.includes(monthKey(year, month));
const isYearSelected = (year: number) => libConfig.calendar.year === year && libConfig.calendar.month === -1;
const isMonthSelected = (year: number, month: number) =>
  libConfig.calendar.year === year && libConfig.calendar.month === month && libConfig.calendar.date === -1;
const isDaySelected = (year: number, month: number, day: number) =>
  libConfig.calendar.year === year && libConfig.calendar.month === month && libConfig.calendar.date === day;

function toggleYear(year: number) {
  expandedYears.value = isYearExpanded(year)
    ? expandedYears.value.filter(value => value !== year)
    : [...expandedYears.value, year];
}

function toggleMonth(year: number, month: number) {
  const key = monthKey(year, month);
  expandedMonths.value = isMonthExpanded(year, month)
    ? expandedMonths.value.filter(value => value !== key)
    : [...expandedMonths.value, key];
}

function selectYear(year: number) {
  libConfig.calendar.year = year;
  libConfig.calendar.month = -1;
  libConfig.calendar.date = -1;
  if (!isYearExpanded(year)) toggleYear(year);
}

function selectMonth(year: number, month: number) {
  libConfig.calendar.year = year;
  libConfig.calendar.month = month;
  libConfig.calendar.date = -1;
  if (!isMonthExpanded(year, month)) toggleMonth(year, month);
}

function selectDay(year: number, month: number, day: number) {
  libConfig.calendar.year = year;
  libConfig.calendar.month = month;
  libConfig.calendar.date = day;
}

function formatMonth(_year: number, month: number) {
  return localeMsg.value.calendar.months_long?.[month - 1]
    || localeMsg.value.calendar.months?.[month - 1]
    || String(month);
}

function formatDay(_year: number, _month: number, day: number) {
  return String(day);
}

function expandSelectedCalendarPath() {
  const year = Number(libConfig.calendar.year);
  const month = Number(libConfig.calendar.month);
  if (!Number.isFinite(year) || !calendar_dates.value[year]) return;

  if (!isYearExpanded(year)) expandedYears.value = [...expandedYears.value, year];
  if (month > 0 && calendar_dates.value[year][month] && !isMonthExpanded(year, month)) {
    expandedMonths.value = [...expandedMonths.value, monthKey(year, month)];
  }
}

/// fetch calendar dates
async function getCalendarDates(preserveFilter = false) {
  const requestVersion = ++calendarRequestVersion;
  const libraryId = libConfig._libraryId;
  isLoading.value = true;
  try {
    const taken_dates = await getTakenDates(config.settings.calendarSort);
    if (!isCalendarMounted || requestVersion !== calendarRequestVersion || libraryId !== libConfig._libraryId) return;
    if (taken_dates) {
      calendar_dates.value = transformArray(taken_dates);
      if (!preserveFilter) validateCalendarSelection();
      if (calendarView.value === 'years') expandSelectedCalendarPath();
    }
  } finally {
    if (isCalendarMounted && requestVersion === calendarRequestVersion) {
      isLoading.value = false;
    }
  }
}

function validateCalendarSelection() {
  const year = libConfig.calendar.year;
  const month = libConfig.calendar.month;
  const date = libConfig.calendar.date;

  if (year === null || year === undefined) return;
  const months = calendar_dates.value[year];
  const isSelectedDateVisible = months
    && (month === -1 || (months[month!] && (date === -1 || months[month!].some(item => item.date === date))));

  if (!isSelectedDateVisible) {
    libConfig.calendar.year = null;
    libConfig.calendar.month = null;
    libConfig.calendar.date = null;
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

</script>
