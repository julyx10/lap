import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createFocusInfoLoader } from '../src/common/focusInfo.js';
import { readFile } from 'node:fs/promises';

test('overlay and status share pending requests and cached unavailable results', async () => {
  let calls = 0;
  let resolve;
  const load = createFocusInfoLoader(() => { calls++; return new Promise(done => { resolve = done; }); });
  const image = load('image.jpg', 12, 1);
  const status = load('image.jpg', 12, 1);
  assert.equal(image, status);
  assert.equal(calls, 1);
  resolve(null);
  assert.equal(await image, null);
  assert.equal(await load('image.jpg', 12, 1), null);
  assert.equal(calls, 1);
});

test('AF cache isolates versions/libraries, expires, bounds size and retries errors', async () => {
  let time = 0;
  let calls = 0;
  const load = createFocusInfoLoader(async () => ++calls, { maxEntries: 2, ttl: 5, now: () => time });
  assert.equal(await load('a', 1, 1), 1);
  assert.equal(await load('a', 2, 1), 2);
  assert.equal(await load('a', 2, 2), 3);
  assert.equal(await load('a', 1, 1), 4);
  time = 6;
  assert.equal(await load('a', 1, 1), 5);
  let attempts = 0;
  const retry = createFocusInfoLoader(async () => { if (++attempts === 1) throw new Error('offline'); return null; });
  await assert.rejects(retry('a'));
  assert.equal(await retry('a'), null);
  assert.equal(attempts, 2);
});

test('both consumers use the same loader and modified_at version', async () => {
  const image = await readFile(new URL('../src/components/Image.vue', import.meta.url), 'utf8');
  const status = await readFile(new URL('../src/components/StatusBar.vue', import.meta.url), 'utf8');
  for (const source of [image, status]) {
    assert.match(source, /import \{ loadFocusInfo \} from '@\/common\/focusInfo.js'/);
    assert.doesNotMatch(source, /invoke.*get_focus_info/);
  }
  assert.match(image, /loadFocusInfo\(path, props.fileVersion, libConfig._libraryId\)/);
  assert.match(status, /loadFocusInfo\(path, currentFile.value\?\.modified_at \|\| 0, libConfig._libraryId\)/);
});
