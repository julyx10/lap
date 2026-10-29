import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import ts from 'typescript';
import { computed, effectScope, ref, watch } from 'vue';

const content = await readFile(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
const dialog = await readFile(new URL('../src/components/BatchRenameDialog.vue', import.meta.url), 'utf8');
const panel = await readFile(new URL('../src/components/SelectionPanel.vue', import.meta.url), 'utf8');
function load(source, start, end, values, result) {
  const offset = source.indexOf(start);
  assert.ok(offset >= 0);
  const code = ts.transpile(source.slice(offset, source.indexOf(end, offset)), { target: ts.ScriptTarget.ES2022 });
  return new Function(...Object.keys(values), `${code}; return ${result};`)(...Object.values(values));
}
function openingFixture() {
  const f = {
    batchRenameSelection: ref(null), batchRenameOpening: false, selectedCount: ref(3),
    libConfig: { _libraryId: 'lib' }, currentContentRequestId: 1, selectedFileIds: new Set([1, 2, 3]), selectMode: ref(true),
    getActionableSelectedItems: () => [{id:1,name:'a.jpg'}, {id:3,name:'c.jpg'}],
    getActionableSelectedItemsForAction: () => { throw new Error('Must not hydrate before opening'); },
    requireOriginalFiles: () => { throw new Error('Backend performs original-file checks'); },
    getCurrentQueryFileIds: async () => [5, 3, 2, 1],
    toast: { error() {} }, t: key => key,
  };
  return f;
}
function open(f) { return load(content, 'async function openBatchRename()', 'function onBatchRenameDone', f, 'openBatchRename'); }

test('panel entry precedes delete and rename routes selection to batch dialog', async () => {
  assert.ok(panel.indexOf("$t('msgbox.batch_rename.title')") < panel.indexOf(':icon="IconTrash"'));
  for (const selected of [true, false]) {
    let called = 0;
    const f = {currentOriginalsUnavailable: () => false, uiStore: {inputStack: []}, selectMode: ref(selected),
      openBatchRename: () => {called++;}, renamingFileName: ref(''), showRenameMsgbox: ref(false),
      fileList: ref([{name:'photo.jpg'}]), selectedItemIndex: ref(0), extractFileName: name => name.split('.')[0]};
    load(content, 'function clickRename()', 'async function clickSetAlbumCover', f, 'clickRename')();
    assert.equal(called, selected ? 1 : 0);
    assert.equal(f.showRenameMsgbox.value, !selected);
  }
});

test('numbering follows the full display order, including unloaded selection', async () => {
  const f = openingFixture(); await open(f)();
  assert.deepEqual(f.batchRenameSelection.value.files.map(file => file.id), [3, 2, 1]);
});

test('stale selections and changed libraries do not open a dialog', async () => {
  for (const change of [f => {f.selectedFileIds.delete(2);}, f => {f.libConfig._libraryId = 'other';}, f => {f.selectMode.value = false;}]) {
    const f = openingFixture();
    f.getCurrentQueryFileIds = async () => {change(f); return [3, 2, 1];};
    await open(f)(); assert.equal(f.batchRenameSelection.value, null);
  }
});

function dialogFixture(context, api = {}) {
  context.mock.timers.enable({apis:['setTimeout']});
  const scope = effectScope();
  const calls = [];
  const values = {ref, computed, watch, config: { settings: { groupRawJpegPairs: true } }, props:{files:[{id:1,name:'old.jpg'},{id:2,name:'other.jpg'}],libraryId:'lib'},
    t: key => key, emit: (...args) => calls.push(args), uiStore:{isInputActive: () => true},
    previewBatchRename: async () => ({items:[{id:1,oldName:'old.jpg',newName:'old_1.jpg',companions:0}],errors:[]}),
    batchRenameFiles: async () => ({items:[{id:1}]}), ...api};
  const model = scope.run(() => load(dialog, 'const nameInput =', 'onMounted(', values,
    '{ name, start, digits, preview, error, pending, saving, canSubmit, submit, close, refreshPreview }'));
  context.after(() => {scope.stop();context.mock.timers.reset();});
  return { ...model, calls, config: values.config };
}
async function tick(context) {context.mock.timers.tick(201); await Promise.resolve(); await Promise.resolve();}

test('stale preview responses cannot enable submitting changed names', async context => {
  const resolvers = [];
  const f = dialogFixture(context, {previewBatchRename: () => new Promise(resolve => resolvers.push(resolve))});
  await tick(context);
  f.name.value = 'new';
  assert.equal(f.canSubmit.value, false);
  await tick(context);
  resolvers[0]({items:[{newName:'old_1.jpg'}],errors:[]}); await Promise.resolve();
  assert.equal(f.pending.value, true); assert.equal(f.preview.value, null);
  resolvers[1]({items:[{newName:'new_1.jpg'}],errors:[]}); await Promise.resolve();
  assert.equal(f.canSubmit.value, true); assert.equal(f.preview.value.items[0].newName, 'new_1.jpg');
});

test('invalid input and name collisions block writes; editing input rechecks the preview', async context => {
  let writes = 0, conflict = true;
  const f = dialogFixture(context, {previewBatchRename: async () => ({items:[{id:1}],errors: conflict ? ['exists'] : []}),
    batchRenameFiles: async () => {writes++;return {items:[{id:1}]};}});
  await tick(context); await f.submit(); assert.equal(writes, 0);
  conflict = false; f.name.value = 'renamed'; await tick(context);
  assert.equal(f.canSubmit.value, true);
  f.start.value = 0; await tick(context); await f.submit(); assert.equal(writes, 0);
  f.start.value = 1; await tick(context); await f.submit();
  assert.equal(writes, 1); assert.equal(f.calls[0][0], 'done');
});

test('preview and commit use the grouping policy captured when the dialog opened', async context => {
  const previews = [], writes = [];
  const f = dialogFixture(context, {
    previewBatchRename: async (...args) => { previews.push(args); return {items:[{id:1,oldName:'old.jpg',newName:'new.jpg'}],errors:[]}; },
    batchRenameFiles: async (...args) => { writes.push(args); return {items:[{id:1}]}; },
  });
  await tick(context);
  f.config.settings.groupRawJpegPairs = false;
  f.name.value = 'changed';
  await tick(context);
  await f.submit();
  assert.equal(previews.length, 2);
  for (const args of previews) { assert.equal(args[0], 'lib'); assert.equal(args[3], true); }
  assert.equal(writes.length, 1);
  assert.equal(writes[0][0], 'lib');
  assert.equal(writes[0][4], true);
  assert.deepEqual(writes[0][3], f.preview.value.items);
});

test('saving blocks a second write and closing until completion', async context => {
  let finish, writes = 0;
  const f = dialogFixture(context, {batchRenameFiles: () => {writes++; return new Promise(resolve => {finish=resolve;});}});
  await tick(context);
  const first = f.submit(); await f.submit(); f.close();
  assert.equal(writes, 1); assert.deepEqual(f.calls, []);
  finish({items:[{id:1}]}); await first;
  assert.equal(f.calls[0][0], 'done'); assert.equal(f.saving.value, false);
});


test('initial preview starts immediately while typing remains debounced', async context => {
  let requests = 0;
  const f = dialogFixture(context, {previewBatchRename: async () => {
    requests++; return {items:[{id:1}],errors:[]};
  }});
  assert.equal(requests, 1);
  f.name.value = 'a'; f.name.value = 'ab'; f.name.value = 'abc';
  assert.equal(requests, 1);
  await tick(context); assert.equal(requests, 2);
});


test('loaded selections open without fetching all query IDs', async () => {
  const f = openingFixture();
  f.getActionableSelectedItems = () => [{id:3}, {id:2}, {id:1}];
  f.getCurrentQueryFileIds = () => { throw new Error('Must reuse loaded display order'); };
  await open(f)();
  assert.deepEqual(f.batchRenameSelection.value.files.map(file => file.id), [3, 2, 1]);
});


test('editing preserves current names and row objects while replacing only target names', async context => {
  let resolve;
  let requests = 0;
  const f = dialogFixture(context, {previewBatchRename: () => {
    requests++;
    return requests === 1
      ? Promise.resolve({items:[{id:1,oldName:'old.jpg',newName:'old_1.jpg',companions:0}],errors:[]})
      : new Promise(done => {resolve = done;});
  }});
  await tick(context);
  const preview = f.preview.value;
  const rows = preview.items;
  const row = rows[0];
  f.name.value = 'new';
  assert.equal(f.preview.value, preview);
  assert.equal(f.preview.value.items, rows);
  assert.equal(row.oldName, 'old.jpg');
  assert.equal(f.canSubmit.value, false);
  await tick(context);
  resolve({items:[{id:1,oldName:'old.jpg',newName:'new_1.jpg',companions:1}],errors:[]});
  await Promise.resolve();
  assert.equal(f.preview.value, preview);
  assert.equal(f.preview.value.items, rows);
  assert.equal(f.preview.value.items[0], row);
  assert.equal(row.oldName, 'old.jpg');
  assert.equal(row.newName, 'new_1.jpg');
  assert.equal(row.companions, 1);
  assert.equal(f.canSubmit.value, true);
});
