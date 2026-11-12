import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';

const source = fs.readFileSync(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
const start = source.indexOf('async function handleImportFilesAdded(');
const end = source.indexOf('// New event handlers for GridView', start);
assert.ok(start >= 0 && end > start);
const code = ts.transpile(source.slice(start, end), { target: ts.ScriptTarget.ES2022 });

function fixture(groupRawJpegPairs) {
  const calls = [];
  const config = { settings: { groupRawJpegPairs } };
  const libConfig = { _libraryId: 'library-a' };
  const handler = new Function(
    'config', 'libConfig', 'reconcileRawJpegPairs', 'refreshImportedAlbumContent',
    `${code}; return handleImportFilesAdded;`,
  )(
    config,
    libConfig,
    async (libraryId, enabled) => calls.push(['reconcile', libraryId, enabled]),
    async albumId => calls.push(['refresh', albumId]),
  );
  return { calls, config, libConfig, handler };
}

test('imported files reconcile RAW pairs before refreshing the album when grouping is enabled', async () => {
  const f = fixture(true);
  await f.handler({ payload: { albumId: 12 } });
  assert.deepEqual(f.calls, [['reconcile', 'library-a', true], ['refresh', 12]]);
});

test('imported files do not reconcile RAW pairs when grouping is disabled', async () => {
  const f = fixture(false);
  await f.handler({ payload: { albumId: 12 } });
  assert.deepEqual(f.calls, [['refresh', 12]]);
});

test('a library switch during reconciliation skips the stale album refresh', async () => {
  const calls = [];
  const config = { settings: { groupRawJpegPairs: true } };
  const libConfig = { _libraryId: 'library-a' };
  const handler = new Function(
    'config', 'libConfig', 'reconcileRawJpegPairs', 'refreshImportedAlbumContent',
    `${code}; return handleImportFilesAdded;`,
  )(
    config,
    libConfig,
    async (libraryId) => {
      calls.push(['reconcile', libraryId]);
      libConfig._libraryId = 'library-b';
    },
    async albumId => calls.push(['refresh', albumId]),
  );
  await handler({ payload: { albumId: 12 } });
  assert.deepEqual(calls, [['reconcile', 'library-a']]);
});

const dragStart = source.indexOf('async function importDroppedFiles(');
const dragEnd = source.indexOf('async function refreshImportedFiles(', dragStart);
assert.ok(dragStart >= 0 && dragEnd > dragStart);
const dragCode = ts.transpile(source.slice(dragStart, dragEnd), { target: ts.ScriptTarget.ES2022 });

test('drag-and-drop paths are deduplicated and imported through the batch pair importer', async () => {
  const calls = [];
  const importDroppedFiles = new Function(
    'importFiles', 'libConfig', 'console',
    `${dragCode}; return importDroppedFiles;`,
  )(
    async (...args) => {
      calls.push(args);
      return [['/drop/b.jpg', 'failed']];
    },
    { _libraryId: 'library-a' },
    { error() {} },
  );

  assert.equal(await importDroppedFiles(['/drop/a.jpg', '/drop/a.jpg', '/drop/b.jpg'], 9, '/album'), 1);
  assert.deepEqual(calls, [[['/drop/a.jpg', '/drop/b.jpg'], 9, '/album', 'library-a']]);
  assert.equal(await importDroppedFiles([], 9, '/album'), 0);
});

test('drag-and-drop reports zero when the batch importer fails', async () => {
  const importDroppedFiles = new Function(
    'importFiles', 'libConfig', 'console',
    `${dragCode}; return importDroppedFiles;`,
  )(
    async () => { throw new Error('library changed'); },
    { _libraryId: 'library-a' },
    { error() {} },
  );

  assert.equal(await importDroppedFiles(['/drop/a.jpg'], 9, '/album'), 0);
});

const apiSource = fs.readFileSync(new URL('../src/common/api.js', import.meta.url), 'utf8');
const apiStart = apiSource.indexOf('export function importFiles(');
const apiEnd = apiSource.indexOf('export async function importFile(', apiStart);
assert.ok(apiStart >= 0 && apiEnd > apiStart);
const importFilesCode = ts.transpile(
  apiSource.slice(apiStart, apiEnd).replace(/export /g, ''),
  { target: ts.ScriptTarget.ES2022 },
);

test('importFiles always installs a channel callback even without a progress listener', async () => {
  const calls = [];
  class FakeChannel {
    constructor(onmessage) { this.onmessage = onmessage; }
  }
  const importFiles = new Function('Channel', 'invoke', 'config', `${importFilesCode}; return importFiles;`)(
    FakeChannel,
    async (command, args) => { calls.push([command, args]); return []; },
    { settings: { groupRawJpegPairs: true } },
  );

  await importFiles(['/drop/a.jpg'], 1, '/album', 'library-a');
  assert.equal(typeof calls[0][1].onProgress.onmessage, 'function');
  assert.equal(calls[0][1].groupRawJpegPairs, true);

  let progress;
  await importFiles(['/drop/b.jpg'], 1, '/album', 'library-a', value => { progress = value; });
  assert.equal(typeof calls[1][1].onProgress.onmessage, 'function');
  calls[1][1].onProgress.onmessage(2);
  assert.equal(progress, 2);
});
