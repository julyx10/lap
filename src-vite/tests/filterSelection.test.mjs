import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { reactive, ref, watch, nextTick, effectScope } from 'vue';
import ts from 'typescript';

const source = readFileSync(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
function between(start, end) {
  const a = source.indexOf(start);
  const b = source.indexOf(end, a);
  assert.ok(a >= 0 && b > a);
  return source.slice(a, b);
}
const code = [
  between('async function restoreSelectionAfterFileListRefresh()', 'function rememberFocusedFileForPresentationRefresh()'),
  between('function rememberFocusedFileForPresentationRefresh()', 'async function waitForGroupedGridLayout()'),
  between('async function scrollToGroupedFile(', 'watch([fileList, contentReady]'),
  between('watch([fileList, contentReady]', 'function clearLoadedSelectionFlags()'),
  between('function refreshContentFromSelectionChange()', 'onMounted(() => {'),
  between('function restoreInitialSelectionIfNeeded()', '\nonBeforeUnmount(() => {'),
  between('/// watch for file list changes', '\nwatch(\n  () => ['),
  'async function loadStartupSelection() {' + between('const appConfig = await getAppConfig();',
    "  window.addEventListener('keydown', handleLocalKeyDown);") + '}',
].join('\n');

function fixture(index = 1) {
  const config = reactive({ main: { sidebarIndex: 9 }, camera: { isCamera: false },
    calendar: { view: 0 }, search: { fileType: 0, sortType: 0 }, settings: { showSubfolderFiles: false } });
  const libConfig = reactive({ _libraryId: 'test', activePane: 'content', collection: {}, library: {},
    album: {}, smartAlbum: {}, person: {}, calendar: {}, tag: {}, location: {},
    camera: { lensMake: 'Nikon', lensModel: 'Lens 1' }, search: { searchText: '' }, rating: {}, culling: {} });
  const selectedItemIndex = ref(index);
  const fileList = ref([1, 2, 3, 4].map(id => ({ id })));
  const contentReady = ref(true);
  const totalFileCount = ref(4);
  const scrolled = [];
  let resultIds = [1, 2, 3, 4];
  let positionQuery = async id => resultIds.indexOf(id);
  let tickWait = async () => {};
  let layoutWait = async () => {};
  const selectMode = ref(false);
  const selectedCount = ref(0);
  const selectedSize = ref(0);
  const selectedFileIds = new Set();
  const deps = { config, libConfig, selectedItemIndex, fileList, contentReady, totalFileCount, watch,
    selectMode, selectedCount, selectedSize, selectedFileIds, scrollPosition: ref(0), console,
    nextTick: async () => { await nextTick(); await tickWait(); },
    getCurrentQueryFileIds: async () => resultIds,
    resetSelectionSummary() { selectedFileIds.clear(); selectedCount.value = 0; selectedSize.value = 0; },
    syncSelectionVersions() {}, isRealFileItem: file => typeof file?.id === 'number',
    getAppConfig: async () => ({ last_selected_item_index: 7 }), updateSelectedImage() {},
    groupedModeActive: ref(false),
    getCurrentQueryFilePosition: id => positionQuery(id), resolveGroupedFileIndex: id => positionQuery(id),
    waitForGroupedGridLayout: () => layoutWait(), getGroupedRowIndexForFileIndex: index => index,
    isGeometryGridStyle: ref(false),
    gridViewRef: ref({ scrollToPosition() {}, scrollToItem: index => scrolled.push(index), scrollToRowIndex: index => scrolled.push(index) }),
    clearSelectionForFileListUpdate() { selectMode.value = false; selectedCount.value = 0; selectedFileIds.clear(); },
    openImageViewer() {} };
  const scope = effectScope();
  const state = scope.run(() => new Function(...Object.keys(deps), ts.transpile(
    'let pendingFocusedFileId = null; let pendingSelectionRestore = null; let pendingScopeSelectionChange = false;'
    + 'let isRestoringFocusedFile = false; let isRestoringSelection = false; let currentContentRequestId = 1;'
    + 'let pendingInitialSelectedIndex = -1; let hasRestoredInitialSelection = false;'
    + 'function updateContent() { pendingScopeSelectionChange = false; contentReady.value = false; currentContentRequestId++; }'
    + code + ';return { refreshContentFromSelectionChange, restoreInitialSelectionIfNeeded, loadStartupSelection, pendingId: () => pendingFocusedFileId, pendingSelection: () => pendingSelectionRestore,'
    + 'seedInitial() { pendingInitialSelectedIndex = 7; hasRestoredInitialSelection = false; },'
    + 'seedMulti() { selectMode.value = true; selectedCount.value = 2; selectedSize.value = 20; selectedFileIds.add(1); selectedFileIds.add(2);'
    + 'pendingSelectionRestore = { selectedIds: new Set([1,2]), selectedSizes: new Map([[1,10],[2,10]]), selectedSize:20, viewportFileId:2, fallbackFileId:1 }; } };',
    { target: ts.ScriptTarget.ES2022 },
  ))(...Object.values(deps)));
  return { ...deps, ...state, scrolled, stop: () => scope.stop(),
    setPositionQuery(query) { positionQuery = query; },
    setTickWait(wait) { tickWait = wait; }, setLayoutWait(wait) { layoutWait = wait; },
    async load(ids) {
      resultIds = ids;
      totalFileCount.value = ids.length;
      fileList.value = ids.map(id => ({ id }));
      state.restoreInitialSelectionIfNeeded();
      contentReady.value = true;
      await nextTick();
      await new Promise(resolve => setImmediate(resolve));
    } };
}

test('filters, sorting and grouping retain the file by ID or fall back to the first item', async () => {
  for (const [ids, expected] of [[[2, 1], 0], [[4, 1, 2], 2], [[1, 3], 0], [[], -1]]) {
    for (const change of [f => { f.config.search.fileType = 3; },
      f => { f.config.search.sortType++; }, f => { f.config.search.groupBy = 1; f.groupedModeActive.value = true; },
      f => { f.config.settings.showSubfolderFiles = true; }]) {
      const f = fixture();
      change(f);
      await nextTick();
      f.refreshContentFromSelectionChange();
      assert.equal(f.pendingId(), 2);
      await f.load(ids);
      assert.equal(f.selectedItemIndex.value, expected);
      assert.equal(f.pendingId(), null);
      f.stop();
    }
  }
});

test('query scope changes cancel queued and late-arriving startup selection restores', async () => {
  for (const lateConfig of [false, true]) {
    const f = fixture(3);
    f.seedInitial();
    f.libConfig.camera.lensModel = 'Lens 2';
    await nextTick();
    f.refreshContentFromSelectionChange();
    if (lateConfig) await f.loadStartupSelection();
    await f.load([100, 101]);
    assert.equal(f.selectedItemIndex.value, 0);
    f.stop();
  }
});

test('startup selection still restores when the query scope has not changed', async () => {
  const f = fixture();
  await f.loadStartupSelection();
  await f.load([100, 101]);
  assert.equal(f.selectedItemIndex.value, 1);
  f.stop();
});

test('changing query scope resets focus and multi-selection', async () => {
  for (const change of [f => { f.config.main.sidebarIndex++; },
    f => { f.libConfig.activePane = 'collection'; }, f => { f.libConfig._libraryId = 'other'; },
    f => { f.libConfig.camera.lensModel = 'Lens 2'; }, f => { f.libConfig.album.folderId = 2; },
    f => { f.libConfig.tag.id = 2; }, f => { f.libConfig.rating.item = 4; },
    f => { f.libConfig.search.searchText = 'new'; }]) {
    const f = fixture(3);
    f.seedMulti();
    change(f);
    await nextTick();
    f.refreshContentFromSelectionChange();
    await f.load([100, 101]);
    assert.equal(f.selectedItemIndex.value, 0);
    assert.equal(f.pendingId(), null);
    assert.equal(f.pendingSelection(), null);
    assert.equal(f.selectMode.value, false);
    assert.equal(f.selectedCount.value, 0);
    f.stop();
  }
});

test('empty results clear the old file ID before the filter is changed again', async () => {
  const f = fixture();
  f.config.search.fileType = 3;
  await nextTick();
  f.refreshContentFromSelectionChange();
  await f.load([]);
  f.config.search.fileType = 0;
  await nextTick();
  f.refreshContentFromSelectionChange();
  await f.load([1, 2, 3]);
  assert.equal(f.selectedItemIndex.value, 0);
  f.stop();
});

test('an old file-position response cannot restore focus into a different scope', async () => {
  const f = fixture();
  let resolvePosition;
  f.setPositionQuery(() => new Promise(resolve => { resolvePosition = resolve; }));
  f.config.search.fileType = 3;
  await nextTick();
  f.refreshContentFromSelectionChange();
  await f.load([1, 2, 3]);
  f.libConfig.camera.lensModel = 'Lens 2';
  await nextTick();
  f.refreshContentFromSelectionChange();
  await f.load([100, 101, 102]);
  resolvePosition(2);
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(f.selectedItemIndex.value, 0);
  assert.deepEqual(f.scrolled, []);
  f.stop();
});

test('scope changes during rendering or grouped scrolling cannot restore stale selection', async () => {
  for (const multi of [false, true]) {
    for (const grouped of [false, true]) {
      const f = fixture();
      let resume;
      const waiting = new Promise(resolve => { resume = resolve; });
      f.groupedModeActive.value = grouped;
      if (grouped) f.setLayoutWait(() => waiting);
      else f.setTickWait(() => waiting);
      if (multi) f.seedMulti();
      else f.refreshContentFromSelectionChange();
      await f.load([1, 2, 3]);
      f.libConfig.camera.lensModel = 'Lens 2';
      await nextTick();
      f.refreshContentFromSelectionChange();
      await f.load([100, 101]);
      resume();
      await new Promise(resolve => setImmediate(resolve));
      assert.equal(f.selectedItemIndex.value, 0);
      assert.equal(f.selectMode.value, false);
      assert.equal(f.selectedCount.value, 0);
      assert.deepEqual([...f.selectedFileIds], []);
      assert.deepEqual(f.scrolled, []);
      f.stop();
    }
  }
});

test('normal multi-selection restoration still follows file IDs after sorting', async () => {
  const f = fixture();
  f.seedMulti();
  await f.load([2, 1, 3]);
  assert.equal(f.selectedItemIndex.value, 0);
  assert.equal(f.selectMode.value, true);
  assert.deepEqual([...f.selectedFileIds], [1, 2]);
  assert.equal(f.selectedCount.value, 2);
  assert.equal(f.selectedSize.value, 20);
  assert.deepEqual(f.scrolled, [0]);
  f.stop();
});
