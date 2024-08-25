<template>

<div class="flex items-center justify-between">
  {{ title }}
  <div class="flex">
    <IconStar class="p-1 hover:text-gray-200 transition-colors duration-300" @click="" />
    <IconTag  class="p-1 hover:text-gray-200 transition-colors duration-300" @click="" />
  </div>
</div>

</template>
  
<script setup>
import { ref, computed, inject  } from 'vue';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const msg = computed(() => messages.value[locale.value]);

// Import the SVG file as a Vue component
import IconStar from '@/assets/star.svg';
import IconTag from '@/assets/tag.svg';

const props = defineProps({
  titlebar: String
});

const g_toolbar_index = inject('g_toolbar_index'); // global toolbar index
const g_albums = inject('g_albums');           // global albums
const g_album_index = inject('g_album_index'); // global album index
const g_child_id = inject('g_child_id');       // global folder id


/// Display the titlebar
const title = computed(() => {
  // console.log('title:', msg.value);

  // album view
  if (g_toolbar_index.value === 1) {
    if (g_album_index.value >= 0) {
      if(g_child_id.value < 0) {
        return g_albums.value[g_album_index.value].name + ' > ' + msg.value.allphotos;
      } else {
        // get the folder name
        let subfolder = getChildren(g_albums.value[g_album_index.value], g_child_id.value);
        console.log('subfolder:', subfolder);
        return g_albums.value[g_album_index.value].name + ' > ' + subfolder.name;
      }
    } else {
      return props.titlebar;
    }
  }
});


/// get the selected sub-folder of the album
function getChildren(album, child_id) {
  if (album.id === child_id) {
    return album;
  } else if (album.children) {
    for (let child of album.children) {
        const result = getChildren(child, child_id);
        if (result) {
          return result;
        }
    }
  }
  return null;
}

</script>
  