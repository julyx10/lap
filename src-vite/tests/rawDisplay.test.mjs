import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
import { computed, ref } from 'vue';

const source = fs.readFileSync(new URL('../src/common/rawDisplay.ts', import.meta.url), 'utf8');
const js = ts.transpile(source.replace(/^import .*;$/gm, '').replace(/export /g, ''), { target: ts.ScriptTarget.ES2022 });
const settings = { rawPreviewSource: 'embedded', groupRawJpegPairs: false, rawPairDisplaySource: 'jpeg' };
const { getRawDisplayOptions, rawDisplayKey, appendRawDisplayParams } = new Function('useConfigStore', js + '; return { getRawDisplayOptions, rawDisplayKey, appendRawDisplayParams };')(() => ({ settings }));

test('preview modes generate three distinct cache keys and request policies', () => {
  const keys = new Set();
  for (const source of ['embedded', 'rendered', 'brightened']) {
      settings.rawPreviewSource = source;
      keys.add(rawDisplayKey());
      const params = new URLSearchParams();
      appendRawDisplayParams(params);
      assert.equal(params.get('rawPreviewMode'), source === 'embedded' ? 'embedded' : 'rendered');
      assert.equal(params.get('rawAutoBright'), String(source === 'brightened'));
  }
  assert.equal(keys.size, 3);
});

test('local RAW override removes paired JPEG preference without changing settings', () => {
  settings.groupRawJpegPairs = true;
  const before = { ...settings };
  const params = new URLSearchParams('rawPairDisplay=jpeg');
  appendRawDisplayParams(params, { mode: 'rendered', autoBright: false, preferPair: false });
  assert.equal(params.get('rawPairDisplay'), 'raw');
  assert.deepEqual(settings, before);
  assert.equal(getRawDisplayOptions().preferPair, true);
});

const imageSource = fs.readFileSync(new URL('../src/components/Image.vue', import.meta.url), 'utf8');
const selectionSource = ts.transpile(imageSource.slice(imageSource.indexOf('const rawModeSource ='), imageSource.indexOf('let rawAbortController')), { target: ts.ScriptTarget.ES2022 });

test('direct RAW buttons override the paired preview and respect embedded availability', () => {
  const defaults = { mode: 'embedded', autoBright: false, preferPair: true };
  const rawOverride = ref(null);
  const rawSource = ref('pair');
  const rawEmbeddedUnavailable = ref(true);
  const rawRequestPending = ref(false);
  const requestedRawOptions = computed(() => rawOverride.value || defaults);
  const { selectRawMode, selectRawPair } = new Function(
    'computed', 'ref', 'rawOverride', 'rawSource', 'rawRequestPending', 'rawSelectionVersion',
    'rawEmbeddedUnavailable', 'requestedRawOptions', 'getRawDisplayOptions',
    'IconEmbedded', 'IconRender', 'IconBrightness',
    selectionSource + '; return { selectRawMode, selectRawPair };',
  )(computed, ref, rawOverride, rawSource, rawRequestPending, ref(0), rawEmbeddedUnavailable,
    requestedRawOptions, () => defaults, {}, {}, {});
  selectRawMode('embedded');
  assert.equal(rawOverride.value, null);
  selectRawMode('brightened');
  assert.deepEqual(rawOverride.value, { mode: 'rendered', autoBright: true, preferPair: false });
  selectRawPair();
  assert.deepEqual(rawOverride.value, { mode: 'rendered', autoBright: true, preferPair: true });
});

const loaderSource = ts.transpile(imageSource.slice(imageSource.indexOf('async function loadRawImage'), imageSource.indexOf('watch(() => [scale.value')), { target: ts.ScriptTarget.ES2022 });

test('a superseded RAW response cannot replace the latest source or leak a blob URL', async () => {
  const requests = [];
  const urls = new Set();
  const load = new Function('fetch', 'props', 'getPreviewUrl', 'appendRawDisplayParams', 'requestedRawOptions', 'Image', 'rawObjectUrls', `let rawAbortController = null; const rawViewerId = 'test-viewer'; ${loaderSource}; return loadRawImage;`)(
    (_url, options) => new Promise(resolve => requests.push({ resolve, signal: options.signal })),
    { fileId: 42, fileVersion: 1 }, () => 'http://preview.localhost/test/42', appendRawDisplayParams,
    { value: { mode: 'rendered', autoBright: true, preferPair: false } },
    class { naturalWidth = 100; naturalHeight = 100; async decode() {} }, urls,
  );
  const older = load('photo.dng');
  const rejected = assert.rejects(older, /cancelled/);
  const newer = load('photo.dng');
  assert.equal(requests[0].signal.aborted, true);
  const response = () => ({ ok: true, blob: async () => new Blob(['image']), headers: new Headers({ 'X-Raw-Source': 'brightened', 'X-Raw-Embedded-Unavailable': 'true', 'X-Raw-Pair': 'JPEG' }) });
  requests[1].resolve(response());
  const latest = await newer;
  requests[0].resolve(response());
  await rejected;
  // preferPair is false: a response header must not expose a paired JPEG.
  assert.deepEqual(latest.raw, { source: 'brightened', unavailable: true, pair: null });
  assert.equal(urls.size, 1);
  URL.revokeObjectURL(latest.src);
});

const utilsSource = fs.readFileSync(new URL('../src/common/utils.ts', import.meta.url), 'utf8');
const previewUrlSource = ts.transpile(utilsSource.slice(utilsSource.indexOf('export function getPreviewUrl('), utilsSource.indexOf('export function shouldUseBackendPreview(')).replace('export ', ''), { target: ts.ScriptTarget.ES2022 });

test('editor URL selects the export pipeline regardless of RAW browsing settings', () => {
  const getPreviewUrl = new Function('isWin', '_thumbLibraryId', 'useUIStore', 'appendRawDisplayParams', previewUrlSource + '; return getPreviewUrl;')(
    false, 'test-library', () => ({ getFileVersion: () => 7 }), appendRawDisplayParams,
  );
  const urls = new Set();
  for (const source of ['embedded', 'rendered', 'brightened']) {
      settings.rawPreviewSource = source;
      settings.groupRawJpegPairs = true;
      const url = getPreviewUrl(42, 'photo.dng', false, 3, true);
      urls.add(url);
      const params = new URL(url).searchParams;
      assert.equal(params.get('forEditing'), 'true');
      assert.equal(params.has('rawPreviewMode'), false);
      assert.equal(params.has('rawAutoBright'), false);
      assert.equal(params.has('rawPairDisplay'), false);
      assert.equal(params.get('v'), '3');
      assert.equal(params.get('u'), '7');
  }
  assert.equal(urls.size, 1);
  const browsing = new URL(getPreviewUrl(42, 'photo.dng')).searchParams;
  assert.equal(browsing.has('forEditing'), false);
  assert.equal(browsing.get('rawPairDisplay'), 'jpeg');
});
