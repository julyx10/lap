<template>

  <div class="flex-1 flex flex-col overflow-hidden" style="user-select: none;">

    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between whitespace-nowrap" data-tauri-drag-region>
      <span class="cursor-default" data-tauri-drag-region>{{ titlebar }}</span>
      <div class="flex text-sm items-center cursor-pointer">
        <div role="tablist" class="tabs-sm tabs-border" >
          <a 
            role="tab"
            class="tab"
            :class="config.calendar.isMonthly ? 'tab-active' : ''" 
            @click="config.calendar.isMonthly=true"
          >
            {{ $t('calendar.month') }}
          </a>
          <a 
            role="tab"
            class="tab"
            :class="!config.calendar.isMonthly ? 'tab-active' : ''" 
            @click="config.calendar.isMonthly=false"
          >
            {{ $t('calendar.day') }}
          </a>
        </div>

        <TButton 
          :icon="config.calendar.sortingAsc ? IconSortingAsc : IconSortingDesc" 
          @click="toggleSortingOrder"
        />
      </div>
    </div>
    
    <template v-if="Object.keys(calendar_dates).length > 0" >
      
        <!-- days of the week in daily calendar -->
        <div v-if="!config.calendar.isMonthly" class="text-sm text-base-content/30 flex flex-col items-center mr-4">
          <div class="grid grid-cols-7 gap-2 text-center">
            <div 
              v-for="(day, index) in localeMsg.calendar.weekdays" 
              :key="index" 
              class="p-2 w-8 flex items-center justify-center"
            >
              {{ day }}
            </div>
          </div>
        </div>  

        <!-- calendar -->
        <div ref="scrollable"
          :class="[
            'flex overflow-x-hidden overflow-y-auto',
            config.calendar.sortingAsc ? 'flex-col' : 'flex-col-reverse'
          ]"
        >
          <div v-for="(months, year) in calendar_dates" 
            :class="[
              'flex',
              config.calendar.sortingAsc ? 'flex-col' : 'flex-col-reverse'
            ]"
          >
            <CalendarMonthly v-if="config.calendar.isMonthly"
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
    <div v-else class="mt-10 flex flex-col items-center justify-center text-base-content/30">
      <IconCalendar class="w-8 h-8" />
      <span class="mt-2">{{ $t('tooltip.not_found.calendar') }}</span>
    </div>
  </div>
  
</template>

<script setup lang="ts">

import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { getTakenDates } from '@/common/api';
import { IconCalendar, IconSortingAsc, IconSortingDesc } from '@/common/icons';

import TButton from '@/components/TButton.vue';
import CalendarMonthly from '@/components/CalendarMonthly.vue';
import CalendarDaily from '@/components/CalendarDaily.vue';

// props
const props = defineProps({
  titlebar: String
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

const scrollable = ref(null); // Ref for the scrollable element
const calendar_dates = ref([]);

onMounted( () => {
  console.log('Calendar.vue mounted');
  getCalendarDates();
});

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
