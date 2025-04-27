<template>

  <div class="fixed inset-0 flex items-center justify-center bg-gray-900 bg-opacity-60 z-50 backdrop-blur-0">
    <div class="w-96 p-4 t-color-bg-light rounded-lg shadow-lg">

      <!-- title -->
      <div class="mb-2 flex items-center justify-between">
        {{ $t('album_profile_title') }}
        <IconClose class="t-icon-size-sm t-icon-hover" @click="clickCancel" />
      </div>

      <!-- name -->
      <!-- <div class="mt-4">
        {{ $t('album_profile_name') }}
        <input
          ref="inputNameRef"
          v-model="inputNameValue"
          type="text"
          maxlength="255"
          class="px-2 py-1 mt-2 w-full border rounded-md t-input-color-bg t-color-border t-input-focus"
          @input="validateInput"
        />
        <p class="h-4 text-red-600 text-xs">{{ inputErrorMessage }}</p>
      </div> -->

      <!-- description -->
      <!-- <div>
        {{ $t('album_profile_description') }}
        <textarea
          v-model="inputDescriptionValue"
          rows="3"
          maxlength="1024"
          :placeholder="$t('album_profile_description_placeholder')"
          class="px-2 py-1 my-2 w-full border rounded-md t-input-color-bg t-color-border t-input-focus resize-y min-h-[80px] max-h-[200px]"
        ></textarea>
      </div> -->

      <!-- two colums table -->
      <div class="mt-4">
        <table class="w-full text-sm text-nowrap">
          <tbody>
            <tr>
              <td>{{ $t('album_profile_name') }}</td>
              <td>
                <input
                  ref="inputNameRef"
                  v-model="inputNameValue"
                  type="text"
                  maxlength="255"
                  class="px-2 py-1 w-full border rounded-md t-input-color-bg t-color-border t-input-focus"
                  @input="validateInput"
                />
                <!-- <p class="h-4 text-red-600 text-xs">{{ inputErrorMessage }}</p> -->
              </td>
            </tr>
            <tr>
              <td>{{ $t('album_profile_description') }}</td>
              <td>
                <textarea
                  v-model="inputDescriptionValue"
                  rows="2"
                  maxlength="1024"
                  :placeholder="$t('album_profile_description_placeholder')"
                  class="px-2 py-1 mt-2 w-full border rounded-md t-input-color-bg t-color-border t-input-focus t-scrollbar-dark resize-y min-h-[30px] max-h-[200px]"
                ></textarea>
              </td>
            </tr>
            <tr class="h-8">
              <td>{{ $t('album_profile_folder') }}</td>
              <td>{{ albumPath }}</td>
            </tr>
            <tr class="h-8">
              <td>{{ $t('album_profile_file') }}</td>
              <td>{{ $t('album_profile_file_info') }}</td>
            </tr>
            <tr class="h-8">
              <td>{{ $t('album_profile_created_time') }}</td>
              <td>{{ createdAt }}</td>
            </tr>
            <tr class="h-8">
              <td>{{ $t('album_profile_modified_time') }}</td>
              <td>{{ modifiedAt }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- buttons -->
      <div class="flex justify-end space-x-4">
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
const inputErrorMessage = ref('');

const okButtonClasses = computed(() => {
  return inputNameValue.value.trim().length > 0
    ? 't-color-bg-highlight-hover t-icon-hover' : 't-icon-disabled';
});

onMounted(() => {
  // window.addEventListener('keydown', handleKeyDown);
  inputNameRef.value?.focus();
});

onUnmounted(() => {
  // window.removeEventListener('keydown', handleKeyDown);
});

// watch(() => props.errorMessage, (newValue) => {
//   if (newValue.length > 0) {
//     inputErrorMessage.value = newValue;
//     emit('reset'); // reset error message
//   }
// });

// const validateInput = () => {
//   if (!isValidFileName(inputNameValue.value)) {
//     inputErrorMessage.value = localeMsg.value.msgbox_input_filename_error;
//   } else {
//     inputErrorMessage.value = '';
//   }
// };

function handleKeyDown(event) {
  // event.stopPropagation();

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
