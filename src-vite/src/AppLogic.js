import { invoke } from '@tauri-apps/api';

export default {
  data() {
    return {
      albums: [],
      error: ''
    };
  },
  methods: {
    async fetchAlbums() {
      try {
        // Call the Tauri command using the global `invoke` function
        this.albums = await invoke('get_albums');
      } catch (error) {
        this.error = 'Failed to load albums: ' + error.message;
        console.error('Failed to fetch albums:', error);
      }
    }
  }
};
