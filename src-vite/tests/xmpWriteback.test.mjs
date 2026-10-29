import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
import { computed, markRaw, ref } from 'vue';

const menuSource = fs.readFileSync(new URL('../src/common/fileMenu.ts', import.meta.url), 'utf8');
const locale = JSON.parse(fs.readFileSync(new URL('../src/locales/en.json', import.meta.url), 'utf8'));
const availability = fs.readFileSync(new URL('../src/common/availability.ts', import.meta.url), 'utf8');
const requiresOriginalAction = new Function(ts.transpile(availability.slice(availability.indexOf('const originalActions')).replace(/export /g, '')) + ';return requiresOriginalAction;')();
const icons = Object.fromEntries([...new Set(menuSource.match(/\bIcon\w+/g))].map(name => [name, {}]));
const menuCode = ts.transpile(menuSource.replace(/^import[\s\S]*?from ['"][^'"]+['"];?\n/gm, '').replace(/export /g, ''), { target: ts.ScriptTarget.ES2022 });
const deps = {
  computed, markRaw, ...icons,
  config: { externalAppsFor: () => [], defaultExternalApp: () => null, main: { sidebarIndex: 0 }, settings: { face: { enabled: false } } },
  libConfig: { album: { id: 1 }, activePane: 'album' }, SIDEBAR: { ALBUM: 1 },
  DEFAULT_PLATFORM: 'mac', getShortcutLabel: () => '', isWin: false,
  isOriginalUnavailable: file => !!file?.unavailable, requiresOriginalAction,
};
const useFileMenuItems = new Function(...Object.keys(deps), menuCode + ';return useFileMenuItems;')(...Object.values(deps));

test('single and selection menus offer XMP save without an overwrite-import action', () => {
  for (const selected of [false, true]) {
    const calls = [];
    const options = { selectMode: ref(selected), selectionCount: ref(2), selectionHasUnavailable: ref(false), selectionMediaKind: ref('image') };
    const file = ref({ id: 1, name: 'photo.jpg', file_type: 1 });
    const menu = useFileMenuItems(file, ref(locale), true, key => key, action => calls.push(action), options);
    const item = menu.value.find(item => item.action?.actionName === 'save-xmp');
    assert.ok(item);
    assert.equal(menu.value.some(item => item.action?.actionName === 'import-xmp'), false);
    item.action();
    assert.deepEqual(calls, ['save-xmp']);
    file.value.unavailable = true;
    options.selectionHasUnavailable.value = true;
    const blocked = menu.value.find(item => item.action?.actionName === 'save-xmp');
    assert.equal(blocked.disabled, true);
    blocked.action();
    assert.deepEqual(calls, ['save-xmp']);
  }
});

const content = fs.readFileSync(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
const actionCode = content.slice(content.indexOf("    'save-xmp': async () => {"), content.indexOf("    'favorite':", content.indexOf("    'save-xmp': async () => {")));
test('XMP save sends the complete selection and library identity, reporting partial failure', async () => {
  const requests = [], messages = [];
  const values = {
    selectMode: ref(true), selectedFileIds: new Set([10, 20, 30]),
    fileList: ref([{ id: 10 }]), selectedItemIndex: ref(0), libConfig: { _libraryId: 'library-a' },
    saveXmpMetadata: async (...args) => { requests.push(args); return { savedIds: [10, 20], failures: [{ fileId: 30, error: 'read-only' }] }; },
    toast: { success: text => messages.push(text), error: text => messages.push(text) },
    t: (key, { count }) => `${key}:${count}`,
  };
  const save = new Function(...Object.keys(values), `return ({${actionCode}})['save-xmp'];`)(...Object.values(values));
  await save();
  assert.deepEqual(requests[0], [[10, 20, 30], 'library-a']);
  assert.deepEqual(messages, ['menu.file.xmp_save_success:2', 'menu.file.xmp_save_failed:1: read-only']);
  values.selectMode.value = false;
  await save();
  assert.deepEqual(requests[1], [[10], 'library-a']);
});

const apiSource = fs.readFileSync(new URL('../src/common/api.js', import.meta.url), 'utf8');
const transferStart = apiSource.indexOf('export function renameFile(');
const transferEnd = apiSource.indexOf('// delete a file\n', transferStart);
assert.ok(transferStart >= 0 && transferEnd > transferStart);
const transferCode = ts.transpile(apiSource.slice(transferStart, transferEnd).replace(/export /g, ''), { target: ts.ScriptTarget.ES2022 });
function transferFixture(options = {}) {
  const calls = [], errors = [];
  const invoke = async (command, args) => {
    calls.push({ command, args });
    if (command === 'begin_file_transfer_batch') {
      if (options.beginError) throw options.beginError;
      return 42;
    }
    if (command === 'end_file_transfer_batch') {
      if (options.endError) throw options.endError;
      return;
    }
    if (options.commandError) throw options.commandError;
    return '/target/photo.jpg';
  };
  const config = { settings: { groupRawJpegPairs: options.groupRawJpegPairs ?? false } };
  const busyNotices = [];
  const api = new Function('invoke', 'console', 'config', 'notifyTransferBusy', transferCode +
    '; return { withFileTransferBatch, moveFile, copyFile, moveFileOutsideLibrary, renameFile };')(
      invoke, { error: (...args) => errors.push(args), log() {} }, config, error => busyNotices.push(error));
  return { ...api, calls, errors, config, busyNotices };
}

test('transfer batch begin failure falls back to commands with a null batch id', async () => {
  const error = new Error('Too many active batches');
  const f = transferFixture({ beginError: error });
  const result = await f.withFileTransferBatch(async batchId => {
    assert.equal(batchId, null);
    return f.copyFile('/source/photo.jpg', '/target', 'keep_both', batchId);
  });
  assert.equal(result, '/target/photo.jpg');
  assert.equal(f.calls.at(-1).args.batchId, null);
  assert.equal(f.calls.at(-1).args.groupRawJpegPairs, false);
  assert.ok(!f.calls.some(call => call.command === 'end_file_transfer_batch'));
  assert.equal(f.errors[0][1], error);
});

test('transfer batch forwards its id to move, copy and move outside the library', async () => {
  const f = transferFixture();
  await f.withFileTransferBatch(async batchId => {
    await f.moveFile(1, '/source/photo.jpg', 2, '/target', 'keep_both', batchId);
    await f.copyFile('/source/photo.jpg', '/target', 'keep_both', batchId);
    await f.moveFileOutsideLibrary(1, '/source/photo.jpg', '/target', 'keep_both', batchId);
  });
  assert.deepEqual(f.calls.slice(1, -1).map(call => [call.command, call.args.batchId, call.args.groupRawJpegPairs]), [
    ['move_file', 42, false], ['copy_file', 42, false], ['move_file_outside_library', 42, false],
  ]);
});

test('transfer batch returns the action result and ends after normal completion', async () => {
  const f = transferFixture();
  const result = { moved: 3 };
  assert.equal(await f.withFileTransferBatch(async () => result), result);
  assert.deepEqual(f.calls.at(-1), { command: 'end_file_transfer_batch', args: { batchId: 42 } });
});

test('transfer batch ends after action failure and preserves the original exception', async () => {
  const error = new Error('Original transfer failure');
  const f = transferFixture({ endError: new Error('Cleanup failure') });
  await assert.rejects(f.withFileTransferBatch(async () => { throw error; }), failure => failure === error);
  assert.equal(f.calls.at(-1).command, 'end_file_transfer_batch');
  assert.equal(f.errors.length, 1);
});

test('transfer batch cleanup failure cannot replace a successful action result', async () => {
  const error = new Error('Cleanup failure');
  const f = transferFixture({ endError: error });
  const result = [{ id: 1 }];
  assert.equal(await f.withFileTransferBatch(async () => result), result);
  assert.equal(f.errors[0][1], error);
});

test('single rename forwards the backend error instead of turning it into null', async () => {
  const error = 'Destination XMP already exists: /target/photo.xmp';
  const f = transferFixture({ commandError: error });
  await assert.rejects(f.renameFile(1, '/source/photo.jpg', 'renamed.jpg'), failure => failure === error);
});

test('content rename displays the backend reason and keeps the dialog open', async () => {
  const start = content.indexOf('const onRenameFile = async');
  const end = content.indexOf('const refreshAffectedAlbums', start);
  assert.ok(start >= 0 && end > start);
  const error = 'XMP sidecar is not a regular file: /source/photo.xmp';
  const f = transferFixture({ commandError: error });
  const view = { requestId: 1, libraryId: 'library-a', focusedId: 1 };
  const guardCalls = [];
  const values = {
    captureOperationView: () => view,
    operationViewIsCurrent: snapshot => { guardCalls.push(snapshot); return snapshot === view; },
    refreshAfterStaleOperation: () => { throw new Error('A failed rename must not refresh optimistically'); },
    requireCurrentOriginals: async () => true, selectedItemIndex: ref(0),
    fileList: ref([{ id: 1, name: 'photo.jpg', file_path: '/source/photo.jpg' }]),
    renamingFileName: ref({ ext: 'jpg' }), combineFileName: (name, ext) => `${name}.${ext}`,
    renameFile: f.renameFile, showRenameMsgbox: ref(true), errorMessage: ref(''),
    localeMsg: ref({ msgbox: { rename_file: { error: 'Rename failed' } } }),
  };
  const rename = new Function(...Object.keys(values), ts.transpile(content.slice(start, end), { target: ts.ScriptTarget.ES2022 }) +
    '; return onRenameFile;')(...Object.values(values));
  await rename('renamed');
  assert.match(values.errorMessage.value, /Rename failed/);
  assert.ok(values.errorMessage.value.includes(error));
  assert.deepEqual(guardCalls, [view, view]);
  assert.equal(values.showRenameMsgbox.value, true);
  assert.equal(values.fileList.value[0].name, 'photo.jpg');
});

test('file info rename reports the backend reason without changing catalog fields', async () => {
  const source = fs.readFileSync(new URL('../src/components/FileInfo.vue', import.meta.url), 'utf8');
  const start = source.indexOf('const finishRename = async');
  const end = source.indexOf('function formatGeoLocation', start);
  assert.ok(start >= 0 && end > start);
  const error = 'Destination XMP already exists: /target/photo.xmp';
  const f = transferFixture({ commandError: error });
  const messages = [];
  const isRenaming = ref(true);
  const values = {
    isRenaming, props: { fileInfo: { id: 1, name: 'photo.jpg', file_path: '/source/photo.jpg' } },
    renamingName: ref('renamed'), extractFileName: () => ({ ext: 'jpg' }), isValidFileName: () => true,
    combineFileName: (name, ext) => `${name}.${ext}`, renameFile: f.renameFile,
    cancelRename: () => { isRenaming.value = false; }, toast: { error: message => messages.push(message) }, t: key => key,
  };
  const rename = new Function(...Object.keys(values), ts.transpile(source.slice(start, end), { target: ts.ScriptTarget.ES2022 }) +
    '; return finishRename;')(...Object.values(values));
  await rename();
  assert.ok(messages[0].includes(error));
  assert.equal(f.calls[0].command, 'rename_file');
  assert.equal(f.calls[0].args.groupRawJpegPairs, false);
  assert.equal(isRenaming.value, false);
  assert.equal(values.props.fileInfo.name, 'photo.jpg');
});
