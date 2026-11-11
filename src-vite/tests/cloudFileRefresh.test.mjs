import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import ts from 'typescript';
import { ref } from 'vue';

const source = fs.readFileSync(new URL('../src/components/RefreshFileInfoDialog.vue', import.meta.url), 'utf8');
const body = ts.transpile(source.split('<script setup lang="ts">')[1].split('</script>')[0]
  .replace(/^import .*;\n/gm, ''), { target: ts.ScriptTarget.ES2022 });
function fixture(responses, options = {}) {
  let responseIndex = 0;
  let mounted;
  const refreshed = [];
  const calls = [];
  const props = { fileIds: options.fileIds ?? responses.map((_, i) => i + 1), libraryId: 'test', freeSpace: options.freeSpace ?? true,
    finishRefresh: async results => refreshed.push(...results) };
  const deps = {
    ref, defineProps: () => props, defineEmits: () => () => {},
    onMounted: callback => { mounted = callback; }, onBeforeUnmount: () => {}, watch: () => {},
    invoke: async (command, args) => {
      calls.push({ command, args });
      if (command === 'begin_file_refresh_batch') {
        if (options.beginError) throw options.beginError;
        return 42;
      }
      if (command === 'end_file_refresh_batch') return;
      const response = responses[responseIndex++];
      if (response instanceof Error) throw response;
      options.afterRefresh?.(args);
      return response;
    },
    config: { settings: { thumbnailSize: 512 } }, libConfig: { _libraryId: 'test' },
    getRawDisplayOptions: () => ({}),
    useUIStore: () => ({ pushInputHandler() {}, removeInputHandler() {} }),
    window: { addEventListener() {}, removeEventListener() {} }, console: { error() {} },
  };
  const state = new Function(...Object.keys(deps), body +
    '; return { processed, succeeded, skipped, failed, warned, releaseRequested, keptLocal, running, cancelled };')(...Object.values(deps));
  return { state, refreshed, calls, run: () => mounted() };
}
const result = (id, extra = {}) => ({ file: { id, cloud_only: false, metadata_pending: false },
  size_delta: 10, skipped: false, release_requested: false, warning: null, error: null, ...extra });

test('accepted asynchronous release requests are reported without claiming the file is already cloud-only', async () => {
  const updated = result(1, { release_requested: true });
  const f = fixture([updated]);
  await f.run();
  assert.equal(f.state.releaseRequested.value, 1);
  assert.equal(f.state.keptLocal.value, 0);
  assert.equal(f.state.succeeded.value, 1);
  assert.equal(f.refreshed[0].file.cloud_only, false);
});

test('thumbnail failure still refreshes the downloaded file, metadata and size delta', async () => {
  const updated = result(1, { error: 'Thumbnail could not be cached' });
  const f = fixture([updated]);
  await f.run();
  assert.deepEqual(f.refreshed, [updated]);
  assert.equal(f.state.failed.value, 1);
  assert.equal(f.state.succeeded.value, 0);
  assert.equal(f.state.keptLocal.value, 1);
});

test('a mixed batch retains partial updates and continues after command and release failures', async () => {
  const partial = result(2, { error: 'Release refused' });
  const complete = result(3, { release_requested: true, file: { id: 3, cloud_only: true } });
  const f = fixture([new Error('Download failed'), partial, complete]);
  await f.run();
  assert.deepEqual(f.refreshed, [partial, complete]);
  assert.equal(f.state.processed.value, 3);
  assert.equal(f.state.failed.value, 2);
  assert.equal(f.state.succeeded.value, 1);
  assert.equal(f.state.running.value, false);
  assert.equal(f.calls[0].command, 'begin_file_refresh_batch');
  assert.deepEqual(f.calls.at(-1), { command: 'end_file_refresh_batch', args: { batchId: 42 } });
  assert.ok(f.calls.slice(1, -1).every(call => call.command === 'index_cloud_file' && call.args.batchId === 42));
});

test('normal refresh forwards the shared index and releases it after completion', async () => {
  const f = fixture([result(1), new Error('Refresh failed')], { freeSpace: false });
  await f.run();
  assert.ok(f.calls.filter(call => call.command === 'refresh_selected_file_info').every(call => call.args.batchId === 42));
  assert.deepEqual(f.calls.at(-1), { command: 'end_file_refresh_batch', args: { batchId: 42 } });
  assert.equal(f.state.failed.value,1);
});
test('cancellation releases the batch without refreshing any files', async () => {
  const f = fixture([result(1)]);
  f.state.cancelled.value = true;
  await f.run();
  assert.deepEqual(f.calls.map(call => call.command), ['begin_file_refresh_batch','end_file_refresh_batch']);
  assert.equal(f.state.running.value,false);
});
test('failure to begin a batch is surfaced and does not invoke refresh commands', async () => {
  const f = fixture([result(1)], { beginError: new Error('Library changed') });
  await f.run();
  assert.deepEqual(f.calls.map(call => call.command), ['begin_file_refresh_batch']);
  assert.equal(f.state.failed.value, 1);
  assert.equal(f.state.running.value,false);
});

test('normal refresh immediately refreshes every selected file once without a staleness check', async () => {
  const f = fixture([result(1), result(2)], { freeSpace: false, fileIds: [1, 1, 2] });
  await f.run();
  const updates = f.calls.filter(call => call.command === 'refresh_selected_file_info');
  assert.deepEqual(updates.map(call => call.args.fileId), [1, 2]);
  assert.ok(updates.every(call => call.args.thumbnailSize === 512 && !('rebuildPreviews' in call.args)));
  assert.deepEqual(f.calls.map(call => call.command), [
    'begin_file_refresh_batch', 'refresh_selected_file_info', 'refresh_selected_file_info', 'end_file_refresh_batch',
  ]);
  assert.equal(f.state.succeeded.value, 2);
});

test('cancellation between files retains completed results and releases the batch', async () => {
  const f = fixture([result(1), result(2)], {
    freeSpace: false, afterRefresh: () => { f.state.cancelled.value = true; },
  });
  await f.run();
  assert.deepEqual(f.refreshed.map(result => result.file.id), [1]);
  assert.equal(f.state.processed.value, 1);
  assert.equal(f.state.running.value, false);
  assert.deepEqual(f.calls.at(-1), { command: 'end_file_refresh_batch', args: { batchId: 42 } });
});

test('skipped originals retain their metadata result without counting as updated', async () => {
  const unchanged = result(1, { skipped: true, file: { id: 1, cloud_only: true, metadata_pending: true } });
  const f = fixture([unchanged], { freeSpace: false });
  await f.run();
  assert.equal(f.state.skipped.value, 1);
  assert.equal(f.state.succeeded.value, 0);
  assert.equal(f.state.failed.value, 0);
  assert.deepEqual(f.refreshed, [unchanged]);
});

test('XMP failure keeps extracted metadata and pending state as a warning', async () => {
  const partial = result(1, { warning: 'XMP import failed: Malformed XML',
    file: { id: 1, icc_profile: 'Display P3', metadata_pending: true } });
  const f = fixture([partial], { freeSpace: false });
  await f.run();
  assert.equal(f.state.succeeded.value, 0);
  assert.equal(f.state.failed.value, 0);
  assert.equal(f.state.warned.value, 1);
  assert.deepEqual(f.refreshed, [partial]);
});

test('cloud-only XMP warnings are reported while retaining skipped metadata and pending state', async () => {
  const partial = result(1, { skipped: true, warning: 'XMP import failed: Malformed XML',
    file: { id: 1, cloud_only: true, metadata_pending: true, last_scan_time: 12345 } });
  const f = fixture([partial], { freeSpace: false });
  await f.run();
  assert.equal(f.state.failed.value, 0);
  assert.equal(f.state.warned.value, 1);
  assert.equal(f.state.succeeded.value, 0);
  assert.deepEqual(f.refreshed, [partial]);
});


test('XMP warnings keep downloaded files local and are separate from failures', async () => {
  const partial = result(1, { warning: 'XMP import failed: Malformed XML',
    file: { id: 1, metadata_pending: true, cloud_only: false } });
  const f = fixture([partial, result(2), new Error('Read failed')]);
  await f.run();
  assert.equal(f.state.processed.value, 3);
  assert.equal(f.state.warned.value, 1);
  assert.equal(f.state.failed.value, 1);
  assert.equal(f.state.succeeded.value, 1);
  assert.equal(f.state.keptLocal.value, 2);
  assert.equal(f.refreshed[0], partial);
});

test('metadata import and preview failures are both counted while retaining updated metadata', async () => {
  const partial = result(1, { warning: 'XMP import failed: Malformed XML', error: 'Preview rebuild failed: Permission denied' });
  const f = fixture([partial], { freeSpace: false });
  await f.run();
  assert.equal(f.state.processed.value, 1);
  assert.equal(f.state.failed.value, 1);
  assert.equal(f.state.warned.value, 1);
  assert.equal(f.state.succeeded.value, 0);
  assert.deepEqual(f.refreshed, [partial]);
});

const content = fs.readFileSync(new URL('../src/components/Content.vue', import.meta.url), 'utf8');
function contentRefresh(startAnchor, endAnchor, dependencies, name) {
  const start = content.indexOf(startAnchor);
  const end = content.indexOf(endAnchor, start);
  assert.ok(start >= 0 && end > start);
  return new Function(...Object.keys(dependencies), ts.transpile(content.slice(start, end), { target: ts.ScriptTarget.ES2022 }) +
    `; return ${name};`)(...Object.values(dependencies));
}

test('single-file panel refresh preserves metadata and distinguishes warnings from errors', async () => {
  for (const extra of [{ warning: 'XMP import failed' }, { warning: 'XMP import failed', error: 'Preview rebuild failed' }]) {
    const updated = result(1, extra);
    const file = { id: 1 };
    const messages = [];
    const deps = {
      libConfig: { _libraryId: 'test' }, refreshFileInfo: async () => updated,
      fileInfoRevision: ref(0), tauriEmit: async () => {}, getFileListThumb: async () => {},
      toast: { warning: text => messages.push(['warning', text]), error: text => messages.push(['error', text]) },
    };
    const refresh = contentRefresh('const refreshFileInfoFromPanel = async', '// update the file info from the file;', deps, 'refreshFileInfoFromPanel');
    await refresh(file);
    assert.deepEqual(file, updated.file);
    assert.deepEqual(messages, [[extra.error ? 'error' : 'warning', [extra.warning, extra.error].filter(Boolean).join('; ')]]);
    assert.equal(deps.fileInfoRevision.value, 1);
  }
});

test('single-file menu refresh reports warnings and failures for both local and skipped originals', async () => {
  for (const skipped of [false, true]) {
    for (const extra of [{ warning: 'XMP import failed' }, { warning: 'XMP import failed', error: 'Preview rebuild failed' }]) {
      const updated = result(1, { skipped, ...extra });
      const file = { id: 1, file_path: '/photo.jpg' };
      const messages = [];
      const thumbnailLoads = [];
      const deps = {
        requireOriginalFiles: async () => true, refreshFileInfo: async () => updated,
        libConfig: { _libraryId: 'test' }, fileInfoRevision: ref(0), tauriEmit: async () => {}, getFileListThumb: async (...args) => thumbnailLoads.push(args),
        updateThumbForFile: async () => { throw new Error("Refresh must not force a second generation"); }, updateSelectedImage: async () => {}, selectedItemIndex: ref(0),
        nextTick: async () => {}, uiStore: { clearActiveAdjustments() {} },
        localeMsg: ref({ tooltip: { update_image: { success: 'Updated', failed: 'Failed' } } }),
        toast: { warning: text => messages.push(['warning', text]), error: text => messages.push(['error', text]), success: text => messages.push(['success', text]) },
      };
      const refresh = contentRefresh('const updateFile = async', '// force-update the thumbnail for the file', deps, 'updateFile');
      await refresh(file, true);
      assert.equal(file.id, updated.file.id);
      assert.equal(thumbnailLoads.length, skipped ? 0 : 1);
      if (!skipped) assert.deepEqual(thumbnailLoads[0][1], { bustCache: true, viewportBound: false });
      assert.deepEqual(messages, [[extra.error ? 'error' : 'warning', [extra.warning, extra.error].filter(Boolean).join('; ')]]);
    }
  }
});

test('all supported locales include the metadata import failure count with its placeholder', () => {
  for (const name of ['en','de','es','fr','hu','it','ja','ko','nl','pl','pt','ru','uk','vi','zh-CN','zh-TW']) {
    const locale = JSON.parse(fs.readFileSync(new URL(`../src/locales/${name}.json`, import.meta.url), 'utf8'));
    assert.match(locale.msgbox.file_refresh.metadata_failed, /\{count\}/, name);
  }
});


test('manual rebuild options and unused locale keys are removed', () => {
  assert.doesNotMatch(source, /rebuildPreviews|rebuild_previews|rebuild_pending/);
  for (const name of ['en','de','es','fr','hu','it','ja','ko','nl','pl','pt','ru','uk','vi','zh-CN','zh-TW']) {
    const locale = JSON.parse(fs.readFileSync(new URL(`../src/locales/${name}.json`, import.meta.url), 'utf8'));
    assert.equal(locale.msgbox.file_refresh.rebuild_previews, undefined, name);
    assert.equal(locale.msgbox.file_refresh.rebuild_pending, undefined, name);
  }
});
