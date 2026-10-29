import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { orientedFocusRect } from '../src/common/focusPoint.js';
import { ref, computed, reactive, watch, effectScope } from 'vue';
import ts from 'typescript';

const focus = { x: 0.25, y: 0.4, width: 0.2, height: 0.1 };

// Execute the actual component's focus watcher with Vue reactivity and deferred IPC.
async function focusWatcherHarness() {
  const source = await readFile(new URL('../src/components/Image.vue', import.meta.url), 'utf8');
  const start = source.indexOf('const showFocus = computed(');
  const end = source.indexOf('onBeforeUnmount(() => { focusRequest++; });', start);
  assert.ok(start >= 0 && end > start);
  const code = ts.transpileModule(source.slice(start, end) + '\nreturn { showFocus, focusRect, focusResult, focusResultClass, focusState, focusImageReady, showFocusOverlay };',
    { compilerOptions: { target: ts.ScriptTarget.ES2022 } }).outputText;
  const props = reactive({ filePath: 'old.jpg', fileVersion: 0, fileType: 1, originalUnavailable: false, isSlideShow: false });
  const imageFilePath = ref(['old.jpg']);
  const activeImage = ref(0);
  const loadError = ref(false);
  const offlinePreview = ref(false);
  const requests = [];
  const invoke = (command, args) => new Promise((resolve, reject) => requests.push({ command, path: args.path, resolve, reject }));
  const scope = effectScope();
  const config = reactive({ settings: { showFocus: false } });
  const loadFocusInfo = path => invoke('get_focus_info', { path });
  const state = scope.run(() => new Function('ref', 'computed', 'watch', 'props', 'imageFilePath', 'activeImage', 'loadError', 'offlinePreview', 'loadFocusInfo', 'orientedFocusRect', 'config', 'libConfig', code)(
    ref, computed, watch, props, imageFilePath, activeImage, loadError, offlinePreview, loadFocusInfo, orientedFocusRect, config, { _libraryId: 1 }));
  const showFocus = computed({ get: () => config.settings.showFocus, set: value => { config.settings.showFocus = value; } });
  return { ...state, showFocus, config, props, imageFilePath, requests, stop: () => scope.stop() };
}

test('focus preference survives serialization and defaults off for existing configurations', async () => {
  const source = await readFile(new URL('../src/stores/configStore.js', import.meta.url), 'utf8');
  const start = source.indexOf('deserialize: (value) => {') + 'deserialize: '.length;
  const end = source.indexOf('\n      },', start) + '\n      }'.length;
  const deserialize = new Function(`return (${source.slice(start, end)});`)();
  assert.equal(deserialize(JSON.stringify({ settings: {} })).settings.showFocus, false);
  for (const showFocus of [true, false]) {
    assert.equal(deserialize(JSON.stringify({ settings: { showFocus } })).settings.showFocus, showFocus);
  }
});

const settle = async () => { await Promise.resolve(); await Promise.resolve(); };
test('maps encoded coordinates through all EXIF orientations', () => {
  const expected = [
    [0.25, 0.4, 0.2, 0.1], [0.75, 0.4, 0.2, 0.1],
    [0.75, 0.6, 0.2, 0.1], [0.25, 0.6, 0.2, 0.1],
    [0.4, 0.25, 0.1, 0.2], [0.6, 0.25, 0.1, 0.2],
    [0.6, 0.75, 0.1, 0.2], [0.4, 0.75, 0.1, 0.2],
  ];
  expected.forEach((values, i) => assert.deepEqual(Object.values(orientedFocusRect({ focus, orientation: i + 1 })), values));
});
test('rejects absent, invalid, out-of-bounds and unknown data', () => {
  for (const info of [null, { focus: null }, { focus: { ...focus, x: NaN } },
    { focus: { ...focus, width: 0 } }, { focus: { ...focus, x: 1 } }, { focus, orientation: 9 }]) {
    assert.equal(orientedFocusRect(info), null);
  }
});
test('overlay follows the image transform and guards source races and RAW crops', async () => {
  const source = await readFile(new URL('../src/components/Image.vue', import.meta.url), 'utf8');
  assert.match(source, /const style = getImageStyle\(activeImage.value\)/);
  assert.match(source, /imageFilePath.value\[activeImage.value\] === props.filePath/);
  assert.match(source, /request !== focusRequest/);
  assert.doesNotMatch(source, /unavailable \|\| type === 3/);
  assert.match(source, /const showFocus = computed\(\(\) => config.settings.showFocus === true\)/);
  assert.doesNotMatch(source, /role="status"/);
  assert.match(source, /focusImageReady.value && !props.isSlideShow/);
  assert.match(source, /focusImageReady.value = false/);
  assert.match(source, /focusImageReady.value = true/);
  assert.match(source, /2 \/ Math.max\(scale.value\[activeImage.value\]/);
  assert.doesNotMatch(source, />AF<\/button>/);
  const toolbar = await readFile(new URL('../src/components/MediaViewer.vue', import.meta.url), 'utf8');
  assert.match(toolbar, /text="AF"/);
  assert.match(toolbar, /:selected="config.settings.showFocus"/);
  assert.match(toolbar, /settings-showFocus-changed/);
});

test('merged RAW primary requests AF metadata even when displaying its JPEG', async () => {
  const h = await focusWatcherHarness();
  try {
    h.props.filePath = 'paired.NEF';
    h.props.fileType = 3;
    h.imageFilePath.value = ['paired.NEF'];
    h.focusImageReady.value = true;
    h.showFocus.value = true;
    assert.equal(h.requests[0].path, 'paired.NEF');
    h.requests[0].resolve({ focus, focus_result: 'focus', orientation: 1 });
    await settle();
    assert.deepEqual(h.focusRect.value, focus);
    assert.equal(h.showFocusOverlay.value, true);
  } finally { h.stop(); }
});

test('late old focus response cannot overwrite the new image rectangle or color', async () => {
  const h = await focusWatcherHarness();
  try {
    h.focusImageReady.value = true;
    h.showFocus.value = true;
    assert.equal(h.requests[0].path, 'old.jpg');
    h.props.filePath = 'new.jpg';
    assert.equal(h.focusRect.value, null);
    assert.equal(h.focusResultClass.value, 'border-gray-400');
    assert.equal(h.showFocusOverlay.value, false);
    h.imageFilePath.value = ['new.jpg'];
    const newRect = { ...focus, x: 0.6 };
    h.requests[1].resolve({ focus: newRect, focus_result: 'focus', orientation: 1 });
    await settle();
    assert.deepEqual(h.focusRect.value, newRect);
    assert.equal(h.focusResultClass.value, 'border-green-500');
    assert.equal(h.showFocusOverlay.value, true);
    h.requests[0].resolve({ focus, focus_result: 'out_of_focus', orientation: 1 });
    await settle();
    assert.deepEqual(h.focusRect.value, newRect);
    assert.equal(h.focusResultClass.value, 'border-green-500');
    assert.equal(h.showFocusOverlay.value, true);
  } finally { h.stop(); }
});

test('disabling focus invalidates a pending response and clears rectangle and color', async () => {
  const h = await focusWatcherHarness();
  try {
    h.focusImageReady.value = true;
    h.showFocus.value = true;
    h.showFocus.value = false;
    h.requests[0].resolve({ focus, focus_result: 'out_of_focus', orientation: 1 });
    await settle();
    assert.equal(h.focusRect.value, null);
    assert.equal(h.focusResultClass.value, 'border-gray-400');
    assert.equal(h.focusState.value, 'idle');
    assert.equal(h.showFocusOverlay.value, false);
  } finally { h.stop(); }
});

test('focus result colors and accessible labels are tied to the current overlay', async () => {
  const source = await readFile(new URL('../src/components/Image.vue', import.meta.url), 'utf8');
  assert.match(source, /focus: 'border-green-500', out_of_focus: 'border-red-500', unknown: 'border-gray-400'/);
  assert.match(source, /info\?\.focus_result === 'focus' \|\| info\?\.focus_result === 'out_of_focus' \? info.focus_result : 'unknown'/);
  assert.match(source, /focusResult.value = 'unknown'/);
  assert.match(source, /request !== focusRequest\) return;[\s\S]*focusResult.value = info/);
  assert.match(source, /flush: 'sync'/);
  const statusBar = await readFile(new URL('../src/components/StatusBar.vue', import.meta.url), 'utf8');
  assert.match(statusBar, /role="status" data-focus-status/);
  assert.match(statusBar, /onCleanup\(\(\) => \{ stale = true;/);
  assert.match(source, /data-focus-overlay aria-hidden="true"/);
  for (const locale of ['en', 'zh-CN']) {
    const messages = JSON.parse(await readFile(new URL(`../src/locales/${locale}.json`, import.meta.url), 'utf8'));
    for (const result of ['focus', 'out_of_focus', 'unknown']) {
      assert.ok(messages.image_viewer[`focus_result_${result}`]);
    }
  }
});
