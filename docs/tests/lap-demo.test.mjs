import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { SourceTextModule, SyntheticModule, createContext } from 'node:vm';
import { createState, addDuplicateFixtures } from '../demo-native/fixtures.mjs';

// Execute the actual browser adapter, replacing only Vue refs and browser APIs.
async function adapter(storage = new Map()) {
  const seed = addDuplicateFixtures(createState());
  const context = createContext({
    URL, console, queueMicrotask, setTimeout: () => 0, clearTimeout: () => {},
    window: {}, location: { href: 'https://example.com/lap/demos/lap/index.html' },
    localStorage: {
      getItem: key => storage.get(key) ?? null,
      setItem: (key, value) => storage.set(key, value),
      removeItem: key => storage.delete(key),
    },
  });
  const module = new SourceTextModule(readFileSync(new URL('../demo-native/tauri.mjs', import.meta.url), 'utf8'), { context });
  await module.link(specifier => {
    if (specifier === 'vue') return new SyntheticModule(['ref'], function () { this.setExport('ref', value => ({ value })); }, { context });
    if (specifier === './fixtures.generated.json') return new SyntheticModule(['default'], function () { this.setExport('default', seed); }, { context });
    throw new Error(`Unexpected import: ${specifier}`);
  });
  await module.evaluate();
  return module.namespace;
}
const ids = rows => Array.from(rows, p => p.id).sort((a, b) => a - b);

test('actual adapter exposes twenty records and two duplicate groups', async () => {
  const a = await adapter();
  assert.equal(a.demoBackend.query().length, 20);
  assert.deepEqual(Array.from(a.demoBackend.groups(), g => Array.from(g.items, i => i.file_id)), [[8, 20], [11, 19]]);
});
test('actual batch metadata changes are visible to queries and survive reload', async () => {
  const storage = new Map(), a = await adapter(storage);
  await a.invoke('batch_update_file_metadata', { params: { fileIds: [2, 3], rating: 4, isFavorite: true, cullingFlag: 2 } });
  assert.deepEqual(ids(a.demoBackend.query({ rating: 4, isFavorite: true, cullingFlag: 2 })), [2, 3]);
  const reloaded = await adapter(storage);
  assert.deepEqual(ids(reloaded.demoBackend.query({ rating: 4 })), [2, 3]);
});
test('editable metadata persists in the browser catalog', async () => {
  const storage = new Map();
  const a = await adapter(storage);
  await a.invoke('edit_file_metadata', { fileIds: [1], changes: { e_title: 'New title', comments: '' } });
  const reloaded = await adapter(storage);
  const file = await reloaded.invoke('get_file_info', { fileId: 1 });
  assert.equal(file.e_title, 'New title');
  assert.equal(file.comments, '');
});
test('tag tree supports subtree queries and keeps assignments when a tag moves', async () => {
  const a = await adapter();
  const tags = await a.invoke('get_all_tags');
  const parent = tags.find(tag => tag.name === 'Outdoors');
  const child = tags.find(tag => tag.name === 'Nature');
  assert.equal(child.parent_id, parent.id);
  assert.equal(child.path, 'Outdoors|Nature');
  assert.equal(a.demoBackend.query({ tagId: parent.id }).length, parent.count);
  const files = ids(a.demoBackend.query({ tagId: child.id }));
  assert(files.length > 0);
  await a.invoke('move_tag', { tagId: child.id, parentId: null });
  assert.deepEqual(ids(a.demoBackend.query({ tagId: child.id })), files);
});
test('actual collection operations are idempotent and leave original folders intact', async () => {
  const a = await adapter(), before = a.demoBackend.state.photos[0].folder;
  await a.invoke('add_files_to_collection', { fileIds: [1, 2], collectionId: 2 });
  await a.invoke('add_files_to_collection', { fileIds: [1], collectionId: 2 });
  assert.deepEqual(ids(a.demoBackend.query({}, 2)), [1, 2]);
  assert.equal(a.demoBackend.state.photos[0].folder, before);
  await a.invoke('remove_files_from_collection', { fileIds: [1], collectionId: 2 });
  assert(!ids(a.demoBackend.query({}, 2)).includes(1));
});
test('actual move and search/type constraints combine', async () => {
  const a = await adapter();
  await a.invoke('move_file', { fileId: 1, newFolderId: 2 });
  assert.equal(a.demoBackend.state.photos[0].folder, 2);
  assert.deepEqual(ids(a.demoBackend.query({ searchFileName: 'Melk', searchFileType: 4, searchFolder: '/Pictures/Sample-Wikimedia/Travel 2024' })), [9]);
  assert.equal(a.demoBackend.query({ searchFileName: 'Melk', searchFileType: 4, searchFolder: '/Pictures/Sample-Wikimedia/Nature 2025' }).length, 0);
});
test('actual dedup keep choice controls deletion and remaining groups', async () => {
  const a = await adapter();
  await a.invoke('dedup_set_keep', { groupId: 8, fileId: 20 });
  assert.equal(a.demoBackend.groups()[0].items.find(i => i.file_id === 20).is_keep, 1);
  const { preview } = await a.invoke('dedup_delete', { request: { preview: true } });
  assert.equal(preview.fileCount, 2);
  assert.equal(a.demoBackend.query().length, 20);
  await a.invoke('dedup_delete', { request: { planToken: preview.token } });
  assert.equal(a.demoBackend.query().length, 18);
  assert.equal(a.demoBackend.groups().length, 0);
  assert(a.demoBackend.state.photos.find(p => p.id === 8).deleted);
});
test('thumbnail and preview mapping share one asset, including duplicate pairs', async () => {
  const a = await adapter();
  assert.equal(a.demoMediaUrl(20), a.demoMediaUrl(8));
  assert.equal(a.demoMediaUrl(19), a.demoMediaUrl(11));
  assert.equal(a.demoMediaUrl(1), 'https://example.com/lap/demos/lap/thumbnails/01.jpg');
  assert.equal(a.demoMediaUrl(999), '');
});
