<template>

  <div class="fixed inset-0 flex items-center justify-center bg-gray-900 bg-opacity-60 z-50 backdrop-blur-0">
    <div class="w-96 p-4 t-color-bg-light border t-color-border rounded-lg shadow-lg">

      <!-- title -->
      <div class="mb-2 flex items-center justify-between">
        {{ $t('album_info_title') }}
        <IconClose class="t-icon-size-sm t-icon-hover" @click="clickCancel" />
      </div>

      <!-- two colums table -->
      <table class="mt-4 w-full text-sm text-nowrap">
        <tbody>
          <tr>
            <td>{{ $t('album_info_name') }}</td>
            <td>
              <input
                ref="inputNameRef"
                v-model="inputNameValue"
                type="text"
                maxlength="255"
                class="px-2 py-1 w-full border rounded-md t-input-color-bg t-color-border t-input-focus"
                @input="validateInput"
              />
            </td>
          </tr>
          <tr>
            <td>{{ $t('album_info_description') }}</td>
            <td>
              <textarea
                v-model="inputDescriptionValue"
                rows="2"
                maxlength="1024"
                :placeholder="$t('album_info_description_placeholder')"
                class="px-2 py-1 mt-2 w-full border rounded-md t-input-color-bg t-color-border t-input-focus t-scrollbar-dark resize-y min-h-[30px] max-h-[200px]"
              ></textarea>
            </td>
          </tr>
          <tr class="h-8">
            <td>{{ $t('album_info_folder') }}</td>
            <td>
              <input
                type="text"
                readonly
                :value="albumPath"
                class="py-1 w-full t-input-color-bg border-none focus:border-none focus:ring-0 focus:outline-none"
              />
            </td>
          </tr>
          <tr class="h-8">
            <td>{{ $t('album_info_images') }}</td>
            <td>{{ totalImageCount >= 0 ? $t('album_info_files_count', {count: totalImageCount.toLocaleString(), size: formatFileSize(totalImageSize) }) : $t('album_info_files_counting') }}</td>
          </tr>
          <tr class="h-8">
            <td>{{ $t('album_info_videos') }}</td>
            <td>{{ totalVideoCount >= 0 ? $t('album_info_files_count', {count: totalVideoCount.toLocaleString(), size: formatFileSize(totalVideoSize) }) : $t('album_info_files_counting') }}</td>
          </tr>
          <tr class="h-8">
            <td>{{ $t('album_info_created_time') }}</td>
            <td>{{ createdAt }}</td>
          </tr>
          <!-- <tr class="h-8">
            <td>{{ $t('album_info_modified_time') }}</td>
            <td>{{ modifiedAt }}</td>
          </tr> -->
        </tbody>
      </table>

      <!-- buttons -->
      <div class="mt-2 flex justify-end space-x-4">
        <button 
          class="px-4 py-1 rounded-full t-color-bg-light t-color-bg-hover t-icon-hover" 
          @click="clickCancel"
        >
          {{ $t('msgbox_cancel')  }}
        </button>
        <button 
          :class="['px-4 py-1 rounded-full t-color-bg-light', 
            okButtonClasses
          ]" 
          @click="clickOk"
        >
          {{ $t('msgbox_ok')  }}
        </button>
      </div>

    </div>
  </div>

</template>

<script setup lang="ts">

import { ref, computed, onMounted, onUnmounted } from 'vue';
// import { useI18n } from 'vue-i18n';
import { countFolder } from '@/common/api';
import { formatFileSize } from '@/common/utils';

import { IconClose } from '@/common/icons';

const props = defineProps({
  inputName: { 
    type: String, 
    default: '' 
  },
  inputDescription: { 
    type: String, 
    default: '' 
  },
  albumPath: { 
    type: String, 
    default: '' 
  },
  createdAt: { 
    type: String, 
    default: '' 
  },
  modifiedAt: { 
    type: String, 
    default: '' 
  },
});

/// i18n
// const { locale, messages } = useI18n();
// const localeMsg = computed(() => messages.value[locale.value]);

const emit = defineEmits(['ok', 'cancel']);

// input 
const inputNameRef = ref(null);
const inputNameValue = ref(props.inputName);
const inputDescriptionValue = ref(props.inputDescription);

// total file count of the album
const totalFolderCount = ref(0);
const totalImageCount = ref(-1);
const totalImageSize = ref(-1);
const totalVideoCount = ref(0);
const totalVideoSize = ref(0);

const okButtonClasses = computed(() => {
  return inputNameValue.value.trim().length > 0
    ? 't-color-bg-highlight-hover t-icon-hover' : 't-icon-disabled';
});

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
  inputNameRef.value?.focus();

  // count folder
  countFolder(props.albumPath).then((res) => {
    [totalFolderCount.value, totalImageCount.value, totalImageSize.value, totalVideoCount.value, totalVideoSize.value] = res;
    console.log('count folder:', res);
  }).catch((err) => {
    console.error('count folder error:', err);
  });
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

function handleKeyDown(event) {
  event.stopPropagation();

  switch (event.key) {
    // case 'Enter':
    //   clickOk();
    //   break;
    case 'Escape':
      clickCancel();
      break;
    default:
      break;
  }
}

const clickOk = () => {
  if (inputNameValue.value.trim().length > 0) {
    emit('ok', inputNameValue.value, inputDescriptionValue.value);
  }
};

const clickCancel = () => {
  emit('cancel');
};

</script>

<style scoped>
</style>
