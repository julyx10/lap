import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import ts from 'typescript';
import { createThumbnailScheduler } from '../src/common/thumbnailScheduler.js';

const tick = () => new Promise(resolve => setImmediate(resolve));
function backend() {
  const calls = [];
  let active = 0;
  let maxActive = 0;
  const load = payloads => new Promise((resolve, reject) => {
    active++;
    maxActive = Math.max(maxActive, active);
    calls.push({ payloads, finish(result = payloads.map(p => p.id)) { active--; resolve(result); },
      fail() { active--; reject(new Error('read failed')); } });
  });
  return { calls, load, get maxActive() { return maxActive; } };
}

test('overlapping requests share batches and one global concurrency cap', async () => {
  const io = backend();
  const scheduler = createThumbnailScheduler({ loadBatch: io.load, priority: () => 0 });
  const first = Array.from({ length: 40 }, (_, id) => scheduler.request(String(id), 'size-512', { id }));
  const duplicate = scheduler.request('0', 'size-512', { id: 0 });
  assert.equal(duplicate, first[0]);
  await tick();
  assert.equal(io.calls.length, 4);
  assert.ok(io.calls.every(call => call.payloads.length <= 4));
  for (let index = 0; index < 10; index++) {
    io.calls[index].finish();
    await tick();
  }
  assert.equal(io.maxActive, 4);
  assert.equal(io.calls.flatMap(call => call.payloads).length, 40);
  assert.deepEqual(await Promise.all(first), Array.from({ length: 40 }, (_, id) => id));
});

test('a new viewport discards queued old work and promotes visible items over prefetch', async () => {
  const io = backend();
  const retained = new Set([0, 1, 2, 3]);
  let visible = 0;
  const scheduler = createThumbnailScheduler({ loadBatch: io.load, concurrency: 1, batchSize: 1,
    priority: p => retained.has(p.id) ? p.id === visible ? 0 : 1 : null });
  const running = scheduler.request('0', 'same', { id: 0 });
  await tick();
  const old = scheduler.request('1', 'same', { id: 1 });
  const prefetch = scheduler.request('2', 'same', { id: 2 });
  const next = scheduler.request('3', 'same', { id: 3 });
  retained.delete(1);
  visible = 3;
  scheduler.refresh();
  assert.equal(await old, undefined);
  io.calls[0].finish();
  await tick();
  assert.equal(io.calls[1].payloads[0].id, 3);
  io.calls[1].finish();
  await tick();
  assert.equal(io.calls[2].payloads[0].id, 2);
  io.calls[2].finish();
  await Promise.all([running, prefetch, next]);
});

for (const explicitFirst of [false, true]) {
  test(`equal-priority explicit demand survives scrolling (${explicitFirst ? 'explicit' : 'grid'} first)`, async () => {
    const io = backend();
    let visible = true;
    const scheduler = createThumbnailScheduler({ loadBatch: io.load, concurrency: 1, batchSize: 1,
      priority: payload => !payload.viewportBound || visible ? 0 : null });
    const blocker = scheduler.request('blocker', 'same', { id: 0, viewportBound: false });
    await tick();
    const demands = [
      { id: 1, viewportBound: explicitFirst ? false : true },
      { id: 1, viewportBound: explicitFirst ? true : false },
    ];
    const first = scheduler.request('selected', 'same', demands[0]);
    const second = scheduler.request('selected', 'same', demands[1]);
    assert.equal(first, second);
    visible = false;
    scheduler.refresh();
    io.calls[0].finish();
    await tick();
    assert.equal(io.calls[1].payloads[0].viewportBound, false);
    io.calls[1].finish();
    assert.deepEqual(await Promise.all([blocker, first, second]), [0, 1, 1]);
    assert.equal(io.calls.length, 2);
  });
}

test('context invalidation between slot reservation and dispatch skips IPC and releases the slot', async () => {
  const io = backend();
  let context = 'old';
  const scheduler = createThumbnailScheduler({ loadBatch: io.load, concurrency: 1,
    priority: payload => payload.context === context ? 0 : null });
  const old = scheduler.request('old:1', 'old', { id: 1, context: 'old' });
  queueMicrotask(() => { context = 'new'; scheduler.refresh(); });
  assert.equal(await old, undefined);
  await tick();
  assert.equal(io.calls.length, 0);
  const next = scheduler.request('new:1', 'new', { id: 1, context: 'new' });
  await tick();
  io.calls[0].finish();
  assert.equal(await next, 1);
});

test('dispatch filters invalidated batch items without shifting result mapping', async () => {
  const io = backend();
  const valid = new Set([1, 2]);
  const scheduler = createThumbnailScheduler({ loadBatch: io.load, concurrency: 1,
    priority: payload => valid.has(payload.id) ? 0 : null });
  const first = scheduler.request('1', 'same', { id: 1 });
  const second = scheduler.request('2', 'same', { id: 2 });
  queueMicrotask(() => { valid.delete(1); scheduler.refresh(); });
  await tick();
  assert.deepEqual(io.calls[0].payloads.map(payload => payload.id), [2]);
  io.calls[0].finish();
  assert.deepEqual(await Promise.all([first, second]), [undefined, 2]);
});

test('disposing after reservation but before dispatch never sends IPC', async () => {
  const io = backend();
  const scheduler = createThumbnailScheduler({ loadBatch: io.load, priority: () => 0 });
  const pending = scheduler.request('1', 'same', { id: 1 });
  queueMicrotask(() => scheduler.dispose());
  assert.equal(await pending, undefined);
  await tick();
  assert.equal(io.calls.length, 0);
});

test('synchronous dispatch failure rejects callers and permits retry', async () => {
  let fail = true;
  const scheduler = createThumbnailScheduler({ priority: () => 0, concurrency: 1,
    loadBatch: payloads => { if (fail) throw new Error('dispatch failed'); return payloads.map(p => p.id); } });
  await assert.rejects(scheduler.request('1', 'same', { id: 1 }), /dispatch failed/);
  fail = false;
  assert.equal(await scheduler.request('1', 'same', { id: 1 }), 1);
});

test('batches keep different size/library policies separate and failures can retry', async () => {
  const io = backend();
  const scheduler = createThumbnailScheduler({ loadBatch: io.load, priority: () => 0, concurrency: 1 });
  const first = scheduler.request('old:1', 'old', { id: 1 });
  const rejected = assert.rejects(first, /read failed/);
  const other = scheduler.request('new:1', 'new', { id: 1 });
  await tick();
  assert.equal(io.calls[0].payloads.length, 1);
  io.calls[0].fail();
  await rejected;
  await tick();
  io.calls[1].finish();
  await other;
  const retry = scheduler.request('old:1', 'old', { id: 1 });
  await tick();
  io.calls[2].finish();
  assert.equal(await retry, 1);
});

test('disposing drops queued work while allowing running requests to finish', async () => {
  const io = backend();
  const scheduler = createThumbnailScheduler({ loadBatch: io.load, priority: () => 0, concurrency: 1, batchSize: 1 });
  const running = scheduler.request('0', 'same', { id: 0 });
  const queued = scheduler.request('1', 'same', { id: 1 });
  await tick();
  scheduler.dispose();
  assert.equal(await queued, undefined);
  assert.equal(await scheduler.request('2', 'same', { id: 2 }), undefined);
  io.calls[0].finish();
  assert.equal(await running, 0);
  await tick();
  assert.equal(io.calls.length, 1);
});

// Exercise the actual Content.vue adapter with delayed IPC, including grouped rows.
const source = await readFile(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
const apiSource = await readFile(new URL('../src/common/api.js', import.meta.url), 'utf8');
const batchApiSource = apiSource.slice(apiSource.indexOf('export async function getFileThumbs('),
  apiSource.indexOf('// get file info', apiSource.indexOf('export async function getFileThumbs(')))
  .replace('export ', '');
const batchApi = invoke => new Function('invoke', 'getRawDisplayOptions', 'libConfig',
  batchApiSource + '; return getFileThumbs;')(invoke, () => ({}), { _libraryId: 'library-a' });

test('production batch API propagates IPC errors and rejects malformed responses', async () => {
  const failure = new Error('Error while fetching offline thumbnail');
  await assert.rejects(batchApi(async () => { throw failure; })([{ fileId: 1 }], 512), error => error === failure);
  for (const response of [[], [null, null], undefined]) {
    await assert.rejects(batchApi(async () => response)([{ fileId: 1 }], 512), /Invalid thumbnail batch response/);
  }
  assert.deepEqual(await batchApi(async () => [null])([{ fileId: 1 }], 512), [null]);
  let args;
  await batchApi(async (command, payload) => { args = payload; return [null]; })([{ fileId: 1 }], 512, true);
  assert.deepEqual(args, { files: [{ fileId: 1 }], thumbnailSize: 512,
    rawDisplayOptions: {}, forceRegenerate: true, trustCached: false, libraryId: 'library-a' });
});

const adapter = ts.transpile(source.slice(source.indexOf('// Track current thumbnail request'),
  source.indexOf('// Open the image viewer window')), { target: ts.ScriptTarget.ES2022 });
function gridFixture(grouped = false, loadThumbs) {
  const io = backend();
  const files = Array.from({ length: 300 }, (_, id) => ({ id: id + 1, file_path: `${id}.jpg`, modified_at: 1 }));
  const rows = { value: grouped ? files.flatMap((file, index) => [
    ...(index % 10 === 0 ? [{ type: 'group', id: `group-${index}` }] : []),
    { type: 'item', file, file_index: index },
  ]) : files };
  const range = { start: 0, end: 3 };
  const memory = new Map();
  const errors = [];
  const libConfig = { _libraryId: 'library-a' };
  let dispose;
  const library = libConfig;
  const api = new Function('createThumbnailScheduler', 'config', 'getFileThumbs', 'gridRows',
    'visibleItemCount', 'lastVisibleRange', 'isItemRow', 'isRealFileItem', 'onBeforeUnmount',
    'getCachedThumbnailDataUrl', 'getThumbnailDataUrl', 'thumbnailPlaceholder', 'console', 'libConfig',
    adapter + '; return { getFileListThumb, invalidateThumbnailRequests, refreshThumbnailViewport, invalidateRawThumbnailPolicy };')(
    createThumbnailScheduler, { settings: { thumbnailSize: 512 } },
    (requests, thumbnailSize, forceRegenerate) => loadThumbs
      ? loadThumbs(requests, io, { thumbnailSize, forceRegenerate }) : io.load(requests), rows, { value: 4 }, range, row => row?.type === 'item',
    file => file && !file.isPlaceholder && typeof file.id === 'number', callback => { dispose = callback; },
    id => memory.get(id), thumb => { memory.set(thumb.id, thumb.url); return thumb.url; },
    'placeholder', { log(...args) { errors.push(args); } }, library);
  const finish = index => io.calls[index].finish(io.calls[index].payloads.map(p => ({
    error_code: 0, id: p.fileId, url: `thumb:${p.fileId}`,
  })));
  return { ...api, io, files, rows, range, memory, errors, finish, library, dispose: () => dispose() };
}

test('Content reports real API failures, releases slots and retries the same files', async () => {
  const grid = gridFixture(false, (requests, io) => batchApi(() => io.load(requests))(requests, 512));
  const files = grid.files.slice(0, 4);
  const failed = grid.getFileListThumb(files);
  await tick();
  grid.io.calls[0].fail();
  await failed;
  assert.ok(grid.errors.some(args => args[1]?.message === 'read failed'));
  assert.ok(files.every(file => !file.thumbnail));
  const malformed = grid.getFileListThumb(files);
  await tick();
  grid.io.calls[1].finish([]);
  await malformed;
  assert.ok(grid.errors.some(args => args[1]?.message.includes('Invalid thumbnail batch response')));
  const retry = grid.getFileListThumb(files);
  await tick();
  grid.finish(2);
  await retry;
  assert.ok(files.every(file => file.thumbnail));
  grid.dispose();
});

test('large explicit refreshes enqueue one batch at a time and yield to visible requests', async () => {
  const grid = gridFixture();
  let done = false;
  const explicit = grid.getFileListThumb(grid.files.slice(200), { viewportBound: false })
    .then(() => { done = true; });
  await tick();
  assert.equal(grid.io.calls.length, 1);
  assert.equal(grid.io.calls[0].payloads.length, 4);
  const visible = grid.getFileListThumb(grid.files.slice(0, 16));
  await tick();
  assert.equal(grid.io.calls.length, 4);
  grid.finish(0);
  await tick();
  assert.deepEqual(grid.io.calls[4].payloads.map(payload => payload.fileId), [13, 14, 15, 16]);
  let next = 1;
  while (!done) {
    assert.ok(next < grid.io.calls.length, 'expected a bounded next refresh batch');
    grid.finish(next++);
    await tick();
  }
  await Promise.all([explicit, visible]);
  assert.ok(grid.files.slice(200).every(file => file.thumbnail));
  assert.equal(grid.io.maxActive, 4);
  grid.dispose();
});

for (const grouped of [false, true]) {
  test(`Content ${grouped ? 'grouped' : 'normal'} grid prioritizes the latest viewport under slow reads`, async () => {
    const grid = gridFixture(grouped);
    const old = grid.getFileListThumb(grid.files);
    await tick();
    assert.equal(grid.io.calls.length, 4);
    grid.range.start = grouped ? grid.rows.value.findIndex(row => row.file?.id === 201) : 200;
    grid.range.end = grid.range.start + 3;
    grid.refreshThumbnailViewport();
    const latest = grid.getFileListThumb(grid.files.slice(200, 204));
    for (let index = 0; index < 4; index++) grid.finish(index);
    await tick();
    assert.equal(grid.io.calls.length, 5);
    assert.deepEqual(grid.io.calls[4].payloads.map(p => p.fileId), [201, 202, 203, 204]);
    grid.finish(4);
    await Promise.all([old, latest]);
    assert.equal(grid.io.maxActive, 4);
    assert.equal(grid.files[16].thumbnail, undefined);
    assert.equal(grid.files[200].thumbnail, 'thumb:201');
    assert.equal(grid.memory.get(1), 'thumb:1'); // Completed old-viewport work remains reusable.
    grid.dispose();
  });
}

test('Content shares in-flight reads and does not apply results after changing library/view', async () => {
  const grid = gridFixture();
  const first = grid.getFileListThumb(grid.files.slice(0, 4));
  const subscribers = grid.files.slice(0, 4).map(file => ({ ...file }));
  const duplicate = grid.getFileListThumb(subscribers);
  await tick();
  assert.equal(grid.io.calls.length, 1);
  grid.finish(0);
  await Promise.all([first, duplicate]);
  assert.ok(subscribers.every(file => file.thumbnail));
  grid.memory.clear();
  const staleFiles = grid.files.slice(4, 8);
  const stale = grid.getFileListThumb(staleFiles);
  await tick();
  grid.invalidateThumbnailRequests();
  grid.finish(1);
  await stale;
  assert.ok(staleFiles.every(file => !file.thumbnail));
  assert.equal(grid.memory.size, 0);
  grid.dispose();
});

test('explicit previews outside the viewport remain available and forced refreshes are independent', async () => {
  const grid = gridFixture();
  const outside = grid.files[250];
  await grid.getFileListThumb([outside]);
  await tick();
  assert.equal(grid.io.calls.length, 0);
  const preview = grid.getFileListThumb([outside], { viewportBound: false });
  await tick();
  grid.finish(0);
  await preview;
  assert.equal(outside.thumbnail, 'thumb:251');
  grid.memory.clear();
  outside.thumbnail = undefined;
  const first = grid.getFileListThumb([outside], { viewportBound: false, bustCache: true });
  const second = grid.getFileListThumb([{ ...outside }], { viewportBound: false, bustCache: true });
  await tick();
  assert.equal(grid.io.calls.length, 3);
  grid.finish(1);
  grid.finish(2);
  await Promise.all([first, second]);
  grid.dispose();
});


test('loaded memory thumbnails are checked once per viewport visit and rechecked after leaving', async () => {
  const grid = gridFixture();
  const file = grid.files[0];
  file.thumbnail = 'old-thumbnail';
  grid.memory.set(file.id, 'old-thumbnail');
  const first = grid.getFileListThumb([file]);
  await tick();
  assert.equal(grid.io.calls.length, 1);
  assert.equal(file.thumbnail, 'old-thumbnail');
  grid.finish(0);
  await first;
  await grid.getFileListThumb([file]);
  assert.equal(grid.io.calls.length, 1);
  grid.range.start = 200; grid.range.end = 204;
  grid.refreshThumbnailViewport();
  grid.range.start = 0; grid.range.end = 3;
  grid.refreshThumbnailViewport();
  const second = grid.getFileListThumb([file]);
  await tick();
  assert.equal(grid.io.calls.length, 2);
  grid.finish(1); await second;
  grid.dispose();
});

test('thumbnail decoding failures request one repair for protocol and base64 cache sources', async () => {
  const source = await readFile(new URL('../src/components/Thumbnail.vue', import.meta.url), 'utf8');
  const start = source.indexOf('function retryThumbnail()');
  const end = source.indexOf('// Robust ResizeObserver', start);
  assert.ok(start >= 0 && end > start);
  for (const src of ['thumb://localhost/lib/1', 'data:image/jpeg;base64,broken']) {
    const calls = [];
    const thumbnailSrc = { value: src };
    const build = new Function('thumbnailSrc', 'getThumbUrl', 'props', 'config', 'isThumbnailLoaded',
      `let thumbnailRetryCount = 0; ${source.slice(start, end)}; return retryThumbnail;`);
    const retry = build(thumbnailSrc, (...args) => { calls.push(args); return 'thumb://localhost/lib/1?t=123'; },
      { file: { id: 1, modified_at: 2 } }, { settings: { thumbnailSize: 512 } }, { value: true });
    retry(); retry();
    assert.equal(calls.length, 1);
    assert.match(thumbnailSrc.value, /repair=true/);
  }
});


test('explicit RAW policy changes request one replacement independently of viewport checks', async () => {
  const policies = [];
  const grid = gridFixture(false, (requests, io, options) => {
    policies.push(options.forceRegenerate);
    return io.load(requests);
  });
  const file = grid.files[0];
  Object.assign(file, { file_type: 3, thumbnail: 'old', rawThumbnailStale: true });
  grid.invalidateRawThumbnailPolicy();
  grid.memory.set(file.id, 'old');
  const replacement = grid.getFileListThumb([file]);
  await tick();
  assert.deepEqual(policies, [true]);
  grid.io.calls[0].finish([null]); // The backend queued the explicit replacement.
  await replacement;
  grid.memory.clear();
  const ready = grid.getFileListThumb([file], { bustCache: true });
  await tick();
  assert.deepEqual(policies, [true, false]);
  grid.finish(1);
  await ready;
  assert.equal(file.thumbnail, 'thumb:1');
  grid.dispose();
});

test('thumbnail-only repair clears frontend caches without fetching file info or reloading the original', async () => {
  const anchor = source.indexOf("unlistenThumbnailReady = await listen('thumbnail_ready'");
  const start = source.indexOf('async (event:', anchor);
  const end = source.indexOf('\n  });', start);
  assert.ok(anchor >= 0 && start > anchor && end > start);
  const callback = ts.transpile(`const handler = ${source.slice(start, end + 4)}; return handler;`,
    { target: ts.ScriptTarget.ES2022 });
  const file = { id: 1, file_path: '/original.jpg', thumbnail: 'corrupt', size: 12 };
  const cleared = [];
  const loaded = [];
  const revision = { value: 3 };
  const deps = {
    libConfig: { _libraryId: 'library' }, config: { settings: { thumbnailSize: 512 } },
    fileList: { value: [file] }, selectedItemIndex: { value: 0 },
    totalFileSize: { value: 12 }, fileInfoRevision: revision,
    clearCachedThumbnailDataUrl: (...args) => cleared.push(args),
    getFileListThumb: (...args) => loaded.push(args),
    getFileInfo: () => { throw new Error('Thumbnail repair must not fetch file info'); },
    nextTick: () => { throw new Error('Thumbnail repair must not reload the original'); },
  };
  const handler = new Function(...Object.keys(deps), callback)(...Object.values(deps));
  await handler({ payload: { library_id: 'library', file_ids: [1], invalidate: true, metadata_changed: false } });
  assert.deepEqual(cleared, [[1, 512]]);
  assert.equal(loaded.length, 1);
  assert.equal(loaded[0][0][0], file);
  assert.deepEqual(loaded[0][1], { bustCache: true });
  assert.equal(file.rawThumbnailStale, true);
  assert.equal(file.file_path, '/original.jpg');
  assert.equal(revision.value, 3);
  assert.equal(deps.totalFileSize.value, 12);
});


test('single-file Refresh rereads an already displayed thumbnail without regenerating it', async () => {
  const policies = [];
  const grid = gridFixture(false, (requests, io, options) => {
    policies.push(options.forceRegenerate);
    return io.load(requests);
  });
  const file = grid.files[0];
  const initial = grid.getFileListThumb([file]);
  await tick();
  grid.finish(0);
  await initial;
  assert.equal(file.thumbnail, 'thumb:1');
  file.modified_at = 2; // The refresh command already updated metadata and disk cache.
  const refreshed = grid.getFileListThumb([file], { bustCache: true, viewportBound: false });
  await tick();
  assert.equal(grid.io.calls.length, 2);
  grid.io.calls[1].finish([{ error_code: 0, id: file.id, url: 'updated-thumbnail' }]);
  await refreshed;
  assert.equal(file.thumbnail, 'updated-thumbnail');
  assert.equal(grid.memory.get(file.id), 'updated-thumbnail');
  assert.deepEqual(policies, [false, false]);
  grid.dispose();
});

test('RAW policy changes cover later albums and libraries once per file, including after row replacement', async () => {
  const policies = [];
  const grid = gridFixture(false, (requests, io, options) => {
    policies.push(options.forceRegenerate);
    return io.load(requests);
  });
  grid.invalidateRawThumbnailPolicy();
  for (const [id, library, expected] of [[1001, 'library-a', true], [1002, 'library-a', true],
    [1001, 'library-a', false], [1001, 'another-library', true]]) {
    grid.invalidateThumbnailRequests(); // Album navigation does not lose the policy revision.
    grid.library._libraryId = library;
    grid.files[0] = { id, file_type: 3, file_path: '/another-album/photo.raw', modified_at: 1 };
    const pending = grid.getFileListThumb([grid.files[0]]);
    await tick();
    assert.equal(policies.at(-1), expected);
    grid.finish(grid.io.calls.length - 1);
    await pending;
  }
  grid.invalidateRawThumbnailPolicy(); // A subsequent policy change applies again.
  const pending = grid.getFileListThumb([grid.files[0]]);
  await tick();
  assert.equal(policies.at(-1), true);
  grid.finish(grid.io.calls.length - 1);
  await pending;
  grid.dispose();
});

test('thumbnail events reject another library and stop after a queued switch', async () => {
  const start = source.indexOf('async (event: any) => {', source.indexOf("listen('thumbnail_ready'"));
  const end = source.indexOf('\n  });', start);
  assert.ok(start >= 0 && end > start);
  const code = ts.transpile(`const callback = ${source.slice(start, end + '\n  }'.length)};`);
  const config = { settings: { thumbnailSize: 512 } };
  const libConfig = { _libraryId: 'library-b' };
  const file = { id: 1, file_path: '/fixture.jpg' };
  const fileList = { value: [file] };
  let loads = 0;
  let resolveInfo;
  const callback = new Function('libConfig', 'fileList', 'config', 'clearCachedThumbnailDataUrl', 'getFileInfo', 'getFileListThumb', 'selectedItemIndex', 'nextTick', `${code}; return callback;`)(
    libConfig, fileList, config, () => {}, () => new Promise(resolve => { resolveInfo = resolve; }),
    () => { loads++; }, { value: 0 }, async () => {},
  );
  await callback({ payload: { library_id: 'library-a', file_ids: [1] } });
  assert.equal(loads, 0);
  assert.equal(resolveInfo, undefined);
  const queued = callback({ payload: { library_id: 'library-b', file_ids: [1] } });
  assert.equal(typeof resolveInfo, 'function');
  libConfig._libraryId = 'library-c';
  resolveInfo({ modified_at: 42 });
  await queued;
  assert.equal(file.modified_at, undefined);
  assert.equal(loads, 0);
});
