import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';

const content = fs.readFileSync(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
const slice = content.slice(
  content.indexOf('async function reconcileDraggedFileMoves'),
  content.indexOf('const isProcessing = ref(false)'),
);
assert.ok(slice.includes('async function reconcileDraggedFileMoves'), 'reconcile slice not found');
assert.ok(slice.includes('function notifyViewerFilesLeftView'), 'notify slice not found');
const js = ts.transpile(slice, { target: ts.ScriptTarget.ES2022 });

function load(files, ids, overrides = {}) {
  const events = [];
  const flag = { value: false };
  const state = {
    currentContentRequestId: 7,
    getMediaItemSize: file => Number(file?.size ?? 0) + Number(file?.companion_size ?? 0),
    fileList: { value: files },
    getCurrentQueryFileIds: async () => ids,
    getFolderName: path => path.split('/').pop(),
    markDedupSourceUpdated: () => {},
    gridViewRef: { value: { getScrollTop: () => 321, scrollToPosition: v => events.push(['scroll', v]) } },
    groupedModeActive: { value: false },
    isGroupingSupportedForCurrentView: () => true,
    initializeGroupedFileList: async () => true,
    resolveGroupedFileIndex: async () => 0,
    getGroupedRowIndexForFileIndex: index => index + 1,
    fetchGroupedRowsRange: async () => {},
    getCurrentQueryCountAndSum: async () => [ids.length, ids.reduce((sum, id) => sum + id * 10, 0)],
    pendingRestoreScrollTop: { value: null },
    selectedFileIds: new Set([2, 3]),
    selectedCount: { value: 2 }, selectedSize: { value: 50 }, selectMode: { value: true },
    lastSelectedIndex: { value: 1 }, keyboardSelectionAnchorIndex: { value: 1 },
    totalFileCount: { value: files.length }, totalFileSize: { value: 60 },
    isSearchLikeView: { value: false },
    getCurrentQueryTimeLine: async () => [], timelineData: { value: [] },
    removeDeletedFilesFromImageViewerSession: () => undefined,
    tauriEmit: async (channel, payload) => events.push([channel, payload]),
    isRealFileItem: file => !!file && !file.isPlaceholder && typeof file.id === 'number',
    nextTick: fn => Promise.resolve().then(fn),
    dragMoveSelectionScrollSuppressed: flag,
    selectedItemIndex: { value: 1 },
    invalidateThumbnailRequests: () => {}, pendingRequests: new Set(['old']), visibleRangeSeqId: 0,
    lastVisibleRange: { start: 0, end: 2 }, countedFolderRequestId: { value: 7 },
    recomputeLoadedGroupSelectedCounts: () => {}, syncSelectionVersions: () => {},
    updateSelectedImage: index => events.push(['details', index]),
    syncSelectionToImageViewer: index => events.push(['viewer', index]),
    updateContent: async () => { throw new Error('Unexpected full refresh'); },
    ...overrides,
  };
  const factory = new Function(...Object.keys(state), js + `;
    return { reconcileDraggedFileMoves, requestId: () => currentContentRequestId, navigate: () => { currentContentRequestId++; } };
  `);
  return { api: factory(...Object.values(state)), state, events };
}
const files = () => [
  { id: 1, file_path: '/a/1.jpg', size: 10 },
  { id: 2, file_path: '/a/2.jpg', size: 20 },
  { id: 3, file_path: '/a/3.jpg', size: 30 },
];
const dest = { path: '/b', folderId: 9, albumId: 4 };
const move = (api, entries = [{ id: 2, path: '/b/2.jpg' }]) => api.reconcileDraggedFileMoves(
  entries, [{ id: 2, size: 20 }, { id: 3, size: 30 }], dest, 7,
);

test('removal tolerates virtual slots and retains unloaded selections', async () => {
  const { api, state, events } = load([files()[0], files()[1], null], [1, 3]);
  await move(api);
  assert.deepEqual(state.fileList.value.map(f => f?.id ?? null), [1, null]);
  assert.deepEqual([...state.selectedFileIds], [3]);
  assert.equal(state.selectedSize.value, 30);
  assert.equal(state.selectedCount.value, 1);
  assert.equal(state.selectMode.value, true);
  assert.equal(state.totalFileSize.value, 40);
  assert.equal(api.requestId(), 8); // old page responses fail their request guard
  assert.equal(state.pendingRequests.size, 0);
  assert.deepEqual(events.find(e => e[0] === 'scroll'), ['scroll', 321]);
  assert.deepEqual(events.find(e => e[0] === 'files-deleted')[1].fileIds, [2]);
});

test('conflict rename reorders retained files before setting focus', async () => {
  const { api, state, events } = load(files(), [1, 3, 2]);
  await move(api, [{ id: 2, path: '/b/z.jpg' }]);
  assert.deepEqual(state.fileList.value.map(f => f.id), [1, 3, 2]);
  assert.equal(state.selectedItemIndex.value, 2);
  assert.equal(state.fileList.value[2].name, 'z.jpg');
  assert.equal(state.fileList.value[2].folder_id, 9);
  assert.equal(events.some(event => event[0] === 'details' || event[0] === 'viewer'), false);
});

test('a new file at the same index refreshes details and the viewer without scrolling to selection', async () => {
  const { api, state, events } = load(files(), [1, 3]);
  await move(api);
  assert.equal(state.selectedItemIndex.value, 1);
  assert.equal(state.fileList.value[1].id, 3);
  assert.deepEqual(events.filter(event => event[0] === 'details' || event[0] === 'viewer'), [
    ['details', 1], ['viewer', 1],
  ]);
  assert.deepEqual(events.find(event => event[0] === 'scroll'), ['scroll', 321]);
});

test('batch membership is evaluated individually', async () => {
  const { api, state, events } = load(files(), [1, 3]);
  await move(api, [{ id: 2, path: '/b/2.jpg' }, { id: 3, path: '/b/3.jpg' }]);
  assert.deepEqual(state.fileList.value.map(f => f.id), [1, 3]);
  assert.equal(state.fileList.value[1].file_path, '/b/3.jpg');
  assert.deepEqual(events.find(e => e[0] === 'files-deleted')[1].fileIds, [2]);
});

test('group rebuild preserves remaining selection even when initializer clears it', async () => {
  let state;
  const loaded = load(files(), [1, 3], {
    groupedModeActive: { value: true },
    initializeGroupedFileList: async (requestId, retained) => {
      assert.equal(requestId, 8);
      assert.deepEqual([...retained.keys()], [1, 3]);
      state.selectedFileIds.clear();
      state.selectedSize.value = 0;
      state.fileList.value = [retained.get(1), retained.get(3)];
      return true;
    },
  });
  state = loaded.state;
  await move(loaded.api);
  assert.deepEqual([...state.selectedFileIds], [3]);
  assert.equal(state.selectedSize.value, 30);
  assert.equal(state.fileList.value[1].isSelected, true);
});

test('retained files also rebuild folder groups', async () => {
  let calls = 0;
  const { api, state } = load(files(), [1, 2, 3], {
    groupedModeActive: { value: true },
    initializeGroupedFileList: async (_requestId, retained) => {
      calls++;
      assert.equal(retained.get(2).folder_id, 9);
      return true;
    },
    resolveGroupedFileIndex: async () => 2,
  });
  await move(api);
  assert.equal(calls, 1);
  assert.equal(state.selectedItemIndex.value, 2);
});

test('grouped focus outside the viewport is hydrated without scrolling to it', async () => {
  let state;
  const loaded = load(files(), [1, 2, 3], {
    groupedModeActive: { value: true },
    initializeGroupedFileList: async () => {
      state.fileList.value = [files()[0], null, null];
      return true;
    },
    resolveGroupedFileIndex: async () => 2,
    fetchGroupedRowsRange: async (start, end) => {
      assert.deepEqual([start, end], [3, 4]);
      state.fileList.value[2] = { ...files()[1], file_path: '/b/2.jpg' };
    },
  });
  state = loaded.state;
  await move(loaded.api);
  assert.equal(state.selectedItemIndex.value, 2);
  assert.equal(state.fileList.value[2].id, 2);
  assert.deepEqual(loaded.events.find(event => event[0] === 'scroll'), ['scroll', 321]);
});

test('a view opened during transfer reconciles without changing its focused file or double-subtracting size', async () => {
  const { api, state } = load([files()[0], files()[2]], [1, 3], {
    totalFileSize: { value: 40 },
  });
  await api.reconcileDraggedFileMoves([{ id: 2, path: '/b/2.jpg' }], [{ id: 2, size: 20 }], dest, 7, true);
  assert.equal(state.fileList.value[state.selectedItemIndex.value].id, 3);
  assert.equal(state.totalFileSize.value, 40);
});

test('a new view retains focus even if it also contains the moved file', async () => {
  const { api, state } = load(files(), [2, 1, 3], { selectedItemIndex: { value: 2 } });
  await api.reconcileDraggedFileMoves([{ id: 2, path: '/b/2.jpg' }], [{ id: 2, size: 20 }], dest, 7, true);
  assert.equal(state.fileList.value[state.selectedItemIndex.value].id, 3);
  assert.equal(state.fileList.value[0].file_path, '/b/2.jpg');
});

test('a stale request does not start reconciliation', async () => {
  const original = files();
  const { api, state } = load(original, []);
  await api.reconcileDraggedFileMoves([{ id: 2, path: '/b/2.jpg' }], [], dest, 6);
  assert.equal(api.requestId(), 7);
  assert.equal(state.fileList.value, original);
});

test('navigation during membership lookup leaves the new view untouched', async () => {
  let resolve;
  const original = files();
  const { api, state } = load(original, [], {
    getCurrentQueryFileIds: () => new Promise(r => { resolve = r; }),
  });
  const pending = move(api);
  api.navigate();
  resolve([1, 3]);
  await pending;
  assert.equal(state.fileList.value, original);
  assert.equal(state.selectedCount.value, 2);
});

test('a late timeline result cannot overwrite the next view', async () => {
  let resolve;
  const { api, state } = load(files(), [1, 3], {
    getCurrentQueryTimeLine: () => new Promise(r => { resolve = r; }),
  });
  await move(api);
  api.navigate();
  state.timelineData.value = ['new-view'];
  resolve(['old-view']);
  await Promise.resolve();
  assert.deepEqual(state.timelineData.value, ['new-view']);
});
