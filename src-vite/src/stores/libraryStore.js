/**
 * Library Store - Per-library configuration
 */
import { defineStore } from 'pinia';
import { getCurrentLibraryState, saveLibraryState, getAppConfig } from '@/common/api';
import { CULLING, LIB_ITEM } from '@/common/constants';
import { setThumbLibraryId } from '@/common/utils';
import { resetLibraryAccessibility } from '@/common/availability';

// The app-level deep watcher calls save() for every store mutation. Keep the
// last actual payload per library so runtime-only count refreshes do not turn
// into redundant backend writes.
const lastSavedPayloads = new Map();

export const createEmptyLibraryCounts = () => ({
  all: 0,
  favorite: 0,
  today: 0,
  rated: 0,
  unrated: 0,
  ratings: { 1: 0, 2: 0, 3: 0, 4: 0, 5: 0 },
  culling: { [CULLING.PICK]: 0, [CULLING.REJECT]: 0, [CULLING.UNREVIEWED]: 0 },
});

export const useLibraryStore = defineStore('libraryStore', {
  state: () => ({
    // Current library ID (for saving)
    _libraryId: null,
    _initialized: false,


    // Per-library state
    /** @type {'main' | 'collection'} */
    activePane: 'main',

    /** @type {{ item: 'all-files' | 'favorites' | 'ratings' | 'culling' | 'subjects' | 'on-this-day', smartId: string | null, ratingsExpanded: boolean, cullingExpanded: boolean, subjectsExpanded: boolean, subjectCounts: Record<string, number>, counts: { all: number, favorite: number, today: number, rated: number, unrated: number, ratings: Record<number, number>, culling: Record<string, number> } }} */
    library: {
      item: LIB_ITEM.ALL,
      smartId: null,
      ratingsExpanded: true,
      cullingExpanded: true,
      subjectsExpanded: true,
      activateTick: 0,
      subjectCounts: {},
      counts: createEmptyLibraryCounts(),
    },

    /** @type {{ id: number, folderId: number | null, folderPath: string, selected: boolean, activateTick: number, counts: Record<string, number> }} */
    album: {
      id: 0,                  // current album id (0: show all files)
      folderId: null,         // current folder id
      folderPath: '',         // current folder path
      selected: false,        // album is selected
      activateTick: 0,        // increments on each album/folder click (even same target)
      counts: {},
    },

    /** @type {{ type: 'system' | 'custom', id: string | number | null }} */
    smartAlbum: {
      type: 'system',         // selected smart album type
      id: null,               // selected smart album id
    },

    /** @type {Array<{ id: string, name: string, description: string, source: 'rules', query: { version: number, match: 'all' | 'any', rules: Array<{ id: string, field: string, operator: string, value: any }> }, group: { type: number }, sort: { type: number, order: number }, coverFileId: number | null, count: number | null, createdAt: number, updatedAt: number }>} */
    smartAlbums: [],          // custom smart albums

    /** @type {{ selectedId: number | null, activateTick: number, counts: Record<string, number> }} */
    collection: {
      selectedId: null,      // selected collection id
      activateTick: 0,
      counts: {},
    },

    /** @type {{ item: number | null }} */
    rating: {
      item: null,             // 0: unrated, 1-5: rated files
    },

    culling: {
      item: 'pick',
    },

    /** @type {{ id: number | null, collapsedTagIds: number[], activateTick: number, counts: Record<string, number> }} */
    tag: {
      id: null,
      collapsedTagIds: [],
      activateTick: 0,
      counts: {},
    },

    /** @type {{ year: number | null, month: number | null, date: number | null }} */
    calendar: {
      year: null,             // selected year
      month: null,            // selected month (1-12)
      date: null,             // selected date (1-31), -1 means selecting a month
    },

    /** @type {{ make: string | null, model: string | null, lensMake: string | null, lensModel: string | null }} */
    camera: {
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

    /** @type {{ searchText: string, searchHistory: (string | { text: string, count: number | null })[], searchHistoryIndex: number }} */
    search: {
      searchText: '',         // AI search text
      searchHistory: [],      // AI search history
      searchHistoryIndex: -1, // current AI search history index
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
        this.collection.selectedId = Number(this.collection.selectedId || 0) > 0
          ? Number(this.collection.selectedId)
          : null;

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
      resetLibraryAccessibility();
      this._initialized = false;
      this.$reset();              // Pinia built-in: restore every field to its initial value
      await this.init();          // re-read current library id + state from backend
    },

    /**
     * Invalidate counts after album exclusions change. Real-time
     * aggregates are runtime-only; delayed counts are also cleared here before
     * their next explicit item activation. Folder file counts are ephemeral
     * and cleared separately via clearFolderFileCounts().
     */
    clearLazySidebarCounts() {
      this.library.counts = createEmptyLibraryCounts();
      this.library.subjectCounts = {};
      this.album.counts = {};
      this.tag.counts = {};
      this.collection.counts = {};
      if (Array.isArray(this.smartAlbums) && this.smartAlbums.some((album) => album?.count != null)) {
        this.smartAlbums = this.smartAlbums.map((album) => ({ ...album, count: null }));
      }
      const history = this.search.searchHistory;
      if (Array.isArray(history) && history.some((item) => typeof item !== 'string' && item?.count != null)) {
        this.search.searchHistory = history.map((item) =>
          typeof item === 'string' ? item : { ...item, count: null },
        );
      }
    },

    async save() {
      if (this._libraryId && this._initialized) {
        try {
          // These panels now obtain their counts from live grouped aggregates.
          // Keep the values in memory for rendering, but never persist them.
          const {
            counts: _libraryCounts,
            activateTick: _libraryActivateTick,
            ...libraryState
          } = this.library;
          const {
            counts: _albumCounts,
            activateTick: _albumActivateTick,
            ...albumState
          } = this.album;
          const {
            counts: _collectionCounts,
            activateTick: _collectionActivateTick,
            ...collectionState
          } = this.collection;
          const {
            counts: _tagCounts,
            activateTick: _tagActivateTick,
            ...tagState
          } = this.tag;
            const stateToSave = {
            library: libraryState,
            album: albumState,
            smartAlbum: this.smartAlbum,
            smartAlbums: this.smartAlbums,
            collection: collectionState,
            rating: this.rating,
            culling: this.culling,
            tag: tagState,
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

          const payload = JSON.stringify(stateToSave);
          if (lastSavedPayloads.get(this._libraryId) === payload) return;
          await saveLibraryState(this._libraryId, stateToSave);
          lastSavedPayloads.set(this._libraryId, payload);
        } catch (error) {
          console.error('Failed to save library state:', error);
        }
      }
    },
  },
});
