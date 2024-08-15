import { invoke } from '@tauri-apps/api';
import { ref } from 'vue'

export default {
  // name: 'AppLogic',
  setup () {
    const albums = ref([]);
    const error = ref(null);

    const fetchAlbums = async () => {
      console.log('Fetching albums...');
      try {
        albums.value = await invoke('get_albums');
      } catch (error) {
        error.value = 'Failed to load albums: ' + error.message;
        console.error('Failed to fetch albums:', error);
      }
    };

    return {
      albums,
      error,
      fetchAlbums
    };
  }
};
