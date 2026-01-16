/**
 * Library Store - Per-library configuration
 */
import { defineStore } from 'pinia';
import { getCurrentLibraryState, saveLibraryState, getAppConfig } from '@/common/api';

export const useLibraryStore = defineStore('libraryStore', {
  state: () => ({
    // Current library ID (for saving)
    _libraryId: null,
    _initialized: false,

    // Per-library state
    album: {
      id: 0,                  // current album id (0: show all files)
      folderId: null,         // current folder id
      folderPath: '',         // current folder path
      selected: false,        // album is selected
    },

    favorite: {
      albumId: null,
      folderId: 0,            // 0 means favorite files (default)
      folderPath: null,
    },

    tag: {
      id: null,
    },

    calendar: {
      year: null,             // selected year
      month: null,            // selected month (1-12)
      date: null,             // selected date (1-31), -1 means selecting a month
    },

    camera: {
      make: null,             // selected camera make
      model: null,            // selected camera model
    },

    location: {
      cc: null,               // country code
      admin1: null,           // admin1 (state/province)
      name: null,             // location name
    },

    search: {
      searchText: '',         // AI search text
      searchHistory: [],      // AI search history
      searchHistoryIndex: -1, // current AI search history index
      similarImageHistory: [],// similar image history
      similarImageHistoryIndex: -1, // current similar image history index
      fileName: '',           // filename search text
    },

    destFolder: {
      albumId: null,          // destination album id
      folderId: null,         // destination folder id
      folderPath: null,       // destination folder path
      selected: false,        // destination album is selected
    },

    index: {
      status: 0,              // 0: idle, 1: indexing
      /** @type {number[]} */
      albumQueue: [],         // indexing album queue
      albumName: '',          // current album name
      indexed: 0,             // current album's indexed count
      total: 0,               // current album's total count
    },
  }),

  actions: {
    async init() {
      try {
        // Get current library ID
        const appConfig = await getAppConfig();
        if (appConfig) {
          this._libraryId = appConfig.current_library_id;
        }

        // Load library state from backend
        const backendState = await getCurrentLibraryState();
        if (backendState) {
          Object.keys(backendState).forEach(key => {
            if (this[key] !== undefined) {
              // Deep merge for objects (like album, search, etc) to preserve structure
              Object.assign(this[key], backendState[key]);
            }
          });
        }

        this._initialized = true;
      } catch (error) {
        console.error('Failed to initialize library state:', error);
        this._initialized = true;
      }
    },

    async save() {
      if (this._libraryId && this._initialized) {
        try {
          const stateToSave = {
            album: this.album,
            favorite: this.favorite,
            tag: this.tag,
            calendar: this.calendar,
            camera: this.camera,
            location: this.location,
            search: this.search,
            destFolder: this.destFolder,
            index: this.index,
          };
          
          await saveLibraryState(this._libraryId, stateToSave);
        } catch (error) {
          console.error('Failed to save library state:', error);
        }
      }
    },
  },
});
