<template>

  <div class="flex-1 flex flex-col overflow-hidden" style="user-select: none;">

    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between" data-tauri-drag-region>
      <span class="cursor-default" data-tauri-drag-region>{{ titlebar }}</span>
      <div class="flex text-sm items-center cursor-pointer">
        <div 
          :class="[
            'px-2 border rounded-l-lg t-color-bg t-color-border t-color-bg-hover text-nowrap',
            config.calendarIsMonthly ? 't-color-text-focus t-color-bg-selected' : ''
          ]"
         @click="config.calendarIsMonthly=true"
        >
          {{ $t('calendar_monthly') }}
        </div>
        <div 
          :class="[
            'px-2 border rounded-r-lg t-color-bg t-color-border t-color-bg-hover text-nowrap',
            !config.calendarIsMonthly ? 't-color-text-focus t-color-bg-selected' : ''
          ]"
         @click="config.calendarIsMonthly=false"
        >
          {{ $t('calendar_daily') }}
        </div>
        <span class="px-2"></span>
        <component :is="config.calendarSortingAsc ? IconSortingAsc : IconSortingDesc" class="t-icon-size-sm t-icon-hover" @click="toggleSortingOrder" />
      </div>
    </div>
    
    <template v-if="Object.keys(calendar_dates).length > 0" >
      
        <!-- days of the week in daily calendar -->
        <div v-if="!config.calendarIsMonthly" class="flex flex-col items-center mr-4">
          <div class="grid grid-cols-7 gap-2 text-center">
            <div 
              v-for="(day, index) in localeMsg.calendar_weekdays" 
              :key="index" 
              class="p-2 w-8 flex items-center justify-center"
            >
              {{ day }}
            </div>
          </div>
        </div>  

        <!-- calendar -->
        <div ref="scrollable"
          :class="['flex overflow-auto t-scrollbar-dark',
            config.calendarSortingAsc ? 'flex-col' : 'flex-col-reverse'
          ]"
        >
          <div v-for="(months, year) in calendar_dates" 
            :class="['flex',
              config.calendarSortingAsc ? 'flex-col' : 'flex-col-reverse'
            ]"
          >
            <CalendarMonthly v-if="config.calendarIsMonthly"
              :year="Number(year)" 
              :months="months"
            />
            <CalendarDaily v-else v-for="(dates, month) in months" 
              :year="Number(year)" 
              :month="Number(month)"
              :dates="dates"
            />
          </div>
        </div>

    </template>

    <!-- Display message if no data are found -->
    <template v-else>
      <div class="mt-10 flex items-center justify-center">
        {{ $t('no_calendar_data') }}
      </div>
    </template>

  </div>
  
</template>

<script setup>

import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { useConfigStore } from '@/stores/configStore';
import { getTakenDates } from '@/common/api';

import CalendarMonthly from '@/components/CalendarMonthly.vue';
import CalendarDaily from '@/components/CalendarDaily.vue';

import IconSortingAsc from '@/assets/sorting-asc.svg';
import IconSortingDesc from '@/assets/sorting-desc.svg';

// props
const props = defineProps({
  titlebar: String
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

// config store
const config = useConfigStore();

const scrollable = ref(null); // Ref for the scrollable element
const calendar_dates = ref([]);

onMounted( () => {
  console.log('Calendar.vue mounted');
  getCalendarDates();
});

const toggleSortingOrder = () => {
  config.calendarSortingAsc = !config.calendarSortingAsc;

  // const element = scrollable.value; // Get the scrollable element
  // element.scrollTop = sortingAsc.value === true ? 0 : element.scrollHeight;
  // console.log('toggleSortingOrder:', sortingAsc.value, element);
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
function transformArray(dates) {
  const result = {};

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
