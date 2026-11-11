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
