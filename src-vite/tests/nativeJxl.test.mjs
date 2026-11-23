import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
const source = fs.readFileSync(new URL('../src/common/nativeJxl.ts', import.meta.url), 'utf8');
function fixture() {
  const js = ts.transpile(source.replace(/export /g, ''), { target: ts.ScriptTarget.ES2022 });
  return new Function(`${js}; return loadNativeJxl;`)();
}
const image = src => ({ src, naturalWidth: 1500, naturalHeight: 1000 });
test('native success avoids fallback', async () => {
  const load = fixture();
  assert.deepEqual(await load('original', () => assert.fail('unexpected fallback'), async src => image(src)), image('original'));
});
test('failure falls back once and remembers only that versioned file', async () => {
  const load = fixture(); const calls = [];
  const decode = async src => { calls.push(src); if (src.startsWith('original')) throw Error('unsupported'); return image(src); };
  await load('original?v=1', () => 'preview1', decode);
  await load('original?v=1', () => 'preview1', decode);
  await load('original?v=2', () => 'preview2', decode);
  assert.deepEqual(calls, ['original?v=1', 'preview1', 'preview1', 'original?v=2', 'preview2']);
});
test('fallback failure is surfaced without a retry loop', async () => {
  const load = fixture(); let calls = 0;
  await assert.rejects(load('original', () => 'preview', async () => { calls++; throw Error('bad image'); }));
  assert.equal(calls, 2);
});
test('one rejected file does not disable native decoding for another', async () => {
  const load = fixture();
  await load('bad', () => 'preview', async src => { if (src === 'bad') throw Error('bad'); return image(src); });
  assert.equal((await load('good', () => assert.fail('unexpected fallback'), async src => image(src))).src, 'good');
});


test('a decode rejection after load falls back to a decoded preview', async () => {
  class FakeImage {
    naturalWidth = 1500;
    naturalHeight = 1000;
    set src(value) { this.url = value; queueMicrotask(() => this.onload()); }
    decode() { return this.url === 'original' ? Promise.reject(Error('decode failed')) : Promise.resolve(); }
  }
  const js = ts.transpile(source.replace(/export /g, ''), { target: ts.ScriptTarget.ES2022 });
  const { loadNativeJxl, loadBrowserImage } = new Function('Image', `${js}; return { loadNativeJxl, loadBrowserImage };`)(FakeImage);
  assert.deepEqual(await loadNativeJxl('original', () => 'preview', loadBrowserImage), image('preview'));
});


test('a late thumbnail cannot replace a completed native or fallback image', async () => {
  const imageSource = fs.readFileSync(new URL('../src/components/Image.vue', import.meta.url), 'utf8');
  const start = imageSource.indexOf('watch(displayThumbnailSrc, async');
  const end = imageSource.indexOf('\n});', start) + 4;
  const js = ts.transpile(imageSource.slice(start, end), { target: ts.ScriptTarget.ES2022 });
  for (const completed of ['original', 'preview']) {
    const src = { value: ['thumbnail', ''] };
    let finish;
    let callback;
    const globals = {
      watch: (_source, fn) => { callback = fn; }, displayThumbnailSrc: {},
      props: { originalUnavailable: false, fileType: 1, filePath: 'test.jxl', fileId: 1, fileVersion: 1 },
      offlinePreview: { value: false }, currentLoadingId: { value: 1 },
      shouldUseBackendPreview: () => true, getFfmpegBackedPreviewExtensions: async () => new Set(),
      getFileExtension: () => 'jxl', activeImage: { value: 0 }, imageSrc: src,
      getPreviewUrl: () => 'preview', getAssetSrc: () => 'original',
      loadPlaceholderResource: () => new Promise(resolve => { finish = resolve; }),
      getCompatibleLayout: () => { throw Error('must not downgrade'); },
      console: { error: error => { throw error; } },
    };
    new Function(...Object.keys(globals), js)(...Object.values(globals));
    const pending = callback('new-thumbnail');
    await Promise.resolve();
    src.value[0] = completed;
    finish({ naturalWidth: 150, naturalHeight: 100 });
    await pending;
    assert.equal(src.value[0], completed);
  }
});
