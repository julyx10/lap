import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
import { ref, watch, nextTick } from 'vue';

const source = fs.readFileSync(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
const start = source.indexOf('function enterAlbumPreviewMode(');
const code = ts.transpile(source.slice(start, source.indexOf('function exitTempViewMode(', start)), {
  target: ts.ScriptTarget.ES2022,
});
const loadingCode = ts.transpile(source.slice(source.indexOf('function showLoadingContent('), source.indexOf('// Similar Search Mode State')), { target: ts.ScriptTarget.ES2022 });
const watcherCode = source.slice(source.indexOf('watch([fileList, contentReady]'), source.indexOf('function clearLoadedSelectionFlags('));

test('folder temp view queries only the containing folder and preserves the original backup', async () => {
  const state = { mode: { value: 'none' }, backup: { value: null }, title: { value: '' }, queries: [] };
  const original = { selection: 42, scrollTop: 1200 };
  const enter = new Function('tempViewMode', 'backupState', 'contentTitle', 'getFileList', 'ref', 'watch', `
    let currentContentRequestId = 0, pendingFocusedFileId = null;
    const getFolderPath = path => path.slice(0, path.lastIndexOf('/'));
    const createViewBackup = () => (${JSON.stringify(original)});
    const invalidateThumbnailRequests = () => {};
    const showQuickView = { value: false }, totalFileCount = {}, totalFileSize = {};
    const currentQueryParams = { value: {} };
    const scrollPosition = {}, selectedItemIndex = ref(0), gridViewRef = {};
    const fileList = ref([{ id: 42 }]), contentReady = ref(true);
    const contentCountIsAuthoritative = {}, timelineData = {}, isLoading = {};
    const clearContentRows = () => { fileList.value = []; };
    const clearSelectionForFileListUpdate = () => {};
    const restoreTagRemovalAnchor = () => {}, restoreSelectionAfterFileListRefresh = () => {};
    const pendingSelectionRestore = null;
    const restoreFocusedFileAfterListRefresh = () => {};
    ${loadingCode}
    const stop = ${watcherCode}
    const getAlbum = async () => ({ path: '/photos', name: 'Photos' });
    const formatFolderBreadcrumb = path => path;
    ${code}
    return { enterAlbumPreviewMode, focusedId: () => pendingFocusedFileId, stop,
      invalidate: () => { currentContentRequestId++; } };
  `)(state.mode, state.backup, state.title, (params, request) => state.queries.push({ params, request }), ref, watch);
  enter.enterAlbumPreviewMode({ id: 42, album_id: 1, file_path: '/photos/day/image.jpg' });
  assert.equal(state.mode.value, 'album');
  assert.deepEqual(state.backup.value, original);
  assert.deepEqual(state.queries[0].params, { searchFolder: '/photos/day', searchAllSubfolders: '' });
  assert.equal(enter.focusedId(), 42);
  await nextTick(); // The empty-list watcher must not discard the pending target.
  assert.equal(enter.focusedId(), 42);
  await Promise.resolve();
  assert.equal(state.title.value, '/photos/day');
  const backup = state.backup.value;
  enter.enterAlbumPreviewMode({ id: 43, album_id: 1, file_path: '/photos/other/image.jpg' });
  assert.equal(state.backup.value, backup);
  enter.invalidate();
  state.title.value = 'Restored view';
  await Promise.resolve();
  assert.equal(state.title.value, 'Restored view');
  enter.stop();
});

test('returning during loading invalidates pending queries and clears the pending folder target', async () => {
  const exitStart = source.indexOf('function exitTempViewMode(');
  const exitCode = ts.transpile(source.slice(exitStart, source.indexOf('function handleTitleClick(', exitStart)), { target: ts.ScriptTarget.ES2022 });
  const original = {
    fileList: [{ id: 7 }], selectedItemIndex: 0, scrollPosition: 900, scrollTop: 900,
    totalFileCount: 1, totalFileSize: 100, contentTitle: 'Original', timelineData: [],
    currentQueryParams: { searchFolder: '/original' }, contentCountIsAuthoritative: true,
  };
  const defaults = {
    backupState: original, fileList: [], groupedModeActive: false, groupedRows: [], totalRowCount: 0,
    groupedTimelineGroups: [], folderGroupRoots: [], groupSelectedCountMap: {}, groupSelectedSizeMap: {},
    totalFileCount: 0, totalFileSize: 0, contentCountIsAuthoritative: false, countedFolderRequestId: 10,
    isLoading: true, contentReady: false, contentTitle: 'Folder', selectedItemIndex: 0, scrollPosition: 0,
    timelineData: [], currentQueryParams: {}, currentQuerySource: 'query', currentSmartQueryParams: null,
    currentCollectionId: null, currentSearchFileIds: [], tempViewMode: 'album', showQuickView: false,
    gridViewRef: null,
  };
  const refs = Object.fromEntries(Object.entries(defaults).map(([key, value]) => [key, ref(value)]));
  const harness = new Function(...Object.keys(refs), `
    let currentContentRequestId = 10, pendingFocusedFileId = 42;
    const groupMetaMap = new Map(), fileIdToGroupId = new Map();
    const clearSelectionForFileListUpdate = () => {}, invalidateThumbnailRequests = () => {};
    const preserveLoadedThumbnails = files => files;
    const setTimeout = () => {}, updateSelectedImage = () => {};
    const isRealFileItem = () => true, getFileListThumb = () => {};
    ${exitCode}
    return { exitTempViewMode, pending: () => pendingFocusedFileId,
      complete: () => { if (currentContentRequestId === 10) {
        fileList.value = [{ id: 42 }]; contentTitle.value = 'Stale folder';
      } } };
  `)(...Object.values(refs));
  harness.exitTempViewMode();
  await Promise.resolve().then(harness.complete);
  assert.deepEqual(refs.fileList.value, original.fileList);
  assert.equal(refs.contentTitle.value, original.contentTitle);
  assert.equal(harness.pending(), null);
  assert.equal(refs.contentReady.value, true);
  assert.equal(refs.isLoading.value, false);
  assert.equal(refs.contentCountIsAuthoritative.value, true);
  assert.equal(refs.countedFolderRequestId.value, 11);
  assert.equal(refs.tempViewMode.value, 'none');
});
