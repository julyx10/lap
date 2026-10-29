import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';

const api = fs.readFileSync(new URL('../src/common/api.js', import.meta.url), 'utf8');
const source = api.replace(/^import .*;$/gm, '').replace(/^export /gm, '');
const settings = { groupRawJpegPairs: false };
const calls = [];
const invoke = async (command, args) => { calls.push({ command, args }); return []; };
const apiFunctions = new Function('invoke', 'config', source + `; return {
  getQueryFiles, getQueryCountAndSum, getGroupedQueryRows, getQueryFileIds,
  getSmartQueryFiles, getSmartQueryCountAndSum, getCollectionFiles, getCollectionCountAndSum,
  deleteFile, deleteFilePermanently, batchDeleteFiles, previewBatchRename, batchRenameFiles,
  renameFile, moveFile, moveFileOutsideLibrary, copyFile, dedupDelete, getFolderFiles,
  getLibraryVisibleCounts, getAlbumVisibleCounts, getCollectionCounts, getTagCounts,
  getTakenDates, getCameraInfo, getLensInfo, getLocationInfo, getPersons, getPersonsPage, getFaceStats
};`)(invoke, { settings, search: {} });

test('query calls carry the current grouping policy without mutating filters', async () => {
  for (const group of [false, true]) {
    settings.groupRawJpegPairs = group;
    const params = { searchFileName: 'test' };
    for (const name of ['getQueryFiles', 'getQueryCountAndSum', 'getGroupedQueryRows', 'getQueryFileIds', 'getSmartQueryFiles', 'getSmartQueryCountAndSum']) {
      await apiFunctions[name](params, 0, 10);
      assert.equal(calls.at(-1).args.params.groupRawJpegPairs, group, name);
    }
    for (const name of ['getCollectionFiles', 'getCollectionCountAndSum']) {
      await apiFunctions[name](1, params, 0, 10);
      assert.equal(calls.at(-1).args.params.groupRawJpegPairs, group, name);
    }
    assert.deepEqual(params, { searchFileName: 'test' });
  }
});

test('file operations carry the current grouping policy', async () => {
  for (const group of [false, true]) {
    settings.groupRawJpegPairs = group;
    for (const [name, args] of Object.entries({ deleteFile: [1, '/file'], deleteFilePermanently: [1, '/file'], batchDeleteFiles: [[], false], previewBatchRename: ['lib', [], {}], batchRenameFiles: ['lib', [], {}, []], renameFile: [1, '/file', 'name'], moveFile: [1, '/file', 2, '/dest', 'keep_both', null], moveFileOutsideLibrary: [1, '/file', '/dest', 'keep_both', null], copyFile: ['/file', '/dest', 'keep_both', null] })) {
      await apiFunctions[name](...args);
      assert.equal(calls.at(-1).args.groupRawJpegPairs, group, name);
      settings.groupRawJpegPairs = !group;
      await apiFunctions[name](...args, group);
      assert.equal(calls.at(-1).args.groupRawJpegPairs, group, `${name} snapshot`);
      settings.groupRawJpegPairs = group;
    }
  }
});

test('statistics, folder and dedup requests carry grouping policy', async () => {
  for (const group of [false, true]) {
    settings.groupRawJpegPairs = group;
    for (const name of ['getLibraryVisibleCounts', 'getAlbumVisibleCounts', 'getCollectionCounts', 'getTagCounts', 'getTakenDates', 'getCameraInfo', 'getLensInfo', 'getLocationInfo', 'getPersons', 'getFaceStats', 'getFolderFiles']) {
      await apiFunctions[name](1, '/folder', true);
      assert.equal(calls.at(-1).args.groupRawJpegPairs, group, name);
    }
    await apiFunctions.getPersonsPage({ sort: 0 });
    assert.equal(calls.at(-1).args.request.groupRawJpegPairs, group);
    await apiFunctions.dedupDelete({ deleteAll: true });
    assert.equal(calls.at(-1).args.request.groupRawJpegPairs, group);
  }
});

const content = fs.readFileSync(new URL('../src/components/Content.vue', import.meta.url), 'utf8');

test('stale operation refreshes only its original library and never mutates the new view', () => {
  const code = ts.transpile(content.slice(content.indexOf('function captureOperationView()'), content.indexOf('function rebuildSelectionAfterListMutation')), { target: ts.ScriptTarget.ES2022 });
  const libConfig = { _libraryId: 'a' };
  const selectionVersion = { value: 1 }, selectedItemIndex = { value: 0 };
  const fileList = { value: [{ id: 7 }] }, selectMode = { value: false }, tempViewMode = { value: 'none' };
  let refreshes = 0, remembered = 0;
  const helpers = new Function('libConfig', 'selectionVersion', 'selectedItemIndex', 'fileList', 'selectMode', 'tempViewMode', 'rememberFocusedFileForPresentationRefresh', 'updateContent', `let currentContentRequestId = 1; ${code}; return { captureOperationView, refreshAfterStaleOperation, changeRequest: () => currentContentRequestId++ };`)(libConfig, selectionVersion, selectedItemIndex, fileList, selectMode, tempViewMode, () => remembered++, () => refreshes++);
  let snapshot = helpers.captureOperationView();
  assert.equal(helpers.refreshAfterStaleOperation(snapshot), false);
  helpers.changeRequest();
  assert.equal(helpers.refreshAfterStaleOperation(snapshot), true);
  assert.equal(refreshes, 1);
  assert.equal(remembered, 1);
  snapshot = helpers.captureOperationView();
  selectionVersion.value++;
  assert.equal(helpers.refreshAfterStaleOperation(snapshot), true);
  assert.equal(refreshes, 2);
  snapshot = helpers.captureOperationView();
  libConfig._libraryId = 'b';
  assert.equal(helpers.refreshAfterStaleOperation(snapshot), true);
  assert.equal(refreshes, 2);
});
const clipboardCode = content.slice(content.indexOf('const getClipboardFilePaths ='), content.indexOf('const clickCopyImages ='));
const getClipboardFilePaths = new Function('config', ts.transpile(clipboardCode, { target: ts.ScriptTarget.ES2022 }) + '; return getClipboardFilePaths;')({ settings });
test('clipboard expands RAW companion only when grouping is enabled', () => {
  const files = [{ file_path: '/a.raw', media_subtype: 'raw_jpeg_pair', live_photo_video_path: '/a.jpg' }];
  settings.groupRawJpegPairs = false;
  assert.deepEqual(getClipboardFilePaths(files), ['/a.raw']);
  settings.groupRawJpegPairs = true;
  assert.deepEqual(getClipboardFilePaths(files), ['/a.raw', '/a.jpg']);
});

test('location watchers distinguish countries with identical place names', () => {
  assert.match(content, /location:\s*\[libConfig\.location\.cc,\s*libConfig\.location\.admin1,\s*libConfig\.location\.name\]/);
  assert.match(content, /libConfig\.location\.cc,\s*libConfig\.location\.admin1,\s*libConfig\.location\.name,\s*\/\/ location/);
  assert.match(content, /locationCc:\s*libConfig\.location\.cc\s*\?\?\s*''/);
});

test('location selection and parent badges use the correct bucket', () => {
  const location = fs.readFileSync(new URL('../src/components/Location.vue', import.meta.url), 'utf8');
  const camera = fs.readFileSync(new URL('../src/components/Camera.vue', import.meta.url), 'utf8');
  assert.match(location, /libConfig\.location\.admin1 === location\.admin1 && \(libConfig\.location\.cc \|\| ''\) === location\.cc/);
  assert.match(location, /libConfig\.location\.name === name && libConfig\.location\.admin1 === location\.admin1/);
  assert.match(location, /location\.parent_count \?\?/);
  assert.match(camera, /item\.parent_count \?\?/);
  const mounted = location.slice(location.indexOf('onMounted(async () => {'), location.indexOf('// Keep the current filter'));
  assert.match(mounted, /restoreLocationSelection\(\)/);
  assert.match(location, /item\.admin1 === libConfig\.location\.admin1 && item\.cc === \(libConfig\.location\.cc \|\| ''\)/);
});

test('group switch invalidates confirmations and refreshes synchronously', () => {
  const start = content.indexOf('watch(() => config.settings.groupRawJpegPairs, () => {');
  const watcher = content.slice(start, content.indexOf("{ flush: 'sync' });", start) + 19);
  assert.match(watcher, /closeTrashMsgbox\(\)/);
  assert.match(watcher, /batchRenameSelection.value = null/);
  assert.match(watcher, /rememberFocusedFileForPresentationRefresh\(\)/);
  assert.match(watcher, /updateContent\(true, selectMode.value, tempViewMode.value === 'none'\)/);
  assert.match(watcher, /flush: 'sync'/);
});
