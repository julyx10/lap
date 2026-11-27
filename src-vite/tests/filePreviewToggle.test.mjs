import ts from 'typescript';
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { computed, ref } from 'vue';

const source = await readFile(new URL('../src/components/FileInfo.vue', import.meta.url), 'utf8');
const storeSource = await readFile(new URL('../src/stores/configStore.js', import.meta.url), 'utf8');

function previewPanels({ isVideoFile, config }) {
  const start = source.indexOf('const canShowHistogram = computed(');
  const end = source.indexOf('const hasPreviewNavigator');
  assert.ok(start >= 0 && end > start, 'preview panel computeds not found');
  return new Function('computed', 'isVideoFile', 'config',
    `${ts.transpile(source.slice(start, end))} return { canShowHistogram: canShowHistogram.value, showThumbnailPanel: showThumbnailPanel.value, showHistogramPanel: showHistogramPanel.value };`
  )(computed, isVideoFile, config);
}

test('thumbnail and histogram are independent toggles (issue #538)', () => {
  const image = ref(false);
  const config = { infoPanel: {} };
  config.infoPanel.showThumbnail = true;
  config.infoPanel.showHistogram = true;
  // Both can be shown simultaneously.
  assert.deepEqual(previewPanels({ isVideoFile: image, config }), {
    canShowHistogram: true, showThumbnailPanel: true, showHistogramPanel: true,
  });
  // Toggling one does not affect the other.
  config.infoPanel.showThumbnail = false;
  assert.deepEqual(previewPanels({ isVideoFile: image, config }), {
    canShowHistogram: true, showThumbnailPanel: false, showHistogramPanel: true,
  });
  config.infoPanel.showHistogram = false;
  assert.deepEqual(previewPanels({ isVideoFile: image, config }), {
    canShowHistogram: true, showThumbnailPanel: false, showHistogramPanel: false,
  });
});

test('histogram stays unavailable for video files', () => {
  const video = ref(true);
  const config = { infoPanel: { showThumbnail: true, showHistogram: true } };
  const { canShowHistogram, showHistogramPanel } = previewPanels({ isVideoFile: video, config });
  assert.equal(canShowHistogram, false);
  assert.equal(showHistogramPanel, false);
});

test('histogram-only image preferences fall back to thumbnails for videos and survive reopening', () => {
  const isVideoFile = ref(false);
  const config = { infoPanel: { showPreview: true, showThumbnail: false, showHistogram: true } };
  assert.equal(previewPanels({ isVideoFile, config }).showHistogramPanel, true);
  isVideoFile.value = true;
  assert.deepEqual(previewPanels({ isVideoFile, config }), {
    canShowHistogram: false, showThumbnailPanel: true, showHistogramPanel: false,
  });
  const start = source.indexOf('function togglePreview()');
  const end = source.indexOf('\n}', start) + '\n}'.length;
  const togglePreview = new Function('config', 'canShowHistogram',
    ts.transpile(source.slice(start, end)) + '\nreturn togglePreview;')(config, { value: false });
  togglePreview();
  togglePreview();
  assert.equal(previewPanels({ isVideoFile, config }).showThumbnailPanel, true);
  assert.deepEqual(config.infoPanel, { showPreview: true, showThumbnail: false, showHistogram: true });
  isVideoFile.value = false;
  assert.deepEqual(previewPanels({ isVideoFile, config }), {
    canShowHistogram: true, showThumbnailPanel: false, showHistogramPanel: true,
  });
});

test('togglePreviewPanel flips the matching flag and ignores histogram for videos', () => {
  const start = source.indexOf('function togglePreviewPanel(');
  const end = source.indexOf('\n}', start) + '\n}'.length;
  assert.ok(start >= 0, 'togglePreviewPanel not found');
  const factory = new Function('canShowHistogram', 'config', ts.transpile(source.slice(start, end)) + '\nreturn togglePreviewPanel;');

  const config = { infoPanel: { showThumbnail: true, showHistogram: false } };
  const image = factory({ value: true }, config);
  image('histogram');
  assert.deepEqual(config.infoPanel, { showThumbnail: true, showHistogram: true });
  image('thumbnail');
  image('thumbnail');
  assert.deepEqual(config.infoPanel, { showThumbnail: true, showHistogram: true });

  config.infoPanel.showHistogram = false;
  const video = factory({ value: false }, config);
  video('histogram');
  assert.equal(config.infoPanel.showHistogram, false, 'video files must not enable the histogram');
  video('thumbnail');
  assert.equal(config.infoPanel.showThumbnail, false);
});

test('collapsing the last visible panel folds the preview section and restores it on re-open', () => {
  const start = source.indexOf('function togglePreview()');
  const end = source.indexOf('\n}', source.indexOf('function togglePreviewPanel(')) + '\n}'.length;
  assert.ok(start >= 0, 'togglePreview not found');
  const factory = new Function('canShowHistogram', 'config', ts.transpile(source.slice(start, end)) + '\nreturn { togglePreview, togglePreviewPanel };');

  // Both on -> turning either one off keeps the section open.
  let config = { infoPanel: { showPreview: true, showThumbnail: true, showHistogram: true } };
  let panels = factory({ value: true }, config);
  panels.togglePreviewPanel('histogram');
  assert.equal(config.infoPanel.showPreview, true, 'section stays open while a panel remains');

  // Turning off the last panel collapses the section and remembers the panel.
  panels.togglePreviewPanel('thumbnail');
  assert.deepEqual(config.infoPanel, { showPreview: false, showThumbnail: false, showHistogram: false, previewRestore: 'thumbnail' });

  // Re-opening restores exactly the last unchecked panel and clears the flag.
  panels.togglePreview();
  assert.deepEqual(config.infoPanel, { showPreview: true, showThumbnail: true, showHistogram: false });

  // Normal collapse (a panel still visible) keeps the checked state on re-open.
  config = { infoPanel: { showPreview: true, showThumbnail: true, showHistogram: false } };
  panels = factory({ value: true }, config);
  panels.togglePreview();
  panels.togglePreview();
  assert.deepEqual(config.infoPanel, { showPreview: true, showThumbnail: true, showHistogram: false });

  // With both panels on, header collapse/expand is also a no-op for the flags.
  config = { infoPanel: { showPreview: true, showThumbnail: true, showHistogram: true } };
  panels = factory({ value: true }, config);
  panels.togglePreview();
  panels.togglePreview();
  assert.deepEqual(config.infoPanel, { showPreview: true, showThumbnail: true, showHistogram: true });

  // Stale histogram flag on a video file falls back to the thumbnail.
  config = { infoPanel: { showPreview: false, showThumbnail: false, showHistogram: false, previewRestore: 'histogram' } };
  panels = factory({ value: false }, config);
  panels.togglePreview();
  assert.deepEqual(config.infoPanel, { showPreview: true, showThumbnail: true, showHistogram: false });
});

test('template renders the panels with independent v-ifs, not a v-else pair', () => {
  assert.match(source, /v-if="showThumbnailPanel"/);
  assert.match(source, /v-if="showHistogramPanel" class="rounded-box/);
  assert.doesNotMatch(source, /v-else class="rounded-box border border-base-content\/5 bg-base-300\/30/,
    'histogram must not be the v-else alternative of the thumbnail');
  assert.match(source, /@click\.stop="togglePreviewPanel\('thumbnail'\)"/);
  assert.match(source, /@click\.stop="togglePreviewPanel\('histogram'\)"/);
  assert.doesNotMatch(source, /setPreviewMode|previewMode/);
});

test('persisted radio-style previewMode migrates to the independent toggles', () => {
  const start = storeSource.indexOf('deserialize: (value) => {');
  assert.ok(start >= 0, 'deserialize not found');
  const bodyStart = start + 'deserialize: (value) => {'.length;
  const body = storeSource.slice(bodyStart, storeSource.lastIndexOf('return state;') + 'return state;'.length);
  const deserialize = new Function('value', body);
  const state = { infoPanel: {} };
  const migrate = (mode) => deserialize(JSON.stringify({ ...state, infoPanel: { previewMode: mode } }));
  assert.deepEqual(migrate('histogram').infoPanel, { showThumbnail: false, showHistogram: true });
  assert.deepEqual(migrate('thumbnail').infoPanel, { showThumbnail: true, showHistogram: false });
  assert.equal('previewMode' in migrate('histogram').infoPanel, false);
});

test('configStore defaults are independent toggles', () => {
  assert.match(storeSource, /showThumbnail: true,/);
  assert.match(storeSource, /showHistogram: false,/);
  assert.doesNotMatch(storeSource, /previewMode: 'thumbnail'/);
});
