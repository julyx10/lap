<template>
    
  <div class="flex-1 flex flex-col overflow-auto" style="user-select: none;">

    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between">
      <span>{{ titlebar }}</span>

      <div class="flex">
        <IconAdd class="p-0.5 t-icon-hover" @click="clickAdd" />
        <IconDelete  
          :class="[
            'p-0.5', 
            gAlbumId ? 't-icon-hover' : 't-icon-disabled'
          ]" 
          @click="gAlbumId ? showDeleteAlbumMsgbox = true : ''" />
        <!-- <IconRefresh class="p-1 hover:text-gray-200 transition-colors duration-300" @click="clickRefresh"/> -->
      </div>
    </div>

    <!-- albums -->
    <div v-if="gAlbums.length > 0" class="flex-grow overflow-auto t-scrollbar-dark">
      <ul>
        <li v-for="album in gAlbums" :key="album.id">
          <div 
            :class="[
              'p-2 flex items-center whitespace-nowrap t-color-bg-hover cursor-pointer', 
              { 
                't-color-text-selected': gAlbumId === album.id, 
                't-color-bg-selected': gAlbumId === album.id && gFolderId === album.folderId
              }
            ]"
            @click="clickAlbum(album)"
          >
            <component :is="album.is_expanded ? IconFolderOpen : IconFolder" class="size-6 pr-1 flex-shrink-0" @click.stop="clickExpandAlbum(album)"/>
            {{ album.name }}
            <!-- {{ album.name }}({{ album.id }}) - {{ album.folderId }} -->
          </div>
          <Folders v-if="album.is_expanded" :albumId="album.id" :children="album.children" />
        </li>
      </ul>
    </div>

    <!-- Display message if no albums are found -->
    <div v-else class="mt-10 flex items-center justify-center">
      {{ $t('no_albums') }}
    </div>

    <MessageBox
      v-if="showDeleteAlbumMsgbox"
      :visible="showDeleteAlbumMsgbox"
      :title="$t('delete_album_msgbox_title')"
      :message="`${$t('delete_album_msgbox_content', { album: getAlbumById(gAlbumId).name })}`"
      :confirmText="$t('delete_album_msgbox_ok')"
      :cancelText="$t('delete_album_msgbox_cancel')"
      @confirm="clickDeleteConfirm"
      @cancel="showDeleteAlbumMsgbox = false"
      @close="showDeleteAlbumMsgbox = false"
    />
  </div>

</template>


<script setup lang="ts">

import { ref, inject, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api';
import { appWindow } from '@tauri-apps/api/window';
import Folders from '@/components/AlbumsFolders.vue';
import MessageBox from '@/components/MessageBox.vue';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

// toolbar icons
import IconAdd from '@/assets/folder-plus.svg';
import IconDelete from '@/assets/folder-minus.svg';
// import IconRefresh from '@/assets/arrow-path.svg';

// folder icon
import IconFolder from '@/assets/folder.svg';
import IconFolderOpen from '@/assets/folder-open.svg';

const gAlbums = inject('gAlbums');       // global albums
const gAlbumId = inject('gAlbumId');     // global album id
const gFolderId = inject('gFolderId');   // global folder id

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const showDeleteAlbumMsgbox = ref(false);

// Fetch albums on mount
onMounted(() => {
  if (gAlbums.value.length === 0) {
    getAlbums();
  }
});

const getAlbumById = (id) => gAlbums.value.find(album => album.id === id);

/// Add albums
const clickAdd = async () => {
  try {
    const new_album = await invoke('add_album', { window: appWindow, title: localeMsg.value.add_album_title });
    gAlbums.value.push(new_album);

    console.log('Add album...', new_album);
  } catch (error) {
    console.log('Failed to add album:', error);
  }
};


/// Delete an album
const clickDeleteConfirm = async () => {
  try {
    if (gAlbumId.value) {
      const result = await invoke('delete_album', { id: getAlbumById(gAlbumId.value).id });

      // delete the album from the list
      gAlbums.value = gAlbums.value.filter(album => album.id !== gAlbumId.value);
      gAlbumId.value = null;

      console.log('Delete album...', result);
    } else {
      console.log('No album selected', gAlbumId.value);
    }
  } catch (error) {
    console.error('Failed to delete album:', error);
  }
};


/// Refresh albums
// const clickRefresh = async () => {
//   await getAlbums(); // Refresh albums

//   gAlbumId.value = null;
//   gFolderId.value = null;

//   console.log('Refresh albums');
// };


/// click a album to select it
const clickAlbum = async (album) => {
  console.log('clickAlbum...', album);
  
  try {
    const result = await invoke('select_folder', {
      albumId: album.id, 
      parentId: 0,
      name: album.name,
      path: album.path
    });
    
    // insert a new property(album.folderId) 
    album.folderId = result.id;

    gFolderId.value = album.folderId;
    gAlbumId.value = album.id;
    
    console.log('add_folder result:', result);
  } catch (error) {
    console.error("Error adding folder:", error);
  }
};


/// click album icon to expand or collapse next level folders
const clickExpandAlbum = async (album) => {
  console.log('clickExpandAlbum...', album);

  // Toggle album expansion
  album.is_expanded = !album.is_expanded; 

  if (album.is_expanded && !album.children) {
    try {
      const folders = await invoke('expand_folder', { path: album.path, isRecursive: false });
      album.children = folders.children;
    } catch (error) {
      console.error('Error fetching folder tree:', error);
    }
  }
};


/// get children folders
async function getAlbums() {
  try {
    const fetchedAlbums = await invoke('get_albums');
    // console.log('fetchedAlbums...', fetchedAlbums);
    if (fetchedAlbums) {
      gAlbums.value = fetchedAlbums.map(album => ({
        ...album, 
        is_expanded: false,
        children: null,
      }));
    } 
    console.log('getAlbums...', gAlbums.value);

  } catch (error) {
    console.error('Failed to fetch albums:', error);
  }
};


</script>