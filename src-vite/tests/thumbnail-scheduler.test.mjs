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
  const api = new Function('createThumbnailScheduler', 'config', 'libConfig', 'getFileThumbs', 'gridRows',
    'visibleItemCount', 'lastVisibleRange', 'isItemRow', 'isRealFileItem', 'onBeforeUnmount',
    'getCachedThumbnailDataUrl', 'getThumbnailDataUrl', 'thumbnailPlaceholder', 'console',
    adapter + '; return { getFileListThumb, invalidateThumbnailRequests, refreshThumbnailViewport };')(
    createThumbnailScheduler, { settings: { thumbnailSize: 512 } }, libConfig,
    requests => loadThumbs ? loadThumbs(requests, io) : io.load(requests), rows, { value: 4 }, range, row => row?.type === 'item',
    file => file && !file.isPlaceholder && typeof file.id === 'number', callback => { dispose = callback; },
    id => memory.get(id), thumb => { memory.set(thumb.id, thumb.url); return thumb.url; },
    'placeholder', { log(...args) { errors.push(args); } });
  const finish = index => io.calls[index].finish(io.calls[index].payloads.map(p => ({
    error_code: 0, id: p.fileId, url: `thumb:${p.fileId}`,
  })));
  return { ...api, io, files, rows, range, memory, errors, libConfig, finish, dispose: () => dispose() };
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
