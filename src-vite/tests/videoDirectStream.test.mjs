import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';

const source = fs.readFileSync(new URL('../src/components/Video.vue', import.meta.url), 'utf8');
const start = source.indexOf('  const loadDirect = async');
const code = ts.transpile(source.slice(start, source.indexOf('  // Matroska files', start)), { target: ts.ScriptTarget.ES2022 });

const utils = fs.readFileSync(new URL('../src/common/utils.ts', import.meta.url), 'utf8');
const mediaUrlStart = utils.indexOf('let originalMediaBaseUrl');
const mediaUrlCode = ts.transpile(
  utils.slice(mediaUrlStart, utils.indexOf('// get country name', mediaUrlStart)).replaceAll('export ', ''),
  { target: ts.ScriptTarget.ES2022 },
);
const mediaBase = 'http://127.0.0.1:54321/media/token/';
const media = new Function('invoke', 'useUIStore', `
  ${mediaUrlCode}
  return { initializeOriginalMedia, getAssetSrc };
`)(
  async () => mediaBase,
  () => ({ getFileVersion: () => 2 }),
);
await media.initializeOriginalMedia();

function setup(filePath) {
  const calls = [], listeners = {}, timers = [];
  const player = {
    reset() { calls.push('reset'); },
    src(value) { calls.push(value); },
    one(name, fn) { listeners[name] = fn; },
    off(name) { delete listeners[name]; },
    load() { calls.push('load'); },
    videoWidth: () => 1920,
    videoHeight: () => 1080,
  };
  const create = new Function('filePath', 'getAssetSrc', 'player', 'resetCompatibilityProcessing', 'loadPrepared', 'handleSuccessfulLoad', 'loadAttemptCleanups', 'setTimeout', 'clearTimeout', 'DIRECT_PLAYBACK_TIMEOUT_MS', `
    let currentLoadId = 1, currentLoadingId = 1, nextUpIndex = 0;
    ${code}
    return { load: loadDirect };
  `);
  const controls = create(filePath, media.getAssetSrc, player, () => {}, mode => calls.push(['prepare', mode]), () => calls.push('success'), [], fn => { timers.push(fn); return timers.length; }, () => {}, 8000);
  return { ...controls, calls, listeners, timers };
}

test('direct playback uses the local media transport and retains fallback', async () => {
  for (const path of ['/nas/clip.mp4', '/Volumes/clip.mov', 'C:\\Videos\\clip.webm']) {
    const t = setup(path);
    await t.load(null);
    assert.deepEqual(t.calls[1], { src: `${mediaBase}${encodeURIComponent(path)}?u=2` });
    assert.ok(!t.calls.some(c => c?.[0] === 'prepare'));
    t.listeners.error();
    t.timers[0]();
    assert.equal(t.calls.filter(c => c?.[0] === 'prepare').length, 1);
  }
});

test('original image URLs preserve special characters and cache versions', () => {
  const path = '/Photos/图 + #?.png';
  const url = new URL(media.getAssetSrc(path, 17));
  assert.equal(decodeURIComponent(url.pathname.split('/').at(-1)), path);
  assert.equal(url.searchParams.get('v'), '17');
  assert.equal(url.searchParams.get('u'), '2');
});

test('direct timeout retains compatibility fallback while a loaded frame settles the attempt', async () => {
  const stalled = setup('/clip.mp4');
  await stalled.load('video/mp4');
  stalled.timers[0]();
  assert.deepEqual(stalled.calls.at(-1), ['prepare', 'compatible']);

  const loaded = setup('/clip.mp4');
  await loaded.load('video/mp4');
  loaded.listeners.loadeddata();
  loaded.timers[0]();
  assert.equal(loaded.calls.at(-1), 'success');
  assert.ok(!loaded.calls.some(c => c?.[0] === 'prepare'));
});
