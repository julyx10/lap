import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const source = readFileSync(new URL('../src/common/api.js', import.meta.url), 'utf8');
const functions = source.slice(source.indexOf('export async function getFileThumb('), source.indexOf('// get file info')).replace(/^export /gm, '');

function createApi() {
  const libConfig = { _libraryId: 'a' };
  const calls = [];
  const invoke = (command, args) => new Promise(resolve => calls.push({ command, args, resolve }));
  const api = new Function('invoke', 'libConfig', 'getRawDisplayOptions', `${functions}; return { getFileThumb, getFileThumbById, getFileThumbs };`)(invoke, libConfig, () => ({}));
  return { api, libConfig, calls };
}

test('thumbnail APIs bind requested library and discard responses after switching', async () => {
  for (const [method, args] of [
    ['getFileThumb', [1, '/a', 1, 0, 256, false, null, 1, 'a']],
    ['getFileThumbById', [1, 256, false, 'a']],
    ['getFileThumbs', [[{ fileId: 1 }], 256, false, false, 'a']],
  ]) {
    const { api, libConfig, calls } = createApi();
    const pending = api[method](...args);
    assert.equal(calls.length, 1);
    assert.equal(calls[0].args.libraryId, 'a');
    libConfig._libraryId = 'b';
    calls[0].resolve(method === 'getFileThumbs' ? [{ file_id: 1 }] : { file_id: 1 });
    assert.deepEqual(await pending, method === 'getFileThumbs' ? [null] : null);
    const stale = await api[method](...args);
    assert.deepEqual(stale, method === 'getFileThumbs' ? [null] : null);
    assert.equal(calls.length, 1, 'a queued old-library request must not dispatch in the new library');
  }
});

test('scheduler and deferred thumbnail callers retain their creation library', () => {
  const content = readFileSync(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
  assert.match(content, /tasks\[0\]\.forceRegenerate, false, tasks\[0\]\.libraryId/);
  assert.match(content, /file, requestId, libraryId, thumbnailSize, viewportBound, forceRegenerate/);
  assert.match(content, /task\.libraryId !== libConfig\._libraryId/);
  const image = readFileSync(new URL('../src/components/Image.vue', import.meta.url), 'utf8');
  assert.match(image, /getFileThumbById\(fileId, config\.settings\.thumbnailSize, false, libraryId\)/);
  assert.match(image, /if \(libraryId !== libConfig\._libraryId\) return ''/);
  const dedup = readFileSync(new URL('../src/components/DedupPane.vue', import.meta.url), 'utf8');
  assert.match(dedup, /async function hydrateSimilarThumbnails[\s\S]*?const libraryId = libConfig\._libraryId/);
  assert.match(dedup, /async function hydrateGroupThumbnails[\s\S]*?const libraryId = libConfig\._libraryId/);
});
