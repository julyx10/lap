/**
 * Library Store - Per-library configuration
 */
import { defineStore } from 'pinia';
import { getCurrentLibraryState, saveLibraryState, getAppConfig } from '@/common/api';
import { setThumbLibraryId } from '@/common/utils';

export const useLibraryStore = defineStore('libraryStore', {
  state: () => ({
    // Current library ID (for saving)
    _libraryId: null,
    _initialized: false,

    // Per-library state
    /** @type {{ id: number, folderId: number | null, folderPath: string, selected: boolean, activateTick: number }} */
    album: {
      id: 0,                  // current album id (0: show all files)
      folderId: null,         // current folder id
      folderPath: '',         // current folder path
      selected: false,        // album is selected
      activateTick: 0,        // increments on each album/folder click (even same target)
    },

    /** @type {{ type: 'system' | 'custom', id: string | number | null }} */
    smartAlbum: {
      type: 'system',         // selected smart album type
      id: null,               // selected smart album id
    },

    /** @type {Array<{ id: string, name: string, description: string, source: 'rules', query: { version: number, match: 'all' | 'any', rules: Array<{ id: string, field: string, operator: string, value: any }> }, sort: { type: number, order: number }, createdAt: number, updatedAt: number }>} */
    smartAlbums: [],          // custom smart albums

    /** @type {{ tab: 'favorite' | 'rating', albumId: number | null, folderId: number, folderPath: string, rating: number | null }} */
    favorite: {
      tab: 'favorite',
      albumId: null,
      folderId: 0,            // 0 means favorite files (default)
      folderPath: '',
      rating: null,           // null: all favorites, 0: unrated, 1-5: rated files
    },

    /** @type {{ id: number | null, smartId: string | null, tab: 'smart' | 'custom' }} */
    tag: {
      id: null,
      smartId: null,
      tab: 'custom',
    },

    /** @type {{ year: number | null, month: number | null, date: number | null }} */
    calendar: {
      year: null,             // selected year
      month: null,            // selected month (1-12)
      date: null,             // selected date (1-31), -1 means selecting a month
    },

    /** @type {{ tab: 'camera' | 'lens', make: string | null, model: string | null, lensMake: string | null, lensModel: string | null }} */
    camera: {
      tab: 'camera',
      make: null,             // selected camera make
      model: null,            // selected camera model
      lensMake: null,         // selected lens make
      lensModel: null,        // selected lens model
    },

    /** @type {{ cc: string | null, admin1: string | null, name: string | null }} */
    location: {
      cc: null,               // country code
      admin1: null,           // admin1 (state/province)
      name: null,             // location name
    },

    /** @type {{ id: number | null, name: string | null }} */
    person: {
      id: null,               // selected person id
      name: null,             // selected person name
    },

    /** @type {{ searchType: number, searchText: string, searchHistory: (string | { text: string, fileId: number | null })[], searchHistoryIndex: number, similarImageHistory: number[], similarImageHistoryIndex: number, fileName: string }} */
    search: {
      searchType: 0,            // 0: ai search, 1: similar image, 2: filename search
      searchText: '',         // AI search text
      searchHistory: [],      // AI search history
      searchHistoryIndex: -1, // current AI search history index
      similarImageHistory: [],// similar image history
      similarImageHistoryIndex: -1, // current similar image history index
      fileName: '',           // filename search text
    },

    /** @type {{ albumId: number | null, folderId: number | null, folderPath: string | null, selected: boolean }} */
    destFolder: {
      albumId: null,          // destination album id
      folderId: null,         // destination folder id
      folderPath: null,       // destination folder path
      selected: false,        // destination album is selected
    },

    index: {
      status: 0,              // 0: idle, 1: indexing, 2: paused
      /** @type {number[]} */
      albumQueue: [],         // indexing album queue
      /** @type {number[]} */
      pausedAlbumIds: [],     // paused albums
      albumName: '',          // current album name
      phase: 'discovering',   // current scan phase
      discovered: 0,          // current album's discovered count
      processed: 0,           // current album's processed count
      searchReady: 0,         // current album's embedding-ready count
      indexed: 0,             // current album's indexed count
      total: 0,               // current album's total count
      searchTotal: 0,         // current album's searchable total
      failed: 0,              // current album's failed count
    },
  }),

  actions: {
    async init() {
      try {
        // Get current library ID
        const appConfig = await getAppConfig();
        if (appConfig) {
          this._libraryId = appConfig.current_library_id;
          setThumbLibraryId(appConfig.current_library_id);
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
        this.index.status = Number(this.index.status || 0);
        this.index.phase = this.index.phase || 'discovering';
        this.index.pausedAlbumIds = Array.isArray(this.index.pausedAlbumIds)
          ? Array.from(new Set(this.index.pausedAlbumIds.map(id => Number(id)).filter(id => id > 0)))
          : [];

        this._initialized = true;

        // Always pause on restart — never auto-resume scanning
        if (this.index.status === 1) {
          this.index.status = 2;
          await this.save();
        }
      } catch (error) {
        console.error('Failed to initialize library state:', error);
        this._initialized = true;
      }
    },

    /**
     * Reset all per-library state to defaults and re-initialize from the
     * backend.  Called after `switchLibrary()` so the UI picks up the new
     * library's persisted state without a full page reload.
     */
    async reload() {
      this._initialized = false;
      this.$reset();              // Pinia built-in: restore every field to its initial value
      await this.init();          // re-read current library id + state from backend
    },

    async save() {
      if (this._libraryId && this._initialized) {
        try {
          const stateToSave = {
            album: this.album,
            smartAlbum: this.smartAlbum,
            smartAlbums: this.smartAlbums,
            favorite: this.favorite,
            tag: this.tag,
            calendar: this.calendar,
            camera: this.camera,
            location: this.location,
            search: this.search,
            destFolder: this.destFolder,
            index: {
              status: this.index.status,
              albumQueue: this.index.albumQueue,
              pausedAlbumIds: this.index.pausedAlbumIds,
              albumName: this.index.albumName,
              phase: this.index.phase,
              discovered: this.index.discovered,
              processed: this.index.processed,
              searchReady: this.index.searchReady,
              indexed: this.index.indexed,
              total: this.index.total,
              searchTotal: this.index.searchTotal,
              failed: this.index.failed,
            },
            person: this.person,
          };
          
          await saveLibraryState(this._libraryId, stateToSave);
        } catch (error) {
          console.error('Failed to save library state:', error);
        }
      }
    },
  },
});
